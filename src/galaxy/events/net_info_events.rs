use crate::{shared::ObjPath, inventory::{InvId, ItemId}, db::ItemStore};

/// Client info event (about inventory, accounts, and the market)
#[derive(Debug)]
pub enum EInfo {
    Error(String, String), // Player, error message
    UpdateInventoryHanger(String, u64), //player, hanger id
    UpdateInventoryId(String, InvId), //player, inventory id
    UpdateBankAccount(String), //player
    ItemStore(String, ItemId),
}