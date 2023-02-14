use bevy_ecs::prelude::*;

use crate::galaxy::{resources::{path_to_entity::PathToEntityMap, star_system_table::SystemMapTable}, components::*};

pub fn process_removals_star_system_table(removals: RemovedComponents<GameObject>, go: Query<&GameObject>, mut sys_table: ResMut<SystemMapTable>, mut path_table: ResMut<PathToEntityMap>) {
    for e in removals.iter() {
        let obj = go.get(e).expect("Removed game object not in query (THIS MEANS IT WAS REMOVED UNSAFELY, OR THAT THE OBJECT WAS ALREADY DUMPED (e.g. it was removed after the removal hooks stage, so the hooks didn't catch it)");
        super::path_table_bookeeping::path_table_removal_hook(e, obj, &mut path_table);
        super::star_system_table_bookeeping::star_system_table_removal_hook(e, &mut sys_table);
    }
}