use serde::{Serialize, Deserialize};

use crate::{shared::ObjPath, galaxy::components::Transform};

#[derive(Serialize, Deserialize)]
pub struct SGate {
    pub path: ObjPath,
    pub dst_sys: String,
    pub transform: Transform
}

#[derive(Serialize, Deserialize)]
pub struct SStation {
    pub path: ObjPath,
    pub transform: Transform
}