use serde::{Serialize, Deserialize};
use crate::{shared::ObjPath, inventory::{InvSlot, InvId}, db::HangerSlot};

// player will be known due to map location

#[derive(Debug, Deserialize)]
pub enum NetIncomingMessage {
    /* LOGIN */
    Login(String, String), //player name, access token
    Disconnect, //player name

    /* Motion */
    WarpTo(ObjPath, ObjPath, f64), //ship, dst, dist
    Approach(ObjPath, ObjPath), //ship, dst
    
    /* Docking */
    Undock(ObjPath), //station path
    Dock(ObjPath, ObjPath), //ship, station path
    
    /* Jumping */
    Jump(ObjPath, ObjPath), //ship, gate

    /* Inventory */
    InvSpaceToSpace(ObjPath, InvSlot, u32, ObjPath, InvSlot), //source object, source slot, source count, dst_container, dst_slot
    InvHangerShipToStation(HangerSlot, InvSlot, u32, InvId, InvSlot), //hanger slot, src_slot, count, dst_ivn, dst_slot
    InvHangerShipToHangerShip(HangerSlot, InvSlot, u32, HangerSlot, InvSlot), //source hanger slot, source slot, count, dest hanger slot, dst slot
    InvStationToShip(InvId, InvSlot, u32, HangerSlot, InvSlot), //source inv, source slot, count, dst ship, dst slot
    InvStationToStation(InvId, InvSlot, u32, InvId, InvSlot), //source inv, source slot, count, dst inv, dst slot
}