use crate::ecs::GameWorld;
use crate::hex::world_to_hex;
use nightshade::prelude::*;

pub fn hover_system(game_world: &mut GameWorld, world: &mut World) {
    let hex_width = game_world.resources.hex_width;
    let hex_depth = game_world.resources.hex_depth;

    if let Some(result) = world.resources.gpu_picking.take_result() {
        if result.entity_id.is_some() {
            let coord = world_to_hex(
                result.world_position.x,
                result.world_position.z,
                hex_width,
                hex_depth,
            );
            let is_land = game_world.resources.passable_tiles.contains(&coord);
            game_world.resources.hovered_tile = if is_land { Some(coord) } else { None };
        } else {
            game_world.resources.hovered_tile = None;
        }
    }

    let mouse_pos = world.resources.input.mouse.position;
    world
        .resources
        .gpu_picking
        .request_pick(mouse_pos.x as u32, mouse_pos.y as u32);
}
