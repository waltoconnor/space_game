use serde::{Serialize, Deserialize};

use crate::galaxy::components::{Ship, Navigation, Transform, GameObject};

// DON'T SERIALIZE THE SENSOR STATE, WE NEED IT TO BE RESET WHEN THE SHIP LOADS BACK IN
#[derive(Serialize, Deserialize, Debug)]
pub struct ShipInSpace {
    pub ship: Ship,
    pub player_name: String,
    pub navigation: Navigation,
    pub transform: Transform,
    pub game_object: GameObject
}