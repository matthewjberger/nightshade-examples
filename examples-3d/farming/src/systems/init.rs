use nightshade::ecs::grass::{
    GrassConfig, GrassSpecies, add_grass_species, enable_grass, enable_grass_interactors,
    spawn_grass_region,
};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::terrain::{NoiseConfig, NoiseType, TerrainConfig};
use nightshade::prelude::*;

use crate::data::{
    ITEM_CAULIFLOWER_SEED, ITEM_PARSNIP_SEED, ITEM_POTATO_SEED, NPC_DEFINITIONS, SHOP_ITEMS,
};
use crate::ecs::{
    Crop, GameEntity, HANDLE, Handle, NPC, Npc, PLAYER, POSITION, Player, Position, Tree,
    VisualEntities, World as GameWorld,
};
use crate::types::{
    CAMERA_DISTANCE, CAMERA_HEIGHT, GROUND_SIZE, PLAYER_RADIUS, PLAYER_STAMINA_MAX, Season, Weather,
};

pub fn initialize(game: &mut GameWorld, world: &mut World) {
    init_graphics(world);
    create_materials(world);

    let ground = spawn_ground(world);
    let grass_region = spawn_grass(world);
    let (player_visual, tool_visual) = spawn_player(game, world);
    let camera = spawn_camera(game, world);
    let sun = spawn_sun(world);

    spawn_npcs(game, world);
    init_resources(game);

    game.resources.visuals = VisualEntities {
        camera: Some(camera),
        sun: Some(sun),
        ground: Some(ground),
        grass_region: Some(grass_region),
        player_visual: Some(player_visual),
        tool_visual: Some(tool_visual),
    };

    crate::systems::terrain::update_chunks(game, world);
}

pub fn recreate_visuals(game: &mut GameWorld, world: &mut World) {
    create_materials(world);

    let ground = spawn_ground(world);
    let grass_region = spawn_grass(world);
    let camera = spawn_camera(game, world);
    let sun = spawn_sun(world);

    let player_pos = game
        .resources
        .player_entity
        .and_then(|e| game.get_position(e))
        .map(|p| p.0)
        .unwrap_or(Vec3::new(0.0, PLAYER_RADIUS, 0.0));

    let player_visual = spawn_mesh(
        world,
        "Sphere",
        player_pos,
        Vec3::new(
            PLAYER_RADIUS * 2.0,
            PLAYER_RADIUS * 2.0,
            PLAYER_RADIUS * 2.0,
        ),
    );
    apply_material(world, player_visual, "PlayerBody");

    let tool_visual = spawn_mesh(
        world,
        "Cube",
        Vec3::new(
            player_pos.x + 0.5,
            player_pos.y + PLAYER_RADIUS * 0.3,
            player_pos.z + 0.3,
        ),
        Vec3::new(0.15, 0.6, 0.15),
    );
    world.set_casts_shadow(tool_visual, CastsShadow);
    apply_material(world, tool_visual, "ToolHoe");

    if let Some(player_entity) = game.resources.player_entity {
        game.set_handle(player_entity, Handle(player_visual));
    }

    game.resources.visuals = VisualEntities {
        camera: Some(camera),
        sun: Some(sun),
        ground: Some(ground),
        grass_region: Some(grass_region),
        player_visual: Some(player_visual),
        tool_visual: Some(tool_visual),
    };

    for popup in &mut game.resources.popups.popups {
        popup.entity = None;
    }

    recreate_npc_visuals(game, world);
    recreate_tree_visuals(game, world);
    recreate_farm_visuals(game, world);
}

fn recreate_npc_visuals(game: &mut GameWorld, world: &mut World) {
    let npc_entities: Vec<_> = game.resources.npcs.clone();

    for npc_entity in npc_entities {
        let Some(npc) = game.get_npc(npc_entity) else {
            continue;
        };
        let npc_type = npc.npc_type;

        let Some(position) = game.get_position(npc_entity) else {
            continue;
        };

        let definition = crate::data::NPC_DEFINITIONS
            .iter()
            .find(|d| d.npc_type == npc_type);

        let color = definition.map(|d| d.color).unwrap_or([0.5, 0.5, 0.5, 1.0]);

        let visual = create_npc_visual(world, position.0, color);
        game.set_handle(npc_entity, Handle(visual));
    }
}

