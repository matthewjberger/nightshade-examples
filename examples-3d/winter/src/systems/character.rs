use crate::constants::{FOX_MODEL, FOX_SCALE, SANTA_HAT_MODEL};
use crate::ecs::{GameWorld, MovementState};
use crate::systems::environment::sample_height;
use nightshade::ecs::animation::components::{AnimationClip, AnimationProperty};
use nightshade::ecs::physics::CharacterControllerComponent;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::prelude::*;

pub fn spawn_character_controller(game_world: &mut GameWorld, world: &mut World) {
    let controller_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | CHARACTER_CONTROLLER,
        1,
    )[0];

    world
        .core
        .set_name(controller_entity, Name("Fox Controller".to_string()));
    let spawn_terrain_y = sample_height(0.0, 0.0, &game_world.resources.terrain_config);
    world.core.set_local_transform(
        controller_entity,
        LocalTransform {
            translation: Vec3::new(0.0, spawn_terrain_y + 1.0, 0.0),
            ..Default::default()
        },
    );

    if let Some(controller) = world.core.get_character_controller_mut(controller_entity) {
        *controller = CharacterControllerComponent::new_capsule(0.5, 0.3);
        controller.max_speed = 3.0;
        controller.acceleration = 15.0;
        controller.jump_impulse = 4.0;
        controller.sprint_speed_multiplier = 2.0;
        controller.crouch_enabled = false;
    }

    game_world.resources.controller_entity = Some(freecs::Entity {
        id: controller_entity.id,
        generation: controller_entity.generation,
    });
}

