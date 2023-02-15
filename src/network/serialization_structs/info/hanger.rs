use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::galaxy::components::Stats;


#[derive(Serialize, Deserialize, Debug)]
pub struct SHanger {
    pub active: Option<SShip>,
    pub hanger_contents: HashMap<u32, SShip>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SShip {
    pub class: String,
    pub name: String,
    pub stats: Stats
}