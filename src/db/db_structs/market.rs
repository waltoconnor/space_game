use std::collections::HashMap;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::inventory::InvId;
use crate::{inventory::{Stack, ItemId}};

#[derive(Debug, Serialize, Deserialize)]
pub struct BuyOrder {
    pub item_id: ItemId,
    pub count: u32,
    pub escrow: i64,
    pub player: String,
    pub location: InvId,
    pub order_id: u64,
    pub time_placed: String,
}

impl BuyOrder {
    pub fn new(item_id: ItemId, count: u32, player: String, escrow: i64, location: InvId) -> Self {
        let time = Utc::now();
        let val = format!("buy{}-{}-{}-{:?}-{}", item_id, count, player, location, time.to_rfc3339());
        let mut s = DefaultHasher::new();
        val.hash(&mut s);
        let id = s.finish();
        BuyOrder { item_id, count, escrow, player, location, order_id: id, time_placed: time.to_rfc3339()}
    }

    pub fn satisfy(&mut self, stack: Stack) -> i64 {
        if self.item_id != stack.id {
            eprintln!("TRYING TO SATISFY BUY ORDER WITH STACK OF WRONG TYPE: {:?} <- {:?}", self, stack);
            return 0;
        }

        if stack.count > self.count {
            eprintln!("TRYING TO SATISFY BUY ORDER WITH TOO MANY ITEMS: {:?} <- {:?}", self, stack);
            return 0;
        }

        let cost_per_item = self.escrow / (self.count as i64);
        let amount_earned = self.count as i64 * cost_per_item;
        self.count -= stack.count;
        self.escrow -= amount_earned;
        return amount_earned;
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SellOrder {
    pub stack: Stack,
    pub player: String,
    pub cost_per_item: i64,
    pub location: InvId,
    pub order_id: u64,
    pub time_placed: String
}

impl SellOrder {
    pub fn new(stack: Stack, cost_per_item: i64, player: String, location: InvId) -> Self {
        let time = Utc::now();
        let val = format!("buy{}-{}-{:?}-{}", stack.id, player, location, time.to_rfc3339());
        let mut s = DefaultHasher::new();
        val.hash(&mut s);
        let id = s.finish();
        SellOrder { stack, player, cost_per_item, location, order_id: id, time_placed: time.to_rfc3339() }
    }

    pub fn buy_from(&mut self, count: u32, money: i64) -> Option<Stack> {
        if count > self.stack.count {
            eprintln!("TRYING TO BUY TOO MANY OF ITEM: {:?} (want {}, have {})", self, count, self.stack.count);
            return None;
        }

        if money != (count as i64) * self.cost_per_item {
            eprintln!("TRYING TO FULFILL SELL ORDER WITH WRONG AMOUNT OF MONEY: {:?} <- (want: {} (at {}), need: {}, gave: {})", self, count, self.cost_per_item, (count as i64) * self.cost_per_item, money);
            return None;
        }

        self.stack.count -= count;
        Some(Stack::new(self.stack.id.clone(), count))
    }

    pub fn is_empty(&self) -> bool {
        self.stack.count == 0
    }
}

pub struct StoreTransaction {
    pub purchasing_player: String,
    pub selling_player: String,
    pub purchased_stack: Stack,
    pub cost: i64,
    pub location: InvId,
    pub order_complete: bool
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ItemStore {
    pub item: ItemId,
    sell_orders: HashMap<u64, SellOrder>,
    buy_orders: HashMap<u64, BuyOrder>
}

impl ItemStore {
    pub fn new(item_id: ItemId) -> Self {
        ItemStore { item: item_id, sell_orders: HashMap::new(), buy_orders: HashMap::new() }
    }

    pub fn add_sell_order(&mut self, player: &String, stack: Stack, cost_per_item: i64, location: InvId) -> Result<(), String> {
        let order = SellOrder::new(stack, cost_per_item, player.clone(), location);
        if self.sell_orders.contains_key(&order.order_id) {
            return Err(String::from("An order with that ID already exists (hash collision)"));
        }
        self.sell_orders.insert(order.order_id, order);
        Ok(())
    }

    pub fn add_buy_order(&mut self, player: &String, item_id: ItemId, count: u32, cost_per_item: i64, location: InvId) -> Result<(), String> {
        let order = BuyOrder::new(item_id, count, player.clone(), count as i64 * cost_per_item, location);
        if self.buy_orders.contains_key(&order.order_id) {
            return Err(String::from("An order with that ID already exists (hash collision)"));
        }
        self.buy_orders.insert(order.order_id, order);
        Ok(())
    }

    pub fn fulfill_buy_order(&mut self, order_id: u64, stack: Stack, location: InvId, selling_player: String) -> Result<StoreTransaction, String> {
        let mut order = self.buy_orders.get_mut(&order_id).ok_or(String::from("Order no longer exists"))?;
        if location != order.location {
            return Err(String::from("Order is not in range"));
        }

        if order.item_id != stack.id {
            return Err(String::from("Buy order not being fulfilled with item of right type"));
        }

        if order.count < stack.count {
            return Err(String::from("Buy order is being fulfilled with too big of stack"));
        }

        let money_earned = order.satisfy(stack.clone());
        let transaction = StoreTransaction {
            purchasing_player: order.player.clone(),
            selling_player: selling_player,
            purchased_stack: stack,
            cost: money_earned,
            location: location,
            order_complete: order.is_empty()
        };

        Ok(transaction)
    }

    pub fn fulfill_sell_order(&mut self, order_id: u64, count: u32, location: InvId, buying_player: String) -> Result<StoreTransaction, String> {
        let mut order = self.sell_orders.get_mut(&order_id).ok_or(String::from("Order no longer exists"))?;
        if location != order.location {
            return Err(String::from("Order is not in range"));
        }

        if order.stack.count < count {
            return Err(String::from("Trying to purchase too many items from sell order"));
        }

        let cost = order.cost_per_item * (count as i64);
        let stack_retreived = order.buy_from(count, cost).ok_or(String::from("Order not fulfilled correctly"))?;
        let transaction = StoreTransaction {
            purchasing_player: buying_player.clone(),
            selling_player: order.player.clone(),
            purchased_stack: stack_retreived,
            cost: cost,
            location: location,
            order_complete: order.is_empty()
        };

        Ok(transaction)
    }
    
    pub fn clear_buy_order(&mut self, order_id: u64) -> Option<bool> {
        let order = self.buy_orders.get(&order_id)?;
        if !order.is_empty() {
            eprintln!("Trying to clear buy order that isn't empty");
            return None;
        }
        self.buy_orders.remove(&order_id);
        Some(true)
    }

    pub fn clear_sell_order(&mut self, order_id: u64) -> Option<bool> {
        let order = self.sell_orders.get(&order_id)?;
        if !order.is_empty() {
            eprintln!("Trying to clear sell order that isn't empty");
            return None;
        }
        self.sell_orders.remove(&order_id);
        Some(true)
    }

    pub fn cancel_sell_order(&mut self, player: &String, order_id: u64) -> Result<Stack, String> {
        if !self.sell_orders.contains_key(&order_id) {
            return Err(String::from("Order no longer exists"));
        };

        let order = self.sell_orders.remove(&order_id).expect("Cannot cancel nonexistent sell order");
        if order.player != *player {
            self.sell_orders.insert(order_id, order);
            eprintln!("TRYING TO CANCEL OTHER PLAYER'S SELL ORDER");
            return Err(String::from("Cannot cancel other player's sell order"));
        }
        let stack = order.stack;
        Ok(stack)
    }

    pub fn cancel_buy_order(&mut self, player: &String, order_id: u64) -> Result<i64, String> {
        if !self.buy_orders.contains_key(&order_id) {
            return Err(String::from("Order no longer exists"));
        };

        let order = self.buy_orders.remove(&order_id).expect("Cannot cancel nonexistent buy order");
        if order.player != *player {
            self.buy_orders.insert(order_id, order);
            eprintln!("TRYING TO CANCEL OTHER PLAYER'S BUY ORDER");
            return Err(String::from("Cannot cancel other player's buy order"));
        }
        let remaining_escrow = order.escrow;
        Ok(remaining_escrow)
    }

    pub fn get_sell_order<'a>(&'a self, order_id: u64) -> Option<&'a SellOrder> {
        self.sell_orders.get(&order_id)
    }

    pub fn get_buy_order<'a>(&'a self, order_id: u64) -> Option<&'a BuyOrder> {
        self.buy_orders.get(&order_id)
    }

}


/* PLAYER METADATA */
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerOutstanding {
    pub name: String,
    pub sell_orders: HashMap<u64, ItemId>, //order id, order item
    pub buy_orders: HashMap<u64, ItemId>, //order id, order item
    pub max_orders: u32,
}

impl PlayerOutstanding {
    pub fn new(name: String) -> Self {
        PlayerOutstanding { name: name, sell_orders: HashMap::new(), buy_orders: HashMap::new(), max_orders: 100 }
    }

    pub fn set_max_orders(&mut self, orders: u32) {
        self.max_orders = orders;
    }

    pub fn add_sell_order(&mut self, order_id: u64, item_id: ItemId) {
        self.sell_orders.insert(order_id, item_id);
    }

    pub fn add_buy_order(&mut self, order_id: u64, item_id: ItemId) {
        self.buy_orders.insert(order_id, item_id);
    }

    pub fn clear_sell_order(&mut self, order_id: u64) {
        self.sell_orders.remove(&order_id);
    }

    pub fn clear_buy_order(&mut self, order_id: u64) {
        self.buy_orders.remove(&order_id);
    }

    pub fn can_place_order(&self) -> bool {
        self.sell_orders.len() + self.buy_orders.len() < self.max_orders as usize
    }
}