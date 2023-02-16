use std::time::Instant;

use bevy_ecs::prelude::*;
use serde::{Serialize, Deserialize};

pub enum LoginState {
    LoggedIn,
    LoggedOut(Instant),
    SafeLogged
}

#[derive(Component)]
pub struct PlayerController {
    pub player_name: String,
    pub login_state: LoginState
}