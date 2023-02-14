use bevy_ecs::prelude::*;

use crate::shared::ObjPath;

#[derive(Component)]
pub struct Gate {
    pub jump_range: f64,
    pub dst_system: String,
    pub dst_gate: ObjPath
}
