use bevy_ecs::prelude::*;
use crate::galaxy::components::*;
use crate::galaxy::events::EInfo;
use crate::galaxy::resources::network_handler::NetworkHandler;
use crate::galaxy::resources::{database_resource::DatabaseResource, path_to_entity::PathToEntityMap};
use crate::network::messages::incoming::NetIncomingMessage;

const INTERACTION_DISTANCE_METERS: f64 = 1000.0;

pub fn sys_manage_inventory_transfers(mut ships: Query<(&mut Ship, &PlayerController, &Transform)>, mut containers: Query<(&mut Container, &Transform)>, hangers: Query<&Hanger>, net: Res<NetworkHandler>, ptm: Res<PathToEntityMap>, db: Res<DatabaseResource>, mut ein: EventWriter<EInfo>) {
    for slot in net.view_incoming() {
        let player = slot.key();
        let msgs = slot.value();
        for msg in msgs.iter() {
            match msg {
                NetIncomingMessage::InvSpaceToSpace(_, _, _, _, _) => space_to_space(&mut ships, &mut containers, &ptm, &db, &mut ein, msg, player),
                NetIncomingMessage::InvHangerShipToHangerShip(_, _, _, _, _) => inv_ship_to_ship(&hangers,&ptm, &db, &mut ein, msg, player),
                NetIncomingMessage::InvHangerShipToStation(_, _, _, _, _) => inv_ship_to_inv(&hangers, &ptm, &db, &mut ein, msg, player),
                NetIncomingMessage::InvStationToShip(_, _, _, _, _) => inv_inv_to_ship(&hangers, &ptm, &db, &mut ein, msg, player),
                NetIncomingMessage::InvStationToStation(_, _, _, _, _) => inv_to_inv(&hangers, &ptm, &db, &mut ein, msg, player),
                _ => ()
            }
        }
    }
}

