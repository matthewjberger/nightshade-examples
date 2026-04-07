use crate::ecs::{GameWorld, Health, TargetKilledEvent, SHOT_BAUBLE, TARGET};
use nightshade::ecs::visibility::components::Visibility;
use nightshade::prelude::*;

use super::effects::spawn_pop_effect;

pub(crate) fn update_healthbar(world: &mut World, health: &Health) {
    let fraction = health.fraction();

    if let Some(transform) = world.core.get_local_transform_mut(health.fill_entity) {
        transform.scale.x = fraction.max(0.001);
        transform.translation.x = -(1.0 - fraction) * 0.5;
    }
    mark_local_transform_dirty(world, health.fill_entity);

    let fill_material_name = format!("HealthBarFill_{}", health.fill_entity.id);
    let color = if fraction > 0.6 {
        nalgebra_glm::vec3(0.2, 0.9, 0.2)
    } else if fraction > 0.3 {
        let transition = (fraction - 0.3) / 0.3;
        nalgebra_glm::lerp(
            &nalgebra_glm::vec3(0.9, 0.6, 0.1),
            &nalgebra_glm::vec3(0.2, 0.9, 0.2),
            transition,
        )
    } else {
        let transition = fraction / 0.3;
        nalgebra_glm::lerp(
            &nalgebra_glm::vec3(0.9, 0.1, 0.1),
            &nalgebra_glm::vec3(0.9, 0.6, 0.1),
            transition,
        )
    };

    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&fill_material_name)
        && let Some(Some(material)) = world
            .resources
            .material_registry
            .registry
            .entries
            .get_mut(index as usize)
    {
        material.base_color = [color.x, color.y, color.z, 1.0];
    }
}

