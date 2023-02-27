use nalgebra::Vector3;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GalaxyMap {
    pub systems: Vec<GMSystem>,
    pub links: Vec<GMLink>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GMSystem {
    pub name: String,
    pub region: String,
    pub sun_temp: u32,
    pub pos: Vector3<f64>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GMLink {
    pub start: String,
    pub end: String
}