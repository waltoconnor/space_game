use std::{f64::consts::PI};

use bevy_ecs::prelude::*;
use nalgebra::{Vector3, UnitQuaternion};

use crate::{galaxy::{components::*, resources::{network_handler::NetworkHandler, path_to_entity::PathToEntityMap, delta_time::DeltaTime}, events::{EInfo}}, network::messages::incoming::NetIncomingMessage, shared::{ObjPath, ObjectType}};

/// PROCESS NON WARP NAVIGATION MESSAGES
/// Stage: COMMAND
pub fn sys_process_navigation_inputs_local(mut players: Query<(&PlayerController, &mut Navigation, &Ship)>, n: Res<NetworkHandler>, ptm: Res<PathToEntityMap>, mut ein: EventWriter<EInfo>) {
    for entry in n.view_incoming().iter() {
        for msg in entry.value().iter() {
            match msg {
                NetIncomingMessage::Approach(ship, dst) => update_navigation_local(&mut players, &ptm, ship, dst, &entry.key(), Action::Approach, &mut ein),
                _ => ()
                /* DO NOT PROCESS WARPS HERE */
            }
        }
    }

}

//TODO: Add visibility check
fn update_navigation_local(q: &mut Query<(&PlayerController, &mut Navigation, &Ship)>, ptm: &Res<PathToEntityMap>, ship_path: &ObjPath, dst: &ObjPath, player: &String, op: Action, ein: &mut EventWriter<EInfo>) {
    // get ship entity
    let ship_ent = match ptm.get(ship_path){
        Some(s) => s,
        None => { eprintln!("Player requesting action for nonexistent ship"); return; }
    };

    //get components
    let (pc, mut nav, _ship) = match q.get_mut(ship_ent) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("Navigation ship not found in space: {}", player);
            return;
        }
    };

    // validate player
    if pc.player_name != *player {
        eprintln!("Player sending command for ship they don't own");
        return;
    }

    if ship_path.sys != dst.sys {
        ein.send(EInfo::Error(player.clone(), String::from("Navigation target no longer in system")));
        //eprintln!("Player trying to warp to object in other system TODO: ALERT THEM");
        return;
    }

    if let WarpState::Warping(_) = nav.warp_state {
        ein.send(EInfo::Error(player.clone(), String::from("You cannot issue navigation commands while warping")));
        //eprintln!("TODO: alert player that they can't issue nav commands while warping");
        return;
    }

    //println!("{:?} ===> {:?}", ship_path, dst);
    nav.cur_action = op;
    nav.warp_state = WarpState::NotWarping;
    nav.target = NavTarget::Obj(dst.clone())
}

/// PROCESS WARP NAVIGATION MESSAGES
/// Stage: COMMAND
// TODO: CHECK VISIBILITY OF TRANSFORM
pub fn sys_process_navigation_inputs_warp(mut players: Query<(&PlayerController, &mut Navigation, &Ship)>, warp_targets: Query<&WarpTarget>, transforms: Query<&Transform>, n: ResMut<NetworkHandler>, ptm: Res<PathToEntityMap>, mut ein: EventWriter<EInfo>) {
    for entry in n.view_incoming().iter() {
        let msgs = entry.value();
        let player = entry.key();
        for msg in msgs.iter() {
            match msg {
                NetIncomingMessage::WarpTo(ship_path, dst, dist) => update_navigation_warp(&mut players, &warp_targets, &transforms, &ptm, ship_path, &dst, player, *dist, &mut ein),
                _ => ()
            }
        }
    }
}

//TODO: Add visibility check
fn update_navigation_warp(q: &mut Query<(&PlayerController, &mut Navigation, &Ship)>, warp_targets: &Query<&WarpTarget>, transforms: &Query<&Transform>, ptm: &Res<PathToEntityMap>, ship_path: &ObjPath, dst: &ObjPath, player: &String, dist: f64, ein: &mut EventWriter<EInfo>) {
    // get ship entity
    let ship_ent = match ptm.get(ship_path){
        Some(s) => s,
        None => { eprintln!("Player requesting action for nonexistent ship"); return; }
    };

    //get components
    let (pc, mut nav, _ship) = match q.get_mut(ship_ent) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("Navigation ship not found in space: {}", player);
            return;
        }
    };

    // validate player
    if pc.player_name != *player {
        eprintln!("Player sending command for ship they don't own");
        return;
    }

    if ship_path.sys != dst.sys {
        ein.send(EInfo::Error(player.clone(), String::from("Warp target no longer in system")));
        //eprintln!("Player trying to warp to object in other system TODO: ALERT THEM");
        return;
    }

    if let WarpState::Warping(_) = nav.warp_state {
        ein.send(EInfo::Error(player.clone(), String::from("You cannot issue navigation commands while warping")));
        //eprintln!("TODO: alert player that they can't issue nav commands while warping");
        return;
    }

    let target_ent = match ptm.get(dst) {
        Some(d) => d,
        None => {
            ein.send(EInfo::Error(player.clone(), String::from("Warp target not found")));
            eprintln!("Warp target entity not found");
            return;
        }
    };

    let wt = match warp_targets.get(target_ent) {
        Ok(w) => NavTarget::Point(w.warp_point),
        Err(_) => {
            match transforms.get(target_ent) {
                Ok(_t) => NavTarget::Obj(dst.clone()),
                Err(_) => {
                    ein.send(EInfo::Error(player.clone(), String::from("That is not a valid warp target")));
                    //eprintln!("Invalid warp target");
                    return;
                }
            }
        }
    };

    nav.cur_action = Action::Warp(dist);
    nav.warp_state = WarpState::Aligning;
    nav.target = wt;
}

