use crate::ecs::{GameWorld, Target, SHOT_BAUBLE, TARGET};
use nightshade::ecs::particles::components::{ColorGradient, EmitterShape, ParticleEmitter};
use nightshade::ecs::visibility::components::Visibility;
use nightshade::prelude::*;

pub fn spawn_targets(game_world: &mut GameWorld, world: &mut World) {
    let positions = [
        (nalgebra_glm::vec3(-6.0, 3.0, -6.0), nalgebra_glm::vec3(0.9, 0.3, 0.3)),
        (nalgebra_glm::vec3(6.0, 4.5, -6.0), nalgebra_glm::vec3(0.3, 0.9, 0.3)),
        (nalgebra_glm::vec3(0.0, 5.0, -10.0), nalgebra_glm::vec3(0.3, 0.5, 0.9)),
        (nalgebra_glm::vec3(-8.0, 2.5, 0.0), nalgebra_glm::vec3(0.9, 0.7, 0.2)),
        (nalgebra_glm::vec3(8.0, 3.5, 0.0), nalgebra_glm::vec3(0.7, 0.3, 0.9)),
        (nalgebra_glm::vec3(-3.0, 6.0, -3.0), nalgebra_glm::vec3(0.2, 0.8, 0.8)),
        (nalgebra_glm::vec3(3.0, 4.0, 3.0), nalgebra_glm::vec3(0.9, 0.4, 0.6)),
        (nalgebra_glm::vec3(0.0, 3.0, 6.0), nalgebra_glm::vec3(0.8, 0.8, 0.3)),
        (nalgebra_glm::vec3(-5.0, 5.5, 5.0), nalgebra_glm::vec3(0.4, 0.6, 0.9)),
        (nalgebra_glm::vec3(5.0, 2.0, -3.0), nalgebra_glm::vec3(0.9, 0.5, 0.3)),
    ];

    for (position, color) in positions {
        let scale = 0.3;
        let entity = spawn_target_sphere(world, position, scale, color);

        let game_entity = game_world.spawn_entities(TARGET, 1)[0];
        game_world.set_target(
            game_entity,
            Target {
                entity,
                position,
                base_scale: scale,
                color,
                popped: false,
                pop_time_ms: 0,
                respawn_delay_ms: 3000,
                pulse_phase: position.x * 2.0 + position.z,
            },
        );
    }
}

fn spawn_target_sphere(
    world: &mut World,
    position: Vec3,
    scale: f32,
    color: Vec3,
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
        mesh.name = "Sphere".to_string();
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
        *bounding_volume = BoundingVolume::from_mesh_type("Sphere");
    }

    world.resources.mesh_render_state.mark_entity_added(entity);

    entity
}

pub fn update_targets(game_world: &mut GameWorld, world: &mut World) {
    let current_time = world.resources.window.timing.uptime_milliseconds;
    let delta_time = world.resources.window.timing.delta_time;

    let bauble_positions: Vec<Vec3> = game_world
        .query_entities(SHOT_BAUBLE)
        .filter_map(|game_entity| {
            let bauble = game_world.get_shot_bauble(game_entity)?;
            world
                .core
                .get_global_transform(bauble.entity)
                .map(|transform| transform.translation())
        })
        .collect();

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

                if let Some(target) = game_world.get_target_mut(game_entity) {
                    target.popped = false;
                }

                world
                    .core
                    .set_visibility(entity, Visibility { visible: true });

                if let Some(transform) = world.core.get_local_transform_mut(entity) {
                    transform.translation = position;
                    transform.scale = nalgebra_glm::vec3(base_scale, base_scale, base_scale);
                }
                mark_local_transform_dirty(world, entity);
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
        let hit = bauble_positions
            .iter()
            .any(|bauble_position| nalgebra_glm::distance(&target_position, bauble_position) < hit_radius);

        if hit {
            if let Some(target) = game_world.get_target_mut(game_entity) {
                target.popped = true;
                target.pop_time_ms = current_time;
            }

            world
                .core
                .set_visibility(entity, Visibility { visible: false });

            spawn_pop_effect(world, target_position, color);
        }
    }
}

fn spawn_pop_effect(world: &mut World, position: Vec3, color: Vec3) {
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
}
