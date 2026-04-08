use crate::ecs::{GameWorld, INTERACTABLE, InteractableKind};
use nightshade::ecs::picking::{PickingOptions, PickingResult, pick_entities};
use nightshade::prelude::*;

pub(super) fn try_start_interaction(
    game_world: &mut GameWorld,
    world: &mut World,
    pick_results: &[PickingResult],
) {
    let config = &game_world.resources.config;
    let max_grab_distance = config.max_grab_distance;

    for result in pick_results {
        let interactable_entities: Vec<freecs::Entity> =
            game_world.query_entities(INTERACTABLE).collect();
        for &ecs_entity in &interactable_entities {
            let Some(interactable) = game_world.get_interactable(ecs_entity) else {
                continue;
            };
            if interactable.engine_entity != result.entity {
                continue;
            }
            match &interactable.kind {
                InteractableKind::Grab => {
                    game_world.resources.interaction.grabbed_entity = Some(result.entity);
                    game_world.resources.interaction.grab_distance =
                        result.distance.min(max_grab_distance);

                    let local_offset = if let Some(rb) =
                        world.core.get_rigid_body(result.entity)
                        && let Some(handle) = rb.handle
                        && let Some(rigid_body) =
                            world.resources.physics.rigid_body_set.get(handle.into())
                    {
                        let body_pos = rigid_body.translation();
                        let body_rot = rigid_body.rotation();
                        let body_quat = nalgebra_glm::quat(body_rot.w, body_rot.i, body_rot.j, body_rot.k);
                        let world_offset = result.world_position
                            - nalgebra_glm::vec3(body_pos.x, body_pos.y, body_pos.z);
                        let inv_rot = nalgebra_glm::quat_conjugate(&body_quat);
                        nalgebra_glm::quat_rotate_vec3(&inv_rot, &world_offset)
                    } else {
                        nalgebra_glm::Vec3::zeros()
                    };

                    world.resources.physics.grab.grab(
                        result.entity,
                        result.distance,
                        game_world.resources.config.min_grab_distance,
                        game_world.resources.config.max_grab_distance,
                        local_offset,
                    );
                    return;
                }
                InteractableKind::Note => {
                    game_world.resources.ui.reading_note = Some(interactable.game_entity);
                    game_world.resources.interaction.require_interact_release = true;
                    return;
                }
                kind => {
                    game_world.resources.interaction.manipulated =
                        Some((interactable.game_entity, kind.clone()));
                    return;
                }
            }
        }
    }
}

pub fn update_interaction_prompt(game_world: &mut GameWorld, world: &mut World) {
    let Some(text_index) = game_world.resources.ui.interaction_prompt_text_index else {
        return;
    };
    let Some(prompt_entity) = game_world.resources.ui.interaction_prompt_entity else {
        return;
    };

    if game_world.resources.interaction.is_any_active()
        || game_world.resources.ui.reading_note.is_some()
    {
        world.resources.text_cache.set_text(text_index, "");
        if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
            hud_text.dirty = true;
        }
        return;
    }

    let Some(camera_entity) = game_world.resources.player.camera_entity else {
        return;
    };
    let Some(_camera_transform) = world.core.get_global_transform(camera_entity) else {
        return;
    };

    let viewport_size = world
        .resources
        .window
        .cached_viewport_size
        .unwrap_or((800, 600));
    let screen_pos =
        nalgebra_glm::vec2(viewport_size.0 as f32 / 2.0, viewport_size.1 as f32 / 2.0);

    let config = &game_world.resources.config;
    let options = PickingOptions {
        max_distance: config.grab_range,
        ignore_invisible: true,
    };

    let pick_results = if world.resources.input.input_mode == InputMode::Gamepad {
        pick_entities_cone(world, screen_pos, config.interact_cone_radius, options)
    } else {
        pick_entities(world, screen_pos, options)
    };

    let interactable_entities: Vec<freecs::Entity> =
        game_world.query_entities(INTERACTABLE).collect();

    let mut can_interact = false;
    let mut can_read = false;

    'outer: for result in &pick_results {
        for &ecs_entity in &interactable_entities {
            let Some(interactable) = game_world.get_interactable(ecs_entity) else {
                continue;
            };
            if interactable.engine_entity != result.entity {
                continue;
            }
            match &interactable.kind {
                InteractableKind::Note => {
                    can_read = true;
                    break 'outer;
                }
                InteractableKind::Grab => {}
                _ => {
                    can_interact = true;
                    break 'outer;
                }
            }
        }
    }

    let prompt_text = if can_read {
        "Read"
    } else if can_interact {
        "Interact"
    } else {
        ""
    };

    world.resources.text_cache.set_text(text_index, prompt_text);
    if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
        hud_text.dirty = true;
    }

    let crosshair_color = if can_interact || can_read {
        nalgebra_glm::Vec4::new(0.2, 1.0, 0.2, 0.9)
    } else {
        nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 0.7)
    };
    for &arm in &game_world.resources.ui.crosshair_arms {
        if let Some(color) = world.ui.get_ui_node_color_mut(arm) {
            color.colors[0] = Some(crosshair_color);
            color.computed_color = crosshair_color;
        }
    }
}

