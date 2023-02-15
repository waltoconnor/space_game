use bevy_ecs::{prelude::*, event::Event};

use crate::{db::database::DB, galaxy::events::{EEvent, EInfo, EState}};

use super::super::resources::*;

pub fn init_resources<'a>(world: &mut World, db: DB) {
    let path_table = path_to_entity::PathToEntityMap::new();
    let entity_table = star_system_table::SystemMapTable::new();
    let network_table = network_handler::NetworkHandler::new();
    let db_res = database_resource::DatabaseResource::new(db);
    let dt_res = delta_time::DeltaTime::new();

    world.insert_resource(path_table);
    world.insert_resource(entity_table);
    world.insert_resource(network_table);
    world.insert_resource(db_res);
    world.insert_resource(dt_res);
    world.init_resource::<Events<EEvent>>();
    world.init_resource::<Events<EInfo>>();
    world.init_resource::<Events<EState>>();
}