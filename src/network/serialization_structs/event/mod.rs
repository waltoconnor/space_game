use serde::{Deserialize, Serialize};

use crate::shared::ObjPath;


#[derive(Deserialize, Serialize)]
pub enum NetOutEvent {
    Dock(ObjPath), //station
    Undock(ObjPath), //ship
}