pub fn load_fox_model(game_world: &mut GameWorld, world: &mut World) {
    tracing::info!("Loading fox model");
    let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(FOX_MODEL);

    match load_result {
        Ok(result) => {
            tracing::info!("Successfully loaded fox model");
            tracing::info!("Loaded {} animations", result.animations.len());

            for (index, anim) in result.animations.iter().enumerate() {
                tracing::info!("Animation {}: {}", index, anim.name);
                let name_lower = anim.name.to_lowercase();
                if name_lower.contains("survey") {
                    game_world.resources.animation_indices.survey = Some(index);
                } else if name_lower.contains("walk") {
                    game_world.resources.animation_indices.walk = Some(index);
                } else if name_lower.contains("run") {
                    game_world.resources.animation_indices.run = Some(index);
                }
            }

            for (name, (rgba_data, width, height)) in result.textures {
                world.queue_command(WorldCommand::LoadTexture {
                    name,
                    rgba_data,
                    width,
                    height,
                });
            }

            for (name, mesh) in result.meshes {
                mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
            }

            let root_bone_indices: std::collections::HashSet<usize> = [0, 1, 2, 3].into();
            let filtered_animations: Vec<AnimationClip> = result
                .animations
                .iter()
                .map(|clip| AnimationClip {
                    name: clip.name.clone(),
                    duration: clip.duration,
                    channels: clip
                        .channels
                        .iter()
                        .filter(|channel| {
                            if channel.target_property == AnimationProperty::Translation {
                                return false;
                            }
                            if root_bone_indices.contains(&channel.target_node)
                                && channel.target_property == AnimationProperty::Rotation
                            {
                                return false;
                            }
                            true
                        })
                        .cloned()
                        .collect(),
                })
                .collect();

            for prefab in result.prefabs {
                let entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
                    world,
                    &prefab,
                    &filtered_animations,
                    &result.skins,
                    Vec3::zeros(),
                );

                game_world.resources.fox_entity = Some(freecs::Entity {
                    id: entity.id,
                    generation: entity.generation,
                });
                tracing::info!("Spawned fox with root entity {:?}", entity);

                if let Some(transform) = world.core.get_local_transform_mut(entity) {
                    transform.scale = Vec3::new(FOX_SCALE, FOX_SCALE, FOX_SCALE);
                }
                world.mark_local_transform_dirty(entity);

                let bone_entities: Vec<Entity> =
                    if let Some(player) = world.core.get_animation_player(entity) {
                        player.node_index_to_entity.values().copied().collect()
                    } else {
                        Vec::new()
                    };

                for bone_entity in bone_entities {
                    if let Some(name) = world.core.get_name(bone_entity)
                        && name.0.to_lowercase().contains("head")
                    {
                        game_world.resources.head_bone_entity = Some(freecs::Entity {
                            id: bone_entity.id,
                            generation: bone_entity.generation,
                        });
                        break;
                    }
                }

                if let Some(player) = world.core.get_animation_player_mut(entity) {
                    if let Some(survey_index) = game_world.resources.animation_indices.survey {
                        player.play(survey_index);
                        player.speed = 0.5;
                        game_world.resources.current_animation = Some(survey_index);
                    } else if !player.clips.is_empty() {
                        player.play(0);
                        player.speed = 0.5;
                        game_world.resources.current_animation = Some(0);
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to load fox model: {}", e);
        }
    }

    load_santa_hat(game_world, world);
}

pub fn load_santa_hat(game_world: &mut GameWorld, world: &mut World) {
    let Some(head_bone) = game_world.resources.head_bone_entity else {
        return;
    };

    let head_entity = nightshade::prelude::Entity {
        id: head_bone.id,
        generation: head_bone.generation,
    };

    let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(SANTA_HAT_MODEL);

    match load_result {
        Ok(result) => {
            for (name, (rgba_data, width, height)) in result.textures {
                world.queue_command(WorldCommand::LoadTexture {
                    name,
                    rgba_data,
                    width,
                    height,
                });
            }

            for (name, mesh) in result.meshes {
                mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
            }

            for prefab in result.prefabs {
                let hat_entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
                    world,
                    &prefab,
                    &result.animations,
                    &result.skins,
                    Vec3::zeros(),
                );

                game_world.resources.santa_hat_entity = Some(freecs::Entity {
                    id: hat_entity.id,
                    generation: hat_entity.generation,
                });

                if let Some(transform) = world.core.get_local_transform_mut(hat_entity) {
                    transform.translation = Vec3::new(0.0, 10.0, 0.0);
                    transform.scale = Vec3::new(1.2, 1.2, 1.2);
                    transform.rotation =
                        nalgebra_glm::quat_angle_axis(std::f32::consts::PI * 0.5, &Vec3::y());
                }

                world.update_parent(hat_entity, Some(Parent(Some(head_entity))));
            }
        }
        Err(e) => {
            tracing::error!("Failed to load santa hat model: {}", e);
        }
    }
}

pub fn sync_fox_to_controller(game_world: &mut GameWorld, world: &mut World) {
    let Some(fox_entity) = game_world.resources.fox_entity else {
        return;
    };
    let Some(controller_entity) = game_world.resources.controller_entity else {
        return;
    };

    let engine_fox = nightshade::prelude::Entity {
        id: fox_entity.id,
        generation: fox_entity.generation,
    };
    let engine_controller = nightshade::prelude::Entity {
        id: controller_entity.id,
        generation: controller_entity.generation,
    };

    let controller_pos = world
        .core
        .get_local_transform(engine_controller)
        .map(|t| t.translation)
        .unwrap_or(Vec3::zeros());

    let target_fox_pos = Vec3::new(controller_pos.x, controller_pos.y - 0.8, controller_pos.z);

    let delta_time = world.resources.window.timing.delta_time;

    if !game_world.resources.fox_position_initialized {
        game_world.resources.smoothed_fox_position = target_fox_pos;
        game_world.resources.fox_position_initialized = true;
    } else {
        let position_lerp_speed = 30.0;
        let t = (position_lerp_speed * delta_time).min(1.0);
        game_world.resources.smoothed_fox_position = game_world.resources.smoothed_fox_position
            + (target_fox_pos - game_world.resources.smoothed_fox_position) * t;
    }

    let (velocity, grounded, is_sprinting) =
        if let Some(controller) = world.core.get_character_controller(engine_controller) {
            (
                controller.velocity,
                controller.grounded,
                controller.is_sprinting,
            )
        } else {
            return;
        };

    let horizontal_speed = (velocity.x.powi(2) + velocity.z.powi(2)).sqrt();
    let start_threshold = 0.5;
    let stop_threshold = 0.2;
    let has_movement = if game_world.resources.was_moving {
        horizontal_speed > stop_threshold
    } else {
        horizontal_speed > start_threshold
    };
    game_world.resources.was_moving = has_movement;

    if has_movement {
        let target_rotation = velocity.x.atan2(velocity.z);
        let rotation_speed = 10.0;

        let mut angle_diff = target_rotation - game_world.resources.fox_rotation;
        while angle_diff > std::f32::consts::PI {
            angle_diff -= 2.0 * std::f32::consts::PI;
        }
        while angle_diff < -std::f32::consts::PI {
            angle_diff += 2.0 * std::f32::consts::PI;
        }
        game_world.resources.fox_rotation += angle_diff * rotation_speed * delta_time;
    }

    game_world.resources.movement_state = if !grounded || (is_sprinting && has_movement) {
        MovementState::Running
    } else if has_movement {
        MovementState::Walking
    } else {
        MovementState::Idle
    };

    if let Some(transform) = world.mutate_local_transform(engine_fox) {
        transform.translation = game_world.resources.smoothed_fox_position;
        transform.rotation =
            nalgebra_glm::quat_angle_axis(game_world.resources.fox_rotation, &Vec3::y());
    }
}

pub fn animation_system(game_world: &mut GameWorld, world: &mut World) {
    let Some(fox_entity) = game_world.resources.fox_entity else {
        return;
    };

    let engine_entity = nightshade::prelude::Entity {
        id: fox_entity.id,
        generation: fox_entity.generation,
    };

    let movement_state = game_world.resources.movement_state;

    let (target_animation, target_speed) = match movement_state {
        MovementState::Idle => (game_world.resources.animation_indices.survey, 0.5),
        MovementState::Walking => (game_world.resources.animation_indices.walk, 1.0),
        MovementState::Running => (game_world.resources.animation_indices.run, 1.0),
    };

    if target_animation != game_world.resources.current_animation {
        if let Some(anim_index) = target_animation
            && let Some(player) = world.core.get_animation_player_mut(engine_entity)
        {
            player.blend_to(anim_index, 0.2);
            player.speed = target_speed;
            game_world.resources.current_animation = Some(anim_index);
        }
    } else if let Some(player) = world.core.get_animation_player_mut(engine_entity) {
        player.speed = target_speed;
    }
}
