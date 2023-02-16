use bevy_ecs::prelude::*;
use serde::{Serialize, Deserialize};
use crate::inventory::{Inventory, Stack, ItemTable};

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct Container {
    pub inv: Inventory
}

impl Container {
    pub fn new(capacity: u32) -> Self {
        Container { inv: Inventory::new(None, Some(capacity)) }
    }

    pub fn new_with_stacks(capacity: u32, stacks: Vec<Stack>, item_table: &ItemTable) -> Self {
        let mut inv = Inventory::new(None, Some(capacity));
        for stack in stacks {
            inv.add_stack(item_table, stack, None);
        }

        Container { inv }
    }
}