use serde::{Serialize, Deserialize};

pub fn load_config(cfg_path: String) -> Config {
    let file = std::fs::read_to_string(cfg_path).expect("Could not open config file");
    serde_json::from_str(file.as_str()).expect("Could not parse config file")
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub network: CfgNetwork,
    pub tick_time_ms: u32,
    pub assets_path: String,
    pub db_path: String,
    pub gameplay_config: CfgGameplay

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CfgNetwork {
    pub websocket_ip: String,
    pub websocket_port: u16,
    pub http_ip: String,
    pub http_port: u16
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CfgGameplay {
    pub starting_system: String,
    pub starting_station: String,
    pub starting_money: i64
}