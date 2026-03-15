use crate::ecs::GameWorld;
use crate::hex::{HexCoord, hex_to_world_position};
use crate::instancing::InstancedTileGroup;
use crate::rendering::generate_hex_outline;
use nightshade::ecs::world::components::Line;
use nightshade::prelude::*;
use std::collections::HashSet;

const DEFAULT_TINT: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const HOVER_TINT: [f32; 4] = [1.5, 1.5, 1.3, 1.0];

fn valid_move_tint(pulse: f32) -> [f32; 4] {
    let glow = 2.0 + 1.0 * pulse;
    [glow, glow, glow, 1.0]
}

fn hover_valid_tint(_pulse: f32) -> [f32; 4] {
    [3.5, 3.5, 3.5, 1.0]
}

pub fn tile_highlight_system(
    game_world: &mut GameWorld,
    world: &mut World,
    instanced_tile_groups: &[InstancedTileGroup],
) {
    let hovered_tile = game_world.resources.hovered_tile;
    let valid_move_tiles = &game_world.resources.valid_move_tiles;
    let has_valid_moves = !valid_move_tiles.is_empty();

    let mut currently_highlighted: HashSet<HexCoord> = valid_move_tiles.clone();
    if let Some(coord) = hovered_tile {
        currently_highlighted.insert(coord);
    }

    let hover_changed = hovered_tile != game_world.resources.previous_hovered_tile;
    let set_changed = currently_highlighted != game_world.resources.previously_highlighted;

    if !set_changed && !hover_changed && !has_valid_moves {
        return;
    }

    let pulse = if has_valid_moves {
        let uptime = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
        (uptime * 3.0).sin() * 0.5 + 0.5
    } else {
        0.0
    };

    let tiles_to_reset: Vec<HexCoord> = game_world
        .resources
        .previously_highlighted
        .difference(&currently_highlighted)
        .copied()
        .collect();

    for group in instanced_tile_groups {
        let Some(instanced_mesh) = world.core.get_instanced_mesh_mut(group.entity) else {
            continue;
        };

        for coord in &tiles_to_reset {
            if let Some(&instance_index) = group.coord_to_instance.get(coord) {
                instanced_mesh.set_instance_tint(instance_index, DEFAULT_TINT);
            }
        }

        for coord in &currently_highlighted {
            if let Some(&instance_index) = group.coord_to_instance.get(coord) {
                let is_hovered = hovered_tile == Some(*coord);
                let is_valid_move = valid_move_tiles.contains(coord);
                let tint = match (is_hovered, is_valid_move) {
                    (true, true) => hover_valid_tint(pulse),
                    (true, false) => HOVER_TINT,
                    (false, true) => valid_move_tint(pulse),
                    (false, false) => DEFAULT_TINT,
                };
                instanced_mesh.set_instance_tint(instance_index, tint);
            }
        }
    }

    game_world.resources.previously_highlighted = currently_highlighted;
    game_world.resources.previous_hovered_tile = hovered_tile;
}

pub fn hover_outline_system(
    game_world: &GameWorld,
    world: &mut World,
    hover_outline_entity: Option<Entity>,
) {
    let Some(entity) = hover_outline_entity else {
        return;
    };

    let hovered_tile = game_world.resources.hovered_tile;

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
