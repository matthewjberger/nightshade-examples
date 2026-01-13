use nightshade::ecs::camera::commands::spawn_camera;
use nightshade::ecs::camera::systems::fly_camera_system;
#[cfg(not(target_arch = "wasm32"))]
use nightshade::ecs::map::save_map;
use nightshade::ecs::map::{Map, MapLight, MapMaterial, MapNode, NodeIndex, spawn_map};
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

fn add_entity_with_mesh(
    map: &mut Map,
    parent: Option<NodeIndex>,
    name: &str,
    transform: LocalTransform,
    mesh_name: &str,
    material: MapMaterial,
) -> NodeIndex {
    let entity_node = MapNode::entity_full(Some(name.to_string()), transform);
    let entity_index = if let Some(parent_idx) = parent {
        map.add_child_node(parent_idx, entity_node)
    } else {
        map.add_root_node(entity_node)
    };
    map.add_child_node(
        entity_index,
        MapNode::mesh_full(mesh_name, Some(material), true, None, None),
    );
    entity_index
}

fn add_entity_with_light(
    map: &mut Map,
    parent: Option<NodeIndex>,
    name: &str,
    transform: LocalTransform,
    light: MapLight,
) -> NodeIndex {
    let entity_node = MapNode::entity_full(Some(name.to_string()), transform);
    let entity_index = if let Some(parent_idx) = parent {
        map.add_child_node(parent_idx, entity_node)
    } else {
        map.add_root_node(entity_node)
    };
    map.add_child_node(entity_index, MapNode::light(light));
    entity_index
}

fn create_shadows_map() -> Map {
    let mut map = Map::new("Shadows Demo");

    add_entity_with_mesh(
        &mut map,
        None,
        "Floor",
        LocalTransform {
            translation: Vec3::new(0.0, -13.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(30.0, 0.1, 20.0),
        },
        "Cube",
        MapMaterial {
            base_color: [0.5, 0.5, 0.7, 1.0],
            roughness: 0.8,
            metallic: 0.0,
            ..Default::default()
        },
    );

    add_entity_with_mesh(
        &mut map,
        None,
        "Torus",
        LocalTransform {
            translation: Vec3::new(0.0, -4.7, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x_axis()),
            scale: Vec3::new(4.0, 4.0, 4.0),
        },
        "Torus",
        MapMaterial {
            base_color: [0.8, 0.3, 0.5, 1.0],
            roughness: 0.5,
            metallic: 0.1,
            ..Default::default()
        },
    );

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
        add_entity_with_mesh(
            &mut map,
            None,
            &format!("Sphere_{}", index),
            LocalTransform {
                translation: Vec3::new(pos[0], pos[1], pos[2]),
                rotation: Quat::identity(),
                scale: Vec3::new(*scale, *scale, *scale),
            },
            "Sphere",
            MapMaterial {
                base_color: *color,
                roughness: 0.4,
                metallic: 0.2,
                ..Default::default()
            },
        );
    }

    add_entity_with_light(
        &mut map,
        None,
        "Sun",
        LocalTransform {
            translation: Vec3::new(0.0, 100.0, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x_axis()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        MapLight::Directional {
            color: [1.0, 1.0, 1.0],
            intensity: 3.0,
            cast_shadows: true,
            shadow_bias: 0.007,
        },
    );

    add_entity_with_light(
        &mut map,
        None,
        "PointLight",
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        MapLight::Point {
            color: [1.0, 0.8, 0.5],
            intensity: 5.0,
            range: 20.0,
            cast_shadows: true,
            shadow_bias: 0.005,
        },
    );

    map
}

fn find_entity_by_name(world: &World, name: &str) -> Option<Entity> {
    world
        .query_entities(NAME)
        .find(|&entity| world.get_name(entity).map(|n| n.0 == name).unwrap_or(false))
}

impl State for ShadowsDemo {
    fn initialize(&mut self, world: &mut World) {
        self.resources.time = 0.0;
        self.resources.light_entity = None;
        self.resources.torus_entity = None;
        self.resources.spheres = Vec::new();

        let map = create_shadows_map();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let map_path = std::path::Path::new("apps/shadows/shadows.json");
            if let Some(parent) = map_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(error) = save_map(&map, map_path) {
                tracing::error!("Failed to save shadows map: {}", error);
            } else {
                tracing::info!("Saved shadows map to {:?}", map_path);
            }
        }

        match spawn_map(world, &map) {
            Ok(result) => {
                tracing::info!(
                    "Loaded shadows map with {} entities",
                    result.node_to_entity.len()
                );
            }
            Err(error) => {
                tracing::error!("Failed to load shadows map: {}", error);
            }
        }

        self.resources.light_entity = find_entity_by_name(world, "Sun");
        self.resources.torus_entity = find_entity_by_name(world, "Torus");

        let sphere_entities: Vec<Entity> = world
            .query_entities(NAME)
            .filter(|&entity| {
                world
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
            if let Some(mut transform) = world.get_local_transform(light_entity).cloned() {
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

                world.set_local_transform(light_entity, transform);
                world.set_local_transform_dirty(light_entity, LocalTransformDirty);
            }
        }

        if let Some(torus_entity) = self.resources.torus_entity {
            if let Some(mut transform) = world.get_local_transform(torus_entity).cloned() {
                transform.rotation =
                    nalgebra_glm::quat_angle_axis(self.resources.time * 2.0, &Vec3::y_axis())
                        * nalgebra_glm::quat_angle_axis(
                            std::f32::consts::FRAC_PI_2,
                            &Vec3::x_axis(),
                        );
                world.set_local_transform(torus_entity, transform);
                world.set_local_transform_dirty(torus_entity, LocalTransformDirty);
            }
        }

        for (sphere_entity, velocity) in &mut self.resources.spheres {
            if let Some(mut transform) = world.get_local_transform(*sphere_entity).cloned() {
                transform.translation.y += *velocity;

                if transform.translation.y > 11.0 {
                    transform.translation.y = 11.0;
                    *velocity = -velocity.abs();
                } else if transform.translation.y < -11.0 {
                    transform.translation.y = -11.0;
                    *velocity = velocity.abs();
                }

                world.set_local_transform(*sphere_entity, transform);
                world.set_local_transform_dirty(*sphere_entity, LocalTransformDirty);
            }
        }

        escape_key_exit_system(world);
        fly_camera_system(world);
    }
}