// Super overly verbose and full of checks here because we want to catch duplication and annihilation bugs really badly
fn space_to_space(ships: &mut Query<(&mut Ship, &PlayerController, &Transform)>, containers: &mut Query<(&mut Container, &Transform)>, ptm: &Res<PathToEntityMap>, db: &Res<DatabaseResource>, _ein: &mut EventWriter<EInfo>, msg: &NetIncomingMessage, player: &String) {
    if let NetIncomingMessage::InvSpaceToSpace(src_path, src_slot, count, dst_path, dst_slot) = msg {
        let src_ent = match ptm.get(src_path) {
            None => { eprintln!("Source inventory does not exist"); return; },
            Some(e) => e
        };
    
        let dst_ent = match ptm.get(dst_path) {
            None => { eprintln!("Dest inventory does not exist"); return; },
            Some(e) => e
        };

        if src_path.t != crate::shared::ObjectType::PlayerShip && dst_path.t != crate::shared::ObjectType::PlayerShip {
            eprintln!("Need to transfer to or from ship");
            return;
        }

        eprintln!("TODO: CHECK DISTANCE TO CONTAINER HERE");
        
        // get the position and stack from the source
        let res_stack = match src_path.t {
            crate::shared::ObjectType::Container | crate::shared::ObjectType::Wreck => containers.get_mut(src_ent).and_then(|(mut i, t)| Ok((t.pos, i.inv.remove_n_from_stack(*src_slot, *count)))),
            crate::shared::ObjectType::PlayerShip => ships.get_mut(src_ent).and_then(|(mut s, pc, t)| {
                if pc.player_name != *player { eprintln!("{} trying to control other player's inventory", player); return Err(bevy_ecs::query::QueryEntityError::NoSuchEntity(src_ent)); }
                Ok((t.pos, s.inventory.remove_n_from_stack(*src_slot, *count)))
            }),
            _ => { eprintln!("Object does not have an inventory"); return; }
        };

        //unwrap the position and stack
        let (src_pos, stack) = match res_stack {
            Err(_) => { eprintln!("Unable to get source component for s2s transfer"); return; },
            Ok((_, None)) => { eprintln!("Source stack could not be accessed"); return; },
            Ok((t, Some(s))) => (t, s)
        };

        //save this for if we need to return the items from the stack back to the original place
        let backup_stack = stack.clone();

        // try placing the stack in the dest inventory, anything that wasn't able to be placed will be returned in an Ok(Some(stack)), Ok(None) means everything was placed, Err(_) means that the entity couldn't be found
        let result = match dst_path.t {
            crate::shared::ObjectType::Container | crate::shared::ObjectType::Wreck => containers.get_mut(dst_ent).and_then(|(mut i, t)|{
                // check the distance
                let dist = t.pos.metric_distance(&src_pos);
                if dist > INTERACTION_DISTANCE_METERS { return Ok(Some(stack)); } //this will prompt the system to try and put back the stack it took
                match i.inv.add_stack(&db.db.item_table, stack, Some(*dst_slot)) {
                    None => Ok(None),
                    Some(s) => Ok(Some(s))
                }
            }),
            crate::shared::ObjectType::PlayerShip => ships.get_mut(dst_ent).and_then(|(mut i, pc, t)|{
                if pc.player_name != *player { eprintln!("{} trying to control other player's inventory", player); return Ok(Some(stack)); }
                let dist = t.pos.metric_distance(&src_pos);
                if dist > INTERACTION_DISTANCE_METERS { return Ok(Some(stack)); } //this will prompt the system to try and put back the stack it took
                match i.inventory.add_stack(&db.db.item_table, stack, Some(*dst_slot)) {
                    None => Ok(None),
                    Some(s) => Ok(Some(s))
                }
            }),
            _ => {
                eprintln!("Dst object does not exist");
                Err(bevy_ecs::query::QueryEntityError::NoSuchEntity(src_ent)) // using this as a dummy error
            }
        };

        // we will have generated either an Ok(x) where x is whatever we couldn't move, or an Err(_), in which case we put the whole stack back
        match result {
            Ok(None) => (), //everything in order
            Ok(Some(extra)) => { //need to put the extra back in the source
                let res = match src_path.t {
                    crate::shared::ObjectType::Container | crate::shared::ObjectType::Wreck => containers.get_mut(src_ent).and_then(|(mut i, _t)| Ok(i.inv.add_stack(&db.db.item_table, extra.clone(), Some(*src_slot)))),
                    crate::shared::ObjectType::PlayerShip => ships.get_mut(src_ent).and_then(|(mut s, _pc, _t)| {
                        //if pc.player_name != *player { eprintln!("{} trying to control other player's inventory", player); return Err(bevy_ecs::query::QueryEntityError::NoSuchEntity(src_ent)); }
                        //Ok((t.pos, s.inventory.remove_n_from_stack(*src_slot, *count)))
                        Ok(s.inventory.add_stack(&db.db.item_table, extra.clone(), Some(*src_slot)))
                    }),
                    _ => { eprintln!("Object does not have an inventory"); return; }
                };
                match res {
                    Ok(None) => (),
                    Ok(Some(dead)) => { eprintln!("WARNING: Could not return extras to source inventory, it has been annihilated: {:?}", dead); }
                    Err(_) => { eprintln!("WARNING: Could not return item to source inventory, it has been annihilated: {:?}", extra); }
                }
            }
            Err(_) => { // need to put the entire stack back
                let res = match src_path.t {
                    crate::shared::ObjectType::Container | crate::shared::ObjectType::Wreck => containers.get_mut(src_ent).and_then(|(mut i, _t)| Ok(i.inv.add_stack(&db.db.item_table, backup_stack.clone(), Some(*src_slot)))),
                    crate::shared::ObjectType::PlayerShip => ships.get_mut(src_ent).and_then(|(mut s, _pc, _t)| {
                        //if pc.player_name != *player { eprintln!("{} trying to control other player's inventory", player); return Err(bevy_ecs::query::QueryEntityError::NoSuchEntity(src_ent)); }
                        //Ok((t.pos, s.inventory.remove_n_from_stack(*src_slot, *count)))
                        Ok(s.inventory.add_stack(&db.db.item_table, backup_stack.clone(), Some(*src_slot)))
                    }),
                    _ => { eprintln!("Object does not have an inventory"); return; }
                };
                match res {
                    Ok(None) => (),
                    Ok(Some(dead)) => { eprintln!("WARNING: While handling dst not found, could not return extras to source inventory, it has been annihilated: {:?}", dead); }
                    Err(_) => { eprintln!("WARNING: While handling dst not found, could not return item to source inventory, it has been annihilated: {:?}", backup_stack); }
                }
            }
        };
    }
    else {
        eprintln!("Wrong message type sent to space_to_space inventory manager");
    }
}

