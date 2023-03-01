
use std::{fmt::Debug, collections::HashMap};

use serde::{Serialize, Deserialize};
use sled::{Tree, Db, IVec};

use crate::{shared::ObjPath, galaxy::{components::{Ship, GameObject, Navigation, Transform, HngId}, bundles::ships::BPlayerShip}, inventory::{ItemTable, Inventory, Stack, InvSlot, ItemId, InvId}};
use super::{db_consts::*, db_structs::{account::*, hanger::PlayerHanger, ship_in_space::ShipInSpace, bank::BankAccount, market::{self, ItemStore}}, HangerSlot, PlayerOutstanding};
use rmp_serde::{to_vec, from_slice};

pub struct DB {
    account: Tree,
    bank: Tree,
    hanger: Tree,
    inventory: Tree,
    market: Tree,
    skills: Tree,
    resources: Tree,
    production: Tree,
    ships_in_space: Tree,
    statistics: Tree,
    overlord: Tree,
    db: Db,

    pub item_table: ItemTable
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoginStatus {
    Good,
    BadPass,
    NoAccount
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateAccountStatus {
    Good,
    NameTaken
}


impl DB {
    pub fn load(path: &String, sled_cache_size: u64, item_table: ItemTable) -> Self {
        let config = sled::Config::default()
            .path(path)
            .cache_capacity(sled_cache_size)
            .use_compression(true)
            .compression_factor(16);
        let db = config.open().expect("Could not open sled database");

        let db = DB { 
            account: db.open_tree(ACCOUNT_TREE).expect("Could not open account tree"), 
            bank: db.open_tree(BANK_TREE).expect("Could not open bank tree"),
            hanger: db.open_tree(HANGER_TREE).expect("Could not open hanger tree"), 
            inventory: db.open_tree(INVENTORY_TREE).expect("Could not open inventory tree"),
            market: db.open_tree(MARKET_TREE).expect("Could not open market tree"),
            skills: db.open_tree(SKILLS_TREE).expect("Could not open skills tree"), 
            resources: db.open_tree(RESOURCES_TREE).expect("Could not open resources tree"), 
            production: db.open_tree(PRODUCTION_TREE).expect("Could not open production tree"), 
            ships_in_space: db.open_tree(IN_SPACE_TREE).expect("Could not open ships in space tree"), 
            statistics: db.open_tree(STATISTICS_TREE).expect("Could not open statistics tree"), 
            overlord: db.open_tree(OVERLORD_TREE).expect("Could not open inventory tree"),
            db: db,
            item_table: item_table.clone()
        };

        db.market_inject_items(&item_table);
        db
    }

    fn ser<T: Serialize + Debug>(&self, data: &T) -> IVec {
        IVec::from(to_vec(data).expect(format!("DB FAILED TO SERIALIZE: {:?}", data).as_str()))
        
    }

    fn deser<'de, T: Deserialize<'de> + Debug>(&self, data: &'de IVec) -> T {
        from_slice(data).expect("DB could not deserialize")
    }

    /* ACCOUNT */
    pub fn account_try_login(&self, name: &String, access_token: &String) -> LoginStatus {
        match self.account.get(name.as_bytes()).expect("DB read error") {
            Some(a) => {
                let acc: Account = self.deser::<Account>(&a);
                if acc.access_token == *access_token { LoginStatus::Good } else { LoginStatus::BadPass }
            }
            None => LoginStatus::NoAccount
        }
    }

    pub fn account_create(&self, name: &String, access_token: &String, home_station: ObjPath) -> CreateAccountStatus {
        if self.account.contains_key(name.as_bytes()).expect("Could not read account db") {
            CreateAccountStatus::NameTaken
        }
        else {
            let acc = Account::new(name.clone(), access_token.clone(), home_station);
            self.account.insert(name.as_bytes(), self.ser(&acc)).expect("Could not write to db");
            CreateAccountStatus::Good
        }

    }

    pub fn account_change_location(&self, name: &String, location: ObjPath){
        match self.account.get(name.as_bytes()).expect("Could not read account from db") {
            None => { eprintln!("Account not found for {} while changing location", name); },
            Some(a) => { 
                let mut acc: Account = self.deser(&a);
                acc.current_location = location;
                self.account.insert(name.as_bytes(), self.ser(&acc)).expect("Could not write account to db during location change");
            }
        };
    }

    pub fn account_get_location(&self, name: &String) -> Option<ObjPath> {
        self.account.get(name.as_bytes()).expect("Could not read account from db").and_then(|x| Some(self.deser::<Account>(&x).current_location))
    }

    pub fn account_get_home_station(&self, name: &String) -> Option<ObjPath> {
        self.account.get(name.as_bytes()).expect("Could not read account from db").and_then(|x| Some(self.deser::<Account>(&x).home_station_path))
    }

    pub fn account_delete(&self, name: &String) {
        eprintln!("TODO: SUPPORT ACCOUNT DELETION, NEED TO CLEAN UP DATA IN ALL TABLES");
    }

    /* BANK */
    fn bank_cook_account_key(&self, name: &String) -> String {
        format!("{}:{}", BANK_ACCOUNT_PREFIX, name)
    }

    fn bank_cook_value_key(&self, name: &String) -> String {
        format!("{}:{}", BANK_VALUE_PREFIX, name)
    }

    pub fn bank_new_account(&self, name: &String) {
        let acct_key = self.bank_cook_account_key(name);
        let val_key = self.bank_cook_value_key(name);
        let acct = BankAccount::new(name);
        let val: i64 = 0;
        self.bank.insert(acct_key.as_bytes(), self.ser(&acct)).expect("Could not write new account to bank tree");
        self.bank.insert(val_key.as_bytes(), self.ser(&val)).expect("Could not write new account val to bank tree");
    }

    pub fn bank_apply_transaction(&self, name: &String, value: i64, reason: String) -> Option<i64> {
        let acct_key = self.bank_cook_account_key(name);
        let val_key = self.bank_cook_value_key(name);
        match (self.bank.get(acct_key.as_bytes()).expect("Could not read bank tree"), self.bank.get(val_key.as_bytes()).expect("Could not read bank tree")) {
            (Some(acct), Some(val)) => {
                let mut bank_acct: BankAccount = self.deser(&acct);
                let mut bank_val: i64 = self.deser(&val);

                if (value > 0 && i64::MAX - value >= bank_val) || bank_val + value >= 0 {
                    bank_val += value;
                    bank_acct.apply_transaction(value, reason);
                    self.bank.insert(acct_key.as_bytes(), self.ser(&bank_acct)).expect("Could not write bank account");
                    self.bank.insert(val_key.as_bytes(), self.ser(&bank_val)).expect("Could not write bank value");
                    Some(value)
                }
                else {
                    None
                }
            },
            (_, _) => {
                eprintln!("COULD NOT FIND BANK ACCOUNT FOR {}", name);
                None
            }
        }
    }

    pub fn bank_get_value(&self, name: &String) -> Option<i64> {
        let val_key = self.bank_cook_value_key(name);
        self.bank.get(val_key.as_bytes()).expect("Could not read bank tree").and_then(|v| Some(self.deser(&v)))
    }

    pub fn bank_get_account_hist(&self, name: &String) -> Option<BankAccount> {
        let acct_key = self.bank_cook_account_key(name);
        self.bank.get(acct_key.as_bytes()).expect("Could not read bank tree").and_then(|v| Some(self.deser(&v)))
    }


    /* HANGER */
    fn hanger_cook_key(&self, name: &String, hanger_id: HngId) -> String {
        format!("{}:{}", name, hanger_id)
    }

    pub fn hanger_ensure(&self, name: &String, hanger_id: HngId) {
        let key = self.hanger_cook_key(name, hanger_id);
        if !self.hanger.contains_key(key.as_bytes()).expect("Could not read hanger tree") {
            let new_hanger = PlayerHanger::new();
            self.hanger.insert(key.as_bytes(), self.ser(&new_hanger)).expect("Could not write new player hanger");
        }
    }

    pub fn hanger_set_active_ship_slot(&self, name: &String, hanger_id: HngId, slot: u32) {
        let key = self.hanger_cook_key(name, hanger_id);
        match self.hanger.get(key.as_bytes()).expect("Could not read hanger tree") {
            Some(h) => {
                let mut h: PlayerHanger = self.deser(&h);
                h.set_active_from_slot(slot);
                self.hanger.insert(key.as_bytes(), self.ser(&h)).expect("Could not push active ship change to hanger tree");
            },
            None => {
                eprintln!("Tried to set active ship in a nonexistent hanger");
            }
        }
    }

    /// RETURNS A COPY OF THE ACTUAL HANGER, CAN NOT MUTATE DIRECTLY
    pub fn hanger_get_ships(&self, name: &String, hanger_id: HngId) -> Option<PlayerHanger> { 
        let key = self.hanger_cook_key(name, hanger_id);
        self.hanger.get(key.as_bytes()).expect("Could not read hanger tree").and_then(|x| Some(self.deser(&x)))
    }

    pub fn hanger_undock(&self, name: &String, hanger_id: HngId) -> Option<Ship> {
        let key = self.hanger_cook_key(name, hanger_id);
        match self.hanger.get(key.as_bytes()).expect("Could not read hanger tree") {
            Some(h) => {
                let mut h: PlayerHanger = self.deser(&h);
                let ship = h.remove_active_ship_undock();
                self.hanger.insert(key.as_bytes(), self.ser(&h)).expect("Could not push undock event to hanger tree");
                ship
            },
            None => {
                eprintln!("Tried to set active ship in a nonexistent hanger");
                None
            }
        }
    }

    pub fn hanger_dock(&self, name: &String, hanger_id: HngId, ship: Ship) {
        let key = self.hanger_cook_key(name, hanger_id.clone());
        match self.hanger.get(key.as_bytes()).expect("Could not read hanger tree") {
            Some(h) => {
                let mut h: PlayerHanger = self.deser(&h);
                h.set_active_from_dock(ship);
                self.hanger.insert(key.as_bytes(), self.ser(&h)).expect("Could not push dock event to hanger tree");
            },
            None => {
                self.hanger_ensure(name, hanger_id.clone());
                self.hanger_dock(name, hanger_id, ship); // if this keeps recursing, we have a big problem
            }
        }
    }

    pub fn hanger_add_ship(&self, name: &String, hanger_id: HngId, ship: Ship) {
        let key = self.hanger_cook_key(name, hanger_id.clone());
        match self.hanger.get(key.as_bytes()).expect("Could not read hanger tree") {
            Some(h) => {
                let mut h: PlayerHanger = self.deser(&h);
                h.insert_ship(ship);
                self.hanger.insert(key.as_bytes(), self.ser(&h)).expect("Could not push ship add to tree");
            },
            None => {
                self.hanger_ensure(name, hanger_id.clone());
                self.hanger_add_ship(name, hanger_id, ship); // if this keeps recursing, we have a big problem
            }
        }
    }

    pub fn hanger_remove_ship(&self, name: &String, hanger_id: HngId, slot: HangerSlot) -> Option<Ship> {
        let key = self.hanger_cook_key(name, hanger_id);
        match self.hanger.get(key.as_bytes()).expect("Could not read hanger tree") {
            Some(h) => {
                let mut h: PlayerHanger = self.deser(&h);
                let ship = h.remove_ship(slot);
                self.hanger.insert(key.as_bytes(), self.ser(&h)).expect("Could not push ship removal to tree");
                ship
            },
            None => {
                eprintln!("Tried to remove ship in a nonexistent hanger");
                None
            }
        }
    }

    /// REMOVES THE HANGER FROM THE DB IF IT IS EMPTY
    pub fn hanger_sweep(&self, name: &String, hanger_id: HngId) {
        let key = self.hanger_cook_key(name, hanger_id);
        match self.hanger.get(key.as_bytes()).expect("Could not read hanger tree") {
            Some(h) => {
                let h: PlayerHanger = self.deser(&h);
                if h.is_empty() {
                    self.hanger.remove(key.as_bytes()).expect("Could not remove hanger during sweep");
                }
            },
            None => {
                eprintln!("Tried to sweep a nonexistent hanger");
            }
        }
    }

    // returns anything that was taken
    pub fn hanger_ship_take_items(&self, name: &String, hanger_id: HngId, slot: HangerSlot, inv_slot: InvSlot, count: u32) -> Option<Stack> {
        let key = self.hanger_cook_key(name, hanger_id);
        match self.hanger.get(key.as_bytes()).expect("Could not read hanger tree") {
            Some(h) => {
                let mut h: PlayerHanger = self.deser(&h);
                let ship = match h.inventory.get_mut(&slot) {
                    Some(s) => s,
                    None => { return None; }
                };
                let stack = ship.inventory.remove_n_from_stack(inv_slot, count);
                self.hanger.insert(key.as_bytes(), self.ser(&h)).expect("Could not push ship removal to tree");
                stack
            },
            None => None
        }
    }

    // returns anything that could not be transferred
    pub fn hanger_ship_add_items(&self, name: &String, hanger_id: HngId, slot: HangerSlot, inv_slot: Option<InvSlot>, stack: Stack) -> Option<Stack> {
        let key = self.hanger_cook_key(name, hanger_id);
        match self.hanger.get(key.as_bytes()).expect("Could not read hanger tree") {
            Some(h) => {
                let mut h: PlayerHanger = self.deser(&h);
                let ship = match h.inventory.get_mut(&slot) {
                    Some(s) => s,
                    None => { return None; }
                };
                let stack = ship.inventory.add_stack(&self.item_table, stack, inv_slot);
                self.hanger.insert(key.as_bytes(), self.ser(&h)).expect("Could not push ship removal to tree");
                stack
            },
            None => None
        }
    }

    /* INVENTORY */
    fn inventory_cook_key(&self, name: &String, inventory_id: InvId) -> String {
        format!("{}:{}", name, inventory_id)
    }

    pub fn inventory_ensure(&self, name: &String, inventory_id: InvId) {
        let key = self.inventory_cook_key(name, inventory_id.clone());
        if self.inventory.contains_key(key.as_bytes()).expect("Could not check for inventory key"){
            return;
        }
        self.inventory.insert(key.as_bytes(), self.ser(&Inventory::new(Some(inventory_id.clone()), None))).expect("Could not ensure inventory");
    }

    fn inventory_run_fn<F, F1>(&self, name: &String, inventory_id: InvId, func: F1) -> F 
    where F1: FnOnce(Option<Inventory>) -> (Option<Inventory>, F) {
        let key = self.inventory_cook_key(name, inventory_id);
        let mut inv: Option<Inventory> = self.inventory.get(key.as_bytes()).expect("Could not read inventory from db").and_then(|inv| Some(self.deser(&inv)));
        let (res_inv, result) = func(inv);
        match res_inv {
            Some(ri) => { self.inventory.insert(key.as_bytes(), self.ser(&ri)).expect("Could not write updated inventory"); },
            None => { eprintln!("inventory_run_fn did not get inventory back from closure"); }
        }
        result
    }

    pub fn inventory_insert_stack(&self, name: &String, inventory_id: InvId, stack: Stack, slot: Option<u32>) -> Option<Stack> {
        self.inventory_ensure(name, inventory_id.clone());
        let res = self.inventory_run_fn(name, inventory_id.clone(), |inv|{
            match inv {
                None => {
                    eprintln!("Inventory not found despite being freshly made");
                    return (None, Some(stack));
                },
                Some(mut i) => {
                    let extra = i.add_stack(&self.item_table, stack, slot);
                    return (Some(i), extra);
                }
            }
        });
        res
    }

    pub fn inventory_remove_stack(&self, name: &String, inventory_id: InvId, slot: u32, count: Option<u32>, ) -> Option<Stack> {
        if !self.inventory.contains_key(&self.inventory_cook_key(name, inventory_id.clone()).as_bytes()).expect("Could not read key from db") { return None; }
        let res = self.inventory_run_fn(name, inventory_id, |inv| {
            match inv {
                None => (None, None),
                Some(mut inv) => {
                    let stack = match count {
                        Some(c) => inv.remove_n_from_stack(slot, c),
                        None => inv.remove_stack(slot)
                    };
                    (Some(inv), stack)
                }
            }
        });
        res
    }

    /// CLONE, SO YOU CANT MUTATE IT
    pub fn inventory_get_inv(&self, name: &String, inventory_id: InvId) -> Option<Inventory> {
        let key = self.inventory_cook_key(name, inventory_id.clone());
        self.inventory.get(key).expect("Could not read inventory db").and_then(|i| self.deser(&i))
    }

    /// WILL IGNORE CAPACITY REQUIREMENTS, DO NOT ALLOW PLAYERS TO INVOKE
    pub fn inventory_insert_stack_free_slot_ignore_capacity(&self, name: &String, inventory_id: InvId, stack: Stack) {
        self.inventory_ensure(name, inventory_id.clone());
        self.inventory_run_fn(name, inventory_id, |inv|{
            let mut inv = inv.expect("Could not unpack ensured inventory"); // we just ensured it exists
            let slot = inv.insert_stack(stack);
            (Some(inv), ())
        });
    }

    pub fn inventory_player_dump_all_inventories(&self, name: &String) -> HashMap<InvId, Inventory> {
        let prefix = self.ser(name);
        let mut r = self.inventory.scan_prefix(prefix);
        let mut hm = HashMap::new();
        while let Some(Ok((key_raw, data_raw))) = r.next() {
            let key: String = self.deser(&key_raw);
            let inv: Inventory = self.deser(&data_raw);
            if let Some((_name, id)) = key.split_once(':') {
                hm.insert(id.to_string(), inv);
            }
            else {
                continue;
            }
        };
        hm
    }

    pub fn inventory_player_list_inventories(&self, name: &String) -> Vec<InvId> {
        let prefix = self.ser(name);
        let r = self.inventory.scan_prefix(prefix);
        r.keys()
            .filter_map(|v| 
                match v { 
                    Ok(d) => self.deser(&d), 
                    Err(_) => None
                }
            )
            .filter_map(|text: String| 
                text.split_once(':')
                .map(|(_name, inv_id)| inv_id.to_string())
            )
            .collect()
    }

    /* MARKET */
    fn market_cook_index_key(&self, name: &String) -> String {
        format!("{}:{}", MARKET_PLAYER_LIST, name)
    }

    fn market_cook_store_key(&self, item_id: ItemId) -> String {
        format!("{}:{:?}", MARKET_ITEM, item_id)
    }

    pub fn market_add_player_index(&self, name: &String) {
        let key = self.market_cook_index_key(name);
        let player_index = market::PlayerOutstanding::new(name.clone());
        self.market.insert(key.as_bytes(), self.ser(&player_index)).expect("Could not write to market tree");
    }

    pub fn market_load_item_store(&self, item_id: ItemId) -> Option<ItemStore> {
        let key = self.market_cook_store_key(item_id.clone());
        self.market.get(key.as_bytes()).expect("Could not read item store from tree").and_then(|is| Some(self.deser(&is)))
    }

    pub fn market_save_item_store(&self, store: &ItemStore) {
        let key = self.market_cook_store_key(store.item.clone());
        self.market.insert(key.as_bytes(), self.ser(store)).expect("Could not write item store to tree");
    }

    pub fn market_add_buy_order_to_player(&self, name: &String, item_id: &ItemId, order_id: u64) {
        let key = self.market_cook_index_key(name);
        match self.market.get(key.as_bytes()).expect("Could not read player index from market tree").and_then(|idx| Some(self.deser::<PlayerOutstanding>(&idx))) {
            Some(mut data) => {
                data.add_buy_order(order_id, item_id.clone());
                self.market.insert(key.as_bytes(), self.ser(&data)).expect("Could not write to market player index");
            },
            None => {
                eprintln!("Unable to update player outstanding orders");
            }
        }
    }

    pub fn market_add_sell_order_to_player(&self, name: &String, item_id: &ItemId, order_id: u64) {
        let key = self.market_cook_index_key(name);
        match self.market.get(key.as_bytes()).expect("Could not read player index from market tree").and_then(|idx| Some(self.deser::<PlayerOutstanding>(&idx))) {
            Some(mut data) => {
                data.add_sell_order(order_id, item_id.clone());
                self.market.insert(key.as_bytes(), self.ser(&data)).expect("Could not write to market player index");
            },
            None => {
                eprintln!("Unable to update player outstanding orders");
            }
        }
    }

    pub fn market_remove_buy_order_from_player(&self, name: &String, order_id: u64) {
        let key = self.market_cook_index_key(name);
        match self.market.get(key.as_bytes()).expect("Could not read player index from market tree").and_then(|idx| Some(self.deser::<PlayerOutstanding>(&idx))) {
            Some(mut data) => {
                data.clear_buy_order(order_id);
                self.market.insert(key.as_bytes(), self.ser(&data)).expect("Could not write to market player index");
            },
            None => {
                eprintln!("Unable to update player outstanding orders");
            }
        }
    }

    pub fn market_remove_sell_order_from_player(&self, name: &String, order_id: u64) {
        let key = self.market_cook_index_key(name);
        match self.market.get(key.as_bytes()).expect("Could not read player index from market tree").and_then(|idx| Some(self.deser::<PlayerOutstanding>(&idx))) {
            Some(mut data) => {
                data.clear_sell_order(order_id);
                self.market.insert(key.as_bytes(), self.ser(&data)).expect("Could not write to market player index");
            },
            None => {
                eprintln!("Unable to update player outstanding orders");
            }
        }
    }

    pub fn market_can_place_new_order(&self, name: &String) -> bool {
        let key = self.market_cook_index_key(name);
        match self.market.get(key.as_bytes()).expect("Could not read player index from market tree").and_then(|idx| Some(self.deser::<PlayerOutstanding>(&idx))) {
            Some(data) => {
                data.can_place_order()
            },
            None => {
                eprintln!("Unable to update player outstanding orders");
                false
            }
        }
    }

    fn market_inject_items(&self, items: &ItemTable) {
        for item in items.keys() {
            let key = self.market_cook_store_key(item.clone());
            let store = ItemStore::new(item.clone());
            self.market.insert(key.as_bytes(), self.ser(&store)).expect("Could not insert new item store during init");
        }
    }

    /* SKILLS */

    /* RESOURCES (planets/moons/asteroid belts) */

    /* PRODUCTION */

    /* SHIPS IN SPACE */

    pub fn sis_load_ship(&self, name: &String) -> Option<BPlayerShip> {
        match self.ships_in_space.remove(name.as_bytes()).expect("Could not read ship from db") {
            Some(s) => {
                let ship: ShipInSpace = self.deser(&s);
                Some(BPlayerShip::load_from_db(ship.ship, &ship.player_name, ship.navigation, ship.transform, ship.game_object))
            },
            None => None
        }
    }

    pub fn sis_save_ship(&self, name: &String, ship: &Ship, nav: &Navigation, transform: &Transform, game_obj: &GameObject) {
        let ss = ShipInSpace {
            player_name: name.clone(),
            ship: ship.clone(),
            navigation: nav.clone(),
            transform: transform.clone(),
            game_object: game_obj.clone()
        };
        self.ships_in_space.insert(name.as_bytes(), self.ser(&ss)).expect("Could not save ship");
    }

    /* STATISTICS */

    /* OVERLORD */


}