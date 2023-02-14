use std::collections::HashSet;

use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Station {
    pub current_players: HashSet<String>
}