/// UDPATES THE POSITIONS OF ALL THE NAVIGATION TARGETS, MUST BE RUN BEFORE sys_tick_navigation
pub fn sys_navigation_update_transform_positions(mut q: Query<(&mut Navigation, &Sensor)>, transforms: Query<&Transform>, ptm: Res<PathToEntityMap>) {
    q.par_for_each_mut(128, |(mut nav, sensor)| {
        let (point, vel) = match &nav.target {
            NavTarget::Obj(o) => {
                if !is_static(o.t) && !sensor.visible_objs.contains(o) && !sensor.lockable_objs.contains(o) {
                    //eprintln!("Lost entity");
                    nav.reset(); //TODO: send lost track of entity message
                    return;
                }

                let ent = match ptm.get(&o) {
                    Some(e) => e,
                    None => { nav.reset(); /* eprintln!("Lost entity 2"); */ return; }
                };

                let transform: &Transform = match transforms.get(ent) {
                    Ok(t) => t,
                    Err(_) => { nav.reset(); /* eprintln!("Lost entity 3"); */ return; }
                };
                (transform.pos, Some(transform.vel))
            },
            NavTarget::Point(p) => (*p, None),
            NavTarget::None => { return; }
        };
        nav.cur_target_pos = Some(point);
        nav.cur_target_vel = vel;
    });
}

fn is_static(t: ObjectType) -> bool {
    match t {
        ObjectType::AsteroidBelt => true,
        ObjectType::Gate => true,
        ObjectType::Planet => true,
        ObjectType::Star => true,
        ObjectType::Station => true,
        _ => false
    }
}

/// TICKS NAVIGATION FOR ALL THINGS, NOT JUST PLAYERS
/// Stage: ACTION
// TODO: make this respect visibility rules
pub fn sys_tick_navigation(mut q: Query<(&mut Navigation, &Ship, &mut Transform)>, _ptm: Res<PathToEntityMap>, dt: Res<DeltaTime>) {
    q.par_for_each_mut(32, |(mut nav, ship, mut ship_transform)| {
        //println!("nav tick");
        let vel = nav.cur_target_vel;
        match nav.cur_target_pos {
            Some(ctp) => update_navigation(&mut nav, ship, &mut ship_transform, ctp, vel, dt.dt),
            None => { /*println!("No target pos");*/ }
        };
    });
}

// fn reset_navigation(nav: &mut Navigation) {
//     nav.cur_action = Action::None;
//     nav.target = NavTarget::None;
//     nav.warp_state = WarpState::NotWarping;
//     //TODO: Send message here
// }

fn update_navigation(nav: &mut Navigation, ship: &Ship, transform: &mut Transform, target_pos: Vector3<f64>, target_vel: Option<Vector3<f64>>, dt: f64) {
    //println!("{:#?}", nav);
    match nav.cur_action {
        Action::Warp(t) => handle_warp_to(nav, ship, transform, target_pos, dt, t),
        Action::AlignTo => handle_align_to(nav, ship, transform, target_pos, dt),
        Action::Approach => handle_approach(nav, ship, transform, target_pos, target_vel, dt),
        Action::KeepAtRange(r) => handle_keep_at_range(nav, ship, transform, target_pos, target_vel, dt, r),
        Action::None => (),
        Action::Orbit(r) => handle_orbit(nav, ship, transform, target_pos, dt, r),
    }
}