pub(crate) fn update_targets(game_world: &mut GameWorld, world: &mut World) {
    let current_time = world.resources.window.timing.uptime_milliseconds;
    let delta_time = world.resources.window.timing.delta_time;

    let baubles: Vec<(freecs::Entity, Entity, Vec3)> = game_world
        .query_entities(SHOT_BAUBLE)
        .filter_map(|game_entity| {
            let bauble = game_world.get_shot_bauble(game_entity)?;
            let position = world
                .core
                .get_global_transform(bauble.entity)
                .map(|transform| transform.translation())?;
            Some((game_entity, bauble.entity, position))
        })
        .collect();

    let mut consumed_baubles: Vec<(freecs::Entity, Entity)> = Vec::new();

    let target_entities: Vec<freecs::Entity> = game_world.query_entities(TARGET).collect();

    for game_entity in target_entities {
        let Some(target) = game_world.get_target(game_entity) else {
            continue;
        };

        if target.popped {
            if current_time.saturating_sub(target.pop_time_ms) >= target.respawn_delay_ms {
                let entity = target.entity;
                let position = target.position;
                let base_scale = target.base_scale;
                let bar_entity = target.health.bar_entity;
                let max_health = target.health.max;

                let emitter_to_despawn = game_world
                    .get_target(game_entity)
                    .and_then(|target| target.pop_emitter_entity);

                if let Some(target) = game_world.get_target_mut(game_entity) {
                    target.popped = false;
                    target.health.current = max_health;
                    target.pop_emitter_entity = None;
                }

                if let Some(emitter) = emitter_to_despawn {
                    despawn_entities_with_cache_cleanup(world, &[emitter]);
                }

                world
                    .core
                    .set_visibility(entity, Visibility { visible: true });
                world
                    .core
                    .set_visibility(bar_entity, Visibility { visible: true });

                let fill_entity = game_world
                    .get_target(game_entity)
                    .map(|target| target.health.fill_entity)
                    .unwrap_or_default();
                world
                    .core
                    .set_visibility(fill_entity, Visibility { visible: true });

                if let Some(transform) = world.core.get_local_transform_mut(entity) {
                    transform.translation = position;
                    transform.scale = nalgebra_glm::vec3(base_scale, base_scale, base_scale);
                }
                mark_local_transform_dirty(world, entity);

                if let Some(target) = game_world.get_target(game_entity) {
                    update_healthbar(world, &target.health);
                }
            }
            continue;
        }

        let entity = target.entity;
        let base_scale = target.base_scale;
        let phase = target.pulse_phase;
        let color = target.color;
        let saved_position = target.position;

        if let Some(target) = game_world.get_target_mut(game_entity) {
            target.pulse_phase += delta_time * 3.0;
        }

        let pulse = 1.0 + 0.15 * (phase + delta_time * 3.0).sin();
        let current_scale = base_scale * pulse;
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.scale = nalgebra_glm::vec3(current_scale, current_scale, current_scale);
        }

        let target_position = world
            .core
            .get_global_transform(entity)
            .map(|transform| transform.translation())
            .unwrap_or(saved_position);

        let hit_radius = base_scale * 1.5;
        let mut hit_count = 0;
        for &(bauble_game_entity, bauble_engine_entity, bauble_position) in &baubles {
            if consumed_baubles.iter().any(|(ge, _)| *ge == bauble_game_entity) {
                continue;
            }
            if nalgebra_glm::distance(&target_position, &bauble_position) < hit_radius {
                consumed_baubles.push((bauble_game_entity, bauble_engine_entity));
                hit_count += 1;
            }
        }

        if hit_count > 0
            && let Some(target) = game_world.get_target_mut(game_entity)
        {
            target.health.damage(hit_count as f32);
        }

        let is_dead = game_world
            .get_target(game_entity)
            .is_some_and(|target| target.health.is_dead());

        if is_dead {
            if let Some(target) = game_world.get_target_mut(game_entity) {
                target.popped = true;
                target.pop_time_ms = current_time;
            }

            game_world.send_target_killed(TargetKilledEvent {
                game_entity,
                position: target_position,
                color,
            });
        } else if hit_count > 0
            && let Some(target) = game_world.get_target(game_entity)
        {
            update_healthbar(world, &target.health);
        }

        if let Some(target) = game_world.get_target(game_entity)
            && !target.popped
        {
            let bar_position = nalgebra_glm::vec3(
                target_position.x,
                target_position.y + base_scale + 0.15,
                target_position.z,
            );

            let camera_position = game_world
                .resources
                .player
                .camera_entity
                .and_then(|camera| world.core.get_global_transform(camera))
                .map(|transform| transform.translation())
                .unwrap_or(Vec3::zeros());

            let to_camera = camera_position - bar_position;
            let yaw = to_camera.x.atan2(to_camera.z);

            if let Some(transform) =
                world.core.get_local_transform_mut(target.health.bar_entity)
            {
                transform.translation = bar_position;
                transform.rotation = nalgebra_glm::quat_angle_axis(
                    yaw,
                    &nalgebra_glm::vec3(0.0, 1.0, 0.0),
                );
            }
            mark_local_transform_dirty(world, target.health.bar_entity);
        }
    }

    for (bauble_game_entity, bauble_engine_entity) in consumed_baubles {
        crate::systems::shooting::despawn_bauble_public(game_world, world, bauble_engine_entity);
        game_world.despawn_entities(&[bauble_game_entity]);
    }
}

pub(crate) fn process_target_killed_events(game_world: &mut GameWorld, world: &mut World) {
    let events: Vec<TargetKilledEvent> = game_world.drain_target_killed().collect();

    for event in events {
        let entity = game_world
            .get_target(event.game_entity)
            .map(|target| target.entity);
        let bar_entity = game_world
            .get_target(event.game_entity)
            .map(|target| target.health.bar_entity);
        let fill_entity = game_world
            .get_target(event.game_entity)
            .map(|target| target.health.fill_entity);

        if let Some(entity) = entity {
            world
                .core
                .set_visibility(entity, Visibility { visible: false });
        }

        if let Some(bar_entity) = bar_entity {
            world
                .core
                .set_visibility(bar_entity, Visibility { visible: false });
        }

        if let Some(fill_entity) = fill_entity {
            world
                .core
                .set_visibility(fill_entity, Visibility { visible: false });
        }

        let emitter = spawn_pop_effect(world, event.position, event.color);
        if let Some(target) = game_world.get_target_mut(event.game_entity) {
            target.pop_emitter_entity = Some(emitter);
        }
    }
}
