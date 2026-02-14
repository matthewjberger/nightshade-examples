use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::mesh::components::{Mesh, Vertex};
use nightshade::ecs::prefab::GltfLoadResult;
use nightshade::ecs::prefab::components::PrefabNode;
use nightshade::ecs::prefab::import_gltf_from_bytes;
use nightshade::ecs::prefab::import_gltf_from_path;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::world::WorldCommand;
use nightshade::prelude::*;
use nightshade::render::wgpu::texture_cache::texture_cache_add_reference;

const NATURE_TREE_OAK_GLB: &[u8] = include_bytes!("../../../assets/kenney/nature/tree_oak.glb");
const NATURE_TREE_DEFAULT_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/nature/tree_default.glb");
const NATURE_TREE_DETAILED_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/nature/tree_detailed.glb");
const NATURE_TREE_PINE_ROUND_A_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/nature/tree_pineRoundA.glb");
const NATURE_TREE_PINE_ROUND_B_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/nature/tree_pineRoundB.glb");
const NATURE_TREE_PALM_GLB: &[u8] = include_bytes!("../../../assets/kenney/nature/tree_palm.glb");
const NATURE_TREE_CONE_GLB: &[u8] = include_bytes!("../../../assets/kenney/nature/tree_cone.glb");
const NATURE_BUSH_GLB: &[u8] = include_bytes!("../../../assets/kenney/nature/plant_bush.glb");
const NATURE_BUSH_LARGE_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/nature/plant_bushLarge.glb");
const NATURE_FLOWER_RED_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/nature/flower_redA.glb");
const NATURE_FLOWER_YELLOW_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/nature/flower_yellowA.glb");
const NATURE_ROCK_SMALL_A_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/nature/rock_smallA.glb");
const NATURE_ROCK_SMALL_B_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/nature/rock_smallB.glb");
const NATURE_ROCK_LARGE_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/nature/rock_largeA.glb");

const FURN_CHAIR_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/chair.glb");
const FURN_CHAIR_CUSHION_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/furniture/chairCushion.glb");
const FURN_CHAIR_DESK_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/chairDesk.glb");
const FURN_TABLE_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/table.glb");
const FURN_TABLE_ROUND_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/furniture/tableRound.glb");
const FURN_TABLE_COFFEE_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/furniture/tableCoffee.glb");
const FURN_DESK_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/desk.glb");
const FURN_SOFA_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/loungeSofa.glb");
const FURN_BED_SINGLE_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/bedSingle.glb");
const FURN_BED_DOUBLE_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/bedDouble.glb");
const FURN_BOOKCASE_OPEN_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/furniture/bookcaseOpen.glb");
const FURN_BOOKCASE_CLOSED_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/furniture/bookcaseClosed.glb");
const FURN_FRIDGE_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/kitchenFridge.glb");
const FURN_STOVE_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/kitchenStove.glb");
const FURN_SINK_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/kitchenSink.glb");
const FURN_CABINET_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/furniture/kitchenCabinet.glb");
const FURN_BATHTUB_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/bathtub.glb");
const FURN_TOILET_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/toilet.glb");
const FURN_TRASHCAN_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/trashcan.glb");
const FURN_BENCH_GLB: &[u8] = include_bytes!("../../../assets/kenney/furniture/bench.glb");
const FURN_BENCH_CUSHION_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/furniture/benchCushion.glb");
const FURN_POTTED_PLANT_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/furniture/pottedPlant.glb");
const FURN_BOX_CLOSED_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/furniture/cardboardBoxClosed.glb");
const FURN_BOX_OPEN_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/furniture/cardboardBoxOpen.glb");
const FURN_LAMP_CEILING_GLB: &[u8] =
    include_bytes!("../../../assets/kenney/furniture/lampSquareCeiling.glb");

