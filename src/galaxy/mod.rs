use bevy_ecs::{world::World, schedule::{Schedule, Stage}};
use dashmap::DashMap;

use crate::{db::database::DB, network::messages::{incoming::NetIncomingMessage, outgoing::NetOutgoingMessage}, inventory::ItemTable};

use self::{runner::{schedule::generate_schedule, init_resources::init_resources}, resources::{network_handler::NetworkHandler, delta_time::DeltaTime}};

pub mod components;
pub mod resources;
pub mod bundles;
pub mod events;
mod runner;
mod systems;

pub mod galaxy_map;

pub struct Galaxy {
    pub world: World,
    schedule: Schedule
}

impl Galaxy {
    pub fn new(mut world: World, db: DB, _item_table: ItemTable) -> Self {
        init_resources(&mut world, db);

        Galaxy {
            world,
            schedule: generate_schedule(),
        }
    }

    fn tick(&mut self, dt: f64) {
        self.world.get_resource_mut::<DeltaTime>().unwrap().dt = dt;
        self.schedule.run(&mut self.world);
    }

    pub fn queue_incoming_message(&self, player: &String, msg: NetIncomingMessage) {
        self.world.get_resource::<NetworkHandler>().unwrap().queue_incoming(player, msg);
    }

    fn dump_outgoing_messages(&mut self) -> DashMap<String, Vec<NetOutgoingMessage>> {
        self.world.get_resource_mut::<NetworkHandler>().unwrap().finish_cycle()
    }

    pub fn run_cycle(&mut self, dt: f64) -> DashMap<String, Vec<NetOutgoingMessage>> {
        self.tick(dt);
        self.world.clear_trackers();
        self.dump_outgoing_messages()
    }
}