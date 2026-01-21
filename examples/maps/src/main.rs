use nightshade::ecs::camera::commands::spawn_camera;
use nightshade::ecs::camera::systems::fly_camera_system;
use nightshade::ecs::graphics::resources::Atmosphere;
use nightshade::ecs::scene::{
    AssetUuid, Scene, SceneEntity, SceneInstancedMesh, SceneLight, SceneMaterial,
    SceneMeshInstance, save_scene, spawn_scene,
};
use nightshade::ecs::world::commands::despawn_recursive_immediate;
use nightshade::prelude::*;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SceneDemo::default())?;
    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SceneDemoMarker;

freecs::ecs! {
    SceneDemo {
        scene_demo_marker: SceneDemoMarker => SCENE_DEMO_MARKER,
    }
    SceneDemoResources {
        scenes: Vec<Scene>,
        current_scene_index: usize,
        root_entities: Vec<Entity>,
    }
}

fn create_light_entity(
    name: &str,
    transform: LocalTransform,
    light: SceneLight,
    parent: Option<AssetUuid>,
) -> SceneEntity {
    let mut entity = SceneEntity::new()
        .with_name(name)
        .with_transform(transform)
        .with_light(light)
        .with_visible(true);
    if let Some(parent_uuid) = parent {
        entity = entity.with_parent(parent_uuid);
    }
    entity
}

fn create_instanced_mesh_entity(
    name: &str,
    transform: LocalTransform,
    mesh_name: &str,
    instances: Vec<SceneMeshInstance>,
    color: [f32; 4],
    parent: Option<AssetUuid>,
) -> SceneEntity {
    let material = SceneMaterial {
        base_color: color,
        roughness: 0.7,
        ..Default::default()
    };
    let instanced_mesh = SceneInstancedMesh {
        mesh_name: Some(mesh_name.to_string()),
        mesh_uuid: None,
        instances,
        material: Some(material),
    };
    let mut entity = SceneEntity::new()
        .with_name(name)
        .with_transform(transform)
        .with_visible(true);
    entity.components.instanced_mesh = Some(instanced_mesh);
    entity.components.casts_shadow = true;
    if let Some(parent_uuid) = parent {
        entity = entity.with_parent(parent_uuid);
    }
    entity
}

fn create_empty_entity(
    name: &str,
    transform: LocalTransform,
    parent: Option<AssetUuid>,
) -> SceneEntity {
    let mut entity = SceneEntity::new()
        .with_name(name)
        .with_transform(transform)
        .with_visible(true);
    if let Some(parent_uuid) = parent {
        entity = entity.with_parent(parent_uuid);
    }
    entity
}

