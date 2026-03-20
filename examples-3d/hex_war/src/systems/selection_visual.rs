use crate::ecs::GameWorld;
use crate::systems::UNIT_SELECTED_COLOR;
use nightshade::ecs::generational_registry::registry_entry_by_name_mut;
use nightshade::prelude::*;

pub fn selection_visual_system(game_world: &GameWorld, world: &mut World) {
    let current_selected: Option<freecs::Entity> = game_world.query_selected().next();
    let previous_selected = game_world.resources.frame_cache.previous_selected_unit;

    if current_selected == previous_selected {
        return;
    }

    if let Some(prev_entity) = previous_selected
        && let Some(unit) = game_world.get_unit(prev_entity)
        && let Some(engine_entity) = game_world.get_engine_entity(prev_entity)
        && let Some(material_ref) = world.core.get_material_ref(engine_entity.0)
    {
        let name = material_ref.name.clone();
        if let Some(material) =
            registry_entry_by_name_mut(&mut world.resources.material_registry.registry, &name)
        {
            material.base_color = unit.faction.color();
        }
    }

    if let Some(curr_entity) = current_selected
        && let Some(engine_entity) = game_world.get_engine_entity(curr_entity)
        && let Some(material_ref) = world.core.get_material_ref(engine_entity.0)
    {
        let name = material_ref.name.clone();
        if let Some(material) =
            registry_entry_by_name_mut(&mut world.resources.material_registry.registry, &name)
        {
            material.base_color = UNIT_SELECTED_COLOR;
        }
    }
}
