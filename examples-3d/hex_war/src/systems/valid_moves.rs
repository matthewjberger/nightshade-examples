use crate::ecs::{Entity, Faction, GameWorld, unit_stats};
use crate::hex::{HexCoord, hex_neighbors, hex_to_world_position};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

#[derive(PartialEq)]
struct AStarNode {
    coord: HexCoord,
    priority: f32,
}

impl Eq for AStarNode {}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .priority
            .partial_cmp(&self.priority)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn astar(
    from: HexCoord,
    to: HexCoord,
    passable: &HashSet<HexCoord>,
    hex_width: f32,
    hex_depth: f32,
) -> Option<Vec<HexCoord>> {
    let goal_world = hex_to_world_position(to.column, to.row, hex_width, hex_depth);

    let mut g_score: HashMap<HexCoord, i32> = HashMap::new();
    let mut came_from: HashMap<HexCoord, HexCoord> = HashMap::new();
    let mut open = BinaryHeap::new();

    g_score.insert(from, 0);
    let from_world = hex_to_world_position(from.column, from.row, hex_width, hex_depth);
    let from_dist =
        ((goal_world.x - from_world.x).powi(2) + (goal_world.z - from_world.z).powi(2)).sqrt();
    open.push(AStarNode {
        coord: from,
        priority: from_dist,
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

                let neighbor_world =
                    hex_to_world_position(neighbor.column, neighbor.row, hex_width, hex_depth);
                let euclidean = ((goal_world.x - neighbor_world.x).powi(2)
                    + (goal_world.z - neighbor_world.z).powi(2))
                .sqrt();

                open.push(AStarNode {
                    coord: neighbor,
                    priority: tentative_g as f32 + euclidean,
                });
            }
        }
    }

    None
}

fn find_sea_path(game_world: &GameWorld, from: HexCoord, to: HexCoord) -> Option<Vec<HexCoord>> {
    let hex_width = game_world.resources.hex_metrics.hex_width;
    let hex_depth = game_world.resources.hex_metrics.hex_depth;

    let passable_land = &game_world.resources.passable_tiles;

    let map_width = game_world.resources.map_params.map_width;
    let map_height = game_world.resources.map_params.map_height;
    let margin = 5;
    let mut passable: HashSet<HexCoord> = HashSet::new();
    for column in -margin..(map_width + margin) {
        for row in -margin..(map_height + margin) {
            let coord = HexCoord { column, row };
            if !passable_land.contains(&coord) || coord == from || coord == to {
                passable.insert(coord);
            }
        }
    }

    astar(from, to, &passable, hex_width, hex_depth)
}

pub fn find_path(game_world: &GameWorld, from: HexCoord, to: HexCoord) -> Option<Vec<HexCoord>> {
    if from == to {
        return Some(vec![from]);
    }

    let passable_tiles = &game_world.resources.passable_tiles;
    let port_tiles = &game_world.resources.port_tiles;

    if !passable_tiles.contains(&from) || !passable_tiles.contains(&to) {
        return None;
    }

    let from_is_port = port_tiles.contains(&from);
    let to_is_port = port_tiles.contains(&to);

    if from_is_port && to_is_port {
        return find_sea_path(game_world, from, to);
    }

    let hex_width = game_world.resources.hex_metrics.hex_width;
    let hex_depth = game_world.resources.hex_metrics.hex_depth;
    astar(from, to, passable_tiles, hex_width, hex_depth)
}

pub fn calculate_valid_moves(
    game_world: &GameWorld,
    unit_entity: Entity,
    unit_hex: HexCoord,
    movement_range: i32,
) -> Vec<HexCoord> {
    let faction = game_world
        .get_unit(unit_entity)
        .map(|u| u.faction)
        .unwrap_or_default();
    calculate_valid_moves_for_faction(game_world, unit_entity, unit_hex, movement_range, faction)
}

fn calculate_valid_moves_for_faction(
    game_world: &GameWorld,
    unit_entity: Entity,
    unit_hex: HexCoord,
    movement_range: i32,
    faction: Faction,
) -> Vec<HexCoord> {
    let passable_tiles = &game_world.resources.passable_tiles;
    let port_tiles = &game_world.resources.port_tiles;
    let unit_position_map = &game_world.resources.unit_position_map;

    let starting_on_friendly_port = port_tiles.contains(&unit_hex)
        && game_world
            .resources
            .tile_map
            .get(&unit_hex)
            .and_then(|&entity| game_world.get_tile(entity))
            .is_some_and(|tile| tile.faction == Some(faction));

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

        if starting_on_friendly_port && current == unit_hex {
            for &port_coord in port_tiles {
                if port_coord == unit_hex {
                    continue;
                }
                if distances.contains_key(&port_coord) {
                    continue;
                }
                let is_friendly = game_world
                    .resources
                    .tile_map
                    .get(&port_coord)
                    .and_then(|&entity| game_world.get_tile(entity))
                    .is_some_and(|tile| tile.faction == Some(faction));
                if is_friendly {
                    distances.insert(port_coord, movement_range);
                }
            }
        }
    }

    distances
        .into_iter()
        .filter(|(coord, distance)| {
            *distance > 0
                && *distance <= movement_range
                && !unit_position_map
                    .get(coord)
                    .is_some_and(|&entity| entity != unit_entity)
        })
        .map(|(coord, _)| coord)
        .collect()
}

pub fn valid_moves_system(game_world: &mut GameWorld) {
    let current_selected: Option<Entity> = game_world.query_selected().next();
    let previous_selected = game_world.resources.frame_cache.previous_selected_unit;

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
        let movement_range = unit_stats(unit.unit_type).movement_range;
        let valid_moves = calculate_valid_moves(game_world, unit_entity, hex_pos.0, movement_range);
        for coord in valid_moves {
            game_world.resources.valid_move_tiles.insert(coord);
        }
    }

    game_world.resources.valid_moves_generation += 1;
    game_world.resources.frame_cache.previous_selected_unit = current_selected;
}
