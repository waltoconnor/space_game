use std::collections::HashSet;

use bevy_ecs::prelude::*;

use crate::shared::ObjPath;

#[derive(Component, Debug)]
pub struct Sensor {
    pub lockable_objs: HashSet<ObjPath>,
    pub visible_objs: HashSet<ObjPath>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectVisibility {
    Lockable, // you can see and lock this
    Visible, // you can see this but not lock it
    Static, // you can always see this
    NotVisible, // you will not see this
}

impl Sensor {
    pub fn new() -> Self {
        Sensor { lockable_objs: HashSet::new(), visible_objs: HashSet::new() }
    }
}