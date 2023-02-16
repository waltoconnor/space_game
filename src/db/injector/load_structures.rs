use std::collections::HashMap;

use bevy_ecs::prelude::*;
use nalgebra::{Vector3, UnitQuaternion};

use crate::{galaxy::{bundles::{structures::{BStation, BGate}, celestials::BPlanet}, components::*}, shared::ObjPath};

use super::{structure_structs::{LStation, LStationList}, galaxy_structs::{LPlanet, LGalaxy, LChildBody}, orbit::{Orbit, orbit_to_csv}};

const AU: f64 = 1.4959e11;

pub fn load_stations(input_stations: LStationList, planets: &HashMap<String, BPlanet>) -> Vec<BStation> {
    let mut stations = vec![];

    for s in input_stations.stations.iter() {
        let planet = planets.get(&s.planet_name).expect("Station could not find planet");
        let orbit = Orbit::from_arr(&s.orbit, planet.celestial.mass_kg);
        let [x, y, z, _, _, _] = orbit_to_csv(&orbit, planet.celestial.mass_kg);
        let offset = Vector3::new(x, y, z);
        let abs = planet.transform.pos + offset;
        let face_away_from_planet = UnitQuaternion::face_towards(&offset, &Vector3::y_axis());
        //TODO: ADD ACTUAL LOGIC FOR THIS
        let warp_point = abs + (offset.normalize() * 1000.0); // warp in 1KM away from hanger door
        let undock_offset = face_away_from_planet.transform_vector(&Vector3::new(0.0, 0.0, 1000.0)); // 1km offset 

        let transform = Transform { pos: abs, rot: face_away_from_planet, vel: Vector3::zeros() };

        let station = BStation::new(&s.system, &s.name, transform, warp_point, undock_offset, 2500.0);
        stations.push(station);
    }

    stations
}

pub fn compute_gates(gal: &LGalaxy, planets: &HashMap<String, BPlanet>, positions: &HashMap<String, Vector3<f64>>) -> Vec<BGate> {
    let mut gates: HashMap<ObjPath, BGate> = HashMap::new();
    let dists = get_max_planet_dist(gal, planets);

    for r in gal.regions.iter() {
        for c in r.connections.iter() {
            let a_sys = &c.a;
            let b_sys = &c.b;
            if check_gates_exist(a_sys, b_sys, &gates) {
                continue;
            }
            

            let a_pos = positions.get(a_sys).expect("Could not get a pos");
            let b_pos = positions.get(b_sys).expect("Could not get b pos");
            let a_t = get_gate_transform(*a_pos, *b_pos, *dists.get(a_sys).expect("Could not find a sys"));
            let b_t = get_gate_transform(*b_pos, *a_pos, *dists.get(b_sys).expect("Could not find b sys"));
            let ((ka, ga), (kb, gb)) = generate_gate_pair(a_sys, b_sys, a_t, b_t);
            gates.insert(ka, ga);
            gates.insert(kb, gb);
        }
    }

    for c in gal.region_connections.iter() {
        let a_sys = &c.sys_a;
        let b_sys = &c.sys_b;

        if check_gates_exist(a_sys, b_sys, &gates) {
            continue;
        }
        

        let a_pos = positions.get(a_sys).expect("Could not get a pos");
        let b_pos = positions.get(b_sys).expect("Could not get b pos");
        let a_t = get_gate_transform(*a_pos, *b_pos, *dists.get(a_sys).expect("Could not find a sys"));
        let b_t = get_gate_transform(*b_pos, *a_pos, *dists.get(b_sys).expect("Could not find b sys"));
        let ((ka, ga), (kb, gb)) = generate_gate_pair(a_sys, b_sys, a_t, b_t);
        gates.insert(ka, ga);
        gates.insert(kb, gb);
    }


    gates.into_iter().map(|(_k, b)| b).collect()
}

fn check_gates_exist(a_sys: &String, b_sys: &String, gates: &HashMap<ObjPath, BGate>) -> bool {
    let a_name = format!("{}->{}", a_sys, b_sys);
    let b_name = format!("{}->{}", b_sys, a_sys);
    let a_path = ObjPath::new(a_sys, crate::shared::ObjectType::Gate, &a_name);
    let b_path = ObjPath::new(b_sys, crate::shared::ObjectType::Gate, &b_name);
    if gates.contains_key(&a_path) && gates.contains_key(&b_path) {
        return true;
    }

    //IF WE ONLY CREATED ONE OF THE GATES, SOMETHING IS BROKEN
    if gates.contains_key(&a_path) || gates.contains_key(&b_path) {
        eprintln!("ERROR: GENERATED ONE GATE OF PAIR");
    }

    false
}

fn generate_gate_pair(a_sys: &String, b_sys: &String, a_transform: Transform, b_transform: Transform) -> ((ObjPath, BGate), (ObjPath, BGate)) {
    let a_name = format!("{}->{}", a_sys, b_sys);
    let b_name = format!("{}->{}", b_sys, a_sys);

    let a_path = ObjPath::new(a_sys, crate::shared::ObjectType::Gate, &a_name);
    let b_path = ObjPath::new(b_sys, crate::shared::ObjectType::Gate, &b_name);

    let a_wip = get_warp_in_point(&a_transform.pos);
    let b_wip = get_warp_in_point(&b_transform.pos);

    let a_g = BGate::new(a_sys, &a_name, b_sys, &b_name, 2500.0, a_transform, a_wip);
    let b_g = BGate::new(b_sys, &b_name, a_sys, &a_name, 2500.0, b_transform, b_wip);

    ((a_path, a_g), (b_path, b_g))
}

fn get_gate_transform(start_pos: Vector3<f64>, end_pos: Vector3<f64>, planet_dist: f64) -> Transform {
    let vec = (end_pos - start_pos).normalize();
    let pos = vec * (planet_dist + 2.0 * AU);

    let up = if vec.angle(&Vector3::new(0.0, 1.0, 0.0)) < 0.01 {
        Vector3::new(1.0, 0.0, 0.0)
    }
    else {
        Vector3::new(0.0, 1.0, 0.0)
    };

    let rot = UnitQuaternion::face_towards(&vec, &up);
    Transform { pos, rot, vel: Vector3::zeros() }
}

fn get_warp_in_point(gate_pos: &Vector3<f64>) -> Vector3<f64> {
    let dist = 2500.0;
    gate_pos * (gate_pos.magnitude() - dist) / gate_pos.magnitude()
}


fn get_max_planet_dist(gal: &LGalaxy, planets: &HashMap<String, BPlanet>) -> HashMap<String, f64> {
    let mut pm = HashMap::new();
    for r in gal.regions.iter() {
        for (name, s) in r.systems.iter() {
            let dist = s.sys.children
                .iter()
                .filter_map(|c| match c { LChildBody::Planet(p) => Some(p), _ => None})
                .map(|p: &LPlanet| planets.get(&p.name).expect("Could not find planet").transform.pos.magnitude() as u64)
                .max()
                .unwrap_or((5.0 * AU) as u64) as f64; //default to five au
            pm.insert(name.clone(), dist);
        }
    }
    pm
}