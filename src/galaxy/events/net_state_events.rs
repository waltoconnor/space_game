use bevy_ecs::prelude::*;

use crate::{galaxy::components::ObjectVisibility, shared::ObjPath};

/// Client state event (about things in space)
pub enum EState {
    // Statics(String, String), // player, system
    // OtherShip(String, ObjPath, ObjectVisibility), //player, ship, visibility
    // OwnShip(String, ObjPath), //player, own ship path
}

