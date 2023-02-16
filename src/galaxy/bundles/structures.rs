use std::{collections::{HashSet, hash_map::DefaultHasher}, hash::Hash, hash::Hasher};

use bevy_ecs::prelude::*;
use nalgebra::Vector3;

use crate::{galaxy::components::*, shared::{ObjPath, ObjectType}};

#[derive(Bundle)]
pub struct BStation {
    pub game_object: GameObject,
    pub hanger: Hanger,
    pub station: Station,
    pub transform: Transform,
    pub warp_target: WarpTarget
}

impl BStation {
    pub fn new(system: &String, name: &String, transform: Transform, warp_point: Vector3<f64>, undock_offset: Vector3<f64>, docking_range: f64) -> Self {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        BStation { 
            game_object: GameObject { path: ObjPath::new(system, ObjectType::Station, name) }, 
            hanger: Hanger { undock_offset: undock_offset, hanger_uid: hash, docking_range_m: docking_range }, 
            station: Station { current_players: HashSet::new() }, 
            transform: transform, 
            warp_target: WarpTarget::new(warp_point) 
        }
    }
}

#[derive(Bundle)]
pub struct BGate {
    pub game_object: GameObject,
    pub gate: Gate,
    pub transform: Transform,
    pub warp_target: WarpTarget
}

impl BGate {
    pub fn new(system: &String, name: &String, dst_sys: &String, dst_gate_name: &String, jump_radius: f64, transform: Transform, warp_point: Vector3<f64>) -> Self {
        BGate { 
            game_object: GameObject { path: ObjPath::new(system, ObjectType::Gate, name) }, 
            gate: Gate { 
                jump_range: jump_radius, 
                dst_system: dst_sys.clone(), 
                dst_gate: ObjPath::new(dst_sys, ObjectType::Gate, dst_gate_name) 
            }, 
            transform: transform, 
            warp_target: WarpTarget::new(warp_point) 
        }
    }
}