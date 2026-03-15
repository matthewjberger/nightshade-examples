use crate::ecs::{Entity, GameWorld, HEX_POSITION, TILE, TileType, UNIT};
use crate::hex::{HexCoord, hex_distance, hex_neighbors};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

#[derive(Eq, PartialEq)]
struct AStarNode {
    coord: HexCoord,
    f_score: i32,
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn astar(from: HexCoord, to: HexCoord, passable: &HashSet<HexCoord>) -> Option<Vec<HexCoord>> {
    let mut g_score: HashMap<HexCoord, i32> = HashMap::new();
    let mut came_from: HashMap<HexCoord, HexCoord> = HashMap::new();
    let mut open = BinaryHeap::new();

    g_score.insert(from, 0);
    open.push(AStarNode {
        coord: from,
        f_score: hex_distance(from, to),
    });

    while let Some(AStarNode { coord: current, .. }) = open.pop() {
        if current == to {
            let mut path = vec![to];
            let mut node = to;
            while let Some(&pred) = came_from.get(&node) {
                path.push(pred);
                node = pred;
            }
            path.reverse();
            return Some(path);
        }

        let current_g = g_score[&current];

        for neighbor in hex_neighbors(current) {
            if !passable.contains(&neighbor) {
                continue;
            }

            let tentative_g = current_g + 1;

            if tentative_g < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g);
                open.push(AStarNode {
                    coord: neighbor,
                    f_score: tentative_g + hex_distance(neighbor, to),
                });
            }
        }
    }

    None
}

fn find_sea_path(game_world: &GameWorld, from: HexCoord, to: HexCoord) -> Option<Vec<HexCoord>> {
    let sea_tiles: HashSet<HexCoord> = game_world
        .query_entities(HEX_POSITION | TILE)
        .filter_map(|entity| {
            let coord = game_world.get_hex_position(entity)?.0;
            let tile = game_world.get_tile(entity)?;
            if tile.tile_type == TileType::Sea {
                Some(coord)
            } else {
                None
            }
        })
        .collect();

    let mut waypoints = sea_tiles.clone();
    waypoints.insert(from);
    waypoints.insert(to);

    astar(from, to, &waypoints)
}

pub fn find_path(game_world: &GameWorld, from: HexCoord, to: HexCoord) -> Option<Vec<HexCoord>> {
    if from == to {
        return Some(vec![from]);
    }

    let passable_tiles: HashSet<HexCoord> = game_world
        .query_entities(HEX_POSITION | TILE)
        .filter_map(|entity| {
            let coord = game_world.get_hex_position(entity)?.0;
            let tile = game_world.get_tile(entity)?;
            if tile.tile_type != TileType::Sea {
                Some(coord)
            } else {
                None
            }
        })
        .collect();

    if !passable_tiles.contains(&from) || !passable_tiles.contains(&to) {
        return None;
    }

    let from_is_port = game_world
        .query_entities(HEX_POSITION | TILE)
        .any(|entity| {
            let Some(coord) = game_world.get_hex_position(entity).map(|h| h.0) else {
                return false;
            };
            let Some(tile) = game_world.get_tile(entity) else {
                return false;
            };
            coord == from && tile.tile_type == TileType::Port
        });

    let to_is_port = game_world
        .query_entities(HEX_POSITION | TILE)
        .any(|entity| {
            let Some(coord) = game_world.get_hex_position(entity).map(|h| h.0) else {
                return false;
            };
            let Some(tile) = game_world.get_tile(entity) else {
                return false;
            };
            coord == to && tile.tile_type == TileType::Port
        });

    if from_is_port && to_is_port {
        return find_sea_path(game_world, from, to);
    }

    astar(from, to, &passable_tiles)
}

pub fn calculate_valid_moves(
    game_world: &GameWorld,
    unit_entity: Entity,
    unit_hex: HexCoord,
    movement_range: i32,
) -> Vec<HexCoord> {
    let unit_positions: HashSet<HexCoord> = game_world
        .query_entities(HEX_POSITION | UNIT)
        .filter(|&entity| entity != unit_entity)
        .filter_map(|entity| game_world.get_hex_position(entity).map(|hex| hex.0))
        .collect();

    let passable_tiles: HashSet<HexCoord> = game_world
        .query_entities(HEX_POSITION | TILE)
        .filter_map(|entity| {
            let coord = game_world.get_hex_position(entity)?.0;
            let tile = game_world.get_tile(entity)?;
            if tile.tile_type != TileType::Sea {
                Some(coord)
            } else {
                None
            }
        })
        .collect();

    let port_tiles: HashSet<HexCoord> = game_world
        .query_entities(HEX_POSITION | TILE)
        .filter_map(|entity| {
            let coord = game_world.get_hex_position(entity)?.0;
            let tile = game_world.get_tile(entity)?;
            if tile.tile_type == TileType::Port {
                Some(coord)
            } else {
                None
            }
        })
        .collect();

    let starting_on_port = port_tiles.contains(&unit_hex);

    let mut distances: HashMap<HexCoord, i32> = HashMap::new();
    let mut queue: VecDeque<HexCoord> = VecDeque::new();

    distances.insert(unit_hex, 0);
    queue.push_back(unit_hex);

    while let Some(current) = queue.pop_front() {
        let current_distance = distances[&current];
        if current_distance >= movement_range {
            continue;
        }

        for neighbor in hex_neighbors(current) {
            if !passable_tiles.contains(&neighbor) {
                continue;
            }
            if distances.contains_key(&neighbor) {
                continue;
            }
            distances.insert(neighbor, current_distance + 1);
            queue.push_back(neighbor);
        }

        if starting_on_port && current == unit_hex {
            for &port_coord in &port_tiles {
                if port_coord == unit_hex {
                    continue;
                }
                if distances.contains_key(&port_coord) {
                    continue;
                }
                distances.insert(port_coord, movement_range);
            }
        }
    }

    distances
        .into_iter()
        .filter(|(coord, distance)| {
            *distance > 0 && *distance <= movement_range && !unit_positions.contains(coord)
        })
        .map(|(coord, _)| coord)
        .collect()
}

pub fn valid_moves_system(game_world: &mut GameWorld) {
    let current_selected: Option<Entity> = game_world.query_selected().next();
    let previous_selected = game_world.resources.previous_selected_unit;

    if current_selected == previous_selected {
        return;
    }

    game_world.resources.valid_move_tiles.clear();

    if let Some(unit_entity) = current_selected
        && let (Some(hex_pos), Some(unit)) = (
            game_world.get_hex_position(unit_entity),
            game_world.get_unit(unit_entity),
        )
    {
        let valid_moves =
            calculate_valid_moves(game_world, unit_entity, hex_pos.0, unit.movement_range);
        for coord in valid_moves {
            game_world.resources.valid_move_tiles.insert(coord);
        }
    }

    game_world.resources.previous_selected_unit = current_selected;
}
