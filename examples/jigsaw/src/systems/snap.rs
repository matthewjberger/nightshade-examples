use crate::ecs::{DRAGGING, ENGINE_ENTITY, GROUP_MEMBERS, PIECE_GROUP, PUZZLE_PIECE, PuzzleWorld};
use nightshade::prelude::*;

pub fn snap_system(puzzle_world: &mut PuzzleWorld, world: &mut World) {
    let dragging_count = puzzle_world.query_entities(DRAGGING).count();
    if dragging_count > 0 {
        return;
    }

    snap_to_board(puzzle_world, world);
    check_puzzle_complete(puzzle_world, world);
}

fn check_puzzle_complete(puzzle_world: &mut PuzzleWorld, world: &World) {
    let cols = puzzle_world.resources.grid_cols;
    let rows = puzzle_world.resources.grid_rows;
    let piece_width = puzzle_world.resources.piece_width;
    let piece_height = puzzle_world.resources.piece_height;

    let pieces: Vec<_> = puzzle_world
        .query_entities(ENGINE_ENTITY | PUZZLE_PIECE)
        .collect();

    let expected_count = cols as usize * rows as usize;
    if pieces.len() != expected_count {
        puzzle_world.resources.puzzle_complete = false;
        return;
    }

    for piece in pieces {
        let puzzle_piece = match puzzle_world.get_puzzle_piece(piece) {
            Some(p) => p,
            None => {
                puzzle_world.resources.puzzle_complete = false;
                return;
            }
        };

        if puzzle_piece.rotation != puzzle_piece.correct_rotation {
            puzzle_world.resources.puzzle_complete = false;
            return;
        }

        let engine_entity = match puzzle_world.get_engine_entity(piece) {
            Some(e) => e.0,
            None => {
                puzzle_world.resources.puzzle_complete = false;
                return;
            }
        };

        let transform = match world.get_local_transform(engine_entity) {
            Some(t) => t,
            None => {
                puzzle_world.resources.puzzle_complete = false;
                return;
            }
        };

        let solved_x = (puzzle_piece.grid_pos.x as f32 - (cols as f32 - 1.0) / 2.0) * piece_width;
        let solved_z = (puzzle_piece.grid_pos.y as f32 - (rows as f32 - 1.0) / 2.0) * piece_height;

        let error_x = (transform.translation.x - solved_x).abs();
        let error_z = (transform.translation.z - solved_z).abs();

        if error_x > 0.01 || error_z > 0.01 {
            puzzle_world.resources.puzzle_complete = false;
            return;
        }
    }

    puzzle_world.resources.puzzle_complete = true;
}

fn snap_to_board(puzzle_world: &mut PuzzleWorld, world: &mut World) {
    let snap_threshold = puzzle_world.resources.snap_threshold;
    let piece_width = puzzle_world.resources.piece_width;
    let piece_height = puzzle_world.resources.piece_height;
    let cols = puzzle_world.resources.grid_cols;
    let rows = puzzle_world.resources.grid_rows;

    let groups: Vec<_> = puzzle_world
        .query_entities(PIECE_GROUP | GROUP_MEMBERS)
        .collect();

    for group_entity in groups {
        let members: Vec<freecs::Entity> = puzzle_world
            .get_group_members(group_entity)
            .map(|g| g.members.clone())
            .unwrap_or_default();

        if members.is_empty() {
            continue;
        }

        let first_piece = members[0];
        let puzzle_piece = match puzzle_world.get_puzzle_piece(first_piece) {
            Some(p) => p,
            None => continue,
        };

        if puzzle_piece.rotation != puzzle_piece.correct_rotation {
            continue;
        }

        let engine_entity = match puzzle_world.get_engine_entity(first_piece) {
            Some(e) => e,
            None => continue,
        };
        let transform = match world.get_local_transform(engine_entity.0) {
            Some(t) => t,
            None => continue,
        };

        let current_x = transform.translation.x;
        let current_z = transform.translation.z;

        let solved_x = (puzzle_piece.grid_pos.x as f32 - (cols as f32 - 1.0) / 2.0) * piece_width;
        let solved_z = (puzzle_piece.grid_pos.y as f32 - (rows as f32 - 1.0) / 2.0) * piece_height;

        let error_x = (current_x - solved_x).abs();
        let error_z = (current_z - solved_z).abs();

        if error_x < snap_threshold && error_z < snap_threshold {
            let dx = solved_x - current_x;
            let dz = solved_z - current_z;

            if dx.abs() < 0.001 && dz.abs() < 0.001 {
                continue;
            }

            if let Some(group_members) = puzzle_world.get_group_members_mut(group_entity) {
                group_members.world_x += dx;
                group_members.world_z += dz;
            }

            for piece in &members {
                if let Some(engine_entity) = puzzle_world.get_engine_entity(*piece) {
                    if let Some(transform) = world.get_local_transform_mut(engine_entity.0) {
                        transform.translation.x += dx;
                        transform.translation.z += dz;
                        transform.translation.y = 0.0;
                    }
                    world.set_local_transform_dirty(engine_entity.0, LocalTransformDirty);
                }
            }
        }
    }
}
