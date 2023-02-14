use serde::{Serialize, Deserialize};

use crate::shared::ObjPath;

// player will be known due to map location

#[derive(Debug, Deserialize)]
pub enum NetIncomingMessage {

    /* Motion */
    WarpTo(ObjPath, ObjPath, f64), //ship, dst, dist
    Approach(ObjPath, ObjPath), //ship, dst
    
    /* Docking */
    Undock(ObjPath), //station path
    Dock(ObjPath, ObjPath), //ship, station path
    
    /* Jumping */
    Jump(ObjPath, ObjPath), //ship, gate
}