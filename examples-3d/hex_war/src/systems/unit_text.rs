use crate::ecs::{GameWorld, UNIT};
use nightshade::prelude::*;

pub fn unit_text_system(game_world: &GameWorld, world: &mut World) {
    for entity in game_world.query_entities(UNIT) {
        let Some(unit) = game_world.get_unit(entity) else {
            continue;
        };

        let Some(text_entity) = unit.text_entity else {
            continue;
        };

        let Some(text_index) = world.core.get_text(text_entity).map(|t| t.text_index) else {
            continue;
        };

        let new_text = unit.soldiers.to_string();
        let unchanged = world
            .resources
            .text_cache
            .get_text(text_index)
            .is_some_and(|t| t == new_text);
        if unchanged {
            continue;
        }

        world.resources.text_cache.set_text(text_index, new_text);

        if let Some(text) = world.core.get_text_mut(text_entity) {
            text.dirty = true;
        }
    }
}
