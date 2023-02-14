use bevy_ecs::prelude::*;
use nalgebra::Vector3;

#[derive(Component)]
pub struct WarpTarget {
    pub warp_point: Vector3<f64>
}

impl WarpTarget {
    pub fn new(target: Vector3<f64>) -> Self {
        WarpTarget { warp_point: target }
    }
}