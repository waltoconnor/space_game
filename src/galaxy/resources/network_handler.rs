use bevy_ecs::prelude::*;
use dashmap::DashMap;

use crate::network::messages::{incoming::NetIncomingMessage, outgoing::NetOutgoingMessage};

#[derive(Resource)]
pub struct NetworkHandler {
    incoming: DashMap<String, Vec<NetIncomingMessage>>,
    outgoing: DashMap<String, Vec<NetOutgoingMessage>>
}

impl NetworkHandler {
    pub fn new() -> Self {
        NetworkHandler { 
            incoming: DashMap::new(), 
            outgoing: DashMap::new() 
        }
    }

    pub fn enqueue_outgoing(&self, player: &String, message: NetOutgoingMessage) {
        self.outgoing.get_mut(player).and_then(|mut p| Some(p.push(message)));
    }

    pub fn finish_cycle(&mut self) -> DashMap<String, Vec<NetOutgoingMessage>> {
        std::mem::replace(&mut self.incoming, DashMap::new());
        std::mem::replace(&mut self.outgoing, DashMap::new())
    }

    pub fn queue_incoming(&self, player: &String, message: NetIncomingMessage) {
        self.incoming
            .get_mut(player)
            .and_then(|mut p| Some(p.push(message)));
    }

    pub fn view_incoming<'a>(&'a self) -> &'a DashMap<String, Vec<NetIncomingMessage>> {
        &self.incoming
    }

}