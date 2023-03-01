use serde::{Serialize, Deserialize};

use crate::{shared::ObjPath, inventory::{Inventory, InvId}, db::ItemStore, galaxy::galaxy_map::GalaxyMap};

use self::hanger::SHanger;

pub mod hanger;

#[derive(Serialize, Deserialize)]
pub enum NetOutInfo {
    Location(ObjPath),
    Hanger(SHanger),
    Inventory(Inventory, InvId), //inv, inv_id
    Bank(i64), //value
    Store(ItemStore), //store
    GalaxyMap(GalaxyMap),
    InvList(Vec<(ObjPath, InvId)>), // station paths, inv ids
    InventoryGameObject(Inventory, ObjPath), //inv, path
}