pub const SEDAN: &str = "kenney_sedan";
pub const SEDAN_SPORTS: &str = "kenney_sedan_sports";
pub const SUV: &str = "kenney_suv";
pub const SUV_LUXURY: &str = "kenney_suv_luxury";
pub const TAXI: &str = "kenney_taxi";
pub const POLICE: &str = "kenney_police";
pub const VAN: &str = "kenney_van";
pub const DELIVERY: &str = "kenney_delivery";
pub const TRUCK: &str = "kenney_truck";
pub const HATCHBACK_SPORTS: &str = "kenney_hatchback_sports";

pub const CAR_MODELS: &[&str] = &[
    SEDAN,
    SEDAN_SPORTS,
    SUV,
    SUV_LUXURY,
    TAXI,
    POLICE,
    VAN,
    DELIVERY,
    TRUCK,
    HATCHBACK_SPORTS,
];

pub const BOAT_FISHING: &str = "kenney_boat_fishing";
pub const BOAT_TUG_A: &str = "kenney_boat_tug_a";
pub const BOAT_TUG_B: &str = "kenney_boat_tug_b";
pub const BOAT_SPEED_A: &str = "kenney_boat_speed_a";
pub const BOAT_SPEED_B: &str = "kenney_boat_speed_b";
pub const BOAT_ROW: &str = "kenney_boat_row";
pub const BUOY: &str = "kenney_buoy";
pub const CARGO_CONTAINER_A: &str = "kenney_cargo_container_a";
pub const CARGO_CONTAINER_B: &str = "kenney_cargo_container_b";
pub const CARGO_PILE: &str = "kenney_cargo_pile";

pub const BOAT_MODELS: &[&str] = &[
    BOAT_FISHING,
    BOAT_TUG_A,
    BOAT_TUG_B,
    BOAT_SPEED_A,
    BOAT_SPEED_B,
    BOAT_ROW,
];

pub const TREE_OAK: &str = "kenney_tree_oak";
pub const TREE_DEFAULT: &str = "kenney_tree_default";
pub const TREE_DETAILED: &str = "kenney_tree_detailed";
pub const TREE_PINE_ROUND_A: &str = "kenney_tree_pine_round_a";
pub const TREE_PINE_ROUND_B: &str = "kenney_tree_pine_round_b";
pub const TREE_PALM: &str = "kenney_tree_palm";
pub const TREE_CONE: &str = "kenney_tree_cone";
pub const BUSH: &str = "kenney_bush";
pub const BUSH_LARGE: &str = "kenney_bush_large";
pub const FLOWER_RED: &str = "kenney_flower_red";
pub const FLOWER_YELLOW: &str = "kenney_flower_yellow";
pub const ROCK_SMALL_A: &str = "kenney_rock_small_a";
pub const ROCK_SMALL_B: &str = "kenney_rock_small_b";
pub const ROCK_LARGE: &str = "kenney_rock_large";

pub const PARK_TREES: &[&str] = &[
    TREE_OAK,
    TREE_DEFAULT,
    TREE_DETAILED,
    TREE_PINE_ROUND_A,
    TREE_PINE_ROUND_B,
    TREE_PALM,
    TREE_CONE,
];

pub const CHAIR: &str = "kenney_chair";
pub const CHAIR_CUSHION: &str = "kenney_chair_cushion";
pub const CHAIR_DESK: &str = "kenney_chair_desk";
pub const TABLE: &str = "kenney_table";
pub const TABLE_ROUND: &str = "kenney_table_round";
pub const TABLE_COFFEE: &str = "kenney_table_coffee";
pub const DESK: &str = "kenney_desk";
pub const SOFA: &str = "kenney_sofa";
pub const BED_SINGLE: &str = "kenney_bed_single";
pub const BED_DOUBLE: &str = "kenney_bed_double";
pub const BOOKCASE_OPEN: &str = "kenney_bookcase_open";
pub const BOOKCASE_CLOSED: &str = "kenney_bookcase_closed";
pub const FRIDGE: &str = "kenney_fridge";
pub const STOVE: &str = "kenney_stove";
pub const SINK: &str = "kenney_sink";
pub const KITCHEN_CABINET: &str = "kenney_kitchen_cabinet";
pub const BATHTUB: &str = "kenney_bathtub";
pub const TOILET: &str = "kenney_toilet";
pub const TRASHCAN: &str = "kenney_trashcan";
pub const BENCH: &str = "kenney_bench";
pub const BENCH_CUSHION: &str = "kenney_bench_cushion";
pub const POTTED_PLANT: &str = "kenney_potted_plant";
pub const BOX_CLOSED: &str = "kenney_box_closed";
pub const BOX_OPEN: &str = "kenney_box_open";
pub const LAMP_CEILING: &str = "kenney_lamp_ceiling";

