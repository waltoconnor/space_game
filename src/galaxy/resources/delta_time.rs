use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct DeltaTime {
    pub dt: f64
}

impl DeltaTime {
    pub fn new() -> Self {
        DeltaTime {
            dt: 0.0
        }
    }
}