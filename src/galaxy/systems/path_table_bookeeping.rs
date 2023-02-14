use bevy_ecs::prelude::*;

use crate::galaxy::{resources::path_to_entity::PathToEntityMap, components::*};

pub fn update_path_table(q: Query<(Entity, &GameObject), Changed<GameObject>>, mut path_table: ResMut<PathToEntityMap>) {
    for (e, go) in q.iter() {
        path_table.update(&go.path, e);
    }
}

pub fn path_table_removal_hook(_e: Entity, obj: &GameObject, path_table: &mut PathToEntityMap) {
    path_table.remove(&obj.path);
}