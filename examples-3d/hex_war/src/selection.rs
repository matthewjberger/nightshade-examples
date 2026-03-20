use crate::ecs::{Entity, GameWorld};

pub fn get_selected_unit(game_world: &GameWorld) -> Option<Entity> {
    game_world.query_selected().next()
}

pub fn select_unit(game_world: &mut GameWorld, unit_entity: Entity) {
    clear_selection(game_world);
    game_world.add_selected(unit_entity);
}

pub fn clear_selection(game_world: &mut GameWorld) {
    let selected_entities: Vec<_> = game_world.query_selected().collect();
    for selected in selected_entities {
        game_world.remove_selected(selected);
    }
    game_world.resources.valid_move_tiles.clear();
}
