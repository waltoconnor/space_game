use crate::{shared::ObjPath, inventory::InvId};

/// Client info event (about inventory, accounts, and the market)
#[derive(Debug)]
pub enum EInfo {
    Error(String, String), // Player, error message
    UpdateInventoryHanger(String, u64), //player, hanger id
    UpdateInventoryId(String, InvId), //player, inventory id
}