pub const MAT_CAR: &str = "KenneyCarColormap";
pub const MAT_WATERCRAFT: &str = "KenneyWatercraftColormap";
pub const MAT_NATURE: &str = "KenneyNature";
pub const MAT_FURNITURE: &str = "KenneyFurniture";

pub fn load_all(world: &mut World) {
    load_car_pack(world);
    load_watercraft_pack(world);
    load_nature_pack(world);
    load_furniture_pack(world);
}

fn merge_meshes_from_prefab(result: &GltfLoadResult, bake_material_colors: bool) -> Option<Mesh> {
    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();
    let identity = Mat4::identity();

    for prefab in &result.prefabs {
        for root_node in &prefab.root_nodes {
            collect_node_meshes(
                root_node,
                &identity,
                &result.meshes,
                bake_material_colors,
                &mut all_vertices,
                &mut all_indices,
            );
        }
    }

    if all_vertices.is_empty() {
        return None;
    }

    let bounding_volume = compute_bounding_volume(&all_vertices);
    Some(Mesh::with_bounding_volume(
        all_vertices,
        all_indices,
        bounding_volume,
    ))
}

fn collect_node_meshes(
    node: &PrefabNode,
    parent_transform: &Mat4,
    meshes: &std::collections::HashMap<String, Mesh>,
    bake_colors: bool,
    all_vertices: &mut Vec<Vertex>,
    all_indices: &mut Vec<u32>,
) {
    let world_transform = parent_transform * node.local_transform.as_matrix();

    if let Some(render_mesh) = &node.components.render_mesh
        && let Some(mesh) = meshes.get(&render_mesh.name)
    {
        let base_color = if bake_colors {
            node.components
                .material
                .as_ref()
                .map(|material| material.base_color)
                .unwrap_or([1.0, 1.0, 1.0, 1.0])
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };

        let vertex_offset = all_vertices.len() as u32;
        let normal_mat = nalgebra_glm::mat4_to_mat3(&world_transform);

        for vertex in &mesh.vertices {
            let position = world_transform
                * Vec4::new(
                    vertex.position[0],
                    vertex.position[1],
                    vertex.position[2],
                    1.0,
                );

            let normal =
                normal_mat * Vec3::new(vertex.normal[0], vertex.normal[1], vertex.normal[2]);
            let normal_len = nalgebra_glm::length(&normal);
            let normal = if normal_len > 1e-6 {
                normal / normal_len
            } else {
                Vec3::new(0.0, 1.0, 0.0)
            };

            let tangent_xyz =
                normal_mat * Vec3::new(vertex.tangent[0], vertex.tangent[1], vertex.tangent[2]);
            let tangent_len = nalgebra_glm::length(&tangent_xyz);
            let tangent_xyz = if tangent_len > 1e-6 {
                tangent_xyz / tangent_len
            } else {
                Vec3::new(1.0, 0.0, 0.0)
            };

            all_vertices.push(Vertex {
                position: [position.x, position.y, position.z],
                normal: [normal.x, normal.y, normal.z],
                tex_coords: vertex.tex_coords,
                tex_coords_1: vertex.tex_coords_1,
                tangent: [
                    tangent_xyz.x,
                    tangent_xyz.y,
                    tangent_xyz.z,
                    vertex.tangent[3],
                ],
                color: [
                    vertex.color[0] * base_color[0],
                    vertex.color[1] * base_color[1],
                    vertex.color[2] * base_color[2],
                    vertex.color[3] * base_color[3],
                ],
            });
        }

        all_indices.extend(mesh.indices.iter().map(|index| index + vertex_offset));
    }

    for child in &node.children {
        collect_node_meshes(
            child,
            &world_transform,
            meshes,
            bake_colors,
            all_vertices,
            all_indices,
        );
    }
}

