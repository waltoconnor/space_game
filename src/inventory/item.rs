use std::collections::{HashSet, HashMap};

use serde::{Serialize, Deserialize};

pub type ItemTable = HashMap<String, Item>;
pub type ItemId = String;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum ItemTag {
    Module,
    Ammo,
    TradeGood,
    Ore,
    Component,
    Ship,
    Structure
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Mapping {
    Module(String), //module id
    Ship(String), //ship class
    Ammo(String), //ammo class
    Structure(String), //structure class
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: ItemId,
    pub tags: HashSet<ItemTag>,
    pub mapping: Mapping,
    pub size_vunits: u32, //1 vunit = 0.01m3 
}