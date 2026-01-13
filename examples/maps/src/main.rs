use nightshade::ecs::camera::commands::spawn_camera;
use nightshade::ecs::camera::systems::fly_camera_system;
use nightshade::ecs::graphics::resources::Atmosphere;
use nightshade::ecs::map::{Map, MapLight, MapMaterial, MapNode, MeshInstance, NodeIndex};
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(MapDemo::default())?;
    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MapDemoMarker;

freecs::ecs! {
    MapDemo {
        map_demo_marker: MapDemoMarker => MAP_DEMO_MARKER,
    }
    MapDemoResources {
        maps: Vec<Map>,
        current_map_index: usize,
        spawned_entities: Vec<Entity>,
    }
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

fn add_entity_with_colored_mesh(
    map: &mut Map,
    parent: Option<NodeIndex>,
    name: &str,
    transform: LocalTransform,
    mesh_name: &str,
    instances: Vec<MeshInstance>,
    color: [f32; 4],
) -> NodeIndex {
    let entity_node = MapNode::entity_full(Some(name.to_string()), transform);
    let entity_index = if let Some(parent_idx) = parent {
        map.add_child_node(parent_idx, entity_node)
    } else {
        map.add_root_node(entity_node)
    };
    let material = MapMaterial {
        base_color: color,
        roughness: 0.7,
        ..Default::default()
    };
    map.add_child_node(
        entity_index,
        MapNode::instanced_mesh_with_material(mesh_name, instances, material),
    );
    entity_index
}

fn add_empty_entity(
    map: &mut Map,
    parent: Option<NodeIndex>,
    name: &str,
    transform: LocalTransform,
) -> NodeIndex {
    let entity_node = MapNode::entity_full(Some(name.to_string()), transform);
    if let Some(parent_idx) = parent {
        map.add_child_node(parent_idx, entity_node)
    } else {
        map.add_root_node(entity_node)
    }
}

fn create_forest_map() -> Map {
    let mut map = Map::new("Forest Map");
    map.atmosphere = Atmosphere::Sky;

    add_entity_with_light(
        &mut map,
        None,
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
        MapLight::directional([1.0, 0.95, 0.8], 4.0),
    );

    add_entity_with_colored_mesh(
        &mut map,
        None,
        "Ground",
        LocalTransform::default(),
        "Cube",
        vec![MeshInstance::new([0.0, -0.25, 0.0]).with_scale([100.0, 0.5, 100.0])],
        [0.3, 0.5, 0.2, 1.0],
    );

    let trees_parent = add_empty_entity(&mut map, None, "Trees", LocalTransform::default());

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
        let tree_parent = add_empty_entity(
            &mut map,
            Some(trees_parent),
            &format!("Tree_{}", index),
            LocalTransform {
                translation: Vec3::new(*x, 0.0, *z),
                ..Default::default()
            },
        );

        let trunk_radius = 0.3 + (index as f32 * 0.05) % 0.3;
        add_entity_with_colored_mesh(
            &mut map,
            Some(tree_parent),
            &format!("Trunk_{}", index),
            LocalTransform::default(),
            "Cylinder",
            vec![
                MeshInstance::new([0.0, trunk_height / 2.0, 0.0]).with_scale([
                    trunk_radius,
                    *trunk_height,
                    trunk_radius,
                ]),
            ],
            [0.45, 0.3, 0.15, 1.0],
        );

        let canopy_radius = 1.5 + (index as f32 * 0.2) % 1.5;
        let green_variation = 0.1 + (index as f32 * 0.02) % 0.2;
        add_entity_with_colored_mesh(
            &mut map,
            Some(tree_parent),
            &format!("Canopy_{}", index),
            LocalTransform::default(),
            "Cone",
            vec![
                MeshInstance::new([0.0, trunk_height + canopy_height / 2.0, 0.0]).with_scale([
                    canopy_radius,
                    *canopy_height,
                    canopy_radius,
                ]),
            ],
            [0.15, 0.5 + green_variation, 0.1, 1.0],
        );
    }

    let rocks_parent = add_empty_entity(&mut map, None, "Rocks", LocalTransform::default());

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
        add_entity_with_colored_mesh(
            &mut map,
            Some(rocks_parent),
            &format!("Rock_{}", index),
            LocalTransform::default(),
            "Sphere",
            vec![MeshInstance::new([pos[0], size * 0.4, pos[2]]).with_scale([
                *size,
                size * 0.7,
                *size,
            ])],
            [gray, gray, gray, 1.0],
        );
    }

    let lights_parent = add_empty_entity(&mut map, None, "ForestLights", LocalTransform::default());

    let light_positions = [
        ([0.0, 3.0, 0.0], [0.4, 0.6, 0.2], 80.0),
        ([-15.0, 5.0, 10.0], [0.3, 0.5, 0.2], 60.0),
        ([18.0, 4.0, -12.0], [0.5, 0.6, 0.3], 50.0),
        ([-10.0, 6.0, -18.0], [0.4, 0.7, 0.3], 55.0),
    ];

    for (index, (pos, color, intensity)) in light_positions.iter().enumerate() {
        add_entity_with_light(
            &mut map,
            Some(lights_parent),
            &format!("ForestLight_{}", index),
            LocalTransform {
                translation: Vec3::new(pos[0], pos[1], pos[2]),
                ..Default::default()
            },
            MapLight::point(*color, *intensity, 25.0),
        );
    }

    map
}

