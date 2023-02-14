use bevy_ecs::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};

use crate::{galaxy::components::*, shared::{ObjPath, ObjectType}};

#[derive(Bundle)]
pub struct BSun {
    pub game_object: GameObject,
    pub sun: Sun,
    pub celestial: Celestial,
    pub transform: Transform,
    pub warp_target: WarpTarget
}


impl BSun {
    pub fn new(system: &String, name: &String, temp_k: u32, spectral_class: &String, radius_m: f64, mass_kg: f64, warp_point: Vector3<f64>) -> Self {
        BSun { 
            game_object: GameObject { path: ObjPath::new(system, ObjectType::Star, name)}, 
            sun: Sun { temp_k: temp_k, spectral_class: spectral_class.clone() }, 
            celestial: Celestial { radius_m, mass_kg }, 
            transform: Transform { pos: Vector3::zeros(), rot: UnitQuaternion::default(), vel: Vector3::zeros() },
            warp_target: WarpTarget::new(warp_point),
        }
    }
}

#[derive(Bundle)]
pub struct BPlanet {
    pub game_object: GameObject,
    pub planet: Planet,
    pub celestial: Celestial,
    pub transform: Transform,
    pub warp_target: WarpTarget,
}

impl BPlanet {
    pub fn new(system: &String, name: &String, planet_type: &String, radius_m: f64, mass_kg: f64, abs_pos: Vector3<f64>, abs_rot: UnitQuaternion<f64>, warp_point: Vector3<f64>) -> Self {
        BPlanet { 
            game_object: GameObject { path: ObjPath::new(system, ObjectType::Planet, name)}, 
            planet: Planet { planet_type: planet_type.clone() }, 
            celestial: Celestial { radius_m, mass_kg }, 
            transform: Transform { pos: abs_pos, rot: abs_rot, vel: Vector3::zeros() },
            warp_target: WarpTarget::new(warp_point),
        }
    }
}

#[derive(Bundle)]
pub struct BMoon {
    pub game_object: GameObject,
    pub moon: Moon,
    pub celestial: Celestial,
    pub transform: Transform,
    pub warp_target: WarpTarget,
}

impl BMoon {
    pub fn new(system: &String, name: &String, moon_type: &String, radius_m: f64, mass_kg: f64, abs_pos: Vector3<f64>, abs_rot: UnitQuaternion<f64>, warp_point: Vector3<f64>) -> Self {
        BMoon { 
            game_object: GameObject { path: ObjPath::new(system, ObjectType::Moon, name)}, 
            moon: Moon { moon_type: moon_type.clone() }, 
            celestial: Celestial { radius_m, mass_kg }, 
            transform: Transform { pos: abs_pos, rot: abs_rot, vel: Vector3::zeros() },
            warp_target: WarpTarget::new(warp_point)
        }
    }
}

#[derive(Bundle)]
pub struct BAsteroidBelt {
    pub game_object: GameObject,
    pub belt: AsteroidBelt,
    pub transform: Transform,
    pub warp_target: WarpTarget
}

impl BAsteroidBelt {
    pub fn new(system: &String, name: &String, abs_pos: Vector3<f64>, abs_rot: UnitQuaternion<f64>, warp_point: Vector3<f64>) -> Self {
        BAsteroidBelt { 
            game_object: GameObject { path: ObjPath::new(system, ObjectType::AsteroidBelt, name)}, 
            belt: AsteroidBelt {},
            transform: Transform { pos: abs_pos, rot: abs_rot, vel: Vector3::zeros() },
            warp_target: WarpTarget::new(warp_point)
        }
    }
}