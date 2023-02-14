use bevy_ecs::{world::World, schedule::{Schedule, Stage}};
use dashmap::DashMap;

use crate::{db::database::DB, network::messages::{incoming::NetIncomingMessage, outgoing::NetOutgoingMessage}};

use self::{runner::{schedule::generate_schedule, init_resources::init_resources}, resources::network_handler::NetworkHandler};

pub mod components;
pub mod resources;
pub mod bundles;
mod runner;
mod systems;

pub struct Galaxy {
    pub world: World,
    schedule: Schedule
}

impl Galaxy {
    pub fn new(mut world: World, db: DB) -> Self {
        init_resources(&mut world, db);

        Galaxy {
            world,
            schedule: generate_schedule()
        }
    }

    pub fn tick(&mut self, dt: f64) {
        self.schedule.run(&mut self.world);
    }

    pub fn queue_incoming_message(&self, player: &String, msg: NetIncomingMessage) {
        self.world.get_resource::<NetworkHandler>().unwrap().queue_incoming(player, msg);
    }

    pub fn dump_outgoing_messages(&mut self) -> DashMap<String, Vec<NetOutgoingMessage>> {
        self.world.get_resource_mut::<NetworkHandler>().unwrap().finish_cycle()
    }
}