fn create_city_map() -> Map {
    let mut map = Map::new("City Map");
    map.atmosphere = Atmosphere::Sunset;

    add_entity_with_light(
        &mut map,
        None,
        "City Sun",
        LocalTransform {
            translation: Vec3::new(0.0, 100.0, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_3, &Vec3::x_axis()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        MapLight::directional([0.8, 0.85, 1.0], 3.0),
    );

    add_entity_with_colored_mesh(
        &mut map,
        None,
        "Street",
        LocalTransform::default(),
        "Cube",
        vec![MeshInstance::new([0.0, -0.5, 0.0]).with_scale([100.0, 0.5, 100.0])],
        [0.25, 0.25, 0.28, 1.0],
    );

    let buildings_parent = add_empty_entity(&mut map, None, "Buildings", LocalTransform::default());

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
        add_entity_with_colored_mesh(
            &mut map,
            Some(buildings_parent),
            &format!("Building_{}", index),
            LocalTransform::default(),
            "Cube",
            vec![MeshInstance::new([pos[0], height / 2.0, pos[2]]).with_scale([5.0, *height, 5.0])],
            building_colors[index % building_colors.len()],
        );
    }

    let street_lights_parent =
        add_empty_entity(&mut map, None, "Street Lights", LocalTransform::default());

    for index in 0..6 {
        let x = -25.0 + (index as f32 * 10.0);
        add_entity_with_light(
            &mut map,
            Some(street_lights_parent),
            &format!("StreetLight_{}", index),
            LocalTransform {
                translation: Vec3::new(x, 5.0, -25.0),
                ..Default::default()
            },
            MapLight::point([1.0, 0.9, 0.7], 50.0, 15.0),
        );
    }

    map
}

fn create_sprawling_world_map() -> Map {
    let mut map = Map::new("Sprawling World");
    map.atmosphere = Atmosphere::Nebula;

    add_entity_with_light(
        &mut map,
        None,
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
        MapLight::directional([1.0, 0.95, 0.8], 5.0),
    );

    let terrain_parent = add_empty_entity(&mut map, None, "Terrain", LocalTransform::default());

    let terrain_size = 500.0;
    let terrain_tiles = 10;
    let tile_size = terrain_size / terrain_tiles as f32;

    for tile_x in 0..terrain_tiles {
        for tile_z in 0..terrain_tiles {
            let x = (tile_x as f32 - terrain_tiles as f32 / 2.0) * tile_size + tile_size / 2.0;
            let z = (tile_z as f32 - terrain_tiles as f32 / 2.0) * tile_size + tile_size / 2.0;

            let variation = ((tile_x + tile_z) as f32 * 0.05) % 0.1;
            let terrain_color = [0.35 + variation, 0.55 + variation, 0.25 + variation, 1.0];

            add_entity_with_colored_mesh(
                &mut map,
                Some(terrain_parent),
                &format!("Terrain_{}_{}", tile_x, tile_z),
                LocalTransform::default(),
                "Cube",
                vec![MeshInstance::new([x, -0.25, z]).with_scale([tile_size, 0.5, tile_size])],
                terrain_color,
            );
        }
    }

    let mountains_parent = add_empty_entity(&mut map, None, "Mountains", LocalTransform::default());

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
        add_entity_with_colored_mesh(
            &mut map,
            Some(mountains_parent),
            &format!("Mountain_{}", index),
            LocalTransform::default(),
            "Cone",
            vec![
                MeshInstance::new([pos[0], height / 2.0, pos[2]])
                    .with_scale([*radius, *height, *radius]),
            ],
            [gray + 0.05, gray, gray - 0.05, 1.0],
        );
    }

    let forest_regions = [
        ([-100.0, 0.0, -50.0], 80.0, 40),
        ([80.0, 0.0, 100.0], 60.0, 30),
        ([-50.0, 0.0, 150.0], 70.0, 35),
        ([150.0, 0.0, -80.0], 50.0, 25),
    ];

    for (region_index, (center, radius, tree_count)) in forest_regions.iter().enumerate() {
        let forest_parent = add_empty_entity(
            &mut map,
            None,
            &format!("Forest_{}", region_index),
            LocalTransform {
                translation: Vec3::new(center[0], center[1], center[2]),
                ..Default::default()
            },
        );

        for tree_index in 0..*tree_count {
            let angle = (tree_index as f32 / *tree_count as f32) * std::f32::consts::TAU * 3.0;
            let dist = (tree_index as f32 / *tree_count as f32) * radius;
            let x = angle.cos() * dist;
            let z = angle.sin() * dist;
            let tree_height = 4.0 + (tree_index as f32 * 0.1) % 3.0;

            add_entity_with_colored_mesh(
                &mut map,
                Some(forest_parent),
                &format!("Tree_{}_{}", region_index, tree_index),
                LocalTransform::default(),
                "Cylinder",
                vec![MeshInstance::new([x, tree_height / 2.0, z]).with_scale([
                    0.3,
                    tree_height,
                    0.3,
                ])],
                [0.4, 0.28, 0.12, 1.0],
            );
        }
    }

    let village_colors = [[1.0, 0.6, 0.2], [0.2, 0.8, 1.0], [1.0, 0.2, 0.6]];

    let village_positions = [[0.0, 0.0, 0.0], [-120.0, 0.0, 80.0], [100.0, 0.0, -100.0]];

    for (village_index, village_center) in village_positions.iter().enumerate() {
        let village_parent = add_empty_entity(
            &mut map,
            None,
            &format!("Village_{}", village_index),
            LocalTransform {
                translation: Vec3::new(village_center[0], village_center[1], village_center[2]),
                ..Default::default()
            },
        );

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

            add_entity_with_colored_mesh(
                &mut map,
                Some(village_parent),
                &format!("Building_{}_{}", village_index, building_index),
                LocalTransform::default(),
                "Cube",
                vec![MeshInstance::new([x, height / 2.0, z]).with_scale([width, height, width])],
                building_color,
            );
        }

        let color = village_colors[village_index];
        add_entity_with_light(
            &mut map,
            Some(village_parent),
            &format!("VillageLight_{}", village_index),
            LocalTransform {
                translation: Vec3::new(0.0, 10.0, 0.0),
                ..Default::default()
            },
            MapLight::point(color, 200.0, 40.0),
        );
    }

    let roads_parent = add_empty_entity(&mut map, None, "Roads", LocalTransform::default());

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

        add_entity_with_colored_mesh(
            &mut map,
            Some(roads_parent),
            &format!("Road_{}", road_index),
            LocalTransform {
                translation: Vec3::new(mid[0], mid[1], mid[2]),
                rotation: nalgebra_glm::quat_angle_axis(angle, &Vec3::y_axis()),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            "Cube",
            vec![MeshInstance::new([0.0, 0.0, 0.0]).with_scale([length, 0.1, 3.0])],
            [0.3, 0.28, 0.25, 1.0],
        );
    }

    let landmarks_parent = add_empty_entity(&mut map, None, "Landmarks", LocalTransform::default());

    add_entity_with_colored_mesh(
        &mut map,
        Some(landmarks_parent),
        "CentralMonument",
        LocalTransform {
            translation: Vec3::new(0.0, 25.0, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x_axis()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        "Torus",
        vec![MeshInstance::new([0.0, 0.0, 0.0]).with_scale([18.0, 18.0, 4.0])],
        [0.95, 0.75, 0.2, 1.0],
    );

    add_entity_with_light(
        &mut map,
        Some(landmarks_parent),
        "MonumentGlow",
        LocalTransform {
            translation: Vec3::new(0.0, 25.0, 0.0),
            ..Default::default()
        },
        MapLight::point([1.0, 0.8, 0.0], 500.0, 60.0),
    );

    add_entity_with_colored_mesh(
        &mut map,
        Some(landmarks_parent),
        "WesternOrb",
        LocalTransform::default(),
        "Sphere",
        vec![MeshInstance::new([-200.0, 40.0, 0.0]).with_uniform_scale(25.0)],
        [0.3, 0.7, 1.0, 1.0],
    );

    add_entity_with_light(
        &mut map,
        Some(landmarks_parent),
        "WesternOrbGlow",
        LocalTransform {
            translation: Vec3::new(-200.0, 40.0, 0.0),
            ..Default::default()
        },
        MapLight::point([0.2, 0.6, 1.0], 800.0, 80.0),
    );

    add_entity_with_colored_mesh(
        &mut map,
        Some(landmarks_parent),
        "EasternTower",
        LocalTransform::default(),
        "Cylinder",
        vec![MeshInstance::new([200.0, 50.0, 0.0]).with_scale([10.0, 100.0, 10.0])],
        [0.85, 0.35, 0.55, 1.0],
    );

    add_entity_with_light(
        &mut map,
        Some(landmarks_parent),
        "TowerBeacon",
        LocalTransform {
            translation: Vec3::new(200.0, 100.0, 0.0),
            ..Default::default()
        },
        MapLight::point([1.0, 0.2, 0.5], 600.0, 70.0),
    );

    let floating_orbs_parent =
        add_empty_entity(&mut map, None, "FloatingOrbs", LocalTransform::default());

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
        add_entity_with_colored_mesh(
            &mut map,
            Some(floating_orbs_parent),
            &format!("FloatingOrb_{}", index),
            LocalTransform::default(),
            "Sphere",
            vec![MeshInstance::new(*pos).with_uniform_scale(3.0)],
            [color[0], color[1], color[2], 1.0],
        );

        add_entity_with_light(
            &mut map,
            Some(floating_orbs_parent),
            &format!("OrbLight_{}", index),
            LocalTransform {
                translation: Vec3::new(pos[0], pos[1], pos[2]),
                ..Default::default()
            },
            MapLight::point(*color, *intensity, 50.0),
        );
    }

    let crystals_parent = add_empty_entity(&mut map, None, "Crystals", LocalTransform::default());

    let crystal_clusters = [
        ([-30.0, 0.0, 80.0], [0.8, 0.2, 1.0]),
        ([45.0, 0.0, -45.0], [0.2, 1.0, 0.8]),
        ([-80.0, 0.0, -30.0], [1.0, 0.4, 0.2]),
        ([110.0, 0.0, 50.0], [0.2, 0.5, 1.0]),
    ];

    for (cluster_index, (pos, color)) in crystal_clusters.iter().enumerate() {
        let cluster_parent = add_empty_entity(
            &mut map,
            Some(crystals_parent),
            &format!("CrystalCluster_{}", cluster_index),
            LocalTransform {
                translation: Vec3::new(pos[0], pos[1], pos[2]),
                ..Default::default()
            },
        );

        for crystal_index in 0..5 {
            let angle = (crystal_index as f32 / 5.0) * std::f32::consts::TAU;
            let offset_x = angle.cos() * 3.0;
            let offset_z = angle.sin() * 3.0;
            let height = 6.0 + (crystal_index as f32 * 2.0);
            let tilt = nalgebra_glm::quat_angle_axis(
                0.2 + crystal_index as f32 * 0.1,
                &Vec3::new(offset_x, 0.0, offset_z).normalize(),
            );

            add_entity_with_colored_mesh(
                &mut map,
                Some(cluster_parent),
                &format!("Crystal_{}_{}", cluster_index, crystal_index),
                LocalTransform {
                    translation: Vec3::new(offset_x, 0.0, offset_z),
                    rotation: tilt,
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
                "Cone",
                vec![MeshInstance::new([0.0, height / 2.0, 0.0]).with_scale([1.5, height, 1.5])],
                [color[0], color[1], color[2], 1.0],
            );
        }

        add_entity_with_light(
            &mut map,
            Some(cluster_parent),
            &format!("CrystalGlow_{}", cluster_index),
            LocalTransform {
                translation: Vec3::new(0.0, 8.0, 0.0),
                ..Default::default()
            },
            MapLight::point(*color, 150.0, 25.0),
        );
    }

    map
}

fn clear_spawned_entities(world: &mut World, entities: &mut Vec<Entity>) {
    for entity in entities.drain(..) {
        world.queue_command(WorldCommand::DespawnRecursive { entity });
    }
}

fn load_map_into_world(world: &mut World, map: &Map, spawned_entities: &mut Vec<Entity>) {
    match spawn_map(world, map) {
        Ok(result) => {
            spawned_entities.extend(result.node_to_entity.values());
            tracing::info!(
                "Loaded map '{}' with {} entities",
                map.name,
                result.node_to_entity.len()
            );
        }
        Err(error) => {
            tracing::error!("Failed to load map '{}': {}", map.name, error);
        }
    }
}

impl State for MapDemo {
    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;

        self.resources.maps = vec![
            create_forest_map(),
            create_city_map(),
            create_sprawling_world_map(),
        ];
        self.resources.current_map_index = 0;
        self.resources.spawned_entities = Vec::new();

        let camera_position = Vec3::new(0.0, 20.0, 40.0);
        let camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(camera);

        if let Some(map) = self.resources.maps.first() {
            load_map_into_world(world, map, &mut self.resources.spawned_entities);
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        fly_camera_system(world);
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        egui::Window::new("Map Demo")
            .default_pos([10.0, 10.0])
            .show(ctx, |ui| {
                ui.heading("Available Maps");
                ui.separator();

                let current_index = self.resources.current_map_index;

                for (index, map) in self.resources.maps.iter().enumerate() {
                    let is_selected = index == current_index;
                    let button_text = if is_selected {
                        format!("* {} (loaded)", map.name)
                    } else {
                        map.name.clone()
                    };

                    if ui.button(&button_text).clicked() && !is_selected {
                        clear_spawned_entities(world, &mut self.resources.spawned_entities);
                        load_map_into_world(world, map, &mut self.resources.spawned_entities);
                        self.resources.current_map_index = index;
                    }
                }

                ui.separator();
                ui.label(format!(
                    "Spawned entities: {}",
                    self.resources.spawned_entities.len()
                ));

                if let Some(map) = self.resources.maps.get(current_index) {
                    ui.label(format!("Prefabs in map: {}", map.prefabs.len()));
                    ui.label(format!("Root nodes: {}", map.root_nodes().len()));
                    ui.label(format!("Total nodes: {}", map.graph.node_count()));
                }
            });
    }
}
