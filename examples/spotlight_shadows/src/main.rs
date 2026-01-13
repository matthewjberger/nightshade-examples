use nightshade::ecs::camera::commands::spawn_camera;
use nightshade::ecs::camera::systems::fly_camera_system;
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::map::{spawn_map, Map, MapLight, MapMaterial, MapNode, NodeIndex};
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SpotlightShadowsDemo::default())?;
    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SpotlightShadowsMarker;

freecs::ecs! {
    SpotlightShadowsDemo {
        spotlight_shadows_marker: SpotlightShadowsMarker => SPOTLIGHT_SHADOWS_MARKER,
    }
    SpotlightShadowsDemoResources {
        time: f32,
        spotlight_entities: Vec<Entity>,
        cube_entities: Vec<Entity>,
        flashlight_entity: Option<Entity>,
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

fn create_spotlight_shadows_map() -> Map {
    let mut map = Map::new("Spotlight Shadows Demo");

    add_entity_with_mesh(
        &mut map,
        None,
        "Floor",
        LocalTransform {
            translation: Vec3::new(0.0, -1.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(40.0, 0.2, 40.0),
        },
        "Cube",
        MapMaterial {
            base_color: [0.3, 0.3, 0.35, 1.0],
            roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        },
    );

    add_entity_with_mesh(
        &mut map,
        None,
        "BackWall",
        LocalTransform {
            translation: Vec3::new(0.0, 4.0, -15.0),
            rotation: Quat::identity(),
            scale: Vec3::new(40.0, 10.0, 0.2),
        },
        "Cube",
        MapMaterial {
            base_color: [0.4, 0.4, 0.45, 1.0],
            roughness: 0.8,
            metallic: 0.0,
            ..Default::default()
        },
    );

    let pillar_positions = [
        ([-8.0, 2.5, -8.0], [0.7, 0.3, 0.3, 1.0]),
        ([-4.0, 2.5, -8.0], [0.3, 0.7, 0.3, 1.0]),
        ([0.0, 2.5, -8.0], [0.3, 0.3, 0.7, 1.0]),
        ([4.0, 2.5, -8.0], [0.7, 0.7, 0.3, 1.0]),
        ([8.0, 2.5, -8.0], [0.7, 0.3, 0.7, 1.0]),
    ];

    for (index, (pos, color)) in pillar_positions.iter().enumerate() {
        add_entity_with_mesh(
            &mut map,
            None,
            &format!("Pillar_{}", index),
            LocalTransform {
                translation: Vec3::new(pos[0], pos[1], pos[2]),
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 5.0, 1.0),
            },
            "Cube",
            MapMaterial {
                base_color: *color,
                roughness: 0.5,
                metallic: 0.1,
                ..Default::default()
            },
        );
    }

    let object_positions = [
        ([-6.0, 0.5, 0.0], 1.0, [0.9, 0.2, 0.2, 1.0], "Sphere"),
        ([-2.0, 0.5, 2.0], 1.0, [0.2, 0.9, 0.2, 1.0], "Cube"),
        ([2.0, 0.5, -2.0], 1.0, [0.2, 0.2, 0.9, 1.0], "Sphere"),
        ([6.0, 0.5, 1.0], 1.0, [0.9, 0.9, 0.2, 1.0], "Cube"),
        ([0.0, 0.8, 5.0], 1.5, [0.9, 0.5, 0.2, 1.0], "Torus"),
    ];

    for (index, (pos, scale, color, mesh)) in object_positions.iter().enumerate() {
        let rotation = if *mesh == "Torus" {
            nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x_axis())
        } else {
            Quat::identity()
        };

        add_entity_with_mesh(
            &mut map,
            None,
            &format!("Object_{}", index),
            LocalTransform {
                translation: Vec3::new(pos[0], pos[1], pos[2]),
                rotation,
                scale: Vec3::new(*scale, *scale, *scale),
            },
            mesh,
            MapMaterial {
                base_color: *color,
                roughness: 0.4,
                metallic: 0.2,
                ..Default::default()
            },
        );
    }

    let spotlight_configs = [
        (
            "Spotlight_Red",
            [-8.0, 8.0, 4.0],
            [1.0, 0.2, 0.2],
            5.0,
            true,
        ),
        (
            "Spotlight_Green",
            [-4.0, 8.0, 4.0],
            [0.2, 1.0, 0.2],
            5.0,
            true,
        ),
        (
            "Spotlight_Blue",
            [0.0, 8.0, 4.0],
            [0.2, 0.2, 1.0],
            5.0,
            true,
        ),
        (
            "Spotlight_Yellow",
            [4.0, 8.0, 4.0],
            [1.0, 1.0, 0.2],
            5.0,
            true,
        ),
        (
            "Spotlight_Magenta",
            [8.0, 8.0, 4.0],
            [1.0, 0.2, 1.0],
            5.0,
            true,
        ),
        (
            "Spotlight_White",
            [0.0, 12.0, 8.0],
            [1.0, 1.0, 1.0],
            8.0,
            true,
        ),
    ];
    let spotlight_range = 100.0;

    for (name, pos, color, intensity, cast_shadows) in spotlight_configs {
        let light_pos = Vec3::new(pos[0], pos[1], pos[2]);
        let target = Vec3::new(0.0, 0.0, -4.0);
        let direction = (target - light_pos).normalize();
        let pitch = direction.y.asin();
        let yaw = direction.z.atan2(direction.x);
        let rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
            * nalgebra_glm::quat_angle_axis(pitch, &Vec3::x());

        add_entity_with_light(
            &mut map,
            None,
            name,
            LocalTransform {
                translation: Vec3::new(pos[0], pos[1], pos[2]),
                rotation,
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            MapLight::Spot {
                color,
                intensity,
                range: spotlight_range,
                inner_cone_angle: 0.4,
                outer_cone_angle: 0.8,
                cast_shadows,
                shadow_bias: 0.0001,
            },
        );
    }

    add_entity_with_light(
        &mut map,
        None,
        "AmbientFill",
        LocalTransform {
            translation: Vec3::new(0.0, 20.0, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x_axis()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        MapLight::Directional {
            color: [0.6, 0.6, 0.7],
            intensity: 1.0,
            cast_shadows: false,
            shadow_bias: 0.0,
        },
    );

    map
}

fn find_entities_starting_with(world: &World, prefix: &str) -> Vec<Entity> {
    world
        .query_entities(NAME)
        .filter(|&entity| {
            world
                .get_name(entity)
                .map(|n| n.0.starts_with(prefix))
                .unwrap_or(false)
        })
        .collect()
}

fn spawn_flashlight(world: &mut World) -> Entity {
    let entity = world.spawn_entities(
        LIGHT | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
        1,
    )[0];

    world.set_light(
        entity,
        Light {
            light_type: LightType::Spot,
            color: nalgebra_glm::vec3(1.0, 0.95, 0.8),
            intensity: 40.0,
            range: 300.0,
            inner_cone_angle: 0.2,
            outer_cone_angle: 0.5,
            cast_shadows: true,
            shadow_bias: 0.0001,
        },
    );

    world.set_local_transform(
        entity,
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    world.set_global_transform(entity, GlobalTransform::default());

    world.set_local_transform_dirty(entity, LocalTransformDirty);

    entity
}

impl State for SpotlightShadowsDemo {
    fn initialize(&mut self, world: &mut World) {
        self.resources.time = 0.0;
        self.resources.spotlight_entities = Vec::new();
        self.resources.cube_entities = Vec::new();
        self.resources.flashlight_entity = None;

        let map = create_spotlight_shadows_map();

        match spawn_map(world, &map) {
            Ok(result) => {
                tracing::info!(
                    "Loaded spotlight shadows map with {} entities",
                    result.node_to_entity.len()
                );
            }
            Err(error) => {
                tracing::error!("Failed to load spotlight shadows map: {}", error);
            }
        }

        self.resources.spotlight_entities = find_entities_starting_with(world, "Spotlight_");
        self.resources.cube_entities = find_entities_starting_with(world, "Object_");

        let camera_position = Vec3::new(0.0, 8.0, 20.0);
        let camera = spawn_camera(world, camera_position, "Main Camera".to_string());

        if let Some(mut transform) = world.get_local_transform(camera).cloned() {
            let target = Vec3::new(0.0, 0.0, 0.0);
            let direction = (target - transform.translation).normalize();
            let pitch = direction.y.asin();
            let yaw = direction.z.atan2(direction.x);
            transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
                * nalgebra_glm::quat_angle_axis(pitch, &Vec3::x());
            world.set_local_transform(camera, transform);
            world.set_local_transform_dirty(camera, LocalTransformDirty);
        }

        world.resources.active_camera = Some(camera);

        let flashlight = spawn_flashlight(world);
        self.resources.flashlight_entity = Some(flashlight);
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta = world.resources.window.timing.delta_time;
        self.resources.time += delta;

        for (index, &spotlight_entity) in self.resources.spotlight_entities.iter().enumerate() {
            if let Some(mut transform) = world.get_local_transform(spotlight_entity).cloned() {
                let phase = index as f32 * std::f32::consts::PI * 0.4;
                let sway_x = (self.resources.time * 0.5 + phase).sin() * 3.0;
                let sway_z = (self.resources.time * 0.3 + phase).cos() * 2.0;

                let base_x = match index {
                    0 => -8.0,
                    1 => -4.0,
                    2 => 0.0,
                    3 => 4.0,
                    4 => 8.0,
                    _ => 0.0,
                };

                let target = Vec3::new(base_x + sway_x, 0.0, -4.0 + sway_z);
                let direction = (target - transform.translation).normalize();

                let pitch = direction.y.asin();
                let yaw = direction.z.atan2(direction.x);

                transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
                    * nalgebra_glm::quat_angle_axis(pitch, &Vec3::x());

                world.set_local_transform(spotlight_entity, transform);
                world.set_local_transform_dirty(spotlight_entity, LocalTransformDirty);
            }
        }

        for (index, &cube_entity) in self.resources.cube_entities.iter().enumerate() {
            if let Some(mut transform) = world.get_local_transform(cube_entity).cloned() {
                let phase = index as f32 * 1.5;
                let bob = (self.resources.time * 2.0 + phase).sin() * 0.3;

                let base_y = match index {
                    0 | 2 => 0.5,
                    1 | 3 => 0.5,
                    4 => 0.8,
                    _ => 0.5,
                };

                transform.translation.y = base_y + bob;

                let rotation_speed = 0.5 + index as f32 * 0.2;
                transform.rotation = nalgebra_glm::quat_angle_axis(
                    self.resources.time * rotation_speed,
                    &Vec3::y_axis(),
                );

                if index == 4 {
                    transform.rotation *=
                        nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x_axis());
                }

                world.set_local_transform(cube_entity, transform);
                world.set_local_transform_dirty(cube_entity, LocalTransformDirty);
            }
        }

        if let Some(flashlight_entity) = self.resources.flashlight_entity {
            if let Some(camera) = world.resources.active_camera {
                if let Some(camera_transform) = world.get_global_transform(camera).cloned() {
                    let camera_position = camera_transform.translation();
                    let camera_forward = camera_transform.forward_vector();

                    let offset_position = camera_position + camera_forward * 0.5;

                    let flashlight_transform = LocalTransform {
                        translation: offset_position,
                        rotation: world
                            .get_local_transform(camera)
                            .map(|t| t.rotation)
                            .unwrap_or(Quat::identity()),
                        scale: Vec3::new(1.0, 1.0, 1.0),
                    };

                    world.set_local_transform(flashlight_entity, flashlight_transform);
                    world.set_local_transform_dirty(flashlight_entity, LocalTransformDirty);
                }
            }
        }

        escape_key_exit_system(world);
        fly_camera_system(world);
    }
}
