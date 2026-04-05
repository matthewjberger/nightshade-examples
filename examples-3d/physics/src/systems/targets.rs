use crate::ecs::{GameWorld, Health, Target, SHOT_BAUBLE, TARGET};
use nightshade::ecs::particles::components::{ColorGradient, EmitterShape, ParticleEmitter};
use nightshade::ecs::transform::components::Parent;
use nightshade::ecs::visibility::components::Visibility;
use nightshade::prelude::*;

pub fn spawn_targets(game_world: &mut GameWorld, world: &mut World) {
    let targets: &[(Vec3, Vec3, f32, f32, &str)] = &[
        (nalgebra_glm::vec3(-6.0, 3.0, -6.0), nalgebra_glm::vec3(0.9, 0.3, 0.3), 0.25, 3.0, "Sphere"),
        (nalgebra_glm::vec3(6.0, 4.5, -6.0), nalgebra_glm::vec3(0.3, 0.9, 0.3), 0.3, 5.0, "Sphere"),
        (nalgebra_glm::vec3(0.0, 5.0, -10.0), nalgebra_glm::vec3(0.3, 0.5, 0.9), 0.4, 8.0, "Cube"),
        (nalgebra_glm::vec3(-8.0, 2.5, 0.0), nalgebra_glm::vec3(0.9, 0.7, 0.2), 0.2, 2.0, "Sphere"),
        (nalgebra_glm::vec3(8.0, 3.5, 0.0), nalgebra_glm::vec3(0.7, 0.3, 0.9), 0.35, 6.0, "Cube"),
        (nalgebra_glm::vec3(-3.0, 6.0, -3.0), nalgebra_glm::vec3(0.2, 0.8, 0.8), 0.2, 2.0, "Sphere"),
        (nalgebra_glm::vec3(3.0, 4.0, 3.0), nalgebra_glm::vec3(0.9, 0.4, 0.6), 0.45, 10.0, "Cube"),
        (nalgebra_glm::vec3(0.0, 3.0, 6.0), nalgebra_glm::vec3(0.8, 0.8, 0.3), 0.3, 4.0, "Sphere"),
        (nalgebra_glm::vec3(-5.0, 5.5, 5.0), nalgebra_glm::vec3(0.4, 0.6, 0.9), 0.25, 3.0, "Sphere"),
        (nalgebra_glm::vec3(5.0, 2.0, -3.0), nalgebra_glm::vec3(0.9, 0.5, 0.3), 0.5, 12.0, "Cylinder"),
    ];

    for &(position, color, scale, max_health, mesh_name) in targets {
        let entity = spawn_target_mesh(world, position, scale, color, mesh_name);
        let (bar_entity, fill_entity) = spawn_healthbar(world, position);

        let game_entity = game_world.spawn_entities(TARGET, 1)[0];
        game_world.set_target(
            game_entity,
            Target {
                entity,
                position,
                base_scale: scale,
                color,
                health: Health::new(max_health, bar_entity, fill_entity),
                popped: false,
                pop_time_ms: 0,
                respawn_delay_ms: 3000,
                pulse_phase: position.x * 2.0 + position.z,
                pop_emitter_entity: None,
            },
        );
    }
}

fn spawn_target_mesh(
    world: &mut World,
    position: Vec3,
    scale: f32,
    color: Vec3,
    mesh_name: &str,
) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(entity) {
        name.0 = "Target".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = nalgebra_glm::vec3(scale, scale, scale);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = mesh_name.to_string();
    }

    let material_name = format!("Target_{}", entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        nightshade::ecs::material::components::Material {
            base_color: [color.x, color.y, color.z, 1.0],
            emissive_factor: [color.x * 2.0, color.y * 2.0, color.z * 2.0],
            roughness: 0.4,
            metallic: 0.6,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
        *bounding_volume = BoundingVolume::from_mesh_type(mesh_name);
    }

    world.resources.mesh_render_state.mark_entity_added(entity);

    entity
}