fn create_forest_scene() -> Scene {
    let mut scene = Scene::new("Forest Scene");
    scene.atmosphere = Atmosphere::Sky;

    let sun = create_light_entity(
        "Sun",
        LocalTransform {
            translation: Vec3::new(5.0, 10.0, 5.0),
            rotation: nalgebra_glm::quat_angle_axis(
                std::f32::consts::FRAC_PI_4,
                &Vec3::new(0.0, 1.0, 0.0),
            ) * nalgebra_glm::quat_angle_axis(
                -std::f32::consts::FRAC_PI_6,
                &Vec3::new(1.0, 0.0, 0.0),
            ),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        SceneLight::Directional {
            color: [1.0, 0.95, 0.8],
            intensity: 4.0,
            cast_shadows: true,
            shadow_bias: 0.0001,
        },
        None,
    );
    scene.add_entity(sun);

    scene.add_entity(create_instanced_mesh_entity(
        "Ground",
        LocalTransform::default(),
        "Cube",
        vec![SceneMeshInstance {
            translation: [0.0, -0.25, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [100.0, 0.5, 100.0],
            color: None,
        }],
        [0.3, 0.5, 0.2, 1.0],
        None,
    ));

    let trees_parent = create_empty_entity("Trees", LocalTransform::default(), None);
    let trees_uuid = trees_parent.uuid;
    scene.add_entity(trees_parent);

    let tree_positions: Vec<(f32, f32, f32, f32)> = (0..80)
        .map(|index| {
            let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
            let theta = index as f32 * golden_angle;
            let r = (index as f32 / 80.0).sqrt() * 45.0 + 5.0;
            let x = theta.cos() * r;
            let z = theta.sin() * r;
            let dist_from_center = (x * x + z * z).sqrt();
            if dist_from_center < 8.0 {
                return (x * 2.0, z * 2.0, 0.0, 0.0);
            }
            let trunk_height = 3.0 + (index as f32 * 0.7) % 4.0;
            let canopy_height = 4.0 + (index as f32 * 0.5) % 3.0;
            (x, z, trunk_height, canopy_height)
        })
        .filter(|(_, _, trunk, _)| *trunk > 0.0)
        .collect();

    for (index, (x, z, trunk_height, canopy_height)) in tree_positions.iter().enumerate() {
        let tree_parent = create_empty_entity(
            &format!("Tree_{}", index),
            LocalTransform {
                translation: Vec3::new(*x, 0.0, *z),
                ..Default::default()
            },
            Some(trees_uuid),
        );
        let tree_uuid = tree_parent.uuid;
        scene.add_entity(tree_parent);

        let trunk_radius = 0.3 + (index as f32 * 0.05) % 0.3;
        scene.add_entity(create_instanced_mesh_entity(
            &format!("Trunk_{}", index),
            LocalTransform::default(),
            "Cylinder",
            vec![SceneMeshInstance {
                translation: [0.0, trunk_height / 2.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [trunk_radius, *trunk_height, trunk_radius],
                color: None,
            }],
            [0.45, 0.3, 0.15, 1.0],
            Some(tree_uuid),
        ));

        let canopy_radius = 1.5 + (index as f32 * 0.2) % 1.5;
        let green_variation = 0.1 + (index as f32 * 0.02) % 0.2;
        scene.add_entity(create_instanced_mesh_entity(
            &format!("Canopy_{}", index),
            LocalTransform::default(),
            "Cone",
            vec![SceneMeshInstance {
                translation: [0.0, trunk_height + canopy_height / 2.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [canopy_radius, *canopy_height, canopy_radius],
                color: None,
            }],
            [0.15, 0.5 + green_variation, 0.1, 1.0],
            Some(tree_uuid),
        ));
    }

    let rocks_parent = create_empty_entity("Rocks", LocalTransform::default(), None);
    let rocks_uuid = rocks_parent.uuid;
    scene.add_entity(rocks_parent);

    let rock_positions: [([f32; 3], f32); 8] = [
        ([-5.0, 0.0, 3.0], 1.2),
        ([4.0, 0.0, -4.0], 0.8),
        ([-3.0, 0.0, -5.0], 1.0),
        ([6.0, 0.0, 2.0], 0.6),
        ([0.0, 0.0, 6.0], 0.9),
        ([-20.0, 0.0, 15.0], 2.0),
        ([25.0, 0.0, -20.0], 2.5),
        ([-15.0, 0.0, -25.0], 1.8),
    ];

    for (index, (pos, size)) in rock_positions.iter().enumerate() {
        let gray = 0.4 + (index as f32 * 0.05) % 0.2;
        scene.add_entity(create_instanced_mesh_entity(
            &format!("Rock_{}", index),
            LocalTransform::default(),
            "Sphere",
            vec![SceneMeshInstance {
                translation: [pos[0], size * 0.4, pos[2]],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [*size, size * 0.7, *size],
                color: None,
            }],
            [gray, gray, gray, 1.0],
            Some(rocks_uuid),
        ));
    }

    let lights_parent = create_empty_entity("ForestLights", LocalTransform::default(), None);
    let lights_uuid = lights_parent.uuid;
    scene.add_entity(lights_parent);

    let light_positions = [
        ([0.0, 3.0, 0.0], [0.4, 0.6, 0.2], 80.0),
        ([-15.0, 5.0, 10.0], [0.3, 0.5, 0.2], 60.0),
        ([18.0, 4.0, -12.0], [0.5, 0.6, 0.3], 50.0),
        ([-10.0, 6.0, -18.0], [0.4, 0.7, 0.3], 55.0),
    ];

    for (index, (pos, color, intensity)) in light_positions.iter().enumerate() {
        scene.add_entity(create_light_entity(
            &format!("ForestLight_{}", index),
            LocalTransform {
                translation: Vec3::new(pos[0], pos[1], pos[2]),
                ..Default::default()
            },
            SceneLight::Point {
                color: *color,
                intensity: *intensity,
                range: 25.0,
                cast_shadows: false,
                shadow_bias: 0.0,
            },
            Some(lights_uuid),
        ));
    }

    scene
}

fn create_city_scene() -> Scene {
    let mut scene = Scene::new("City Scene");
    scene.atmosphere = Atmosphere::Sunset;

    scene.add_entity(create_light_entity(
        "City Sun",
        LocalTransform {
            translation: Vec3::new(0.0, 100.0, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_3, &Vec3::x_axis()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        SceneLight::Directional {
            color: [0.8, 0.85, 1.0],
            intensity: 3.0,
            cast_shadows: true,
            shadow_bias: 0.0001,
        },
        None,
    ));

    scene.add_entity(create_instanced_mesh_entity(
        "Street",
        LocalTransform::default(),
        "Cube",
        vec![SceneMeshInstance {
            translation: [0.0, -0.5, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [100.0, 0.5, 100.0],
            color: None,
        }],
        [0.25, 0.25, 0.28, 1.0],
        None,
    ));

    let buildings_parent = create_empty_entity("Buildings", LocalTransform::default(), None);
    let buildings_uuid = buildings_parent.uuid;
    scene.add_entity(buildings_parent);

    let building_colors: [[f32; 4]; 7] = [
        [0.7, 0.5, 0.4, 1.0],
        [0.5, 0.6, 0.7, 1.0],
        [0.8, 0.7, 0.5, 1.0],
        [0.6, 0.5, 0.6, 1.0],
        [0.9, 0.85, 0.75, 1.0],
        [0.55, 0.65, 0.6, 1.0],
        [0.75, 0.6, 0.55, 1.0],
    ];

    let building_positions = [
        ([-15.0, 0.0, -15.0], 8.0),
        ([15.0, 0.0, -15.0], 12.0),
        ([-15.0, 0.0, 15.0], 6.0),
        ([15.0, 0.0, 15.0], 10.0),
        ([0.0, 0.0, 0.0], 15.0),
        ([-8.0, 0.0, 0.0], 7.0),
        ([8.0, 0.0, 0.0], 9.0),
    ];

    for (index, (pos, height)) in building_positions.iter().enumerate() {
        scene.add_entity(create_instanced_mesh_entity(
            &format!("Building_{}", index),
            LocalTransform::default(),
            "Cube",
            vec![SceneMeshInstance {
                translation: [pos[0], height / 2.0, pos[2]],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [5.0, *height, 5.0],
                color: None,
            }],
            building_colors[index % building_colors.len()],
            Some(buildings_uuid),
        ));
    }

    let street_lights_parent =
        create_empty_entity("Street Lights", LocalTransform::default(), None);
    let street_lights_uuid = street_lights_parent.uuid;
    scene.add_entity(street_lights_parent);

    for index in 0..6 {
        let x = -25.0 + (index as f32 * 10.0);
        scene.add_entity(create_light_entity(
            &format!("StreetLight_{}", index),
            LocalTransform {
                translation: Vec3::new(x, 5.0, -25.0),
                ..Default::default()
            },
            SceneLight::Point {
                color: [1.0, 0.9, 0.7],
                intensity: 50.0,
                range: 15.0,
                cast_shadows: false,
                shadow_bias: 0.0,
            },
            Some(street_lights_uuid),
        ));
    }

    scene
}

fn create_sprawling_world_scene() -> Scene {
    let mut scene = Scene::new("Sprawling World");
    scene.atmosphere = Atmosphere::Nebula;

    scene.add_entity(create_light_entity(
        "Sun",
        LocalTransform {
            translation: Vec3::new(5.0, 10.0, 5.0),
            rotation: nalgebra_glm::quat_angle_axis(
                std::f32::consts::FRAC_PI_4,
                &Vec3::new(0.0, 1.0, 0.0),
            ) * nalgebra_glm::quat_angle_axis(
                -std::f32::consts::FRAC_PI_6,
                &Vec3::new(1.0, 0.0, 0.0),
            ),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        SceneLight::Directional {
            color: [1.0, 0.95, 0.8],
            intensity: 5.0,
            cast_shadows: true,
            shadow_bias: 0.0001,
        },
        None,
    ));

    let terrain_parent = create_empty_entity("Terrain", LocalTransform::default(), None);
    let terrain_uuid = terrain_parent.uuid;
    scene.add_entity(terrain_parent);

    let terrain_size = 500.0;
    let terrain_tiles = 10;
    let tile_size = terrain_size / terrain_tiles as f32;

    for tile_x in 0..terrain_tiles {
        for tile_z in 0..terrain_tiles {
            let x = (tile_x as f32 - terrain_tiles as f32 / 2.0) * tile_size + tile_size / 2.0;
            let z = (tile_z as f32 - terrain_tiles as f32 / 2.0) * tile_size + tile_size / 2.0;

            let variation = ((tile_x + tile_z) as f32 * 0.05) % 0.1;
            let terrain_color = [0.35 + variation, 0.55 + variation, 0.25 + variation, 1.0];

            scene.add_entity(create_instanced_mesh_entity(
                &format!("Terrain_{}_{}", tile_x, tile_z),
                LocalTransform::default(),
                "Cube",
                vec![SceneMeshInstance {
                    translation: [x, -0.25, z],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [tile_size, 0.5, tile_size],
                    color: None,
                }],
                terrain_color,
                Some(terrain_uuid),
            ));
        }
    }

    let mountains_parent = create_empty_entity("Mountains", LocalTransform::default(), None);
    let mountains_uuid = mountains_parent.uuid;
    scene.add_entity(mountains_parent);

    let mountain_positions = [
        ([-180.0, 0.0, -180.0], 60.0, 25.0),
        ([-150.0, 0.0, -200.0], 45.0, 20.0),
        ([-200.0, 0.0, -150.0], 55.0, 22.0),
        ([180.0, 0.0, 180.0], 70.0, 30.0),
        ([150.0, 0.0, 200.0], 50.0, 18.0),
        ([200.0, 0.0, 150.0], 40.0, 15.0),
        ([-180.0, 0.0, 180.0], 65.0, 28.0),
        ([180.0, 0.0, -180.0], 55.0, 24.0),
    ];

    for (index, (pos, height, radius)) in mountain_positions.iter().enumerate() {
        let gray = 0.45 + (index as f32 * 0.03) % 0.15;
        scene.add_entity(create_instanced_mesh_entity(
            &format!("Mountain_{}", index),
            LocalTransform::default(),
            "Cone",
            vec![SceneMeshInstance {
                translation: [pos[0], height / 2.0, pos[2]],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [*radius, *height, *radius],
                color: None,
            }],
            [gray + 0.05, gray, gray - 0.05, 1.0],
            Some(mountains_uuid),
        ));
    }

    let forest_regions = [
        ([-100.0, 0.0, -50.0], 80.0, 40),
        ([80.0, 0.0, 100.0], 60.0, 30),
        ([-50.0, 0.0, 150.0], 70.0, 35),
        ([150.0, 0.0, -80.0], 50.0, 25),
    ];

    for (region_index, (center, radius, tree_count)) in forest_regions.iter().enumerate() {
        let forest_parent = create_empty_entity(
            &format!("Forest_{}", region_index),
            LocalTransform {
                translation: Vec3::new(center[0], center[1], center[2]),
                ..Default::default()
            },
            None,
        );
        let forest_uuid = forest_parent.uuid;
        scene.add_entity(forest_parent);

        for tree_index in 0..*tree_count {
            let angle = (tree_index as f32 / *tree_count as f32) * std::f32::consts::TAU * 3.0;
            let dist = (tree_index as f32 / *tree_count as f32) * radius;
            let x = angle.cos() * dist;
            let z = angle.sin() * dist;
            let tree_height = 4.0 + (tree_index as f32 * 0.1) % 3.0;

            scene.add_entity(create_instanced_mesh_entity(
                &format!("Tree_{}_{}", region_index, tree_index),
                LocalTransform::default(),
                "Cylinder",
                vec![SceneMeshInstance {
                    translation: [x, tree_height / 2.0, z],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [0.3, tree_height, 0.3],
                    color: None,
                }],
                [0.4, 0.28, 0.12, 1.0],
                Some(forest_uuid),
            ));
        }
    }

    let village_colors = [[1.0, 0.6, 0.2], [0.2, 0.8, 1.0], [1.0, 0.2, 0.6]];

    let village_positions = [[0.0, 0.0, 0.0], [-120.0, 0.0, 80.0], [100.0, 0.0, -100.0]];

    for (village_index, village_center) in village_positions.iter().enumerate() {
        let village_parent = create_empty_entity(
            &format!("Village_{}", village_index),
            LocalTransform {
                translation: Vec3::new(village_center[0], village_center[1], village_center[2]),
                ..Default::default()
            },
            None,
        );
        let village_uuid = village_parent.uuid;
        scene.add_entity(village_parent);

        let building_count = 8 + village_index * 4;
        let base_color = village_colors[village_index];
        for building_index in 0..building_count {
            let angle = (building_index as f32 / building_count as f32) * std::f32::consts::TAU;
            let dist = 10.0 + (building_index as f32 * 2.0) % 15.0;
            let x = angle.cos() * dist;
            let z = angle.sin() * dist;
            let height = 3.0 + (building_index as f32 * 0.5) % 5.0;
            let width = 2.0 + (building_index as f32 * 0.3) % 2.0;

            let variation = (building_index as f32 * 0.05) % 0.2;
            let building_color = [
                (base_color[0] * 0.6 + variation).min(1.0),
                (base_color[1] * 0.6 + variation).min(1.0),
                (base_color[2] * 0.6 + variation).min(1.0),
                1.0,
            ];

            scene.add_entity(create_instanced_mesh_entity(
                &format!("Building_{}_{}", village_index, building_index),
                LocalTransform::default(),
                "Cube",
                vec![SceneMeshInstance {
                    translation: [x, height / 2.0, z],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [width, height, width],
                    color: None,
                }],
                building_color,
                Some(village_uuid),
            ));
        }

        let color = village_colors[village_index];
        scene.add_entity(create_light_entity(
            &format!("VillageLight_{}", village_index),
            LocalTransform {
                translation: Vec3::new(0.0, 10.0, 0.0),
                ..Default::default()
            },
            SceneLight::Point {
                color,
                intensity: 200.0,
                range: 40.0,
                cast_shadows: false,
                shadow_bias: 0.0,
            },
            Some(village_uuid),
        ));
    }

    let roads_parent = create_empty_entity("Roads", LocalTransform::default(), None);
    let roads_uuid = roads_parent.uuid;
    scene.add_entity(roads_parent);

    let road_segments = [
        ([0.0, 0.1, 0.0], [-120.0, 0.1, 80.0]),
        ([0.0, 0.1, 0.0], [100.0, 0.1, -100.0]),
        ([-120.0, 0.1, 80.0], [-180.0, 0.1, -180.0]),
        ([100.0, 0.1, -100.0], [180.0, 0.1, -180.0]),
    ];

    for (road_index, (start, end)) in road_segments.iter().enumerate() {
        let mid: [f32; 3] = [
            (start[0] + end[0]) * 0.5,
            (start[1] + end[1]) * 0.5,
            (start[2] + end[2]) * 0.5,
        ];
        let diff: [f32; 3] = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
        let length = (diff[0] * diff[0] + diff[1] * diff[1] + diff[2] * diff[2]).sqrt();
        let angle = diff[2].atan2(diff[0]);

        let rotation = nalgebra_glm::quat_angle_axis(angle, &Vec3::y_axis());

        scene.add_entity(create_instanced_mesh_entity(
            &format!("Road_{}", road_index),
            LocalTransform {
                translation: Vec3::new(mid[0], mid[1], mid[2]),
                rotation,
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            "Cube",
            vec![SceneMeshInstance {
                translation: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [length, 0.1, 3.0],
                color: None,
            }],
            [0.3, 0.28, 0.25, 1.0],
            Some(roads_uuid),
        ));
    }

    let landmarks_parent = create_empty_entity("Landmarks", LocalTransform::default(), None);
    let landmarks_uuid = landmarks_parent.uuid;
    scene.add_entity(landmarks_parent);

    scene.add_entity(create_instanced_mesh_entity(
        "CentralMonument",
        LocalTransform {
            translation: Vec3::new(0.0, 25.0, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x_axis()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        "Torus",
        vec![SceneMeshInstance {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [18.0, 18.0, 4.0],
            color: None,
        }],
        [0.95, 0.75, 0.2, 1.0],
        Some(landmarks_uuid),
    ));

    scene.add_entity(create_light_entity(
        "MonumentGlow",
        LocalTransform {
            translation: Vec3::new(0.0, 25.0, 0.0),
            ..Default::default()
        },
        SceneLight::Point {
            color: [1.0, 0.8, 0.0],
            intensity: 500.0,
            range: 60.0,
            cast_shadows: false,
            shadow_bias: 0.0,
        },
        Some(landmarks_uuid),
    ));

    scene.add_entity(create_instanced_mesh_entity(
        "WesternOrb",
        LocalTransform::default(),
        "Sphere",
        vec![SceneMeshInstance {
            translation: [-200.0, 40.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [25.0, 25.0, 25.0],
            color: None,
        }],
        [0.3, 0.7, 1.0, 1.0],
        Some(landmarks_uuid),
    ));

    scene.add_entity(create_light_entity(
        "WesternOrbGlow",
        LocalTransform {
            translation: Vec3::new(-200.0, 40.0, 0.0),
            ..Default::default()
        },
        SceneLight::Point {
            color: [0.2, 0.6, 1.0],
            intensity: 800.0,
            range: 80.0,
            cast_shadows: false,
            shadow_bias: 0.0,
        },
        Some(landmarks_uuid),
    ));

    scene.add_entity(create_instanced_mesh_entity(
        "EasternTower",
        LocalTransform::default(),
        "Cylinder",
        vec![SceneMeshInstance {
            translation: [200.0, 50.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [10.0, 100.0, 10.0],
            color: None,
        }],
        [0.85, 0.35, 0.55, 1.0],
        Some(landmarks_uuid),
    ));

    scene.add_entity(create_light_entity(
        "TowerBeacon",
        LocalTransform {
            translation: Vec3::new(200.0, 100.0, 0.0),
            ..Default::default()
        },
        SceneLight::Point {
            color: [1.0, 0.2, 0.5],
            intensity: 600.0,
            range: 70.0,
            cast_shadows: false,
            shadow_bias: 0.0,
        },
        Some(landmarks_uuid),
    ));

    let floating_orbs_parent = create_empty_entity("FloatingOrbs", LocalTransform::default(), None);
    let floating_orbs_uuid = floating_orbs_parent.uuid;
    scene.add_entity(floating_orbs_parent);

    let orb_data = [
        ([50.0, 30.0, 50.0], [0.0, 1.0, 0.5], 300.0),
        ([-60.0, 25.0, -60.0], [1.0, 0.0, 0.8], 250.0),
        ([80.0, 35.0, -70.0], [0.5, 0.2, 1.0], 350.0),
        ([-90.0, 20.0, 60.0], [1.0, 0.5, 0.0], 280.0),
        ([0.0, 50.0, -150.0], [0.0, 0.8, 1.0], 400.0),
        ([130.0, 45.0, 130.0], [1.0, 1.0, 0.2], 320.0),
        ([-140.0, 40.0, -120.0], [0.8, 0.2, 0.2], 280.0),
        ([70.0, 55.0, 160.0], [0.2, 1.0, 0.2], 350.0),
    ];

    for (index, (pos, color, intensity)) in orb_data.iter().enumerate() {
        scene.add_entity(create_instanced_mesh_entity(
            &format!("FloatingOrb_{}", index),
            LocalTransform::default(),
            "Sphere",
            vec![SceneMeshInstance {
                translation: *pos,
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [3.0, 3.0, 3.0],
                color: None,
            }],
            [color[0], color[1], color[2], 1.0],
            Some(floating_orbs_uuid),
        ));

        scene.add_entity(create_light_entity(
            &format!("OrbLight_{}", index),
            LocalTransform {
                translation: Vec3::new(pos[0], pos[1], pos[2]),
                ..Default::default()
            },
            SceneLight::Point {
                color: *color,
                intensity: *intensity,
                range: 50.0,
                cast_shadows: false,
                shadow_bias: 0.0,
            },
            Some(floating_orbs_uuid),
        ));
    }

    let crystals_parent = create_empty_entity("Crystals", LocalTransform::default(), None);
    let crystals_uuid = crystals_parent.uuid;
    scene.add_entity(crystals_parent);

    let crystal_clusters = [
        ([-30.0, 0.0, 80.0], [0.8, 0.2, 1.0]),
        ([45.0, 0.0, -45.0], [0.2, 1.0, 0.8]),
        ([-80.0, 0.0, -30.0], [1.0, 0.4, 0.2]),
        ([110.0, 0.0, 50.0], [0.2, 0.5, 1.0]),
    ];

    for (cluster_index, (pos, color)) in crystal_clusters.iter().enumerate() {
        let cluster_parent = create_empty_entity(
            &format!("CrystalCluster_{}", cluster_index),
            LocalTransform {
                translation: Vec3::new(pos[0], pos[1], pos[2]),
                ..Default::default()
            },
            Some(crystals_uuid),
        );
        let cluster_uuid = cluster_parent.uuid;
        scene.add_entity(cluster_parent);

        for crystal_index in 0..5 {
            let angle = (crystal_index as f32 / 5.0) * std::f32::consts::TAU;
            let offset_x = angle.cos() * 3.0;
            let offset_z = angle.sin() * 3.0;
            let height = 6.0 + (crystal_index as f32 * 2.0);
            let tilt = nalgebra_glm::quat_angle_axis(
                0.2 + crystal_index as f32 * 0.1,
                &Vec3::new(offset_x, 0.0, offset_z).normalize(),
            );

            scene.add_entity(create_instanced_mesh_entity(
                &format!("Crystal_{}_{}", cluster_index, crystal_index),
                LocalTransform {
                    translation: Vec3::new(offset_x, 0.0, offset_z),
                    rotation: tilt,
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
                "Cone",
                vec![SceneMeshInstance {
                    translation: [0.0, height / 2.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.5, height, 1.5],
                    color: None,
                }],
                [color[0], color[1], color[2], 1.0],
                Some(cluster_uuid),
            ));
        }

        scene.add_entity(create_light_entity(
            &format!("CrystalGlow_{}", cluster_index),
            LocalTransform {
                translation: Vec3::new(0.0, 8.0, 0.0),
                ..Default::default()
            },
            SceneLight::Point {
                color: *color,
                intensity: 150.0,
                range: 25.0,
                cast_shadows: false,
                shadow_bias: 0.0,
            },
            Some(cluster_uuid),
        ));
    }

    scene
}

fn load_scene_into_world(world: &mut World, scene: &Scene, root_entities: &mut Vec<Entity>) {
    match spawn_scene(world, scene, None) {
        Ok(result) => {
            root_entities.extend(result.root_entities.iter().copied());
            tracing::info!(
                "Loaded scene '{}' with {} root entities ({} total entities)",
                scene.header.name,
                result.root_entities.len(),
                result.uuid_to_entity.len()
            );
        }
        Err(error) => {
            tracing::error!("Failed to load scene '{}': {}", scene.header.name, error);
        }
    }
}

fn clear_scene(world: &mut World, root_entities: &mut Vec<Entity>) {
    for entity in root_entities.drain(..) {
        despawn_recursive_immediate(world, entity);
    }
    world.resources.mesh_render_state.request_full_rebuild();
}

fn save_scene_to_disk(scene: &mut Scene) {
    let filename = format!(
        "{}.json",
        scene.header.name.to_lowercase().replace(' ', "_")
    );
    if let Err(error) = save_scene(scene, Path::new(&filename)) {
        tracing::error!("Failed to save scene '{}': {}", scene.header.name, error);
    } else {
        tracing::info!("Saved scene to {}", filename);
    }
}

impl State for SceneDemo {
    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;

        self.resources.scenes = vec![
            create_forest_scene(),
            create_city_scene(),
            create_sprawling_world_scene(),
        ];

        for scene in &mut self.resources.scenes {
            save_scene_to_disk(scene);
        }

        self.resources.current_scene_index = 0;

        let camera_position = Vec3::new(0.0, 20.0, 40.0);
        let camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(camera);

        if let Some(scene) = self.resources.scenes.first() {
            load_scene_into_world(world, scene, &mut self.resources.root_entities);
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        fly_camera_system(world);
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        let current_index = self.resources.current_scene_index;
        let scene_names: Vec<String> = self
            .resources
            .scenes
            .iter()
            .map(|s| s.header.name.clone())
            .collect();
        let root_count = self.resources.root_entities.len();
        let scene_info = self
            .resources
            .scenes
            .get(current_index)
            .map(|scene| scene.entities.len());

        let clicked_index = egui::Window::new("Scene Demo")
            .default_pos([10.0, 10.0])
            .show(ctx, |ui| {
                ui.heading("Available Scenes");
                ui.separator();

                let mut result: Option<usize> = None;

                for (index, name) in scene_names.iter().enumerate() {
                    let is_selected = index == current_index;
                    let button_text = if is_selected {
                        format!("* {} (loaded)", name)
                    } else {
                        name.clone()
                    };

                    if ui.button(&button_text).clicked() && !is_selected {
                        result = Some(index);
                    }
                }

                ui.separator();
                ui.label(format!("Root entities: {}", root_count));

                if let Some(total) = scene_info {
                    ui.label(format!("Total entities in scene: {}", total));
                }

                result
            })
            .and_then(|response| response.inner)
            .flatten();

        if let Some(new_index) = clicked_index {
            clear_scene(world, &mut self.resources.root_entities);
            if let Some(scene) = self.resources.scenes.get(new_index) {
                load_scene_into_world(world, scene, &mut self.resources.root_entities);
            }
            self.resources.current_scene_index = new_index;
        }
    }
}
