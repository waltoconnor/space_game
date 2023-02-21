use std::collections::VecDeque;
use chrono::{DateTime, Utc, NaiveDateTime};
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

    // pub fn can_apply_transaction(&mut self, val: i64) -> bool {
    //     if val > 0 && i64::MAX - val > self.cur_val {
    //         eprintln!("Account is at max value");
    //         return false;
    //     }
    //     else {
    //         self.cur_val + val >= 0
    //     }

    // }

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