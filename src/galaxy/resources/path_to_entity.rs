use std::collections::HashMap;

use bevy_ecs::{system::Resource, prelude::Entity};

use crate::shared::ObjPath;

#[derive(Resource, Debug)]
pub struct PathToEntityMap {
    table: HashMap<ObjPath, Entity>
}

impl PathToEntityMap {
    pub fn new() -> Self {
        PathToEntityMap { table: HashMap::new() }
    }

    pub fn get(&self, path: &ObjPath) -> Option<Entity> {
        self.table.get(path).and_then(|id| Some(*id))
    }

    pub fn update(&mut self, path: &ObjPath, entity: Entity) {
        // println!("PT Updating {:?} ({:?})", entity, path);
        self.table.insert(path.clone(), entity);
    }

    pub fn remove(&mut self, path: &ObjPath) {
        println!("PT Removing {:?}", path);
        self.table.remove(path);
    }
}