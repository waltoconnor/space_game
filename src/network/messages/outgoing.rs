use serde::{Serialize, Deserialize};

use crate::network::serialization_structs::{state::NetOutState, info::NetOutInfo, event::NetOutEvent};

#[derive(Serialize, Deserialize)]
pub enum NetOutgoingMessage {
    State(NetOutState),
    Event(NetOutEvent),
    Info(NetOutInfo),
    LoginBad,
    LoginOk
}