struct TreeVisualData {
    entity: GameEntity,
    tree: Tree,
    position: Vec3,
}

fn recreate_tree_visuals(game: &mut GameWorld, world: &mut World) {
    let tree_data: Vec<TreeVisualData> = game
        .resources
        .trees
        .by_chunk
        .values()
        .flat_map(|trees| trees.iter())
        .filter_map(|&tree_entity| {
            let tree = *game.get_tree(tree_entity)?;
            let position = game.get_position(tree_entity)?.0;
            Some(TreeVisualData {
                entity: tree_entity,
                tree,
                position,
            })
        })
        .collect();

    for data in tree_data {
        let visuals = create_tree_visual(
            world,
            data.position,
            data.tree.trunk_height,
            data.tree.trunk_radius,
            data.tree.tree_scale,
        );
        game.set_handle(data.entity, Handle(visuals.trunk));
        game.modify_tree(data.entity, |tree| {
            tree.trunk_visual = Some(visuals.trunk);
            tree.foliage_visuals = [
                Some(visuals.foliage[0]),
                Some(visuals.foliage[1]),
                Some(visuals.foliage[2]),
            ];
        });
    }
}

struct TileVisualData {
    entity: GameEntity,
    coords: (i32, i32),
    is_watered: bool,
}

struct CropVisualData {
    entity: GameEntity,
    coords: (i32, i32),
    crop: Crop,
}

fn recreate_farm_visuals(game: &mut GameWorld, world: &mut World) {
    let tile_data: Vec<TileVisualData> = game
        .resources
        .farm
        .tiles
        .iter()
        .map(|(&coords, &entity)| {
            let is_watered = game.get_tile(entity).map(|t| t.watered).unwrap_or(false);
            TileVisualData {
                entity,
                coords,
                is_watered,
            }
        })
        .collect();

    for data in tile_data {
        let tile_pos = crate::ecs::tile_center(data.coords.0, data.coords.1);

        let visual = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | RENDER_MESH | MATERIAL_REF,
            1,
        )[0];

        world.set_local_transform(
            visual,
            LocalTransform {
                translation: Vec3::new(tile_pos.x, 0.02, tile_pos.z),
                rotation: Quat::identity(),
                scale: Vec3::new(
                    crate::types::TILE_SIZE * 0.9,
                    0.05,
                    crate::types::TILE_SIZE * 0.9,
                ),
            },
        );
        world.set_render_mesh(visual, RenderMesh::new("Cube"));
        mark_local_transform_dirty(world, visual);
        world.resources.mesh_render_state.mark_entity_added(visual);

        let material = if data.is_watered {
            "WateredSoil"
        } else {
            "TilledSoil"
        };
        apply_material(world, visual, material);

        game.set_handle(data.entity, Handle(visual));
    }

    let crop_data: Vec<CropVisualData> = game
        .resources
        .farm
        .crops
        .iter()
        .filter_map(|(&coords, &entity)| {
            let crop = *game.get_crop(entity)?;
            Some(CropVisualData {
                entity,
                coords,
                crop,
            })
        })
        .collect();

    for data in crop_data {
        let tile_pos = crate::ecs::tile_center(data.coords.0, data.coords.1);
        let scale = crate::data::get_crop_scale(data.crop.growth_stage, data.crop.max_growth_stage);

        let visual = world.spawn_entities(
            LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | RENDER_MESH
                | MATERIAL_REF
                | CASTS_SHADOW,
            1,
        )[0];

        world.set_local_transform(
            visual,
            LocalTransform {
                translation: Vec3::new(tile_pos.x, scale * 0.5, tile_pos.z),
                rotation: Quat::identity(),
                scale: Vec3::new(scale * 0.4, scale, scale * 0.4),
            },
        );
        world.set_render_mesh(visual, RenderMesh::new("Cube"));
        world.set_casts_shadow(visual, CastsShadow);
        mark_local_transform_dirty(world, visual);
        world.resources.mesh_render_state.mark_entity_added(visual);

        let material_name = crate::data::get_crop_material_name(
            data.crop.crop_type,
            data.crop.growth_stage,
            data.crop.max_growth_stage,
        );
        apply_material(world, visual, material_name);

        game.set_handle(data.entity, Handle(visual));
    }
}