fn compute_bounding_volume(vertices: &[Vertex]) -> BoundingVolume {
    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

    for vertex in vertices {
        min.x = min.x.min(vertex.position[0]);
        min.y = min.y.min(vertex.position[1]);
        min.z = min.z.min(vertex.position[2]);
        max.x = max.x.max(vertex.position[0]);
        max.y = max.y.max(vertex.position[1]);
        max.z = max.z.max(vertex.position[2]);
    }

    let obb = OrientedBoundingBox::from_aabb(min, max);
    let sphere_radius = nalgebra_glm::length(&obb.half_extents);
    BoundingVolume::new(obb, sphere_radius)
}

fn register_glb_from_path(
    world: &mut World,
    path: &str,
    mesh_name: &str,
    load_textures: bool,
    bake_material_colors: bool,
) {
    let result = match import_gltf_from_path(std::path::Path::new(path)) {
        Ok(result) => result,
        Err(error) => {
            tracing::error!("Failed to load Kenney GLB from path {path} for {mesh_name}: {error}");
            return;
        }
    };

    if load_textures {
        for (name, (rgba_data, width, height)) in &result.textures {
            texture_cache_add_reference(&mut world.resources.texture_cache, name);
            world.queue_command(WorldCommand::LoadTexture {
                name: name.clone(),
                rgba_data: rgba_data.clone(),
                width: *width,
                height: *height,
            });
        }
    }

    if let Some(mesh) = merge_meshes_from_prefab(&result, bake_material_colors) {
        mesh_cache_insert(&mut world.resources.mesh_cache, mesh_name.to_string(), mesh);
    }
}

fn register_glb_from_bytes(
    world: &mut World,
    glb_bytes: &[u8],
    mesh_name: &str,
    load_textures: bool,
    bake_material_colors: bool,
) {
    let result = match import_gltf_from_bytes(glb_bytes) {
        Ok(result) => result,
        Err(error) => {
            tracing::error!("Failed to load Kenney GLB for {mesh_name}: {error}");
            return;
        }
    };

    if load_textures {
        for (name, (rgba_data, width, height)) in &result.textures {
            texture_cache_add_reference(&mut world.resources.texture_cache, name);
            world.queue_command(WorldCommand::LoadTexture {
                name: name.clone(),
                rgba_data: rgba_data.clone(),
                width: *width,
                height: *height,
            });
        }
    }

    if let Some(mesh) = merge_meshes_from_prefab(&result, bake_material_colors) {
        mesh_cache_insert(&mut world.resources.mesh_cache, mesh_name.to_string(), mesh);
    }
}

fn create_textured_material(
    world: &mut World,
    material_name: &str,
    texture_name: &str,
    roughness: f32,
    metallic: f32,
) {
    texture_cache_add_reference(&mut world.resources.texture_cache, texture_name);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.to_string(),
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(texture_name.to_string()),
            roughness,
            metallic,
            ..Default::default()
        },
    );
}

fn create_solid_material(
    world: &mut World,
    material_name: &str,
    base_color: [f32; 4],
    roughness: f32,
    metallic: f32,
) {
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.to_string(),
        Material {
            base_color,
            roughness,
            metallic,
            ..Default::default()
        },
    );
}

