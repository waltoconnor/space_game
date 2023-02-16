use std::collections::HashMap;

use bevy_ecs::{system::Resource, prelude::Entity};

use crate::shared::ObjPath;

#[derive(Resource, Debug)]
pub struct PathToEntityMap {
    table: HashMap<ObjPath, Entity>,
    reverse: HashMap<Entity, ObjPath>
}

impl PathToEntityMap {
    pub fn new() -> Self {
        PathToEntityMap { table: HashMap::new(), reverse: HashMap::new() }
    }

    pub fn get(&self, path: &ObjPath) -> Option<Entity> {
        self.table.get(path).and_then(|id| Some(*id))
    }

    pub fn update(&mut self, path: &ObjPath, entity: Entity) {
        // println!("PT Updating {:?} ({:?})", entity, path);
        self.table.insert(path.clone(), entity);
        self.reverse.insert(entity, path.clone());
    }

    pub fn remove(&mut self, path: &ObjPath) {
        println!("PT Removing {:?}", path);
        let ent = self.table.remove(path);
        match ent {
            Some(e) => { self.reverse.remove(&e); },
            None => ()
        };
    }

    pub fn get_path_from_entity(&self, entity: Entity) -> Option<ObjPath> {
        self.reverse.get(&entity).and_then(|p| Some(p.clone()))
    }

    pub fn remove_by_ent(&mut self, entity: Entity) {
        let to_remove = self.reverse.remove(&entity);
        match to_remove {
            Some(p) => { self.table.remove(&p); },
            None => ()
        };
    }
}