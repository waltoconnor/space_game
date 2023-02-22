use std::collections::VecDeque;
use chrono::Utc;
use serde::{Serialize, Deserialize};

const TRANACTION_COUNT: usize = 500;

#[derive(Debug, Deserialize, Serialize)]
pub struct BankAccount {
    pub player: String,
    pub history: VecDeque<Transaction>
}

impl BankAccount {
    pub fn new(player: &String) -> Self {
        BankAccount { player: player.clone(), history: VecDeque::new() }
    }

    /// VAL IS THE ABSOLUTE CHANGE IN ACCOUNT VALUE (neg = withdraw, pos = despoit)
    pub fn apply_transaction(&mut self, val: i64, reason: String) {
        self.history.push_front(Transaction::new(val, reason));
        if self.history.len() > TRANACTION_COUNT {
            self.history.pop_back();
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    time_string: String,
    amount: i64,
    time: String
}

impl Transaction {
    pub fn new(val: i64, reason: String) -> Self {
        let dt = Utc::now();
        Transaction { time_string: reason, amount: val, time: dt.to_rfc3339() }
    }
}