fn load_car_pack(world: &mut World) {
    let car_glbs: &[(&str, &str)] = &[
        ("assets/kenney/cars/sedan.glb", SEDAN),
        ("assets/kenney/cars/sedan-sports.glb", SEDAN_SPORTS),
        ("assets/kenney/cars/suv.glb", SUV),
        ("assets/kenney/cars/suv-luxury.glb", SUV_LUXURY),
        ("assets/kenney/cars/taxi.glb", TAXI),
        ("assets/kenney/cars/police.glb", POLICE),
        ("assets/kenney/cars/van.glb", VAN),
        ("assets/kenney/cars/delivery.glb", DELIVERY),
        ("assets/kenney/cars/truck.glb", TRUCK),
        ("assets/kenney/cars/hatchback-sports.glb", HATCHBACK_SPORTS),
    ];

    let mut colormap_registered = false;
    for (path, mesh_name) in car_glbs {
        register_glb_from_path(world, path, mesh_name, !colormap_registered, false);
        if !colormap_registered {
            colormap_registered = true;
        }
    }

    if let Some(texture_name) = find_colormap_texture_name_from_path("assets/kenney/cars/sedan.glb")
    {
        create_textured_material(world, MAT_CAR, &texture_name, 0.45, 0.3);
    } else {
        create_solid_material(world, MAT_CAR, [0.6, 0.6, 0.65, 1.0], 0.45, 0.3);
    }
}

fn load_watercraft_pack(world: &mut World) {
    let watercraft_glbs: &[(&str, &str)] = &[
        (
            "assets/kenney/watercraft/boat-fishing-small.glb",
            BOAT_FISHING,
        ),
        ("assets/kenney/watercraft/boat-tug-a.glb", BOAT_TUG_A),
        ("assets/kenney/watercraft/boat-tug-b.glb", BOAT_TUG_B),
        ("assets/kenney/watercraft/boat-speed-a.glb", BOAT_SPEED_A),
        ("assets/kenney/watercraft/boat-speed-b.glb", BOAT_SPEED_B),
        ("assets/kenney/watercraft/boat-row-small.glb", BOAT_ROW),
        ("assets/kenney/watercraft/buoy.glb", BUOY),
        (
            "assets/kenney/watercraft/cargo-container-a.glb",
            CARGO_CONTAINER_A,
        ),
        (
            "assets/kenney/watercraft/cargo-container-b.glb",
            CARGO_CONTAINER_B,
        ),
        ("assets/kenney/watercraft/cargo-pile-a.glb", CARGO_PILE),
    ];

    let mut colormap_registered = false;
    for (path, mesh_name) in watercraft_glbs {
        register_glb_from_path(world, path, mesh_name, !colormap_registered, false);
        if !colormap_registered {
            colormap_registered = true;
        }
    }

    if let Some(texture_name) =
        find_colormap_texture_name_from_path("assets/kenney/watercraft/boat-fishing-small.glb")
    {
        create_textured_material(world, MAT_WATERCRAFT, &texture_name, 0.55, 0.2);
    } else {
        create_solid_material(world, MAT_WATERCRAFT, [0.5, 0.5, 0.55, 1.0], 0.55, 0.2);
    }
}