fn inv_ship_to_ship(hanger: &Query<&Hanger>, ptm: &Res<PathToEntityMap>, db: &Res<DatabaseResource>, ein: &mut EventWriter<EInfo>, msg: &NetIncomingMessage, player: &String){
    if let NetIncomingMessage::InvHangerShipToHangerShip(src_h, src_slot, count, dst_h, dst_slot) = msg {
        let player_loc = match db.db.account_get_location(player) { Some(p) => p, None => { eprintln!("inv_hanger_to_ship: Player not in account table"); return; }};
        let ent = match ptm.get(&player_loc) { Some(e) => e, None => { eprintln!("inv_hanger_to_ship: Player path not found in ptm"); return; }};
        let h = match hanger.get(ent) { Ok(h) => h, Err(_) => { eprintln!("inv_hanger_to_ship: Hanger entity not found"); return; }};
        let id = h.hanger_uid.clone();
        let stack = match db.db.hanger_ship_take_items(player, id.clone(), *src_h, *src_slot, *count) { Some(s) => s, None => { eprintln!("inv_hanger_to_ship: No item stack in souce ship"); return; }};
        let extra =  db.db.hanger_ship_add_items(player, id.clone(), *dst_h, Some(*dst_slot), stack);
        ein.send(EInfo::UpdateInventoryHanger(player.clone(), id.clone()));
        let finish = match extra {
            Some(s) => db.db.hanger_ship_add_items(player, id, *src_h, Some(*src_slot), s),
            None => None,
        };
        match finish {
            None => (),
            Some(s) => { eprintln!("inv_hanger_to_ship: Could not return items to source ship, they have been annihilated: {:?}", s); }
        };
    }
    else {
        eprintln!("inv_ship_to_ship GOT WRONG MESSAGE TYPE");
    }
}

fn inv_ship_to_inv(hanger: &Query<&Hanger>, ptm: &Res<PathToEntityMap>, db: &Res<DatabaseResource>, ein: &mut EventWriter<EInfo>, msg: &NetIncomingMessage, player: &String){
    if let NetIncomingMessage::InvHangerShipToStation(hanger_slot, src_slot,count , dst_inv, dst_slot) = msg {
        let player_loc = match db.db.account_get_location(player) { Some(p) => p, None => { eprintln!("inv_ship_to_inv: Player not in account table"); return; }};
        let ent = match ptm.get(&player_loc) { Some(e) => e, None => { eprintln!("inv_ship_to_inv: Player path not found in ptm"); return; }};
        let h = match hanger.get(ent) { Ok(h) => h, Err(_) => { eprintln!("inv_ship_to_inv: Hanger entity not found"); return; }};
        let id = h.hanger_uid.clone();
        if id != *dst_inv { eprintln!("Mismatch between target inventory and found inventory for station, got {}, expected {} (TODO: SUPPORT MULTIPLE INVENTORIES)", *dst_inv, id); return; }; 
        let stack = match db.db.hanger_ship_take_items(player, id.clone(), *hanger_slot, *src_slot, *count) { Some(s) => s, None => { eprintln!("inv_ship_to_inv: No item stack in souce ship"); return; }};
        let extra =  db.db.inventory_insert_stack(player, id.clone(), stack, Some(*dst_slot));
        ein.send(EInfo::UpdateInventoryHanger(player.clone(), id.clone()));
        ein.send(EInfo::UpdateInventoryId(player.clone(), dst_inv.clone()));

        let finish = match extra {
            Some(s) => db.db.hanger_ship_add_items(player, id, *hanger_slot, Some(*src_slot), s),
            None => None,
        };
        match finish {
            None => (),
            Some(s) => { eprintln!("inv_ship_to_inv: Could not return items to source ship, they have been annihilated: {:?}", s); }
        };
    }
    else {
        eprintln!("inv_ship_to_inv GOT WRONG MESSAGE TYPE");
    }
}

