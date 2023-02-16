use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::galaxy::components::Ship;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerHanger {
    pub inventory: HashMap<u32, Ship>,
    pub active: Option<Ship>
}

impl PlayerHanger {
    pub fn new() -> Self {
        PlayerHanger { inventory: HashMap::new(), active: None }
    }

    fn slot_free(&self, slot: u32) -> bool {
        !self.inventory.contains_key(&slot)
    }

    fn get_open_slot(&self) -> Option<u32> {
        for i in 0..self.inventory.len() + 1 {
            if !self.inventory.contains_key(&(i as u32)) {
                return Some(i as u32);
            }
        }
        return None;
    }

    pub fn insert_ship(&mut self, ship: Ship) {
        let open_slot = self.get_open_slot().expect("COULD NOT FIND OPEN INVENTORY SLOT");
        self.inventory.insert(open_slot, ship);
    }

    pub fn remove_ship(&mut self, slot: u32) -> Option<Ship> {
        self.inventory.remove(&slot).and_then(|ship| Some(ship))
    }

    fn get_last_ship_slot(&self) -> Option<u32> {
        if self.inventory.len() == 0 {
            return None;
        }
        else {
            self.inventory.iter().map(|(key, _)| *key).last()
        }
    }

    pub fn set_active_from_slot(&mut self, slot: u32) {
        if self.slot_free(slot) {
            eprintln!("Trying to activate ship from empty slot");
            return;
        }

        if self.active.is_none() {
            self.active = self.remove_ship(slot);    
        }
        else {
            let mut tmp = None;
            std::mem::swap(&mut tmp, &mut self.active);
            //tmp now has the formerly active ship
            self.active = self.remove_ship(slot); //slot is now empty for sure
            match tmp {
                Some(s) => self.inventory.insert(slot, s),
                None => None
            };
        }
    }

    pub fn set_active_from_dock(&mut self, ship: Ship) {
        let old = std::mem::replace(&mut self.active, Some(ship));
        match old {
            Some(o) => self.insert_ship(o),
            None => ()
        }
    }

    pub fn remove_active_ship_undock(&mut self) -> Option<Ship> {
        std::mem::replace(&mut self.active, None)
    }

    pub fn is_empty(&self) -> bool {
        self.active.is_none() && self.inventory.len() == 0
    }
}