fn handle_warp_to(nav: &mut Navigation, ship: &Ship, transform: &mut Transform, target_pos: Vector3<f64>, dt: f64, target_dist: f64) {
    match nav.warp_state {
        WarpState::NotWarping => { eprintln!("Handle warp to called on ship that isn't warping"); },
        WarpState::Aligning => { 
            handle_align_to(nav, ship, transform, target_pos, dt); 
            let diff = target_pos - transform.pos;
            let up = if diff.angle(&Vector3::new(0.0, 1.0, 0.0)) < 0.01 {
                Vector3::new(1.0, 0.0, 0.0)
            }
            else {
                Vector3::new(0.0, 1.0, 0.0)
            };

            let rot_to_target = UnitQuaternion::face_towards(&diff, &up);
            if transform.rot.angle_to(&rot_to_target) < 5.0 * (2.0 * PI) / 360.0 {
                println!("Warping");
                nav.warp_state = WarpState::Warping(0.0);
            }
        },
        WarpState::Warping(n) => {
            // SPOOL UP
            if n < 1.0 {
                let new_n = n + (dt as f32) / ship.stats.warp_spool_s;
                if new_n >= 1.0 {
                    nav.warp_state = WarpState::Warping(1.0);
                }
                else {
                    nav.warp_state = WarpState::Warping(new_n);
                }
                return;
            }

            transform.vel = Vector3::zeros();
            let dist_to_object = transform.pos.metric_distance(&target_pos);

            // we want to warp to a point n meters away from the object
            let real_target_point = transform.pos.lerp(&target_pos, 1.0 - (target_dist / dist_to_object));
            let real_dist = transform.pos.metric_distance(&real_target_point);

            
            
            if real_dist > 11000000.0 { //11,000 KM (we try to hit 10K KM, and if we undershoot we still activate the deceleration)
                let warp_target_point = transform.pos.lerp(&real_target_point, 1.0 - (10000000.0 / dist_to_object));
                let warp_target_dist = transform.pos.metric_distance(&warp_target_point);
                let lerp_amount = ((ship.stats.warp_speed_ms * dt) / warp_target_dist).min(1.0);
                if lerp_amount > 0.99 {
                    transform.pos = warp_target_point;
                }
                else {
                    transform.pos = transform.pos.lerp(&real_target_point, lerp_amount);
                }
            }
            else {
                let lerp_amount = 0.1 * (dt / 0.1); //cover 0.1 of the distance to the target each 10th second once we are within 10,000KM
                transform.pos = transform.pos.lerp(&real_target_point, lerp_amount);
                if real_dist < 10.0 { //1 meter
                    nav.reset();
                }
            };

            //println!("Dist from actual and target: {}, real_dist: {}, lerp: {}", target_pos.metric_distance(&real_target_point), real_dist, lerp_amount);
            
        }
    }
}

fn handle_align_to(_nav: &mut Navigation, ship: &Ship, transform: &mut Transform, target_pos: Vector3<f64>, dt: f64) {
    //println!("Aligning t={:?}", transform);
    let diff = target_pos - transform.pos;
    align_to_vector(transform, ship, diff, dt)
}

fn handle_approach(nav: &mut Navigation, ship: &Ship, transform: &mut Transform, target_pos: Vector3<f64>, target_vel: Option<Vector3<f64>>, dt: f64) {
    //println!("Approaching t={:?}", transform);
    let tvel = match target_vel { Some(v) => v, None => Vector3::zeros() };
    let rel_vel = tvel - transform.vel;
    let dist = target_pos.metric_distance(&transform.pos);
    let dir_to_target = (target_pos - transform.pos).normalize();

    if dist < 10.0 && rel_vel.magnitude() < 1.0 { //if we are at the target and within 1 m/s, just call it close enough
        transform.vel = tvel;
        return;
    }

    let closing_rate = rel_vel.dot(&dir_to_target);
    let bad_vel = rel_vel - (dir_to_target * closing_rate); // undesierable velocity component
    
    let max_accel = ship.stats.thrust_n / ship.stats.mass_kg;
    let time_to_decelerate = closing_rate / max_accel;
    let dist_to_decelerate = 0.5 * max_accel * time_to_decelerate * time_to_decelerate; // 1/2 * a * t^2
    let dist_to_stop_accelerating = 2.0 * dist_to_decelerate;

    //three phases:
    // 1 - get up to closing speed
    // 2 - make corrections
    // 3 - slow down from closing speed

    if dist > dist_to_stop_accelerating {
        // we are really far away, burn right towards it
        // this generates a vector showing how long we have to burn in each direction to hit our target velocity (roughly speaking)
        let direction_vec = (dir_to_target * dist / max_accel) + bad_vel / max_accel; 
        align_to_vector(transform, ship, direction_vec, dt);
        if is_aligned(transform, direction_vec, 0.1) {
            // compute the acceleration vector given our current engine direction, and apply the burn
            let dv = transform.rot.transform_vector(&Vector3::new(0.0, 0.0, max_accel)) * dt;
            transform.vel += dv;
        }
    }
    else if dist <= dist_to_stop_accelerating && dist > dist_to_decelerate {
        // we are in the middle third, coast and make corrections
        let direction_vec = bad_vel;
        let needed_accel = bad_vel.magnitude() / max_accel / dt;
        let cur_accel = (max_accel).min(needed_accel); 
        align_to_vector(transform, ship, direction_vec, dt);
        if is_aligned(transform, direction_vec, 0.1) {
            //println!("Burning to correct error");
            // compute the acceleration vector given our current engine direction, and apply the burn

            let dv = transform.rot.transform_vector(&Vector3::new(0.0, 0.0, cur_accel)) * dt;
            transform.vel += dv;
        }
    }
    else {
        // we are in the final third, slow down
        //println!("Killing relative velocity");
        kill_relative_velocity(nav, ship, transform, target_pos, target_vel, dt);
    }
}

