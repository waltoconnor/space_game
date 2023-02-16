use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::galaxy::components::Ship;

pub type HangerSlot = u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerHanger {
    pub inventory: HashMap<HangerSlot, Ship>,
    pub active: Option<u32>
}

impl PlayerHanger {
    pub fn new() -> Self {
        PlayerHanger { inventory: HashMap::new(), active: None }
    }

    fn slot_free(&self, slot: HangerSlot) -> bool {
        !self.inventory.contains_key(&slot)
    }

    fn get_open_slot(&self) -> Option<HangerSlot> {
        for i in 0..self.inventory.len() + 1 {
            if !self.inventory.contains_key(&(i as u32)) {
                return Some(i as u32);
            }
        }
        return None;
    }

    /// returns slot it was inserted in to
    pub fn insert_ship(&mut self, ship: Ship) -> u32 {
        let open_slot = self.get_open_slot().expect("COULD NOT FIND OPEN INVENTORY SLOT");
        self.inventory.insert(open_slot, ship);
        open_slot
    }

    pub fn remove_ship(&mut self, slot: HangerSlot) -> Option<Ship> {
        self.inventory.remove(&slot).and_then(|ship| Some(ship))
    }

    fn get_last_ship_slot(&self) -> Option<HangerSlot> {
        if self.inventory.len() == 0 {
            return None;
        }
        else {
            self.inventory.iter().map(|(key, _)| *key).last()
        }
    }

    pub fn set_active_from_slot(&mut self, slot: u32) {
        if self.inventory.contains_key(&slot) {
            self.active = Some(slot);
        }
    }

    pub fn set_active_from_dock(&mut self, ship: Ship) {
        let slot = self.insert_ship(ship);
        self.active = Some(slot);
    }

    pub fn remove_active_ship_undock(&mut self) -> Option<Ship> {
        self.active.and_then(|a| self.inventory.remove(&a))
    }

    pub fn is_empty(&self) -> bool {
        self.inventory.len() == 0
    }
}