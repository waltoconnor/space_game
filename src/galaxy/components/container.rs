use bevy_ecs::prelude::*;
use serde::{Serialize, Deserialize};
use crate::inventory::{Inventory, Stack, ItemTable};

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct Container {
    pub inv: Inventory,
    pub access_dist: f64
}

impl Container {
    pub fn new(capacity: u32, access_dist: f64) -> Self {
        Container { inv: Inventory::new(None, Some(capacity)), access_dist }
    }

    pub fn new_with_stacks(capacity: u32, stacks: Vec<Stack>, item_table: &ItemTable, access_dist: f64) -> Self {
        let mut inv = Inventory::new(None, Some(capacity));
        for stack in stacks {
            inv.add_stack(item_table, stack, None);
        }

        Container { inv, access_dist }
    }
}