use std::collections::HashMap;

use bevy_ecs::prelude::*;
use crate::{galaxy::{resources::{path_to_entity::PathToEntityMap, network_handler::NetworkHandler, star_system_table::SystemMapTable, database_resource::DatabaseResource}, events::{EEvent, EInfo, EState}}, network::{serialization_structs::{state::{SSystem, SPlayerShip_OTHER, NetOutState, SPlayerShip_OWN}, event::NetOutEvent, info::{NetOutInfo, hanger::SHanger}}, messages::{outgoing::NetOutgoingMessage}}, inventory::Inventory};

use super::super::components::*;

/* LOGIC IS NOT CHECKED IN HERE FOR THE MOST PART, INSTEAD IF WE GET AN EVENT, WE ASSUME IT IS LEGIT */

pub fn sys_dispatch_static_data(
    suns: Query<(&Sun, &GameObject, &Celestial)>, 
    planets: Query<(&Planet, &GameObject, &Celestial, &Transform)>, 
    moons: Query<(&Moon, &GameObject, &Celestial, &Transform)>, 
    belts: Query<(&AsteroidBelt, &GameObject, &Transform)>, 
    gates: Query<(&Gate, &GameObject, &Transform)>, 
    stations: Query<(&Station, &GameObject, &Transform)>, 
    //ptm: Res<PathToEntityMap>, 
    sys_map: Res<SystemMapTable>,
    net: Res<NetworkHandler>,
    mut events: EventReader<EEvent>,
){
    events.iter().for_each(|e| {
        let (sys, player) = match e {
            EEvent::Undock(player, ship) => (ship.sys.clone(), player),
            EEvent::Jump(player, ship_path) => (ship_path.sys.clone(), player),
            _ => { return; }
        };

        let ents = match sys_map.get_entities_in_system(&sys) {
            None => { eprintln!("Could not find system in entity map"); return; },
            Some(h) => h
        };

        let mut ser_sys = match SSystem::new(&suns, ents){
            None => { eprintln!("Sun not found for system {}", sys); return; },
            Some(ss) => ss
        };

        ser_sys.add_planets(&planets, ents);
        ser_sys.add_moons(&moons, ents);
        ser_sys.add_belts(&belts, ents);

        ser_sys.add_gates(&gates, ents);
        ser_sys.add_stations(&stations, ents);

        net.enqueue_outgoing(player, NetOutgoingMessage::State(crate::network::serialization_structs::state::NetOutState::System(ser_sys)));
    });

}

