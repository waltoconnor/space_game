use serde::{Serialize, Deserialize};

use crate::{shared::ObjPath, galaxy::components::Transform};


#[derive(Serialize, Deserialize)]
pub struct SSun {
    pub path: ObjPath,
    pub temp_k: u32,
    pub spectral_class: String,
    pub radius_m: f64,
    pub mass_kg: f64
}

#[derive(Serialize, Deserialize)]
pub struct SPlanet {
    pub path: ObjPath,
    pub planet_type: String,
    pub radius_m: f64,
    pub mass_kg: f64,
    pub transform: Transform
}

#[derive(Serialize, Deserialize)]
pub struct SMoon {
    pub path: ObjPath,
    pub moon_type: String,
    pub radius_m: f64,
    pub mass_kg: f64,
    pub transform: Transform
}

#[derive(Serialize, Deserialize)]
pub struct SAsteroidBelt {
    pub path: ObjPath,
    pub transform: Transform
}