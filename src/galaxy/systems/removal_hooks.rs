use bevy_ecs::prelude::*;

use crate::galaxy::{resources::{path_to_entity::PathToEntityMap, star_system_table::SystemMapTable}, components::*};

pub fn process_removals_star_system_table(removals: RemovedComponents<GameObject>,  mut sys_table: ResMut<SystemMapTable>, mut path_table: ResMut<PathToEntityMap>) {
    for e in removals.iter() {
        super::path_table_bookeeping::path_table_removal_hook(e, &mut path_table);
        super::star_system_table_bookeeping::star_system_table_removal_hook(e, &mut sys_table);
    }
}