fn init_graphics(world: &mut World) {
    world.resources.graphics.atmosphere = Atmosphere::Hdr;
    world.resources.graphics.show_grid = false;
    world.resources.user_interface.enabled = false;
    world.resources.retained_ui.enabled = true;
    world.resources.graphics.selection_outline_enabled = true;
    world.resources.graphics.selection_outline_color = [1.0, 0.6, 0.0, 1.0];
    world.resources.graphics.fog = Some(Fog {
        start: 25.0,
        end: 60.0,
        color: [0.45, 0.55, 0.5],
    });
}

fn spawn_ground(world: &mut World) -> Entity {
    let ground = spawn_mesh(
        world,
        "Cube",
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(GROUND_SIZE, 1.0, GROUND_SIZE),
    );
    material_registry_insert(
        &mut world.resources.material_registry,
        "Ground".to_string(),
        Material {
            base_color: [0.15, 0.35, 0.12, 1.0],
            roughness: 0.95,
            metallic: 0.0,
            ..Default::default()
        },
    );
    apply_material(world, ground, "Ground");
    ground
}

fn spawn_grass(world: &mut World) -> Entity {
    let plane = world.spawn_entities(
        LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | RENDER_MESH | MATERIAL_REF,
        1,
    )[0];
    world.set_local_transform(
        plane,
        LocalTransform {
            translation: Vec3::new(0.0, -0.01, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1000.0, 0.1, 1000.0),
        },
    );
    world.set_render_mesh(plane, RenderMesh::new("Cube"));
    material_registry_insert(
        &mut world.resources.material_registry,
        "GrassPlane".to_string(),
        Material {
            base_color: [0.25, 0.38, 0.18, 1.0],
            roughness: 0.95,
            metallic: 0.0,
            ..Default::default()
        },
    );
    apply_material(world, plane, "GrassPlane");

    let terrain_config = TerrainConfig {
        width: 500.0,
        depth: 500.0,
        resolution_x: 64,
        resolution_z: 64,
        height_scale: 0.0,
        noise: NoiseConfig {
            seed: 42,
            frequency: 0.01,
            octaves: 1,
            lacunarity: 2.0,
            persistence: 0.5,
            noise_type: NoiseType::Perlin,
        },
        uv_scale: [1.0, 1.0],
    };

    let mut config = GrassConfig::default()
        .with_density(64)
        .with_wind(0.6, 1.0)
        .with_wind_direction(1.0, 0.3)
        .with_stream_radius(100.0);
    config.lod_distances = [15.0, 40.0, 80.0, 100.0];
    config.lod_density_scales = [1.0, 0.5, 0.2, 0.05];

    let region = spawn_grass_region(world, config);
    nightshade::ecs::grass::set_grass_terrain(world, region, terrain_config);
    add_grass_species(world, region, GrassSpecies::meadow(), 4.0);
    add_grass_species(world, region, GrassSpecies::short(), 3.0);
    add_grass_species(world, region, GrassSpecies::tall(), 1.0);
    enable_grass(world, region, true);
    enable_grass_interactors(world, region, true);

    region
}

fn spawn_player(game: &mut GameWorld, world: &mut World) -> (Entity, Entity) {
    let position = Vec3::new(0.0, PLAYER_RADIUS, 0.0);

    let visual = spawn_mesh(
        world,
        "Sphere",
        position,
        Vec3::new(
            PLAYER_RADIUS * 2.0,
            PLAYER_RADIUS * 2.0,
            PLAYER_RADIUS * 2.0,
        ),
    );
    material_registry_insert(
        &mut world.resources.material_registry,
        "PlayerBody".to_string(),
        Material {
            base_color: [0.2, 0.6, 0.3, 1.0],
            roughness: 0.4,
            metallic: 0.1,
            ..Default::default()
        },
    );
    apply_material(world, visual, "PlayerBody");

    let tool = spawn_mesh(
        world,
        "Cube",
        Vec3::new(
            position.x + 0.5,
            position.y + PLAYER_RADIUS * 0.3,
            position.z + 0.3,
        ),
        Vec3::new(0.15, 0.6, 0.15),
    );
    world.set_casts_shadow(tool, CastsShadow);
    apply_material(world, tool, "ToolHoe");

    let player_entity = game.spawn_entities(HANDLE | POSITION | PLAYER, 1)[0];
    game.set_handle(player_entity, Handle(visual));
    game.set_position(player_entity, Position(position));
    game.set_player(
        player_entity,
        Player {
            facing: Vec3::new(0.0, 0.0, 1.0),
            height: 0.0,
            vertical_velocity: 0.0,
            grounded: true,
            stamina: PLAYER_STAMINA_MAX,
            max_stamina: PLAYER_STAMINA_MAX,
            ..Default::default()
        },
    );
    game.resources.player_entity = Some(player_entity);

    (visual, tool)
}

