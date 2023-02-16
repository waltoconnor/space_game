use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use super::{ItemTable, ItemId};

pub type InvSlot = u32;
pub type InvId = u64;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Inventory {
    id: Option<InvId>, //used for inventories in the database
    capacity_vunits: Option<u32>, //1 vunit = 0.01m3
    inv: HashMap<InvSlot, Stack>, //slot, stack
}

impl Inventory {
    pub fn new(id: Option<InvId>, capacity: Option<u32>) -> Self {
        Inventory { id: id, inv: HashMap::new(), capacity_vunits: capacity }
    }

    pub fn add_stack(&mut self, item_table: &ItemTable, mut stack: Stack, slot: Option<InvSlot>) -> Option<Stack> {
        match self.capacity_vunits {
            Some(cap) => {
                let vol_per_item = item_table.get(&stack.id).expect("GOT INVALID ITEM ID").size_vunits;
                let used_vol = self.get_cap_used(item_table);
                let free_vol = cap - used_vol;
                let max_count = free_vol / vol_per_item;
                let count = max_count.min(stack.count);
                let insert_stack = stack.take_n(count);
                match (insert_stack, slot) {
                    (None, _) => { return Some(stack); }, //weird stuff going on
                    (Some(is), None) => {
                        self.insert_stack(is);
                        if stack.count == 0 {
                            return None;
                        }
                        else{
                            return Some(stack);
                        }
                    },
                    (Some(is), Some(slot)) => {
                        self.insert_stack_at_slot(is, slot);
                        if stack.count == 0 {
                            return None;
                        }
                        else {
                            return Some(stack);
                        }
                    }
                }
            },
            None => { 
                match slot {
                    None => { self.insert_stack(stack); None },
                    Some(slot) => { self.insert_stack_at_slot(stack, slot); None }
                }
            }
        }
    }


    pub fn remove_stack(&mut self, slot: InvSlot) -> Option<Stack> {
        self.inv.remove(&slot)
    }

    pub fn remove_n_from_stack(&mut self, slot: InvSlot, count: u32) -> Option<Stack> {
        self.inv.get_mut(&slot).and_then(|s| s.take_n(count))
    }

    /// THE STACK MUST BE ABLE TO FIT
    fn insert_stack(&mut self, stack: Stack) {
        for (k, v) in self.inv.iter_mut() {
            if v.id == stack.id {
                let result = v.add(stack);
                if result.is_some() {
                    eprintln!("Failed to stack items, annihilating {:?}", result);
                }
                return;
            }
        }
        //if we are here, this is a new item type
        let slot = self.get_first_free_slot();
        self.inv.insert(slot, stack);
    }

    /// THE STACK MUST BE ABLE TO FIT
    fn insert_stack_at_slot(&mut self, stack: Stack, slot: InvSlot) {
        if let Some(v) = self.inv.get_mut(&slot) {
            if v.id == stack.id {
                let result = v.add(stack);
                if result.is_some() {
                    eprintln!("Failed to stack items, annihilating {:?}", result);
                }
                return;
            }
        }
        else {
            eprintln!("Tried to insert items in to incompatible slot, allocating new slot");
            let slot = self.get_first_free_slot();
            self.inv.insert(slot, stack);
        }
    }

    fn get_first_free_slot(&self) -> InvSlot {
        for i in 0..=self.inv.len() as u32 { // case 1: slot 0..n are filled: slot n+1 is open. case 2: slots after n are filled -> one of the first n slots are open
            if !self.inv.contains_key(&i) {
                return i;
            }
        }
        eprintln!("UNABLE TO FIND OPEN SLOT IN INVENTORY");
        return 0;
    }

    pub fn get_cap_used(&self, item_table: &ItemTable) -> u32 {
        self.inv.iter().map(|(k, v)| item_table.get(&v.id).and_then(|i| Some(i.size_vunits)).unwrap_or(0)).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.inv.len() == 0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stack {
    id: ItemId,
    count: u32,
}

impl Stack {
    pub fn new(id: ItemId, count: u32) -> Self {
        Stack { id, count }
    }

    pub fn take_n(&mut self, count: u32) -> Option<Stack> {
        if count == self.count {
            self.count = 0;
            Some(Stack::new(self.id.clone(), count))
        }
        else if count > self.count{
            None
        }
        else {
            self.count -= count;
            Some(Stack::new(self.id.clone(), count))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    //returns what was not used
    pub fn add(&mut self, stack: Stack) -> Option<Stack> {
        if stack.id != self.id {
            Some(stack)
        }
        else {
            self.count += stack.count;
            None
        }
        /* TODO: Decide if we want a stack size limit (no) */
    }
}