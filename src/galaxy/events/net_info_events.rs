/// Client info event (about inventory, accounts, and the market)
#[derive(Debug)]
pub enum EInfo {
    Error(String, String), // Player, error message
}