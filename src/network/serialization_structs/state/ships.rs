use serde::{Serialize, Deserialize};

use crate::{shared::ObjPath, galaxy::components::{Stats, Transform, Navigation, Action, NavTarget, WarpState, ObjectVisibility}};

#[derive(Serialize, Deserialize)]
pub struct SPlayerShip_OTHER {
    pub path: ObjPath,
    pub ship_class: String,
    pub ship_name: String,
    pub transform: Transform,
    pub player_name: String,
    pub vis: ObjectVisibility
}

#[derive(Serialize, Deserialize)]
pub struct SPlayerShip_OWN {
    pub path: ObjPath,
    pub ship_class: String,
    pub ship_name: String,
    pub transform: Transform,
    pub stats: Stats,
    pub nav_action: Action,
    pub nav_target: NavTarget,
    pub warp_state: WarpState
}