use serde::{Serialize, Deserialize};

use crate::{shared::ObjPath, inventory::{Inventory, InvId}, db::ItemStore};

use self::hanger::SHanger;

pub mod hanger;

#[derive(Serialize, Deserialize)]
pub enum NetOutInfo {
    Location(ObjPath),
    Hanger(SHanger),
    Inventory(Inventory, InvId), //inv, inv_id
    Bank(i64), //value
    Store(ItemStore), //store
}