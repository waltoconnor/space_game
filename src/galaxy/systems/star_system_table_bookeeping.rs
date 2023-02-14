use bevy_ecs::prelude::*;

use crate::galaxy::{components::*, resources::star_system_table::SystemMapTable};

pub fn update_star_system_table(q: Query<(Entity, &GameObject), Changed<GameObject>>, mut system_table: ResMut<SystemMapTable>) {
    for (e, go) in q.iter() {
        system_table.update_changed_entity(&go.path.sys, e);
    }
}

pub fn star_system_table_removal_hook(e: Entity, sys_table: &mut SystemMapTable) {
    sys_table.update_remove_entity(e);
}