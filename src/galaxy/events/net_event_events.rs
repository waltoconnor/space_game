use bevy_ecs::prelude::*;

use crate::shared::ObjPath;

/// "Client event" event, about explosions and other things like that 
#[derive(Debug)]
pub enum EEvent {
    Undock(String, ObjPath), //player, ship
    Dock(String, ObjPath), //player, station
    Jump(String, String), //player, dst system
}