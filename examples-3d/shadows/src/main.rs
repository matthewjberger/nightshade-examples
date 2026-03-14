use std::path::Path;

use nightshade::ecs::camera::commands::spawn_camera;
use nightshade::ecs::camera::systems::fly_camera_system;
use nightshade::ecs::scene::{
    save_scene, spawn_scene, Scene, SceneComponents, SceneEntity, SceneLight, SceneMaterial,
    SceneMesh,
};
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(ShadowsDemo::default())?;
    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ShadowsMarker;

freecs::ecs! {
    ShadowsDemo {
        shadows_marker: ShadowsMarker => SHADOWS_MARKER,
    }
    ShadowsDemoResources {
        time: f32,
        light_entity: Option<Entity>,
        torus_entity: Option<Entity>,
        spheres: Vec<(Entity, f32)>,
    }
}

fn create_mesh_entity(
    name: &str,
    transform: LocalTransform,
    mesh_name: &str,
    base_color: [f32; 4],
    roughness: f32,
    metallic: f32,
) -> SceneEntity {
    SceneEntity::new()
        .with_name(name)
        .with_transform(transform)
        .with_components(SceneComponents {
            mesh: Some(
                SceneMesh::from_name(mesh_name).with_material(SceneMaterial {
                    base_color,
                    roughness,
                    metallic,
                    ..Default::default()
                }),
            ),
            casts_shadow: true,
            visible: true,
            ..Default::default()
        })
}

fn create_light_entity(name: &str, transform: LocalTransform, light: SceneLight) -> SceneEntity {
    SceneEntity::new()
        .with_name(name)
        .with_transform(transform)
        .with_components(SceneComponents {
            light: Some(light),
            visible: true,
            ..Default::default()
        })
}

