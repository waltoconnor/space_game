
use std::fmt::Debug;

use serde::{Serialize, Deserialize};
use sled::{Tree, Db, IVec};

use crate::{shared::ObjPath, galaxy::{components::{Ship, GameObject, Navigation, Transform}, bundles::ships::BPlayerShip}, inventory::{ItemTable, Inventory, Stack, InvSlot}};
use super::{db_consts::*, db_structs::{account::*, hanger::PlayerHanger, ship_in_space::ShipInSpace}, HangerSlot};
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

        DB { 
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
            item_table
        }
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

    /* HANGER */
    fn hanger_cook_key(&self, name: &String, hanger_id: u64) -> String {
        format!("{}:{}", name, hanger_id)
    }

    pub fn hanger_ensure(&self, name: &String, hanger_id: u64) {
        let key = self.hanger_cook_key(name, hanger_id);
        if !self.hanger.contains_key(key.as_bytes()).expect("Could not read hanger tree") {
            let new_hanger = PlayerHanger::new();
            self.hanger.insert(key.as_bytes(), self.ser(&new_hanger)).expect("Could not write new player hanger");
        }
    }

    pub fn hanger_set_active_ship_slot(&self, name: &String, hanger_id: u64, slot: u32) {
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
    pub fn hanger_get_ships(&self, name: &String, hanger_id: u64) -> Option<PlayerHanger> { 
        let key = self.hanger_cook_key(name, hanger_id);
        self.hanger.get(key.as_bytes()).expect("Could not read hanger tree").and_then(|x| Some(self.deser(&x)))
    }

    pub fn hanger_undock(&self, name: &String, hanger_id: u64) -> Option<Ship> {
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

    pub fn hanger_dock(&self, name: &String, hanger_id: u64, ship: Ship) {
        let key = self.hanger_cook_key(name, hanger_id);
        match self.hanger.get(key.as_bytes()).expect("Could not read hanger tree") {
            Some(h) => {
                let mut h: PlayerHanger = self.deser(&h);
                h.set_active_from_dock(ship);
                self.hanger.insert(key.as_bytes(), self.ser(&h)).expect("Could not push dock event to hanger tree");
            },
            None => {
                self.hanger_ensure(name, hanger_id);
                self.hanger_dock(name, hanger_id, ship); // if this keeps recursing, we have a big problem
            }
        }
    }

    pub fn hanger_add_ship(&self, name: &String, hanger_id: u64, ship: Ship) {
        let key = self.hanger_cook_key(name, hanger_id);
        match self.hanger.get(key.as_bytes()).expect("Could not read hanger tree") {
            Some(h) => {
                let mut h: PlayerHanger = self.deser(&h);
                h.insert_ship(ship);
                self.hanger.insert(key.as_bytes(), self.ser(&h)).expect("Could not push ship add to tree");
            },
            None => {
                self.hanger_ensure(name, hanger_id);
                self.hanger_add_ship(name, hanger_id, ship); // if this keeps recursing, we have a big problem
            }
        }
    }

    pub fn hanger_remove_ship(&self, name: &String, hanger_id: u64, slot: HangerSlot) -> Option<Ship> {
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
    pub fn hanger_sweep(&self, name: &String, hanger_id: u64) {
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
    pub fn hanger_ship_take_items(&self, name: &String, hanger_id: u64, slot: HangerSlot, inv_slot: InvSlot, count: u32) -> Option<Stack> {
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
    pub fn hanger_ship_add_items(&self, name: &String, hanger_id: u64, slot: HangerSlot, inv_slot: Option<InvSlot>, stack: Stack) -> Option<Stack> {
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
    fn inventory_cook_key(&self, name: &String, inventory_id: u64) -> String {
        format!("{}:{}", name, inventory_id)
    }

    pub fn inventory_ensure(&self, name: &String, inventory_id: u64) {
        let key = self.inventory_cook_key(name, inventory_id);
        if self.inventory.contains_key(key.as_bytes()).expect("Could not check for inventory key"){
            return;
        }
        self.inventory.insert(key.as_bytes(), self.ser(&Inventory::new(Some(inventory_id), None))).expect("Could not ensure inventory");
    }

    fn inventory_run_fn<F, F1>(&self, name: &String, inventory_id: u64, func: F1) -> F 
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

    pub fn inventory_insert_stack(&self, name: &String, inventory_id: u64, stack: Stack, slot: Option<u32>) -> Option<Stack> {
        self.inventory_ensure(name, inventory_id);
        let res = self.inventory_run_fn(name, inventory_id, |inv|{
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

    pub fn inventory_remove_stack(&self, name: &String, inventory_id: u64, slot: u32, count: Option<u32>, ) -> Option<Stack> {
        if !self.inventory.contains_key(&self.inventory_cook_key(name, inventory_id).as_bytes()).expect("Could not read key from db") { return None; }
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
    pub fn inventory_get_inv(&self, name: &String, inventory_id: u64) -> Option<Inventory> {
        let key = self.inventory_cook_key(name, inventory_id);
        self.inventory.get(key).expect("Could not read inventory db").and_then(|i| self.deser(&i))
    }

    /* MARKET */

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