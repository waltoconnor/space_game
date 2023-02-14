use std::collections::HashMap;

use bevy_ecs::prelude::*;
use nalgebra::Vector3;

use super::{ship::Ship, transform::Transform};

#[derive(Component)]
pub struct Hanger {
    //pub player_hangers: HashMap<String, PlayerHanger>,
    pub undock_offset: Vector3<f64>,
    pub hanger_uid: u64,
    pub docking_range_m: f64,
}

pub struct PlayerHanger {
    inventory: HashMap<u32, Ship>
}

impl PlayerHanger {
    pub fn new() -> Self {
        PlayerHanger { inventory: HashMap::new() }
    }

    pub fn slot_free(&self, slot: u32) -> bool {
        !self.inventory.contains_key(&slot)
    }

    pub fn get_open_slot(&self) -> Option<u32> {
        for i in 0..self.inventory.len() + 1 {
            if !self.inventory.contains_key(&(i as u32)) {
                return Some(i as u32);
            }
        }
        return None;
    }

    pub fn insert_ship(&mut self, ship: Ship, slot: u32) -> Option<Ship>{
        let v = self.inventory.insert(slot, ship);
        if v.is_some() {
            eprintln!("ERROR: MOVED SHIP IN TO OCCUPIED SLOT");
        }
        v
    }

    pub fn remove_ship(&mut self, slot: u32) -> Option<Ship> {
        self.inventory.remove(&slot)
    }

    pub fn get_last_ship_slot(&self) -> Option<u32> {
        if self.inventory.len() == 0 {
            return None;
        }
        else {
            self.inventory.iter().map(|(i, _)| *i).last()
        }
    }
}

