use serde::{Serialize, Deserialize};

use crate::shared::ObjPath;

use self::hanger::SHanger;

pub mod hanger;

#[derive(Serialize, Deserialize)]
pub enum NetOutInfo {
    Location(ObjPath),
    Hanger(SHanger)
}