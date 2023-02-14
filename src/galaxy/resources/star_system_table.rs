use std::collections::{HashSet, HashMap};

use bevy_ecs::prelude::*;

#[derive(Resource, Debug)]
pub struct SystemMapTable {
    // map from systems to hashset of entity ids
    pub sys_table: HashMap<String, HashSet<Entity>>,
    
    // map from entities to their current system
    pub entity_table: HashMap<Entity, String>
}

impl<'a> SystemMapTable {
    pub fn new() -> Self {
        SystemMapTable { 
            sys_table: HashMap::new(),
            entity_table: HashMap::new()
        }
    }

    pub fn update_changed_entity(&mut self, system: &String, entity: Entity) {
        // println!("SM Updating {:?} ({})", entity, system);
        // inserts element in to sys_table, creating new entry if required
        match self.sys_table.get_mut(system) {
            None => {
                let mut new_set = HashSet::new();
                new_set.insert(entity);
                self.sys_table.insert(system.clone(), new_set);
            },
            Some(hs) => { hs.insert(entity); }
        }

        match self.entity_table.get_mut(&entity) {
            Some(cur_sys_ptr) => {
                // if the entity was already present, remove its old entry from the system table
                self.sys_table.get_mut(cur_sys_ptr).expect("Old system does not exist").remove(&entity);
                // then update the referenced system to the new system
                *cur_sys_ptr = system.clone();
            },
            None => {
                // this is a new entity, so just add it to the table
                self.entity_table.insert(entity, system.clone());
            }
        }
    }

    pub fn update_remove_entity(&mut self, entity: Entity) {
        println!("SM Removing {:?}", entity);
        let sys = match self.entity_table.remove(&entity) {
            Some(sys) => sys,
            None => {
                eprintln!("Entity queued for removal does not appear in the entity table");
                return;
            }
        };

        match self.sys_table.get_mut(&sys) {
            Some(hs) => { hs.remove(&entity); },
            None => {
                eprintln!("Entity removed from entity_table does not appear in expected slot in sys table");
                return;
            }
        }
    }

    pub fn get_entities_in_system(&'a self, sys: &String) -> Option<&'a HashSet<Entity>> {
        self.sys_table.get(sys)
    }

    pub fn get_system_of_entity(&'a self, entity: Entity) -> Option<&'a String> {
        self.entity_table.get(&entity)
    }

    
}