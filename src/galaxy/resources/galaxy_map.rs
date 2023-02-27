use bevy_ecs::prelude::*;

use crate::galaxy::galaxy_map::GalaxyMap;

#[derive(Resource)]
pub struct GalaxyMapRes {
    pub gmap: GalaxyMap
}