use serde::{Serialize, Deserialize};

use crate::shared::ObjPath;


#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub name: String,
    pub access_token: String,
    pub current_location: ObjPath,
    pub home_station_path: ObjPath,
}

impl Account {
    pub fn new(name: String, access_token: String, home_station: ObjPath) -> Self {
        Account { name: name, access_token: access_token, current_location: home_station.clone(), home_station_path: home_station }
    }
}