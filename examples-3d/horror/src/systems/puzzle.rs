use crate::ecs::{ENGINE_ENTITY, GameWorld, LEVER, LeverAction};
use crate::systems::monster::start_cutscene;
use nightshade::ecs::world::commands::find_entity_by_name;
use nightshade::prelude::*;

pub fn check_puzzle_state(game_world: &mut GameWorld, world: &mut World) {
    let mut should_start_cutscene = false;

    let lever_entities: Vec<freecs::Entity> =
        game_world.query_entities(LEVER | ENGINE_ENTITY).collect();

    let has_unlock_exit_lever = lever_entities.iter().any(|&game_entity| {
        game_world
            .get_lever(game_entity)
            .is_some_and(|lever| lever.action == LeverAction::UnlockExit)
    });

    for game_entity in lever_entities {
        let Some(lever) = game_world.get_lever(game_entity) else {
            continue;
        };
        let should_activate = lever.current_angle > 0.3;
        let action = lever.action;
        let activated = lever.activated;
        let light_entity = lever.light_entity;
        let light_material_name = lever.light_material_name.clone();

        if !should_activate || activated {
            continue;
        }

        match action {
            LeverAction::RestorePower => {
                game_world.get_lever_mut(game_entity).unwrap().activated = true;
                game_world.resources.power_restored = true;
                activate_lever_light(world, light_entity, &light_material_name);
                if let Some(generator_entity) = game_world.resources.generator_audio_entity
                    && let Some(source) = world.core.get_audio_source_mut(generator_entity)
                {
                    source.playing = true;
                }
            }
            LeverAction::UnlockExit => {
                if !game_world.resources.power_restored {
                    continue;
                }
                game_world.get_lever_mut(game_entity).unwrap().activated = true;
                game_world.resources.exit_unlocked = true;
                if let Some(exit_door_game_entity) = game_world.resources.exit_door
                    && let Some(door) = game_world.get_door_mut(exit_door_game_entity)
                {
                    door.locked = false;
                    door.angular_velocity = -3.0;
                }
                activate_lever_light(world, light_entity, &light_material_name);
                should_start_cutscene = true;
            }
        }
    }

    if should_start_cutscene && has_unlock_exit_lever {
        start_cutscene(game_world, world);
    }

    if game_world.resources.exit_unlocked
        && !game_world.resources.game_won
        && !game_world.resources.cutscene.active
        && !game_world.resources.monster.active
        && let Some(player_entity) = game_world.resources.player_entity
        && let Some(transform) = world.core.get_local_transform(player_entity)
    {
        let at_exit_zone = if let Some(exit_zone_entity) = find_entity_by_name(world, "ExitZone") {
            if let Some(zone_transform) = world.core.get_local_transform(exit_zone_entity) {
                let distance =
                    nalgebra_glm::distance(&transform.translation, &zone_transform.translation);
                distance < 3.0
            } else {
                false
            }
        } else {
            transform.translation.z < -24.0
        };

        if at_exit_zone {
            game_world.resources.game_won = true;
        }
    }
}

fn activate_lever_light(world: &mut World, light_entity: Entity, material_name: &str) {
    if let Some(light) = world.core.get_light_mut(light_entity) {
        light.color = nalgebra_glm::vec3(0.2, 1.0, 0.2);
        light.intensity = 3.0;
    }
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(material_name)
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
}
