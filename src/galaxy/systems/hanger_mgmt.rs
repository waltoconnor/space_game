use bevy_ecs::prelude::*;
use crate::galaxy::components::*;
use crate::galaxy::events::EInfo;
use crate::galaxy::resources::network_handler::NetworkHandler;
use crate::galaxy::resources::{database_resource::DatabaseResource, path_to_entity::PathToEntityMap};
use crate::network::messages::incoming::NetIncomingMessage;

pub fn hanger_mgmt(hangers: Query<&Hanger>, ptm: Res<PathToEntityMap>, net: Res<NetworkHandler>, db: Res<DatabaseResource>, mut ein: EventWriter<EInfo>) {
    for slot in net.view_incoming() {
        let player = slot.key();
        let msgs = slot.value();
        for msg in msgs.iter() {
            match msg {
                NetIncomingMessage::SetActiveShip(hanger_slot) => {
                    if let Some(cur_loc) = db.db.account_get_location(player) {
                        if let Some(hanger_ent) = ptm.get(&cur_loc) {
                            if let Ok(hanger) = hangers.get(hanger_ent) {
                                db.db.hanger_set_active_ship_slot(player, hanger.hanger_uid.clone(), *hanger_slot);
                                ein.send(EInfo::UpdateInventoryHanger(player.clone(), hanger.hanger_uid.clone()));
                            }
                        }
                        else {
                            eprintln!("Player is requesting to change active ship in a hanger that doesn't exist (or the entity they are in is not a hanger)");
                        }
                    }
                    else {
                        eprintln!("No active location found for connected account");
                    }
                },
                _ => ()
            }
        }
    }
}