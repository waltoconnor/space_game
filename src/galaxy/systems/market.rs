use std::collections::HashMap;

use bevy_ecs::prelude::*;

use crate::db::{ItemStore};
use crate::galaxy::events::EInfo;
use crate::{galaxy::resources::{database_resource::DatabaseResource, network_handler::NetworkHandler}, inventory::ItemId};
use crate::network::messages::incoming::NetIncomingMessage;

pub fn sys_process_market(db: Res<DatabaseResource>, net: Res<NetworkHandler>, mut ein: EventWriter<EInfo>) {
    let mut local_cache = HashMap::new();
    for set in net.view_incoming() {
        let player = set.key();
        let msgs = set.value();
        for msg in msgs {
            match msg {
                NetIncomingMessage::GetStore(item_id) => {
                    ein.send(EInfo::ItemStore(player.clone(), item_id.clone()));
                },
                NetIncomingMessage::CancelBuyOrder(item_id, order_id) => {
                    ensure_item_store_in_cache(&db, &mut local_cache, item_id);
                    let store = local_cache.get_mut(item_id).unwrap(); //guarenteed by above
                    match store.cancel_buy_order(player, *order_id) {
                        Ok(d) => {
                            db.db.bank_apply_transaction(player, d, format!("Cancelled buy order for {}", store.item));
                            db.db.market_remove_buy_order_from_player(player, *order_id);
                            ein.send(EInfo::UpdateBankAccount(player.clone()))
                        },
                        Err(msg) => {
                            ein.send(EInfo::Error(player.clone(), msg));
                        }
                    }
                },
                NetIncomingMessage::CancelSellOrder(item_id, order_id) => {
                    ensure_item_store_in_cache(&db, &mut local_cache, item_id);
                    let store = local_cache.get_mut(item_id).unwrap();
                    let order = match store.get_sell_order(*order_id) {
                        Some(o) => o,
                        None => { ein.send(EInfo::Error(player.clone(), String::from("Requested sell order no longer exists"))); continue; }
                    };
                    let inv_id = order.location.clone();
                    ein.send(EInfo::UpdateInventoryId(player.clone(), order.location.clone()));
                    let stack = store.cancel_sell_order(player, *order_id).expect("Could not cancel sell order that was just validated");
                    db.db.inventory_insert_stack_free_slot_ignore_capacity(player, inv_id, stack);
                    db.db.market_remove_sell_order_from_player(player, *order_id);
                },
                NetIncomingMessage::FulfillBuyOrder(item_id, order_id, inv_id, inv_slot, count) => {
                    // give money to selling player, take item from selling player, give item to buying player, remove money from buying escrow
                    ensure_item_store_in_cache(&db, &mut local_cache, item_id);
                    let store = local_cache.get_mut(item_id).unwrap();
                    let stack = match db.db.inventory_remove_stack(player, inv_id.clone(), *inv_slot, Some(*count)){
                        Some(s) => s,
                        None => { ein.send(EInfo::Error(player.clone(), String::from("Source inventory slot does not have items"))); continue; }
                    };
                    
                    match store.fulfill_buy_order(*order_id, stack.clone(), inv_id.clone(), player.clone()){
                        Ok(t) => {
                            db.db.bank_apply_transaction(player, t.cost, format!("Sold {}x{} to {}", t.purchased_stack.id, *count, t.purchasing_player)).expect("Could not apply bank transaction");
                            db.db.inventory_insert_stack_free_slot_ignore_capacity(&t.purchasing_player, t.location.clone(), stack);
                            if t.order_complete {
                                if let Some(_res) = store.clear_buy_order(*order_id){
                                    db.db.market_remove_buy_order_from_player(&t.purchasing_player, *order_id);
                                }
                            }
                            ein.send(EInfo::UpdateBankAccount(player.clone()));
                            ein.send(EInfo::UpdateInventoryId(player.clone(), inv_id.clone()));
                            ein.send(EInfo::UpdateInventoryId(t.purchasing_player.clone(), t.location.clone()));

                        },
                        Err(e) => { 
                            db.db.inventory_insert_stack_free_slot_ignore_capacity(player, inv_id.clone(), stack); //if there was a problem, return the stack
                            ein.send(EInfo::Error(player.clone(), e)); 
                            continue; 
                        }
                    };
                },
                NetIncomingMessage::FulfillSellOrder(item_id, order_id, count) => {
                    ensure_item_store_in_cache(&db, &mut local_cache, item_id);
                    let store = local_cache.get_mut(item_id).unwrap();
                    let order = match store.get_sell_order(*order_id) {
                        None => { ein.send(EInfo::Error(player.clone(), String::from("Requested sell order no longer exists"))); continue; },
                        Some(o) => o
                    };

                    let money = db.db.bank_get_value(player).expect("Player has no bank account");
                    if money < order.cost_per_item * (*count as i64) {
                        ein.send(EInfo::Error(player.clone(), String::from("Insufficent funds"))); continue;
                    }

                    match store.fulfill_sell_order(*order_id, *count, order.location.clone(), player.clone()) {
                        Ok(t) => {
                            db.db.inventory_insert_stack_free_slot_ignore_capacity(&t.purchasing_player, t.location.clone(), t.purchased_stack);
                            db.db.bank_apply_transaction(&t.purchasing_player, -t.cost, format!("Purchased {}x{} from {}", item_id, *count, t.selling_player));
                            db.db.bank_apply_transaction(&t.selling_player, t.cost, format!("Sold {}x{} to {}", item_id, *count, t.purchasing_player));
                            if t.order_complete {
                                if let Some(_res) = store.clear_sell_order(*order_id){
                                    db.db.market_remove_sell_order_from_player(&t.selling_player, *order_id);
                                }
                            }
                            ein.send(EInfo::UpdateBankAccount(t.purchasing_player.clone()));
                            ein.send(EInfo::UpdateBankAccount(t.selling_player.clone()));
                            ein.send(EInfo::UpdateInventoryId(t.purchasing_player.clone(), t.location.clone()));
                        },
                        Err(e) => {
                            ein.send(EInfo::Error(player.clone(), e));
                            continue;
                        }
                    };
                },
                NetIncomingMessage::PlaceBuyOrder(item_id, location, count, price_per_item) => {
                    if !db.db.market_can_place_new_order(player) {
                        ein.send(EInfo::Error(player.clone(), String::from("No remaining order slots")));
                        continue;
                    }

                    ensure_item_store_in_cache(&db, &mut local_cache, item_id);
                    let store = local_cache.get_mut(item_id).unwrap();

                    let money = db.db.bank_get_value(player).expect("Could not get player bank value");
                    if money < price_per_item * (*count as i64) {
                        ein.send(EInfo::Error(player.clone(), String::from("Insufficent funds"))); continue;
                    }

                    match store.add_buy_order(player, item_id.clone(), *count, *price_per_item, location.clone()) {
                        Ok(order_id) => {
                            db.db.market_add_buy_order_to_player(player, item_id, order_id);
                            db.db.bank_apply_transaction(player, -money, format!("Placed buy order for {}x{}", item_id, *count));
                            ein.send(EInfo::UpdateBankAccount(player.clone()));
                        },
                        Err(e) => {
                            ein.send(EInfo::Error(player.clone(), e));
                            continue;
                        }
                    };
                },
                NetIncomingMessage::PlaceSellOrder(inventory_id, item_slot, count, price_per_item) => {
                    if !db.db.market_can_place_new_order(player) {
                        ein.send(EInfo::Error(player.clone(), String::from("No remaining order slots")));
                        continue;
                    }

                    let item_stack = match db.db.inventory_remove_stack(player, inventory_id.clone(), *item_slot, Some(*count)) {
                        None => {
                            ein.send(EInfo::Error(player.clone(), String::from("Invalid item stack attempting to be sold (not enough items in stack?)")));
                            continue;
                        },
                        Some(s) => s
                    };
                    ensure_item_store_in_cache(&db, &mut local_cache, &item_stack.id);
                    let id = item_stack.id.clone();
                    let store = local_cache.get_mut(&item_stack.id).unwrap();
                    match store.add_sell_order(player, item_stack, *price_per_item, inventory_id.clone()) {
                        Ok(order_id) => {
                            db.db.market_add_sell_order_to_player(player, &id, order_id);
                            ein.send(EInfo::UpdateInventoryId(player.clone(), inventory_id.clone()));
                        },
                        Err(e) => {
                            ein.send(EInfo::Error(player.clone(), e));
                            continue;
                        }
                    };
                },
                _ => ()
            }
        }
    }
    flush_cache(&db, &local_cache);
}

fn ensure_item_store_in_cache(db: &DatabaseResource, cache: &mut HashMap<ItemId, ItemStore>, item_id: &ItemId) {
    if !cache.contains_key(item_id) {
        let market = db.db.market_load_item_store(item_id.clone()).expect("Could not load item store in to cache");
        cache.insert(item_id.clone(), market);
    }
}

fn flush_cache(db: &DatabaseResource, cache: &HashMap<ItemId, ItemStore>) {
    for (_name, item_store) in cache.into_iter() {
        db.db.market_save_item_store(item_store);
    }
}