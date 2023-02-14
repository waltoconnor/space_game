use bevy_ecs::prelude::*;

use crate::shared::{ObjPath, ObjectType};

#[derive(Component)]
pub struct GameObject {
    pub path: ObjPath
}

impl GameObject {
    pub fn new(system: &String, t: ObjectType, name: &String) -> Self {
        GameObject { path: ObjPath::new(system, t, name) }
    }
}