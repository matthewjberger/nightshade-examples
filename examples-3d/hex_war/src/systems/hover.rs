use crate::ecs::{GameWorld, HEX_POSITION, TILE};
use crate::hex::world_to_hex;
use nightshade::prelude::*;

pub fn hover_system(game_world: &mut GameWorld, world: &mut World) {
    let mouse_pos = world.resources.input.mouse.position;

    let hex_width = game_world.resources.hex_width;
    let hex_depth = game_world.resources.hex_depth;

    world
        .resources
        .gpu_picking
        .request_pick(mouse_pos.x as u32, mouse_pos.y as u32);

    if let Some(result) = world.resources.gpu_picking.take_result() {
        if result.entity_id.is_some() {
            let coord = world_to_hex(
                result.world_position.x,
                result.world_position.z,
                hex_width,
                hex_depth,
            );
            let exists = game_world
                .query_entities(HEX_POSITION | TILE)
                .any(|entity| {
                    game_world
                        .get_hex_position(entity)
                        .is_some_and(|hex| hex.0 == coord)
                });
            game_world.resources.hovered_tile = if exists { Some(coord) } else { None };
        } else {
            game_world.resources.hovered_tile = None;
        }
    }
}
