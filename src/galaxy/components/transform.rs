use bevy_ecs::prelude::*;
use nalgebra::{Vector3, UnitQuaternion};
use serde::{Serialize, Deserialize};

use crate::network::messages::outgoing::NetOutgoingMessage;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub pos: Vector3<f64>,
    pub rot: UnitQuaternion<f64>,
    pub vel: Vector3<f64>
}

impl Transform {
    pub fn to_mv(&self, name: String) -> NetOutgoingMessage {
        let rot = self.rot.coords.as_slice();
        NetOutgoingMessage::Mv(name, 
            [self.pos.x, self.pos.y, self.pos.z], 
            [self.vel.x as f32, self.vel.y as f32, self.vel.z as f32], 
            [rot[0] as f32, rot[1] as f32, rot[2] as f32, rot[3] as f32]
        )
    }
}