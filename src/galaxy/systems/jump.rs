use bevy_ecs::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};
use rand::Rng;
use crate::{galaxy::{components::*, resources::{network_handler::NetworkHandler, path_to_entity::PathToEntityMap, database_resource::DatabaseResource}, events::{EInfo, EEvent}}, network::messages::incoming::NetIncomingMessage, shared::ObjPath};

pub fn sys_process_jump_inputs(mut players: Query<(&PlayerController, &mut Transform, &mut GameObject, &mut Navigation)>, gates: Query<(&Gate, &Transform, &GameObject), Without<PlayerController>>, ptm: Res<PathToEntityMap>, n: Res<NetworkHandler>, db: Res<DatabaseResource>, mut eev: EventWriter<EEvent>, mut ein: EventWriter<EInfo>) {
    let mut rng = rand::thread_rng();
    for player in n.view_incoming() {
        let player_name = player.key();
        for msg in player.value() {
            match msg {
                NetIncomingMessage::Jump(ship_path, gate_path) => {
                    if ship_path.sys != gate_path.sys {
                        eprintln!("Not in same system as gate");
                        continue;
                    }

                    let ship_ent = match ptm.get(ship_path) {
                        None => {
                            eprintln!("Trying to jump with ship not in table");
                            continue;
                        },
                        Some(s) => s
                    };

                    let (pc, mut pc_transform, mut go, mut nav) = players.get_mut(ship_ent).expect("Could not get ship entity for jump");
                    if pc.player_name != *player_name {
                        eprintln!("Trying to jump with other player's ship");
                        continue;
                    }

                    let gate_ent = match ptm.get(gate_path) {
                        None => {
                            ein.send(EInfo::Error(player_name.clone(), String::from("Gate not found")));
                            eprintln!("Jumping on nonexistent gate");
                            continue;
                        },
                        Some(g) => g
                    };

                    let (gate, g_transform, g_go) = gates.get(gate_ent).expect("Could not get gate entity");

                    let dist = pc_transform.pos.metric_distance(&g_transform.pos);
                    if dist >= gate.jump_range {
                        ein.send(EInfo::Error(player_name.clone(), format!("Too far away to jump, must be within {} meters", gate.jump_range)));
                        eprintln!("Too far away to jump");
                        continue;
                    }

                    let dst_gate_ent = match ptm.get(&gate.dst_gate) {
                        Some(dst) => dst,
                        None => {
                            ein.send(EInfo::Error(player_name.clone(), format!("Gate destination ({}) not found", gate.dst_gate.name)));
                            eprintln!("Gate is connected to nonexistent dst: {:?} -> {:?}", g_go.path, gate.dst_gate);
                            continue;
                        }
                    };

                    let (dst_gate, dst_gate_transform, dst_go) = gates.get(dst_gate_ent).expect("Could not get dst gate");
                    nav.reset();
                    pc_transform.vel = Vector3::zeros();
                    pc_transform.pos = dst_gate_transform.pos + (Vector3::<f64>::new(rng.gen::<f64>() - 0.5, rng.gen::<f64>() - 0.5, rng.gen::<f64>() - 0.5).normalize() * 1000.0);
                    go.path = ObjPath::new(&dst_go.path.sys, go.path.t.clone(), &go.path.name);
                    eev.send(EEvent::Jump(player_name.clone(), dst_go.path.sys.clone()));
                    db.db.account_change_location(player_name, go.path.clone());
                },
                _ => ()
            }
        }
    }
}