pub(super) fn pick_entities_cone(
    world: &World,
    center: Vec2,
    radius: f32,
    options: PickingOptions,
) -> Vec<PickingResult> {
    let mut all_results: Vec<PickingResult> = Vec::new();
    let mut seen_entities = std::collections::HashSet::new();

    let offsets = [
        (0.0, 0.0),
        (1.0, 0.0),
        (-1.0, 0.0),
        (0.0, 1.0),
        (0.0, -1.0),
        (0.707, 0.707),
        (-0.707, 0.707),
        (0.707, -0.707),
        (-0.707, -0.707),
        (0.5, 0.0),
        (-0.5, 0.0),
        (0.0, 0.5),
        (0.0, -0.5),
    ];

    for (offset_x, offset_y) in offsets {
        let screen_pos =
            nalgebra_glm::vec2(center.x + offset_x * radius, center.y + offset_y * radius);

        let results = pick_entities(world, screen_pos, options);
        for result in results {
            if !seen_entities.contains(&result.entity) {
                seen_entities.insert(result.entity);
                all_results.push(result);
            }
        }
    }

    all_results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    all_results
}

#[cfg(feature = "openxr")]
pub(super) fn pick_entities_from_ray(
    world: &World,
    origin: Vec3,
    direction: Vec3,
    options: PickingOptions,
) -> Vec<PickingResult> {
    let mut results = Vec::new();

    for entity in world
        .core
        .query_entities(nightshade::ecs::world::BOUNDING_VOLUME)
    {
        let Some(bounding_volume) = world.core.get_bounding_volume(entity) else {
            continue;
        };
        let Some(global_transform) = world.core.get_global_transform(entity) else {
            continue;
        };

        if options.ignore_invisible
            && let Some(visible) = world.core.get_visibility(entity)
            && !visible.visible
        {
            continue;
        }

        let transformed_bv = bounding_volume.transform(&global_transform.0);

        let to_center = transformed_bv.obb.center - origin;
        let projection = nalgebra_glm::dot(&to_center, &direction);
        let closest_point = if projection < 0.0 {
            origin
        } else {
            origin + direction * projection
        };

        let distance_to_sphere =
            nalgebra_glm::distance(&closest_point, &transformed_bv.obb.center);
        if distance_to_sphere > transformed_bv.sphere_radius {
            continue;
        }

        if let Some(distance) = transformed_bv.obb.intersect_ray(origin, direction)
            && distance <= options.max_distance
        {
            let world_position = origin + direction * distance;
            results.push(PickingResult {
                entity,
                distance,
                world_position,
            });
        }
    }

    results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    results
}