fn spawn_healthbar(world: &mut World, position: Vec3) -> (Entity, Entity) {
    let bar_width = 0.8;
    let bar_height = 0.08;
    let bar_y_offset = 0.6;

    let background = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | RENDER_MESH
            | MATERIAL_REF | BOUNDING_VOLUME | VISIBILITY,
        1,
    )[0];

    if let Some(transform) = world.core.get_local_transform_mut(background) {
        transform.translation = nalgebra_glm::vec3(position.x, position.y + bar_y_offset, position.z);
        transform.scale = nalgebra_glm::vec3(bar_width, bar_height, 0.02);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(background) {
        mesh.name = "Cube".to_string();
    }

    let bg_material_name = format!("HealthBarBg_{}", background.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        bg_material_name.clone(),
        create_textured_material(nalgebra_glm::vec3(0.1, 0.1, 0.1), 0.9, 0.0),
    );
    if let Some(&index) = world.resources.material_registry.registry.name_to_index.get(&bg_material_name) {
        world.resources.material_registry.registry.add_reference(index);
    }
    world.core.set_material_ref(background, MaterialRef::new(bg_material_name));

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(background) {
        *bounding_volume = BoundingVolume::from_mesh_type("Cube");
    }

    world.resources.mesh_render_state.mark_entity_added(background);

    let fill = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | RENDER_MESH
            | MATERIAL_REF | BOUNDING_VOLUME | VISIBILITY | PARENT,
        1,
    )[0];

    if let Some(transform) = world.core.get_local_transform_mut(fill) {
        transform.translation = nalgebra_glm::vec3(0.0, 0.0, 0.01);
        transform.scale = nalgebra_glm::vec3(1.0, 1.0, 1.1);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(fill) {
        mesh.name = "Cube".to_string();
    }

    let fill_material_name = format!("HealthBarFill_{}", fill.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        fill_material_name.clone(),
        create_textured_material(nalgebra_glm::vec3(0.2, 0.9, 0.2), 0.8, 0.0),
    );
    if let Some(&index) = world.resources.material_registry.registry.name_to_index.get(&fill_material_name) {
        world.resources.material_registry.registry.add_reference(index);
    }
    world.core.set_material_ref(fill, MaterialRef::new(fill_material_name));

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(fill) {
        *bounding_volume = BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(parent) = world.core.get_parent_mut(fill) {
        *parent = Parent(Some(background));
    }

    world.resources.mesh_render_state.mark_entity_added(fill);

    (background, fill)
}

pub fn update_healthbar(world: &mut World, health: &Health) {
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

pub fn update_targets(game_world: &mut GameWorld, world: &mut World) {
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

            world
                .core
                .set_visibility(entity, Visibility { visible: false });

            if let Some(target) = game_world.get_target(game_entity) {
                world
                    .core
                    .set_visibility(target.health.bar_entity, Visibility { visible: false });
                world
                    .core
                    .set_visibility(target.health.fill_entity, Visibility { visible: false });
            }

            let emitter = spawn_pop_effect(world, target_position, color);
            if let Some(target) = game_world.get_target_mut(game_entity) {
                target.pop_emitter_entity = Some(emitter);
            }
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
        super::shooting::despawn_bauble_public(game_world, world, bauble_engine_entity);
        game_world.despawn_entities(&[bauble_game_entity]);
    }
}

fn spawn_pop_effect(world: &mut World, position: Vec3, color: Vec3) -> Entity {
    let entity = world.spawn_entities(
        PARTICLE_EMITTER | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
        1,
    )[0];

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
    }

    let mut emitter = ParticleEmitter::firework_explosion(position, color, 30);
    emitter.shape = EmitterShape::Sphere { radius: 0.3 };
    emitter.one_shot = true;
    emitter.burst_count = 30;
    emitter.initial_velocity_min = 3.0;
    emitter.initial_velocity_max = 8.0;
    emitter.velocity_spread = 1.0;
    emitter.particle_lifetime_min = 0.4;
    emitter.particle_lifetime_max = 0.8;
    emitter.size_start = 0.15;
    emitter.size_end = 0.02;
    emitter.gravity = nalgebra_glm::vec3(0.0, -5.0, 0.0);
    emitter.drag = 0.3;
    emitter.emissive_strength = 8.0;
    emitter.color_gradient = ColorGradient {
        colors: vec![
            (0.0, nalgebra_glm::vec4(color.x, color.y, color.z, 1.0)),
            (0.3, nalgebra_glm::vec4(color.x * 1.2, color.y * 1.2, color.z * 1.2, 0.9)),
            (0.7, nalgebra_glm::vec4(color.x * 0.5, color.y * 0.5, color.z * 0.5, 0.5)),
            (1.0, nalgebra_glm::vec4(0.1, 0.1, 0.1, 0.0)),
        ],
    };

    world.core.set_particle_emitter(entity, emitter);

    entity
}
