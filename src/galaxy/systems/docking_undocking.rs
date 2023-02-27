use bevy_ecs::prelude::*;

use crate::{galaxy::{components::*, resources::{network_handler::NetworkHandler, path_to_entity::PathToEntityMap, database_resource::DatabaseResource}, bundles::ships::BPlayerShip, events::{EEvent, EInfo}}, network::messages::incoming::NetIncomingMessage, shared::ObjPath};

pub fn sys_process_dock(players: Query<(&PlayerController, &Ship, &Transform)>, hangers: Query<(&Hanger, &Transform)>, mut commands: Commands, n: Res<NetworkHandler>, ptm: Res<PathToEntityMap>, db: Res<DatabaseResource>, mut eev: EventWriter<EEvent>, mut ein: EventWriter<EInfo>) {
    for player in n.view_incoming() {
        let name = player.key();
        for msg in player.value() {
            match msg {
                NetIncomingMessage::Dock(ship, station) => handle_dock(&players, &hangers, &mut commands, &ptm, ship, station, &db, name, &mut eev, &mut ein),
                NetIncomingMessage::Undock(hanger_path) => handle_undock(&hangers, &ptm, hanger_path, &db, name, &mut commands, &mut eev, &mut ein),
                _ => ()
            }
        }
        
    }
}

fn handle_dock(players: &Query<(&PlayerController, &Ship, &Transform)>, hangers: &Query<(&Hanger, &Transform)>, commands: &mut Commands, ptm: &Res<PathToEntityMap>, ship: &ObjPath, station: &ObjPath, db: &Res<DatabaseResource>, player_name: &String, eev: &mut EventWriter<EEvent>, ein: &mut EventWriter<EInfo>) {
    if ship.sys != station.sys {
        eprintln!("Can't dock to station in other system");
        return;
    }
    
    let docking_ent = match ptm.get(ship) {
        Some(s) => s,
        None => { eprintln!("Ship entity not found for docking"); return; }
    };

    let station_ent = match ptm.get(station) {
        Some(s) => s,
        None => {
            ein.send(EInfo::Error(player_name.clone(), "Hanger not found on object".to_string())); 
            eprintln!("Hanger entity not found for docking"); 
            return; 
        }
    };
    
    let (pc, p_ship, p_transform) = match players.get(docking_ent) {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Docking nonexistent ship");
            return;
        }
    };

    if pc.player_name != *player_name {
        eprintln!("Can't control another player's ship");
        return;
    }

    let (hanger, h_transform) = match hangers.get(station_ent) {
        Ok(h) => h,
        Err(_) => {
            ein.send(EInfo::Error(player_name.clone(), "Hanger not found on object".to_string())); 
            eprintln!("Docking to nonexistent hanger");
            return;
        }
    };

    // CHECK IF CAN DOCK

    // CHECK IF IN RANGE
    if !(h_transform.pos.metric_distance(&p_transform.pos) < hanger.docking_range_m) {
        ein.send(EInfo::Error(player_name.clone(), "You are not in range to dock".to_string())); 
        eprintln!("Player out of range to dock");
        return;
    }    

    db.db.hanger_dock(player_name, hanger.hanger_uid.clone(), p_ship.clone());
    db.db.account_change_location(player_name, station.clone());
    eev.send(EEvent::Dock(player_name.clone(), station.clone()));
    ein.send(EInfo::UpdateInventoryId(player_name.clone(), hanger.hanger_uid.clone()));
    ein.send(EInfo::UpdateInventoryHanger(player_name.clone(), hanger.hanger_uid.clone()));
    commands.entity(docking_ent).despawn();
}

fn handle_undock(hangers: &Query<(&Hanger, &Transform)>, ptm: &Res<PathToEntityMap>, hanger_path: &ObjPath, db: &Res<DatabaseResource>, player_name: &String, commands: &mut Commands, eev: &mut EventWriter<EEvent>, ein: &mut EventWriter<EInfo>) {
    let loc = match db.db.account_get_location(player_name) {
        Some(loc) => loc,
        None => { eprintln!("Player has no location when trying undock"); return; }
    };

    if loc != *hanger_path {
        eprintln!("Player trying to undock from hanger they are not in");
        ein.send(EInfo::Error(player_name.clone(), String::from("You cannot undock from a station you are not in")));
    }

    let hanger_ent = match ptm.get(hanger_path) {
        Some(h) => h,
        None => {
            eprintln!("Player undocking from nonexistent station");
            return;
        }
    };

    let (hanger, h_transform) = match hangers.get(hanger_ent) {
        Ok(h) => h,
        Err(_) => {
            eprintln!("HANGER NO LONGER EXISTS");
            eprintln!("TODO: move to home station if this hanger doesn't exist");
            return;
        }
    };

    let ship = db.db.hanger_undock(player_name, hanger.hanger_uid.clone());

    match ship {
        None => {
            ein.send(EInfo::Error(player_name.clone(), "You do not have an active ship".to_string())); 
            eprintln!("Player has no active ship");
            return;
        },
        Some(s) => {
            let mut t = h_transform.clone();
            t.pos += hanger.undock_offset;
            let ship_name = format!("{}:{}", player_name, s.ship_name);
            let new_ship = BPlayerShip::new(player_name, t, s, &hanger_path.sys, &ship_name);
            db.db.account_change_location(player_name, new_ship.game_obj.path.clone());
            eev.send(EEvent::Undock(player_name.clone(), new_ship.game_obj.path.clone()));
            commands.spawn(new_ship);
        }
    }


}