pub fn sys_dispatch_other_ships(
    sensor: Query<(&PlayerController, &Sensor)>,
    ships: Query<(&Ship, &PlayerController, &GameObject, &Transform)>,
    net: Res<NetworkHandler>,
    ptm: Res<PathToEntityMap>,
    mut est: EventReader<EState>,
){
    // sensor.par_for_each(16, |(pc, s)|{
    //     match pc.login_state {
    //         LoginState::LoggedOut(_) => { /* println!("Ship for {} logged out", pc.player_name); */ return; },
    //         _ => ()
    //     };

    //     let visible = s.visible_objs.iter().filter_map(|obj| ptm.get(obj)); // just silently ignore broken stuff, TODO: DEAL WITH THIS
    //     for v in visible {
    //         // println!("Sensor: {} can see {:?}", pc.player_name, ptm.get_path_from_entity(v));
    //         let (os, opc, ogo, ot) = match ships.get(v) {
    //             Err(_) => { eprintln!("Ship not found for sensor"); continue; },
    //             Ok(s) => s
    //         };

    //         let other_ship = SPlayerShip_OTHER {
    //             path: ogo.path.clone(),
    //             ship_class: os.ship_class.clone(),
    //             ship_name: os.ship_name.clone(),
    //             transform: ot.clone(),
    //             player_name: opc.player_name.clone()
    //         };

    //         net.enqueue_outgoing(&pc.player_name, NetOutgoingMessage::State(NetOutState::OtherShip(other_ship)));
    //     }

    //     let lockable = s.lockable_objs.iter().filter_map(|obj| ptm.get(obj)); // just silently ignore broken stuff, TODO: DEAL WITH THIS
    //     for v in lockable {
    //         // println!("Sensor: {} can see {:?}", pc.player_name, ptm.get_path_from_entity(v));
    //         let (os, opc, ogo, ot) = match ships.get(v) {
    //             Err(_) => { eprintln!("Ship not found for sensor"); continue; },
    //             Ok(s) => s
    //         };

    //         let other_ship = SPlayerShip_OTHER {
    //             path: ogo.path.clone(),
    //             ship_class: os.ship_class.clone(),
    //             ship_name: os.ship_name.clone(),
    //             transform: ot.clone(),
    //             player_name: opc.player_name.clone()
    //         };

    //         net.enqueue_outgoing(&pc.player_name, NetOutgoingMessage::State(NetOutState::OtherShip(other_ship)));
    //     }
    // });

    let mut update_map = HashMap::new();

    for event in est.iter() {
        match event {
            EState::OtherShip(player, ship_path, vis) => {
                if let Some(oship_ent) = ptm.get(&ship_path) {
                    if let Ok((os, opc, ogo, ot)) = ships.get(oship_ent) {
                        let other_ship = SPlayerShip_OTHER {
                            path: ogo.path.clone(),
                            ship_class: os.ship_class.clone(),
                            ship_name: os.ship_name.clone(),
                            transform: ot.clone(),
                            player_name: opc.player_name.clone(),
                            vis: vis.clone()
                        };
                        update_map.entry(player).or_insert(vec![]).push(other_ship);
                    }
                }
                else {
                    eprintln!("Sensed ship not found in ptm ({} seeing {:?})", player, ship_path);
                }
            }
        }
    }

    for (pc, s) in sensor.iter() {
        match pc.login_state {
            LoginState::LoggedOut(_) => { return; },
            _ => ()
        };

        update_map.remove(&pc.player_name).map(|ships| {
            for s in ships {
                net.enqueue_outgoing(&pc.player_name, NetOutgoingMessage::State(NetOutState::OtherShip(s)));
            }
        });
    }
}

pub fn sys_dispatch_other_ships_movement(
    moved: Query<(&Transform, &GameObject), Changed<Transform>>, 
    sensor: Query<(&PlayerController, &Sensor)>, 
    net: Res<NetworkHandler>,
    ptm: Res<PathToEntityMap>,
){
    sensor.par_for_each(32, |(pc, s)| {
        let mut updates: Vec<Entity> = s.visible_objs.iter().filter_map(|path| ptm.get(path)).collect();
        updates.extend(s.lockable_objs.iter().filter_map(|path| ptm.get(path)));

        for ent in updates {
            if let Ok((t, go)) = moved.get(ent) {
                net.enqueue_outgoing(&pc.player_name, NetOutgoingMessage::Mv(go.path.name.clone(), [t.pos.x, t.pos.y, t.pos.z], [t.vel.x as f32, t.vel.y as f32, t.vel.z as f32], [t.rot.w as f32, t.rot.i as f32, t.rot.j as f32, t.rot.j as f32]));
            }
        }
    });
}

pub fn sys_dispatch_own_ship(
    ships: Query<(&Ship, &PlayerController, &GameObject, &Transform, &Navigation), Or<(Changed<Ship>, Changed<Navigation>)>>,
    net: Res<NetworkHandler>
){
    ships.par_for_each(16, |(s, pc, go, t, n)| {
        match pc.login_state {
            LoginState::LoggedOut(_) => { return; },
            _ => ()
        };
        //println!("Rotation: {:?}", t.rot);
        let ship = SPlayerShip_OWN {
            path: go.path.clone(),
            ship_class: s.ship_class.clone(),
            ship_name: s.ship_name.clone(),
            transform: t.clone(),
            stats: s.stats.clone(),
            nav_action: n.cur_action.clone(),
            nav_target: n.target.clone(),
            warp_state: n.warp_state.clone()
        };

        net.enqueue_outgoing(&pc.player_name, NetOutgoingMessage::State(NetOutState::OwnShip(ship)));
    })
}

