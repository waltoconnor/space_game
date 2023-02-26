use std::collections::{HashSet, HashMap};

use serde::{Serialize, Deserialize};

use crate::inventory::{ItemTag, Mapping, ItemTable, ItemId, Item};


#[derive(Serialize, Deserialize, Debug)]
pub struct LItem {
    tags: HashSet<ItemTag>,
    mapping: Mapping,
    size: u32,
    tech_level: u8
}

pub fn load_item(items: HashMap<ItemId, LItem>) -> ItemTable {
    items.into_iter().map(|(k, v)| (k.clone(), Item{ id: k, tags: v.tags, mapping: v.mapping, size_vunits: v.size, tech_level: v.tech_level })).collect()
}