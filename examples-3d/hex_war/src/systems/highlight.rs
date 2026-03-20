use crate::ecs::GameWorld;
use crate::hex::hex_to_world_position;
use crate::hex_overlay_pass::SharedOverlayData;
use crate::rendering::generate_hex_outline;
use nightshade::ecs::world::components::Line;
use nightshade::prelude::*;

pub fn tile_highlight_system(game_world: &mut GameWorld, overlay_data: &SharedOverlayData) {
    let generation = game_world.resources.valid_moves_generation;
    if generation == game_world.resources.previous_highlight_generation {
        return;
    }
    game_world.resources.previous_highlight_generation = generation;

    let valid_move_tiles = &game_world.resources.valid_move_tiles;
    let hex_width = game_world.resources.hex_width;
    let hex_depth = game_world.resources.hex_depth;

    let mut data = overlay_data.lock().unwrap();
    data.hex_width = hex_width;
    data.hex_depth = hex_depth;
    data.positions.clear();

    for coord in valid_move_tiles {
        let world_pos = hex_to_world_position(coord.column, coord.row, hex_width, hex_depth);
        data.positions.push([world_pos.x, world_pos.y, world_pos.z]);
    }
}

pub fn hover_outline_system(
    game_world: &mut GameWorld,
    world: &mut World,
    hover_outline_entity: Option<Entity>,
) {
    let Some(entity) = hover_outline_entity else {
        return;
    };

    let hovered_tile = game_world.resources.hovered_tile;
    let previous_hovered = game_world.resources.previous_hovered_tile;
    let generation = game_world.resources.valid_moves_generation;
    let previous_generation = game_world.resources.previous_highlight_generation;

    if hovered_tile == previous_hovered && generation == previous_generation {
        return;
    }
    game_world.resources.previous_hovered_tile = hovered_tile;

    match hovered_tile {
        Some(coord) => {
            let hex_width = game_world.resources.hex_width;
            let hex_depth = game_world.resources.hex_depth;
            let tile_center = hex_to_world_position(coord.column, coord.row, hex_width, hex_depth);
            let outline_lines = generate_hex_outline(tile_center, hex_width, hex_depth, 0.12);

            let is_valid_move = game_world.resources.valid_move_tiles.contains(&coord);
            let outline_color = if is_valid_move {
                nalgebra_glm::vec4(0.3, 1.0, 0.3, 1.0)
            } else {
                nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0)
            };

            let colored_lines: Vec<Line> = outline_lines
                .into_iter()
                .map(|mut line| {
                    line.color = outline_color;
                    line
                })
                .collect();

            if let Some(lines_component) = world.core.get_lines_mut(entity) {
                lines_component.lines = colored_lines;
                lines_component.mark_dirty();
            }
            if let Some(visibility) = world.core.get_visibility_mut(entity) {
                visibility.visible = true;
            }
        }
        None => {
            if let Some(visibility) = world.core.get_visibility_mut(entity) {
                visibility.visible = false;
            }
        }
    }
}