fn create_shadows_scene() -> Scene {
    let mut scene = Scene::new("Shadows Demo");

    scene.add_entity(create_mesh_entity(
        "Floor",
        LocalTransform {
            translation: Vec3::new(0.0, -13.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(30.0, 0.1, 20.0),
        },
        "Cube",
        [0.5, 0.5, 0.7, 1.0],
        0.8,
        0.0,
    ));

    scene.add_entity(create_mesh_entity(
        "Torus",
        LocalTransform {
            translation: Vec3::new(0.0, -4.7, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x_axis()),
            scale: Vec3::new(4.0, 4.0, 4.0),
        },
        "Torus",
        [0.8, 0.3, 0.5, 1.0],
        0.5,
        0.1,
    ));

    let sphere_positions = [
        ([-12.0, -8.0, -6.0], 1.2, [0.9, 0.2, 0.3, 1.0]),
        ([-8.0, -4.0, 4.0], 0.8, [0.2, 0.8, 0.3, 1.0]),
        ([-4.0, 0.0, -8.0], 1.5, [0.3, 0.3, 0.9, 1.0]),
        ([0.0, 4.0, 2.0], 1.0, [0.9, 0.9, 0.2, 1.0]),
        ([4.0, -6.0, -4.0], 0.7, [0.9, 0.5, 0.2, 1.0]),
        ([8.0, 2.0, 6.0], 1.3, [0.5, 0.2, 0.9, 1.0]),
        ([12.0, -2.0, -2.0], 0.9, [0.2, 0.9, 0.9, 1.0]),
        ([-10.0, 6.0, 0.0], 1.1, [0.9, 0.2, 0.9, 1.0]),
        ([-6.0, -10.0, 8.0], 0.6, [0.6, 0.6, 0.2, 1.0]),
        ([2.0, 8.0, -6.0], 1.4, [0.2, 0.6, 0.6, 1.0]),
        ([6.0, -8.0, 4.0], 0.85, [0.8, 0.4, 0.2, 1.0]),
        ([10.0, 0.0, -8.0], 1.25, [0.4, 0.8, 0.4, 1.0]),
        ([-14.0, 4.0, 2.0], 0.95, [0.4, 0.4, 0.8, 1.0]),
        ([-2.0, -6.0, 6.0], 1.35, [0.8, 0.8, 0.4, 1.0]),
        ([14.0, 6.0, 0.0], 0.75, [0.8, 0.2, 0.6, 1.0]),
        ([-8.0, 10.0, -4.0], 1.05, [0.2, 0.8, 0.6, 1.0]),
        ([8.0, -10.0, 2.0], 0.65, [0.6, 0.2, 0.8, 1.0]),
        ([0.0, -2.0, -10.0], 1.15, [0.6, 0.8, 0.2, 1.0]),
    ];

    for (index, (pos, scale, color)) in sphere_positions.iter().enumerate() {
        scene.add_entity(create_mesh_entity(
            &format!("Sphere_{}", index),
            LocalTransform {
                translation: Vec3::new(pos[0], pos[1], pos[2]),
                rotation: Quat::identity(),
                scale: Vec3::new(*scale, *scale, *scale),
            },
            "Sphere",
            *color,
            0.4,
            0.2,
        ));
    }

    scene.add_entity(create_light_entity(
        "Sun",
        LocalTransform {
            translation: Vec3::new(0.0, 100.0, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x_axis()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        SceneLight::Directional {
            color: [1.0, 1.0, 1.0],
            intensity: 3.0,
            cast_shadows: true,
            shadow_bias: 0.007,
        },
    ));

    scene.add_entity(create_light_entity(
        "PointLight",
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        SceneLight::Point {
            color: [1.0, 0.8, 0.5],
            intensity: 5.0,
            range: 20.0,
            cast_shadows: true,
            shadow_bias: 0.005,
        },
    ));

    scene
}

fn find_entity_by_name(world: &World, name: &str) -> Option<Entity> {
    world.core.query_entities(NAME).find(|&entity| {
        world
            .core
            .get_name(entity)
            .map(|n| n.0 == name)
            .unwrap_or(false)
    })
}

impl State for ShadowsDemo {
    fn initialize(&mut self, world: &mut World) {
        self.resources.time = 0.0;
        self.resources.light_entity = None;
        self.resources.torus_entity = None;
        self.resources.spheres = Vec::new();

        let mut scene = create_shadows_scene();

        if let Err(error) = save_scene(&mut scene, Path::new("shadows_demo.json")) {
            tracing::error!("Failed to save scene: {}", error);
        }

        match spawn_scene(world, &scene, None) {
            Ok(result) => {
                tracing::info!(
                    "Loaded shadows scene with {} entities",
                    result.uuid_to_entity.len()
                );
            }
            Err(error) => {
                tracing::error!("Failed to load shadows scene: {}", error);
            }
        }

        self.resources.light_entity = find_entity_by_name(world, "Sun");
        self.resources.torus_entity = find_entity_by_name(world, "Torus");

        let sphere_entities: Vec<Entity> = world
            .core
            .query_entities(NAME)
            .filter(|&entity| {
                world
                    .core
                    .get_name(entity)
                    .map(|n| n.0.starts_with("Sphere_"))
                    .unwrap_or(false)
            })
            .collect();

        let mut rng = rand::rng();
        for sphere_entity in sphere_entities {
            let velocity = rng.random_range(-0.09..0.09);
            self.resources.spheres.push((sphere_entity, velocity));
        }

        let camera_position = Vec3::new(0.0, 10.0, 20.0);
        let camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(camera);
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta = world.resources.window.timing.delta_time;
        self.resources.time += delta;

        if let Some(light_entity) = self.resources.light_entity {
            if let Some(mut transform) = world.core.get_local_transform(light_entity).cloned() {
                let x = 50.0 * self.resources.time.sin();
                let z = 50.0 * self.resources.time.cos();
                transform.translation.x = x;
                transform.translation.z = z;

                let target = Vec3::zeros();
                let direction = (target - transform.translation).normalize();

                let pitch = direction.y.asin();
                let yaw = direction.z.atan2(direction.x);

                transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
                    * nalgebra_glm::quat_angle_axis(pitch, &Vec3::x());

                world.core.set_local_transform(light_entity, transform);
                world
                    .core
                    .set_local_transform_dirty(light_entity, LocalTransformDirty);
            }
        }

        if let Some(torus_entity) = self.resources.torus_entity {
            if let Some(mut transform) = world.core.get_local_transform(torus_entity).cloned() {
                transform.rotation =
                    nalgebra_glm::quat_angle_axis(self.resources.time * 2.0, &Vec3::y_axis())
                        * nalgebra_glm::quat_angle_axis(
                            std::f32::consts::FRAC_PI_2,
                            &Vec3::x_axis(),
                        );
                world.core.set_local_transform(torus_entity, transform);
                world
                    .core
                    .set_local_transform_dirty(torus_entity, LocalTransformDirty);
            }
        }

        for (sphere_entity, velocity) in &mut self.resources.spheres {
            if let Some(mut transform) = world.core.get_local_transform(*sphere_entity).cloned() {
                transform.translation.y += *velocity;

                if transform.translation.y > 11.0 {
                    transform.translation.y = 11.0;
                    *velocity = -velocity.abs();
                } else if transform.translation.y < -11.0 {
                    transform.translation.y = -11.0;
                    *velocity = velocity.abs();
                }

                world.core.set_local_transform(*sphere_entity, transform);
                world
                    .core
                    .set_local_transform_dirty(*sphere_entity, LocalTransformDirty);
            }
        }

        escape_key_exit_system(world);
        fly_camera_system(world);
    }
}