fn spawn_camera(game: &GameWorld, world: &mut World) -> Entity {
    let player_pos = game
        .resources
        .player_entity
        .and_then(|e| game.get_position(e))
        .map(|p| p.0)
        .unwrap_or(Vec3::zeros());

    let cam_pos = player_pos + Vec3::new(0.0, CAMERA_HEIGHT, CAMERA_DISTANCE);
    let camera = world.spawn_entities(
        LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | CAMERA,
        1,
    )[0];

    if let Some(transform) = world.get_local_transform_mut(camera) {
        transform.translation = cam_pos;
        let direction = nalgebra_glm::normalize(&(player_pos - cam_pos));
        let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&direction, &Vec3::y()));
        let up = nalgebra_glm::cross(&right, &direction);
        transform.rotation =
            nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[right, up, -direction]));
    }
    mark_local_transform_dirty(world, camera);

    world.set_camera(
        camera,
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 45.0_f32.to_radians(),
                z_far: Some(500.0),
                z_near: 0.1,
            }),
            smoothing: Some(Smoothing::default()),
        },
    );
    world.resources.active_camera = Some(camera);

    camera
}

fn spawn_sun(world: &mut World) -> Entity {
    let sun = nightshade::prelude::spawn_sun(world);
    if let Some(light) = world.get_light_mut(sun) {
        light.cast_shadows = true;
        light.intensity = 1.5;
    }
    sun
}

fn spawn_npcs(game: &mut GameWorld, world: &mut World) {
    for definition in NPC_DEFINITIONS {
        let visual = create_npc_visual(world, definition.position, definition.color);

        let npc_entity = game.spawn_entities(HANDLE | POSITION | NPC, 1)[0];
        game.set_handle(npc_entity, Handle(visual));
        game.set_position(npc_entity, Position(definition.position));
        game.set_npc(
            npc_entity,
            Npc {
                npc_type: definition.npc_type,
                friendship: 0,
                talked_today: false,
            },
        );
        game.resources.npcs.push(npc_entity);
    }
}

fn init_resources(game: &mut GameWorld) {
    game.resources.day = 1;
    game.resources.season = Season::Spring;
    game.resources.hour = 6.0;
    game.resources.weather = Weather::Sunny;
    game.resources.money = 500;

    game.resources.inventory.hotbar[6].item_id = Some(ITEM_PARSNIP_SEED);
    game.resources.inventory.hotbar[6].quantity = 15;
    game.resources.inventory.hotbar[7].item_id = Some(ITEM_CAULIFLOWER_SEED);
    game.resources.inventory.hotbar[7].quantity = 5;
    game.resources.inventory.hotbar[8].item_id = Some(ITEM_POTATO_SEED);
    game.resources.inventory.hotbar[8].quantity = 10;

    game.resources.shop_items = SHOP_ITEMS.to_vec();
}

fn apply_material(world: &mut World, entity: Entity, name: &str) {
    if let Some(&idx) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(idx);
    }
    world.set_material_ref(entity, MaterialRef::new(name.to_string()));
}

