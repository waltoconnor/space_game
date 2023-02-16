use bevy_ecs::prelude::*;

use crate::galaxy::components::*;

#[derive(Bundle)]
pub struct BPlayerShip {
    pub game_obj: GameObject,
    pub ship: Ship,
    pub transform: Transform,
    pub pc: PlayerController,
    pub nav: Navigation,
    pub sig: Signature,
    pub sensor: Sensor,
    // health
}

impl BPlayerShip {
    pub fn new(player_name: &String, transform: Transform, ship: Ship, system: &String, ship_name: &String) -> BPlayerShip {
        let pc = PlayerController {
            player_name: player_name.clone(),
            login_state: LoginState::LoggedIn
        };

        let nav = Navigation::new();
        let go = GameObject::new(system, crate::shared::ObjectType::PlayerShip, ship_name);

        BPlayerShip { ship: ship, transform: transform, pc: pc, nav: nav, game_obj: go, sig: Signature::new(10.0), sensor: Sensor::new() }
    }

    pub fn load_from_db(ship: Ship, player: &String, nav: Navigation, transform: Transform, game_obj: GameObject) -> Self {
        eprintln!("TODO: SET SIGNATURE AND STATS IN LOAD");
        BPlayerShip { game_obj, ship, transform, pc: PlayerController { player_name: player.clone(), login_state: LoginState::LoggedIn }, nav, sig: Signature::new(10.0), sensor: Sensor::new() }
    }
}