pub fn sys_dispatch_own_ship_movement(
    moved: Query<(&Transform, &GameObject, &PlayerController), Changed<Transform>>, 
    net: Res<NetworkHandler>,
){
    moved.par_for_each(16, |(t, go, pc)| {
        net.enqueue_outgoing(&pc.player_name, NetOutgoingMessage::Mv(go.path.name.clone(), [t.pos.x, t.pos.y, t.pos.z], [t.vel.x as f32, t.vel.y as f32, t.vel.z as f32], [t.rot.w as f32, t.rot.i as f32, t.rot.j as f32, t.rot.j as f32]));
    });
}

pub fn sys_dispatch_ev_dock_undock_jump(
    mut eev: EventReader<EEvent>,
    net: Res<NetworkHandler>
){
    for e in eev.iter() {
        match e {
            EEvent::Dock(player, station) => net.enqueue_outgoing(player, NetOutgoingMessage::Event(NetOutEvent::Dock(station.clone()))),
            EEvent::Undock(player, ship) => net.enqueue_outgoing(player, NetOutgoingMessage::Event(NetOutEvent::Undock(ship.clone()))),
            EEvent::Jump(player, ship) => {
                net.enqueue_outgoing(player, NetOutgoingMessage::Info(NetOutInfo::Location(ship.clone())));
                net.enqueue_outgoing(player, NetOutgoingMessage::Event(NetOutEvent::Jump(ship.clone())));   
            },
            _ => ()
        }
    }
}


pub fn sys_dispatch_inv_bank_updates(
    mut inf: EventReader<EInfo>,
    net: Res<NetworkHandler>,
    db: Res<DatabaseResource>
){
    for e in inf.iter(){
        match e {
            EInfo::UpdateInventoryHanger(player, hanger_id) => {
                let hanger = db.db.hanger_get_ships(player, hanger_id.clone());
                if let Some(h) = hanger {
                    net.enqueue_outgoing(player, NetOutgoingMessage::Info(NetOutInfo::Hanger(SHanger::from_hanger(&h, hanger_id.clone()))))
                }
            },
            EInfo::UpdateInventoryId(player, inv_id) => {
                let inv = db.db.inventory_get_inv(player, inv_id.clone());
                if let Some(i) = inv {
                    net.enqueue_outgoing(player, NetOutgoingMessage::Info(NetOutInfo::Inventory(i, inv_id.clone())));
                }
                else {
                    net.enqueue_outgoing(player, NetOutgoingMessage::Info(NetOutInfo::Inventory(Inventory::new(Some(inv_id.clone()), None), inv_id.clone())))
                }
            },
            EInfo::UpdateBankAccount(player) => {
                let val = db.db.bank_get_value(player).expect("Could not get player bank value");
                net.enqueue_outgoing(player, NetOutgoingMessage::Info(NetOutInfo::Bank(val)));
            },
            EInfo::ItemStore(player, item_id) => {
                if let Some(store) = db.db.market_load_item_store(item_id.clone()) {
                    net.enqueue_outgoing(player, NetOutgoingMessage::Info(NetOutInfo::Store(store)));
                }
            },
            EInfo::UpdateInventoryList(player, inv_list) => {
                net.enqueue_outgoing(player, NetOutgoingMessage::Info(NetOutInfo::InvList(inv_list.clone())));
            },
            _ => ()
        }
    }
}

pub fn sys_dispatch_ship_inventory_requests(
    ships: Query<&Ship>,
    mut inf: EventReader<EInfo>,
    net: Res<NetworkHandler>,
    ptm: Res<PathToEntityMap>,
){
    for e in inf.iter() {
        match e {
            EInfo::UpdateInventoryShip(player, ship_path) => {
                if let Some(ship_ent) = ptm.get(ship_path) {
                    if let Ok(ship) = ships.get(ship_ent) {
                        net.enqueue_outgoing(player, NetOutgoingMessage::Info(NetOutInfo::InventoryGameObject(ship.inventory.clone(), ship_path.clone())));
                    }
                    else {
                        eprintln!("Ship entity not found")
                    }
                }
                else {
                    eprintln!("Player ship not found: {}-{:?}", player, ship_path);
                }
            },
            _ => ()
        }
    }
}

/* TODO: Implement game object inventory requests */