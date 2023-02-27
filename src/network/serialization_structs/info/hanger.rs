use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use crate::db::PlayerHanger;
use crate::galaxy::components::HngId;
use crate::{galaxy::components::{Stats, Ship}, inventory::Inventory};


#[derive(Serialize, Deserialize, Debug)]
pub struct SHanger {
    pub active: Option<u32>,
    pub hanger_contents: HashMap<u32, SShip>,
    pub id: HngId
}

impl SHanger {
    pub fn from_hanger(h: &PlayerHanger, id: HngId) -> Self {
        SHanger { active: h.active, hanger_contents: h.inventory.iter().map(|(k,v)| (*k, SShip::from_ship(v))).collect(), id }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SShip {
    pub class: String,
    pub name: String,
    pub stats: Stats,
    pub inv: Inventory
}

impl SShip {
    pub fn from_ship(s: &Ship) -> Self {
        SShip { class: s.ship_class.clone(), name: s.ship_name.clone(), stats: s.stats.clone(), inv: s.inventory.clone() }
    }
}