fn load_nature_pack(world: &mut World) {
    let nature_glbs: &[(&[u8], &str)] = &[
        (NATURE_TREE_OAK_GLB, TREE_OAK),
        (NATURE_TREE_DEFAULT_GLB, TREE_DEFAULT),
        (NATURE_TREE_DETAILED_GLB, TREE_DETAILED),
        (NATURE_TREE_PINE_ROUND_A_GLB, TREE_PINE_ROUND_A),
        (NATURE_TREE_PINE_ROUND_B_GLB, TREE_PINE_ROUND_B),
        (NATURE_TREE_PALM_GLB, TREE_PALM),
        (NATURE_TREE_CONE_GLB, TREE_CONE),
        (NATURE_BUSH_GLB, BUSH),
        (NATURE_BUSH_LARGE_GLB, BUSH_LARGE),
        (NATURE_FLOWER_RED_GLB, FLOWER_RED),
        (NATURE_FLOWER_YELLOW_GLB, FLOWER_YELLOW),
        (NATURE_ROCK_SMALL_A_GLB, ROCK_SMALL_A),
        (NATURE_ROCK_SMALL_B_GLB, ROCK_SMALL_B),
        (NATURE_ROCK_LARGE_GLB, ROCK_LARGE),
    ];

    let mut textures_loaded = false;
    for (glb_bytes, mesh_name) in nature_glbs {
        register_glb_from_bytes(world, glb_bytes, mesh_name, !textures_loaded, true);
        if !textures_loaded {
            textures_loaded = true;
        }
    }

    if let Some(texture_name) = find_colormap_texture_name_from_bytes(NATURE_TREE_OAK_GLB) {
        create_textured_material(world, MAT_NATURE, &texture_name, 0.85, 0.0);
    } else {
        create_solid_material(world, MAT_NATURE, [1.0, 1.0, 1.0, 1.0], 0.85, 0.0);
    }
}

fn load_furniture_pack(world: &mut World) {
    let furniture_glbs: &[(&[u8], &str)] = &[
        (FURN_CHAIR_GLB, CHAIR),
        (FURN_CHAIR_CUSHION_GLB, CHAIR_CUSHION),
        (FURN_CHAIR_DESK_GLB, CHAIR_DESK),
        (FURN_TABLE_GLB, TABLE),
        (FURN_TABLE_ROUND_GLB, TABLE_ROUND),
        (FURN_TABLE_COFFEE_GLB, TABLE_COFFEE),
        (FURN_DESK_GLB, DESK),
        (FURN_SOFA_GLB, SOFA),
        (FURN_BED_SINGLE_GLB, BED_SINGLE),
        (FURN_BED_DOUBLE_GLB, BED_DOUBLE),
        (FURN_BOOKCASE_OPEN_GLB, BOOKCASE_OPEN),
        (FURN_BOOKCASE_CLOSED_GLB, BOOKCASE_CLOSED),
        (FURN_FRIDGE_GLB, FRIDGE),
        (FURN_STOVE_GLB, STOVE),
        (FURN_SINK_GLB, SINK),
        (FURN_CABINET_GLB, KITCHEN_CABINET),
        (FURN_BATHTUB_GLB, BATHTUB),
        (FURN_TOILET_GLB, TOILET),
        (FURN_TRASHCAN_GLB, TRASHCAN),
        (FURN_BENCH_GLB, BENCH),
        (FURN_BENCH_CUSHION_GLB, BENCH_CUSHION),
        (FURN_POTTED_PLANT_GLB, POTTED_PLANT),
        (FURN_BOX_CLOSED_GLB, BOX_CLOSED),
        (FURN_BOX_OPEN_GLB, BOX_OPEN),
        (FURN_LAMP_CEILING_GLB, LAMP_CEILING),
    ];

    let mut textures_loaded = false;
    for (glb_bytes, mesh_name) in furniture_glbs {
        register_glb_from_bytes(world, glb_bytes, mesh_name, !textures_loaded, true);
        if !textures_loaded {
            textures_loaded = true;
        }
    }

    if let Some(texture_name) = find_colormap_texture_name_from_bytes(FURN_CHAIR_GLB) {
        create_textured_material(world, MAT_FURNITURE, &texture_name, 0.75, 0.0);
    } else {
        create_solid_material(world, MAT_FURNITURE, [1.0, 1.0, 1.0, 1.0], 0.75, 0.0);
    }
}

fn find_colormap_texture_name_from_path(path: &str) -> Option<String> {
    let result = import_gltf_from_path(std::path::Path::new(path)).ok()?;
    result.textures.into_keys().next()
}

fn find_colormap_texture_name_from_bytes(glb_bytes: &[u8]) -> Option<String> {
    let result = import_gltf_from_bytes(glb_bytes).ok()?;
    result.textures.into_keys().next()
}