fn create_materials(world: &mut World) {
    let materials = [
        ("TilledSoil", [0.35, 0.25, 0.15, 1.0], 0.9, 0.0),
        ("WateredSoil", [0.25, 0.18, 0.1, 1.0], 0.7, 0.0),
        ("CropGrowth1", [0.3, 0.5, 0.2, 1.0], 0.8, 0.0),
        ("CropGrowth2", [0.35, 0.55, 0.25, 1.0], 0.8, 0.0),
        ("CropGrowth3", [0.4, 0.6, 0.3, 1.0], 0.8, 0.0),
        ("CropGrowth4", [0.45, 0.65, 0.35, 1.0], 0.8, 0.0),
        ("CropMature_Parsnip", [0.95, 0.9, 0.7, 1.0], 0.7, 0.0),
        ("CropMature_Cauliflower", [0.95, 0.95, 0.9, 1.0], 0.6, 0.0),
        ("CropMature_Potato", [0.7, 0.55, 0.35, 1.0], 0.8, 0.0),
        ("CropMature_Tomato", [0.9, 0.2, 0.15, 1.0], 0.5, 0.0),
        ("CropMature_Corn", [0.95, 0.85, 0.3, 1.0], 0.7, 0.0),
        ("CropMature_Pumpkin", [0.95, 0.5, 0.1, 1.0], 0.6, 0.0),
        ("ToolHoe", [0.5, 0.35, 0.2, 1.0], 0.6, 0.3),
        ("ToolWateringCan", [0.3, 0.4, 0.5, 1.0], 0.4, 0.5),
        ("ToolAxe", [0.6, 0.6, 0.65, 1.0], 0.3, 0.7),
        ("ToolPickaxe", [0.5, 0.5, 0.55, 1.0], 0.35, 0.6),
        ("ToolScythe", [0.55, 0.55, 0.6, 1.0], 0.3, 0.65),
        ("ToolSword", [0.7, 0.7, 0.75, 1.0], 0.2, 0.8),
        ("TreeTrunk", [0.4, 0.28, 0.18, 1.0], 0.85, 0.0),
        ("TreeFoliage", [0.2, 0.45, 0.15, 1.0], 0.9, 0.0),
    ];

    for (name, color, roughness, metallic) in materials {
        material_registry_insert(
            &mut world.resources.material_registry,
            name.to_string(),
            Material {
                base_color: color,
                roughness,
                metallic,
                ..Default::default()
            },
        );
    }
}

pub fn apply_material_by_name(world: &mut World, entity: Entity, name: &str) {
    apply_material(world, entity, name);
}

pub fn create_npc_visual(world: &mut World, position: Vec3, color: [f32; 4]) -> Entity {
    let visual = spawn_mesh(
        world,
        "Cylinder",
        Vec3::new(position.x, 1.0, position.z),
        Vec3::new(1.0, 2.0, 1.0),
    );
    world.set_casts_shadow(visual, CastsShadow);

    let mat_name = format!("Npc_{}", visual.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        mat_name.clone(),
        Material {
            base_color: color,
            roughness: 0.6,
            metallic: 0.1,
            ..Default::default()
        },
    );
    apply_material(world, visual, &mat_name);

    visual
}

pub struct TreeVisuals {
    pub trunk: Entity,
    pub foliage: [Entity; 3],
}

pub fn create_tree_visual(
    world: &mut World,
    position: Vec3,
    trunk_height: f32,
    trunk_radius: f32,
    tree_scale: f32,
) -> TreeVisuals {
    let x = position.x;
    let z = position.z;

    let trunk = spawn_mesh(
        world,
        "Cylinder",
        Vec3::new(x, trunk_height / 2.0, z),
        Vec3::new(trunk_radius * 2.0, trunk_height, trunk_radius * 2.0),
    );
    apply_material(world, trunk, "TreeTrunk");
    world.set_casts_shadow(trunk, CastsShadow);

    let tier_heights = [1.8 * tree_scale, 1.5 * tree_scale, 1.2 * tree_scale];
    let tier_radii = [2.0 * tree_scale, 1.5 * tree_scale, 1.0 * tree_scale];
    let tier_offsets = [0.0, 1.0 * tree_scale, 1.8 * tree_scale];

    let mut foliage = [Entity::default(); 3];
    for tier in 0..3 {
        let foliage_y = trunk_height + tier_offsets[tier] + tier_heights[tier] / 2.0;
        let foliage_entity = spawn_mesh(
            world,
            "Cone",
            Vec3::new(x, foliage_y, z),
            Vec3::new(
                tier_radii[tier] * 2.0,
                tier_heights[tier],
                tier_radii[tier] * 2.0,
            ),
        );
        apply_material(world, foliage_entity, "TreeFoliage");
        world.set_casts_shadow(foliage_entity, CastsShadow);
        foliage[tier] = foliage_entity;
    }

    TreeVisuals { trunk, foliage }
}
