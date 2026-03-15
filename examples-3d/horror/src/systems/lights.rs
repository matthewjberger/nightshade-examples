use crate::ecs::{
    ENGINE_ENTITY, EngineEntity, GameWorld, OVERHEAD_LIGHT, SPARK_PARTICLE, SparkParticle,
};
use nightshade::prelude::*;

pub fn update_overhead_lights(game_world: &mut GameWorld, world: &mut World) {
    let dt = world.resources.window.timing.delta_time;
    let total_time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

    let light_entities: Vec<freecs::Entity> = game_world
        .query_entities(OVERHEAD_LIGHT | ENGINE_ENTITY)
        .collect();

    let mut spark_spawns: Vec<Entity> = Vec::new();

    for game_entity in light_entities {
        let Some(engine_entity) = game_world.get_engine_entity(game_entity) else {
            continue;
        };
        let fixture_entity = engine_entity.0;

        let Some(light_state) = game_world.get_overhead_light_mut(game_entity) else {
            continue;
        };

        light_state.spark_timer += dt;

        let light_entity = light_state.light_entity;
        let base_intensity = light_state.base_intensity;

        if light_state.is_sparking {
            let spark_progress = light_state.spark_timer;

            if spark_progress < 0.5 {
                let flicker = ((spark_progress * 50.0).sin() * 0.5 + 0.5).powi(2);
                if let Some(light) = world.core.get_light_mut(light_entity) {
                    light.intensity = base_intensity * flicker * 3.0;
                    light.color = nalgebra_glm::vec3(1.0, 0.6 + flicker * 0.3, 0.3);
                }
            } else {
                light_state.is_sparking = false;
                light_state.spark_timer = 0.0;
                light_state.next_spark_time = 3.0 + (total_time * 7.0).sin().abs() * 8.0;
                if let Some(light) = world.core.get_light_mut(light_entity) {
                    light.intensity = base_intensity;
                    light.color = nalgebra_glm::vec3(1.0, 0.9, 0.7);
                }
            }
        } else {
            let subtle_flicker = 1.0 + (total_time * 3.0 + base_intensity * 10.0).sin() * 0.05;
            let should_spark = light_state.spark_timer >= light_state.next_spark_time;

            if let Some(light) = world.core.get_light_mut(light_entity) {
                light.intensity = base_intensity * subtle_flicker;
            }

            if should_spark {
                light_state.is_sparking = true;
                light_state.spark_timer = 0.0;
                spark_spawns.push(fixture_entity);
            }
        }
    }

    for fixture_entity in spark_spawns {
        spawn_spark_particles(game_world, world, fixture_entity);
    }
}

pub fn update_spark_particles(game_world: &mut GameWorld, world: &mut World) {
    let dt = world.resources.window.timing.delta_time;

    let expired: Vec<freecs::Entity> = game_world
        .query_entities(SPARK_PARTICLE | ENGINE_ENTITY)
        .filter(|&game_entity| {
            game_world
                .get_spark_particle(game_entity)
                .is_some_and(|spark| spark.lifetime <= 0.0)
        })
        .collect();

    for game_entity in &expired {
        if let Some(engine_entity) = game_world.get_engine_entity(*game_entity) {
            world.queue_despawn_entity(engine_entity.0);
        }
    }
    game_world.despawn_entities(&expired);

    let active_sparks: Vec<freecs::Entity> = game_world.query_entities(SPARK_PARTICLE).collect();
    for game_entity in active_sparks {
        if let Some(spark) = game_world.get_spark_particle_mut(game_entity) {
            spark.lifetime -= dt;
        }
    }
}

fn spawn_spark_particles(game_world: &mut GameWorld, world: &mut World, fixture_entity: Entity) {
    let fixture_pos = world
        .core
        .get_local_transform(fixture_entity)
        .map(|t| t.translation)
        .unwrap_or(Vec3::zeros());

    let material_name = "spark_shared".to_string();

    for spark_index in 0..8 {
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

        let angle = (spark_index as f32 / 8.0) * std::f32::consts::TAU;
        let spread = 0.1 + (spark_index % 3) as f32 * 0.05;
        let offset = nalgebra_glm::vec3(angle.cos() * spread, -0.1, angle.sin() * spread);

        if let Some(name) = world.core.get_name_mut(entity) {
            name.0 = "Spark".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = fixture_pos + offset;
            transform.scale = nalgebra_glm::vec3(0.02, 0.02, 0.02);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
            mesh.name = "Sphere".to_string();
        }

        world.register_material(
            entity,
            material_name.clone(),
            Material {
                base_color: [1.0, 0.7, 0.2, 1.0],
                emissive_factor: [2.0, 1.0, 0.3],
                roughness: 0.1,
                metallic: 0.9,
                ..Default::default()
            },
        );

        if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Sphere");
        }

        let game_entity = game_world.spawn_entities(ENGINE_ENTITY | SPARK_PARTICLE, 1)[0];
        game_world.set_engine_entity(game_entity, EngineEntity(entity));
        game_world.set_spark_particle(game_entity, SparkParticle { lifetime: 0.5 });
    }
}
