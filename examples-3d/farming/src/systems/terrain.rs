use nightshade::ecs::grass::update_grass_player_position;
use nightshade::prelude::*;
use rand::{Rng, SeedableRng};

use crate::ecs::{
    CameraMode, HANDLE, Handle, POSITION, Position, TREE, Tree, TreeState, World as GameWorld,
    chunk_coords,
};
use crate::types::{CHUNK_SIZE, RENDER_DISTANCE};

pub fn update_chunks(game: &mut GameWorld, world: &mut World) {
    let player_pos = get_player_position(game);
    let player_chunk = chunk_coords(player_pos.x, player_pos.z);

    let load_center = match game.resources.camera_mode {
        CameraMode::TopDown => player_chunk,
        CameraMode::ThirdPerson => {
            let yaw = game.resources.camera_yaw;
            let forward_x = -yaw.sin();
            let forward_z = -yaw.cos();
            let offset_distance = 8.0;
            let look_pos_x = player_pos.x + forward_x * offset_distance;
            let look_pos_z = player_pos.z + forward_z * offset_distance;
            chunk_coords(look_pos_x, look_pos_z)
        }
    };

    let mut chunks_to_load = Vec::new();
    let mut chunks_to_unload = Vec::new();

    for dx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for dz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let chunk = (load_center.0 + dx, load_center.1 + dz);
            if !game.resources.loaded_chunks.contains(&chunk) {
                chunks_to_load.push(chunk);
            }
        }
    }

    for &chunk in &game.resources.loaded_chunks.clone() {
        let (chunk_x, chunk_z) = chunk;
        let dx_player = (chunk_x - player_chunk.0).abs();
        let dz_player = (chunk_z - player_chunk.1).abs();
        let dx_center = (chunk_x - load_center.0).abs();
        let dz_center = (chunk_z - load_center.1).abs();

        let too_far_from_player =
            dx_player > RENDER_DISTANCE + 2 || dz_player > RENDER_DISTANCE + 2;
        let too_far_from_center =
            dx_center > RENDER_DISTANCE + 2 || dz_center > RENDER_DISTANCE + 2;

        if too_far_from_player && too_far_from_center {
            chunks_to_unload.push(chunk);
        }
    }

    for chunk in chunks_to_unload {
        unload_chunk(game, world, chunk);
    }

    for chunk in chunks_to_load {
        load_chunk(game, world, chunk);
    }
}

fn get_player_position(game: &GameWorld) -> Vec3 {
    game.resources
        .player_entity
        .and_then(|entity| game.get_position(entity))
        .map(|pos| pos.0)
        .unwrap_or(Vec3::zeros())
}

fn load_chunk(game: &mut GameWorld, world: &mut World, chunk: (i32, i32)) {
    game.resources.loaded_chunks.insert(chunk);

    let seed = (chunk.0 as u64).wrapping_mul(73856093) ^ (chunk.1 as u64).wrapping_mul(19349663);
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let chunk_base_x = chunk.0 as f32 * CHUNK_SIZE;
    let chunk_base_z = chunk.1 as f32 * CHUNK_SIZE;

    let is_farm_area = chunk.0 == 0 && chunk.1 == 0;
    let tree_count: u32 = if is_farm_area {
        0
    } else {
        rng.random_range(3..6)
    };

    let mut chunk_trees = Vec::new();

    for _ in 0..tree_count {
        let x = chunk_base_x + rng.random_range(1.0..CHUNK_SIZE - 1.0);
        let z = chunk_base_z + rng.random_range(1.0..CHUNK_SIZE - 1.0);

        let trunk_height = 2.0 + rng.random_range(0.0..1.5);
        let trunk_radius = 0.2 + rng.random_range(0.0..0.1);
        let tree_scale = 0.8 + rng.random_range(0.0..0.5);

        let position = Vec3::new(x, 0.0, z);
        let visuals = crate::systems::init::create_tree_visual(
            world,
            position,
            trunk_height,
            trunk_radius,
            tree_scale,
        );

        let tier_heights = [1.8 * tree_scale, 1.5 * tree_scale, 1.2 * tree_scale];
        let tier_radii = [2.0 * tree_scale, 1.5 * tree_scale, 1.0 * tree_scale];
        let tier_offsets = [0.0, 1.0 * tree_scale, 1.8 * tree_scale];

        let foliage_y_offsets = [
            trunk_height + tier_offsets[0] + tier_heights[0] / 2.0,
            trunk_height + tier_offsets[1] + tier_heights[1] / 2.0,
            trunk_height + tier_offsets[2] + tier_heights[2] / 2.0,
        ];

        let original_trunk_scale = Vec3::new(trunk_radius * 2.0, trunk_height, trunk_radius * 2.0);
        let original_foliage_scales = [
            Vec3::new(tier_radii[0] * 2.0, tier_heights[0], tier_radii[0] * 2.0),
            Vec3::new(tier_radii[1] * 2.0, tier_heights[1], tier_radii[1] * 2.0),
            Vec3::new(tier_radii[2] * 2.0, tier_heights[2], tier_radii[2] * 2.0),
        ];

        let tree_entity = game.spawn_entities(HANDLE | POSITION | TREE, 1)[0];
        game.set_handle(tree_entity, Handle(visuals.trunk));
        game.set_position(tree_entity, Position(position));
        game.set_tree(
            tree_entity,
            Tree {
                chunk,
                health: 3.0,
                max_health: 3.0,
                state: TreeState::Standing,
                fall_direction: Vec3::x(),
                fall_progress: 0.0,
                shrink_progress: 0.0,
                trunk_height,
                trunk_radius,
                tree_scale,
                trunk_visual: Some(visuals.trunk),
                foliage_visuals: [
                    Some(visuals.foliage[0]),
                    Some(visuals.foliage[1]),
                    Some(visuals.foliage[2]),
                ],
                foliage_y_offsets,
                original_trunk_scale,
                original_foliage_scales,
            },
        );

        chunk_trees.push(tree_entity);
    }

    game.resources.trees.by_chunk.insert(chunk, chunk_trees);
}

fn unload_chunk(game: &mut GameWorld, world: &mut World, chunk: (i32, i32)) {
    game.resources.loaded_chunks.remove(&chunk);

    if let Some(tree_entities) = game.resources.trees.by_chunk.remove(&chunk) {
        for tree_entity in tree_entities {
            if let Some(tree) = game.get_tree(tree_entity) {
                if let Some(trunk) = tree.trunk_visual {
                    world.queue_despawn_entity(trunk);
                }
                for foliage in tree.foliage_visuals.iter().flatten() {
                    world.queue_despawn_entity(*foliage);
                }
            }

            if game.resources.targeted_tree == Some(tree_entity) {
                game.resources.targeted_tree = None;
            }

            game.queue_despawn_entity(tree_entity);
        }
        game.apply_commands();
    }
}

pub fn update_grass(game: &GameWorld, world: &mut World) {
    let Some(region) = game.resources.visuals.grass_region else {
        return;
    };
    let player_pos = get_player_position(game);
    update_grass_player_position(world, region, Vec3::new(player_pos.x, 0.0, player_pos.z));
}

pub fn update_ground(game: &GameWorld, world: &mut World) {
    let Some(ground) = game.resources.visuals.ground else {
        return;
    };
    let player_pos = get_player_position(game);
    if let Some(transform) = world.get_local_transform_mut(ground) {
        transform.translation.x = player_pos.x;
        transform.translation.z = player_pos.z;
    }
    mark_local_transform_dirty(world, ground);
}
