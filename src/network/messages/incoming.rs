use serde::{Serialize, Deserialize};
use crate::{shared::ObjPath, inventory::{InvSlot, InvId, ItemId}, db::HangerSlot, galaxy::components::HngId};

// player will be known due to map location

#[derive(Debug, Deserialize)]
pub enum NetIncomingMessage {
    /* LOGIN */
    Login(String, String), //player name, access token
    Disconnect, //player name

    /* Motion */
    WarpTo(ObjPath, ObjPath, f64), //ship, dst, dist
    Approach(ObjPath, ObjPath), //ship, dst
    MNav(ObjPath, f64, f64, f64, f64), //net rotation and thrust time x,y,z,t (integrate [-1, 1] (or [0,1] for thrust) axis input over the reporting period)
    
    /* Docking */
    Undock(ObjPath), //station path
    Dock(ObjPath, ObjPath), //ship, station path
    
    /* Jumping */
    Jump(ObjPath, ObjPath), //ship, gate

    /* Hanger */
    SetActiveShip(HangerSlot), // hanger slot
    /* TODO: request list of all hangers */
    HangerRequestShips(HngId), // hanger id

    /* Inventory */
    InvSpaceToSpace(ObjPath, InvSlot, u32, ObjPath, InvSlot), //source object, source slot, source count, dst_container, dst_slot
    InvHangerShipToStation(HangerSlot, InvSlot, u32, InvId, InvSlot), //hanger slot, src_slot, count, dst_ivn, dst_slot
    InvHangerShipToHangerShip(HangerSlot, InvSlot, u32, HangerSlot, InvSlot), //source hanger slot, source slot, count, dest hanger slot, dst slot
    InvStationToShip(InvId, InvSlot, u32, HangerSlot, InvSlot), //source inv, source slot, count, dst ship, dst slot
    InvStationToStation(InvId, InvSlot, u32, InvId, InvSlot), //source inv, source slot, count, dst inv, dst slot
    InvRequestInventoryList, //requesting list of all paths/ids with inventories
    InvRequestInventory(InvId), //requesting dump of specific station inventory
    InvRequestShip(ObjPath), // requesting ship onboard inventory
    InvRequestGameObject(ObjPath), // TODO: figure out rules for this

    /* Market */
    PlaceBuyOrder(ItemId, InvId, u32, i64), //item, location, count, price PER ITEM
    FulfillBuyOrder(ItemId, u64, InvId, InvSlot, u32), //item, order id, inventory id, inventory slot, count
    CancelBuyOrder(ItemId, u64), //item, order id
    PlaceSellOrder(InvId, InvSlot, u32, i64), //inventory, slot, count, cost per
    FulfillSellOrder(ItemId, u64, u32), //item, order id, count
    CancelSellOrder(ItemId, u64), //item, order id
    GetStore(ItemId), // item id
}