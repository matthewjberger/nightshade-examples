use crate::ecs::{ENGINE_ENTITY, GameWorld, LEVER, LeverAction};
use crate::systems::monster::start_cutscene;
use nightshade::prelude::*;

pub fn check_puzzle_state(game_world: &mut GameWorld, world: &mut World) {
    let mut should_start_cutscene = false;

    let lever_entities: Vec<freecs::Entity> =
        game_world.query_entities(LEVER | ENGINE_ENTITY).collect();

    for game_entity in lever_entities {
        let Some(lever) = game_world.get_lever(game_entity) else {
            continue;
        };
        let should_activate = lever.current_angle > 0.3;
        let action = lever.action;
        let activated = lever.activated;
        let light_entity = lever.light_entity;
        let light_material_name = lever.light_material_name.clone();

        match action {
            LeverAction::RestorePower => {
                if should_activate && !activated {
                    game_world.get_lever_mut(game_entity).unwrap().activated = true;
                    game_world.resources.power_restored = true;
                    if let Some(light) = world.core.get_light_mut(light_entity) {
                        light.color = nalgebra_glm::vec3(0.2, 1.0, 0.2);
                        light.intensity = 3.0;
                    }
                    if let Some(&index) = world
                        .resources
                        .material_registry
                        .registry
                        .name_to_index
                        .get(&light_material_name)
                        && let Some(Some(material)) = world
                            .resources
                            .material_registry
                            .registry
                            .entries
                            .get_mut(index as usize)
                    {
                        material.base_color = [0.2, 1.0, 0.2, 1.0];
                        material.emissive_factor = [0.2, 1.0, 0.2];
                    }
                    if let Some(generator_entity) = game_world.resources.generator_audio_entity
                        && let Some(source) = world.core.get_audio_source_mut(generator_entity)
                    {
                        source.playing = true;
                    }
                }
            }
            LeverAction::UnlockExit => {
                if should_activate && game_world.resources.power_restored && !activated {
                    game_world.get_lever_mut(game_entity).unwrap().activated = true;
                    game_world.resources.exit_unlocked = true;
                    if let Some(exit_door_game_entity) = game_world.resources.exit_door
                        && let Some(door) = game_world.get_door_mut(exit_door_game_entity)
                    {
                        door.locked = false;
                        door.angular_velocity = -3.0;
                    }
                    if let Some(light) = world.core.get_light_mut(light_entity) {
                        light.color = nalgebra_glm::vec3(0.2, 1.0, 0.2);
                        light.intensity = 3.0;
                    }
                    if let Some(&index) = world
                        .resources
                        .material_registry
                        .registry
                        .name_to_index
                        .get(&light_material_name)
                        && let Some(Some(material)) = world
                            .resources
                            .material_registry
                            .registry
                            .entries
                            .get_mut(index as usize)
                    {
                        material.base_color = [0.2, 1.0, 0.2, 1.0];
                        material.emissive_factor = [0.2, 1.0, 0.2];
                    }
                    should_start_cutscene = true;
                }
            }
        }
    }

    if should_start_cutscene {
        start_cutscene(game_world, world);
    }

    if game_world.resources.exit_unlocked
        && !game_world.resources.game_won
        && !game_world.resources.cutscene.active
        && !game_world.resources.monster.active
        && let Some(player_entity) = game_world.resources.player_entity
        && let Some(transform) = world.core.get_local_transform(player_entity)
        && transform.translation.z < -24.0
    {
        game_world.resources.game_won = true;
    }
}
