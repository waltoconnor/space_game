use bevy_ecs::prelude::*;
use nalgebra::Vector3;
use crate::galaxy::bundles::ships::BPlayerShip;
use crate::galaxy::components::*;
use crate::galaxy::events::{EEvent, EInfo};
use crate::galaxy::resources::galaxy_map::GalaxyMapRes;
use crate::galaxy::resources::{database_resource::DatabaseResource, network_handler::NetworkHandler, path_to_entity::PathToEntityMap};
use crate::network::messages::incoming::NetIncomingMessage;
use crate::network::messages::outgoing::NetOutgoingMessage;
use crate::network::serialization_structs::info::NetOutInfo;
use crate::shared::ObjectType;

pub fn sys_dispatch_login_info(
    mut ships: Query<(&mut PlayerController, &Ship, &mut Transform, &mut Navigation, &GameObject, Entity)>,
    hangers: Query<&Hanger>,
    ptm: Res<PathToEntityMap>,
    net: Res<NetworkHandler>,
    db: Res<DatabaseResource>,
    gmap: Res<GalaxyMapRes>,
    mut command: Commands,
    mut eev: EventWriter<EEvent>,
    mut ein: EventWriter<EInfo>,
){
    for entry in net.view_incoming() {
        let player = entry.key();
        let msgs = entry.value();
        for msg in msgs {
            match msg {
                NetIncomingMessage::Login(player, _) => {
                    let loc = match db.db.account_get_location(player) {
                        None => {
                            eprintln!("Account has no location set");
                            continue;
                        },
                        Some(l) => l
                    };

                    // if we find the dest entity in the "in space ship list", set the ships status as logged in
                    if let Some(dst_entity) = ptm.get(&loc) {
                        match ships.get_mut(dst_entity) {
                            Ok(mut s) => {
                                s.0.login_state = LoginState::LoggedIn; 
                                eev.send(EEvent::Undock(player.clone(), loc));
                                ein.send(EInfo::UpdateInventoryShip(player.clone(), s.4.path.clone()));
                                println!("Reset player's ship"); 
                                continue; 
                            },
                            Err(_) => ()
                        };
                    }

                    if loc.t == ObjectType::PlayerShip {
                        match db.db.sis_load_ship(player) {
                            Some(s) => {
                                command.spawn(BPlayerShip::load_from_db(s.ship, player, s.nav, s.transform, s.game_obj));
                                /* TODO: the player is not in the PTEM by the time the inventory request executes*/
                                /* MOVE THIS TO THE BOOK KEEPING SECTION SO IT GETS HANDLED NEXT FRAME */
                                //ein.send(EInfo::UpdateInventoryShip(player.clone(), loc.clone())); 
                            },
                            None => {
                                eprintln!("Ship not found in db, TODO: reset player to home");
                                continue;
                            }
                        };
                        eev.send(EEvent::Undock(player.clone(), loc.clone()));
                    }
                    else {
                        if let Some(ent) = ptm.get(&loc) {
                            if let Ok(hanger) = hangers.get(ent) {
                                ein.send(EInfo::UpdateInventoryId(player.clone(), hanger.hanger_uid.clone()));
                                ein.send(EInfo::UpdateInventoryHanger(player.clone(), hanger.hanger_uid.clone()));
                            }
                            else {
                                eprintln!("During login, found login station but didnn't find hanger");
                            }
                        }
                        else {
                            eprintln!("During login, could not find hanger attatched to login station");
                        }
                        
                    }

                    net.enqueue_outgoing(player, NetOutgoingMessage::Info(NetOutInfo::GalaxyMap(gmap.gmap.clone())));
                    net.enqueue_outgoing(player, NetOutgoingMessage::Info(NetOutInfo::Location(loc)));
                    if let Some(money) = db.db.bank_get_value(player) {
                        net.enqueue_outgoing(player, NetOutgoingMessage::Info(NetOutInfo::Bank(money)));
                    }
                    else {
                        eprintln!("ERROR: NO BANK ACCOUNT FOR {}", player);
                    }
                    /* TODO: HANDLE SKILLS AND BANK ACCOUNT */
                },
                NetIncomingMessage::Disconnect => {
                    let path = match db.db.account_get_location(player) {
                        Some(p) => p,
                        None => {
                            eprintln!("Couldn't find location for {}", player);
                            continue;
                        }
                    };

                    if path.t != ObjectType::PlayerShip {
                        return;
                    }

                    let ent = match ptm.get(&path) {
                        Some(ent) => ent,
                        None => {
                            eprintln!("Couldn't find entity for player ship: {}", player);
                            continue;
                        }
                    };

                    ships.get_mut(ent).as_mut().map(|pc| pc.0.login_state = LoginState::LoggedOut(std::time::Instant::now())).expect("Could not set safe log");
                }
                _ => ()
            }
        }
    }

    ships.for_each_mut(|(pc, ship, mut transform, mut nav, go, ent)| {
        let safe_log_time = match pc.login_state {
            LoginState::LoggedIn => { return; },
            LoginState::LoggedOut(time) => time,
            LoginState::SafeLogged => { return; }
        };

        if let WarpState::Warping(_) = nav.warp_state {
            return; //wait for warp to finish
        }

        nav.reset();

        let elapsed = safe_log_time.elapsed().as_secs();

        transform.vel *= 0.9;
        
        if elapsed > 10 {
            transform.vel = Vector3::zeros();
            eprintln!("TODO: add safe logout duration as setting");
            db.db.sis_save_ship(&pc.player_name, ship, &nav, &transform, go);
            command.entity(ent).despawn();
        }
    });
}

pub fn logon_bookeeping_handle_send_initial_info(new_ships: Query<(&Ship, &GameObject, &PlayerController), Added<PlayerController>>, mut ein: EventWriter<EInfo>) {
    for (_s, go, pc) in new_ships.iter() {
        ein.send(EInfo::UpdateInventoryShip(pc.player_name.clone(), go.path.clone()));
    }
}