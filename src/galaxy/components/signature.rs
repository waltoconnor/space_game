use bevy_ecs::prelude::*;

#[derive(Component, Debug)]
pub struct Signature {
    size_m: f64
}

impl Signature {
    pub fn new(size_m: f64) -> Self {
        Signature { size_m }
    }
}