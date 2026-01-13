use crate::state::{HorrorDemo, LeverAction};
use crate::systems::monster::start_cutscene;
use nightshade::prelude::*;

pub fn check_puzzle_state(demo: &mut HorrorDemo, world: &mut World) {
    let mut should_start_cutscene = false;

    for lever_index in 0..demo.levers.len() {
        let lever = &demo.levers[lever_index];
        let should_activate = lever.current_angle > 0.3;
        let action = lever.action.clone();
        let activated = lever.activated;
        let light_entity = lever.light_entity;
        let light_material_name = lever.light_material_name.clone();

        match action {
            LeverAction::RestorePower => {
                if should_activate && !activated {
                    demo.levers[lever_index].activated = true;
                    demo.power_restored = true;
                    if let Some(light) = world.get_light_mut(light_entity) {
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
                    if let Some(generator_entity) = demo.generator_audio_entity
                        && let Some(source) = world.get_audio_source_mut(generator_entity)
                    {
                        source.playing = true;
                    }
                }
            }
            LeverAction::UnlockExit => {
                if should_activate && demo.power_restored && !activated {
                    demo.levers[lever_index].activated = true;
                    demo.exit_unlocked = true;
                    if let Some(door) = demo.doors.last_mut() {
                        door.locked = false;
                        door.angular_velocity = -3.0;
                    }
                    if let Some(light) = world.get_light_mut(light_entity) {
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
        start_cutscene(demo, world);
    }

    if demo.exit_unlocked
        && !demo.game_won
        && !demo.cutscene.active
        && !demo.monster.active
        && let Some(player_entity) = demo.player_entity
        && let Some(transform) = world.get_local_transform(player_entity)
        && transform.translation.z < -24.0
    {
        demo.game_won = true;
    }
}