fn inv_inv_to_ship(hanger: &Query<&Hanger>, ptm: &Res<PathToEntityMap>, db: &Res<DatabaseResource>, ein: &mut EventWriter<EInfo>, msg: &NetIncomingMessage, player: &String){
    if let NetIncomingMessage::InvStationToShip(src_inv_id, src_slot, count, hanger_slot, dst_slot) = msg {
        let player_loc = match db.db.account_get_location(player) { Some(p) => p, None => { eprintln!("inv_inv_to_ship: Player not in account table"); return; }};
        let ent = match ptm.get(&player_loc) { Some(e) => e, None => { eprintln!("inv_inv_to_ship: Player path not found in ptm"); return; }};
        let h = match hanger.get(ent) { Ok(h) => h, Err(_) => { eprintln!("inv_inv_to_ship: Hanger entity not found"); return; }};
        let id = h.hanger_uid.clone();
        if id != *src_inv_id { eprintln!("inv_inv_to_ship Mismatch between target inventory and found inventory for station (TODO: SUPPORT MULTIPLE INVENTORIES)"); return; }; 
        let stack = match db.db.inventory_remove_stack(player, id.clone(), *src_slot, Some(*count)) { Some(s) => s, None => { eprintln!("inv_inv_to_ship: No item stack in souce inv"); return; }};
        let extra = db.db.hanger_ship_add_items(player, id.clone(), *hanger_slot, Some(*dst_slot), stack);
        ein.send(EInfo::UpdateInventoryId(player.clone(), src_inv_id.clone()));
        ein.send(EInfo::UpdateInventoryHanger(player.clone(), id.clone()));
        let finish = match extra {
            Some(s) => db.db.inventory_insert_stack(player, src_inv_id.clone(), s, Some(*src_slot)),
            None => None,
        };
        match finish {
            None => (),
            Some(s) => { eprintln!("inv_inv_to_ship: Could not return items to source ship, they have been annihilated: {:?}", s); }
        };
    }
    else {
        eprintln!("inv_inv_to_ship GOT WRONG MESSAGE TYPE");
    }
}

fn inv_to_inv(hanger: &Query<&Hanger>, ptm: &Res<PathToEntityMap>, db: &Res<DatabaseResource>, ein: &mut EventWriter<EInfo>, msg: &NetIncomingMessage, player: &String){
    if let NetIncomingMessage::InvStationToStation(src_id, src_slot, count, dst_id, dst_slot) = msg {
        let player_loc = match db.db.account_get_location(player) { Some(p) => p, None => { eprintln!("inv_to_inv: Player not in account table"); return; }};
        let ent = match ptm.get(&player_loc) { Some(e) => e, None => { eprintln!("inv_to_inv: Player path not found in ptm"); return; }};
        let h = match hanger.get(ent) { Ok(h) => h, Err(_) => { eprintln!("inv_to_inv: Hanger entity not found"); return; }};
        let id = h.hanger_uid.clone();
        if id != *src_id || id != *dst_id { eprintln!("inv_to_inv Mismatch between target inventory and found inventory for station (TODO: SUPPORT MULTIPLE INVENTORIES)"); return; }; 
        let stack = match db.db.inventory_remove_stack(player, src_id.clone(), *src_slot, Some(*count)) { Some(s) => s, None => { eprintln!("inv_to_inv: No item stack in souce inv"); return; }};
        let extra = db.db.inventory_insert_stack(player, dst_id.clone(), stack, Some(*dst_slot));
        ein.send(EInfo::UpdateInventoryId(player.clone(), id));
        let finish = match extra {
            Some(s) => db.db.inventory_insert_stack(player, src_id.clone(), s, Some(*src_slot)),
            None => None,
        };
        match finish {
            None => (),
            Some(s) => { eprintln!("inv_to_inv: Could not return items to source ship, they have been annihilated: {:?}", s); }
        };
    }
}