use crate::constants::ROOM_HEIGHT;
use crate::state::{HorrorDemo, OverheadLightState};
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::physics::create_textured_material;
use nightshade::prelude::*;

pub fn spawn_overhead_lights(demo: &mut HorrorDemo, world: &mut World) {
    let light_positions = [
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.1, 4.0),
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.1, -2.0),
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.1, -8.0),
        nalgebra_glm::vec3(-3.0, ROOM_HEIGHT - 0.1, -14.0),
        nalgebra_glm::vec3(3.0, ROOM_HEIGHT - 0.1, -14.0),
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.1, -18.0),
        nalgebra_glm::vec3(9.0, ROOM_HEIGHT - 0.1, -16.0),
        nalgebra_glm::vec3(-9.0, ROOM_HEIGHT - 0.1, -16.0),
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.1, -26.0),
    ];

    for (index, &position) in light_positions.iter().enumerate() {
        let fixture_material =
            create_textured_material(nalgebra_glm::vec3(0.2, 0.2, 0.22), 0.6, 0.5);

        let fixture_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | VISIBILITY,
            1,
        )[0];

        if let Some(name) = world.get_name_mut(fixture_entity) {
            name.0 = format!("Light Fixture {}", index);
        }

        if let Some(transform) = world.get_local_transform_mut(fixture_entity) {
            transform.translation = position;
            transform.scale = nalgebra_glm::vec3(0.6, 0.08, 0.2);
        }

        if let Some(mesh) = world.get_render_mesh_mut(fixture_entity) {
            mesh.name = "Cube".to_string();
        }

        let material_name = format!("LightFixture_{}", fixture_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            fixture_material,
        );
        if let Some(&mat_index) = world
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
                .add_reference(mat_index);
        }
        world.set_material_ref(fixture_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.get_bounding_volume_mut(fixture_entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
        }

        let light_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | LIGHT,
            1,
        )[0];

        if let Some(name) = world.get_name_mut(light_entity) {
            name.0 = format!("Overhead Light {}", index);
        }

        if let Some(transform) = world.get_local_transform_mut(light_entity) {
            transform.translation = position - nalgebra_glm::vec3(0.0, 0.1, 0.0);
        }

        let base_intensity = 1.5 + (index % 3) as f32 * 0.3;

        if let Some(light) = world.get_light_mut(light_entity) {
            *light = Light {
                light_type: LightType::Point,
                color: nalgebra_glm::vec3(1.0, 0.9, 0.7),
                intensity: base_intensity,
                range: 8.0,
                inner_cone_angle: 0.0,
                outer_cone_angle: 0.0,
                cast_shadows: false,
                shadow_bias: 0.0,
            };
        }

        demo.overhead_lights.push(OverheadLightState {
            entity: fixture_entity,
            light_entity,
            base_intensity,
            spark_timer: 0.0,
            next_spark_time: 2.0 + (index as f32 * 1.7) % 5.0,
            is_sparking: false,
        });
    }
}

pub fn update_overhead_lights(demo: &mut HorrorDemo, world: &mut World) {
    let dt = world.resources.window.timing.delta_time;
    let total_time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

    for light_state in &mut demo.overhead_lights {
        light_state.spark_timer += dt;

        if light_state.is_sparking {
            let spark_progress = light_state.spark_timer;

            if spark_progress < 0.5 {
                let flicker = ((spark_progress * 50.0).sin() * 0.5 + 0.5).powi(2);
                let intensity = light_state.base_intensity * flicker * 3.0;

                if let Some(light) = world.get_light_mut(light_state.light_entity) {
                    light.intensity = intensity;
                    light.color = nalgebra_glm::vec3(1.0, 0.6 + flicker * 0.3, 0.3);
                }
            } else {
                light_state.is_sparking = false;
                light_state.spark_timer = 0.0;
                light_state.next_spark_time = 3.0 + (total_time * 7.0).sin().abs() * 8.0;

                if let Some(light) = world.get_light_mut(light_state.light_entity) {
                    light.intensity = light_state.base_intensity;
                    light.color = nalgebra_glm::vec3(1.0, 0.9, 0.7);
                }
            }
        } else {
            let subtle_flicker =
                1.0 + (total_time * 3.0 + light_state.base_intensity * 10.0).sin() * 0.05;
            if let Some(light) = world.get_light_mut(light_state.light_entity) {
                light.intensity = light_state.base_intensity * subtle_flicker;
            }

            if light_state.spark_timer >= light_state.next_spark_time {
                light_state.is_sparking = true;
                light_state.spark_timer = 0.0;

                spawn_spark_particles(world, light_state.entity);
            }
        }
    }
}

fn spawn_spark_particles(world: &mut World, fixture_entity: Entity) {
    let fixture_pos = world
        .get_local_transform(fixture_entity)
        .map(|t| t.translation)
        .unwrap_or(Vec3::zeros());

    let spark_material = Material {
        base_color: [1.0, 0.7, 0.2, 1.0],
        emissive_factor: [2.0, 1.0, 0.3],
        roughness: 0.1,
        metallic: 0.9,
        ..Default::default()
    };

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

        if let Some(name) = world.get_name_mut(entity) {
            name.0 = "Spark".to_string();
        }

        if let Some(transform) = world.get_local_transform_mut(entity) {
            transform.translation = fixture_pos + offset;
            transform.scale = nalgebra_glm::vec3(0.02, 0.02, 0.02);
        }

        if let Some(mesh) = world.get_render_mesh_mut(entity) {
            mesh.name = "Sphere".to_string();
        }

        let material_name = format!("Spark_{}", entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            spark_material.clone(),
        );
        if let Some(&mat_index) = world
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
                .add_reference(mat_index);
        }
        world.set_material_ref(entity, MaterialRef::new(material_name));

        if let Some(bv) = world.get_bounding_volume_mut(entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Sphere");
        }
    }
}
