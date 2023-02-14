use serde::{Serialize, Deserialize};



#[derive(Serialize, Deserialize, Debug)]
pub struct LStation {
    pub name: String,
    pub system: String,
    pub planet_name: String,
    pub orbit: [f64; 6]
}

#[derive(Deserialize, Debug)]
pub struct LStationList {
    pub stations: Vec<LStation>
}