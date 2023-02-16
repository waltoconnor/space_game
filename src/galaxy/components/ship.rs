

use bevy_ecs::prelude::*;
use serde::{Serialize, Deserialize};

use crate::inventory::Inventory;

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Ship {
    pub ship_name: String,
    pub ship_class: String, // TODO: MAKE SHIP CLASS ITS OWN TYPE
    pub stats: Stats,
    pub inventory: Inventory,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stats {
    pub warp_speed_ms: f64,
    pub thrust_n: f64,
    pub ang_vel_rads: f64,
    pub mass_kg: f64
}