fn kill_relative_velocity(nav: &mut Navigation, ship: &Ship, transform: &mut Transform, target_pos: Vector3<f64>, target_vel: Option<Vector3<f64>>, dt: f64) {
    let tvel = match target_vel { Some(v) => v, None => Vector3::zeros() };
    let rel_vel = tvel - transform.vel;

    if rel_vel.magnitude() < 5.0 && transform.pos.metric_distance(&target_pos) < 500.0 {
        transform.vel = tvel;
    }

    let burn_dir = rel_vel.normalize();
    align_to_vector(transform, ship, burn_dir, dt);
    if is_aligned(transform, burn_dir, 0.01) {
        let max_accel = ship.stats.thrust_n / ship.stats.mass_kg;
        let needed_accel = rel_vel.magnitude() / max_accel / dt;
        let cur_accel = (max_accel).min(needed_accel); 
        transform.vel += burn_dir * cur_accel;
    }
}

fn handle_keep_at_range(nav: &mut Navigation, ship: &Ship, transform: &mut Transform, target_pos: Vector3<f64> ,target_vel: Option<Vector3<f64>>, dt: f64, r: f64) {
    eprintln!("TODO: keep at range");
}

fn handle_orbit(nav: &mut Navigation, ship: &Ship, transform: &mut Transform, target_pos: Vector3<f64>, dt: f64, r: f64) {
    eprintln!("TODO: orbit")
}

/* UTILITY FUNCTIONS */
fn align_to_vector(transform: &mut Transform, ship: &Ship, v: Vector3<f64>, dt: f64) {
    let up = if v.angle(&Vector3::new(0.0, 1.0, 0.0)) < 0.01 {
        Vector3::new(1.0, 0.0, 0.0)
    }
    else {
        Vector3::new(0.0, 1.0, 0.0)
    };

    let rot_to_target = UnitQuaternion::face_towards(&v, &up);
    if rot_to_target.i.is_nan() || rot_to_target.j.is_nan() || rot_to_target.k.is_nan() || rot_to_target.w.is_nan() {
        return; // THIS HAPPENS IF THE ROTATION IS ZERO
    }

    let angle_to = transform.rot.angle_to(&rot_to_target);
    let slerp_amount = ((ship.stats.ang_vel_rads * dt) / angle_to).min(1.0);
    //println!("slerp: {}", slerp_amount);
    transform.rot = transform.rot.slerp(&rot_to_target, slerp_amount);
    if transform.rot.i.is_nan() {
        println!("ROT IS NAN: slerp_amount: {}, angle_to: {}, rot_to_target: {:?}", slerp_amount, angle_to, rot_to_target);
        transform.rot = rot_to_target; //just force it through
    }
}

fn is_aligned(transform: &Transform, v: Vector3<f64>, epsilon_rad: f64) -> bool {
    let up = if v.angle(&Vector3::new(0.0, 1.0, 0.0)) < 0.01 {
        Vector3::new(1.0, 0.0, 0.0)
    }
    else {
        Vector3::new(0.0, 1.0, 0.0)
    };

    let rot_to_target = UnitQuaternion::face_towards(&v, &up);
    transform.rot.angle_to(&rot_to_target) < epsilon_rad
}


/// THIS UPDATES POSITIONS
/// Stage: CONSEQUENCE
pub fn sys_tick_transforms(mut t: Query<&mut Transform>, dt: Res<DeltaTime>) {
    t.par_for_each_mut(500, |mut transform| {
        let vel = transform.vel;
        transform.pos += vel * dt.dt
    });
}