use bevy_ecs::prelude::*;
use nalgebra::{Vector3, UnitQuaternion};
use serde::{Serialize, Deserialize};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub pos: Vector3<f64>,
    pub rot: UnitQuaternion<f64>,
    pub vel: Vector3<f64>
}
