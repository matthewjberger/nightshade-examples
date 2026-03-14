use crate::ecs::{DRAGGING, ENGINE_ENTITY, PUZZLE_PIECE, PuzzleWorld, Z_ORDER};
use nightshade::prelude::*;

const PIECE_BASE_HEIGHT: f32 = 0.003;
const DRAGGING_LIFT_HEIGHT: f32 = 0.25;
const Z_ORDER_HEIGHT_STEP: f32 = 0.0005;

pub fn render_system(puzzle_world: &PuzzleWorld, world: &mut World) {
    let dragging_entities: Vec<_> = puzzle_world.query_entities(DRAGGING).collect();
    let hovered = puzzle_world.resources.hovered_piece;

    let mut hovered_engine_entity = None;

    for piece_entity in puzzle_world.query_entities(ENGINE_ENTITY | PUZZLE_PIECE | Z_ORDER) {
        let engine_entity = match puzzle_world.get_engine_entity(piece_entity) {
            Some(e) => e.0,
            None => continue,
        };

        let z_order = puzzle_world
            .get_z_order(piece_entity)
            .map(|z| z.0)
            .unwrap_or(0);
        let is_dragging = dragging_entities.contains(&piece_entity);
        let is_hovered = hovered == Some(piece_entity);

        if let Some(transform) = world.core.get_local_transform_mut(engine_entity) {
            let base_y = PIECE_BASE_HEIGHT + z_order as f32 * Z_ORDER_HEIGHT_STEP;
            let target_y = if is_dragging {
                base_y + DRAGGING_LIFT_HEIGHT
            } else {
                base_y
            };
            if (transform.translation.y - target_y).abs() > 0.001 {
                transform.translation.y = target_y;
                world
                    .core
                    .set_local_transform_dirty(engine_entity, LocalTransformDirty);
            }
        }

        if is_hovered && !is_dragging {
            hovered_engine_entity = Some(engine_entity);
        }
    }

    world.resources.graphics.selection_outline_enabled = hovered_engine_entity.is_some();
    world.resources.graphics.bounding_volume_selected_entity = hovered_engine_entity;
}

pub fn drop_pieces_system(puzzle_world: &PuzzleWorld, world: &mut World) {
    let dragging_count = puzzle_world.query_entities(DRAGGING).count();
    if dragging_count > 0 {
        return;
    }

    for piece_entity in puzzle_world.query_entities(ENGINE_ENTITY | Z_ORDER) {
        let engine_entity = match puzzle_world.get_engine_entity(piece_entity) {
            Some(e) => e.0,
            None => continue,
        };

        let z_order = puzzle_world
            .get_z_order(piece_entity)
            .map(|z| z.0)
            .unwrap_or(0);
        let target_y = PIECE_BASE_HEIGHT + z_order as f32 * Z_ORDER_HEIGHT_STEP;

        if let Some(transform) = world.core.get_local_transform_mut(engine_entity) {
            let diff = transform.translation.y - target_y;
            if diff.abs() > 0.01 {
                transform.translation.y = target_y + diff * 0.8;
                if (transform.translation.y - target_y).abs() < 0.01 {
                    transform.translation.y = target_y;
                }
                world
                    .core
                    .set_local_transform_dirty(engine_entity, LocalTransformDirty);
            }
        }
    }
}
