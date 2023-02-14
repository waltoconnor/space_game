use bevy_ecs::system::Resource;

use crate::db::database::DB;


#[derive(Resource)]
pub struct DatabaseResource {
    pub db: DB
}

impl DatabaseResource {
    pub fn new(db: DB) -> Self {
        DatabaseResource { db }
    } 
}