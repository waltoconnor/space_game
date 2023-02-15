use serde::{Serialize, Deserialize};

use self::hanger::SHanger;

pub mod hanger;

#[derive(Serialize, Deserialize)]
pub enum NetOutInfo {
    Hanger(SHanger)
}