use crate::ecs::{Map, TileType};
use nightshade::prelude::rand::prelude::*;
use std::collections::VecDeque;

pub fn generate_dungeon(width: i32, height: i32, seed: u64) -> Map {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut map = Map::new(width, height);

    random_fill(&mut map, &mut rng, 45);

    for _ in 0..5 {
        smooth(&mut map);
    }

    connect_regions(&mut map, &mut rng);

    ensure_border_walls(&mut map);

    place_stairs(&mut map, &mut rng);

    map
}

fn random_fill(map: &mut Map, rng: &mut StdRng, wall_chance: u32) {
    for y in 0..map.height {
        for x in 0..map.width {
            let is_border = x == 0 || x == map.width - 1 || y == 0 || y == map.height - 1;
            if is_border || rng.random_range(0..100) < wall_chance {
                map.set_tile(x, y, TileType::Wall);
            } else {
                map.set_tile(x, y, TileType::Floor);
            }
        }
    }
}

fn smooth(map: &mut Map) {
    let mut new_tiles = map.tiles.clone();

    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let wall_count = count_neighbors(map, x, y, TileType::Wall);
            let index = map.index(x, y);

            if wall_count >= 5 {
                new_tiles[index] = TileType::Wall;
            } else if wall_count <= 3 {
                new_tiles[index] = TileType::Floor;
            }
        }
    }

    map.tiles = new_tiles;
}

fn count_neighbors(map: &Map, center_x: i32, center_y: i32, tile_type: TileType) -> i32 {
    let mut count = 0;

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let x = center_x + dx;
            let y = center_y + dy;

            if !map.in_bounds(x, y) || map.get_tile(x, y) == tile_type {
                count += 1;
            }
        }
    }

    count
}

fn connect_regions(map: &mut Map, rng: &mut StdRng) {
    let regions = find_regions(map);

    if regions.is_empty() {
        create_fallback_cave(map);
        return;
    }

    let largest_region = regions.iter().max_by_key(|region| region.len()).unwrap();

    for y in 0..map.height {
        for x in 0..map.width {
            if map.get_tile(x, y) == TileType::Floor {
                let index = map.index(x, y);
                if !largest_region.contains(&index) {
                    map.set_tile(x, y, TileType::Wall);
                }
            }
        }
    }

    let floor_count: usize = map
        .tiles
        .iter()
        .filter(|tile| **tile == TileType::Floor)
        .count();
    let min_floor = ((map.width * map.height) as f32 * 0.3) as usize;

    if floor_count < min_floor {
        expand_cave(map, rng, min_floor - floor_count);
    }
}

fn find_regions(map: &Map) -> Vec<Vec<usize>> {
    let mut visited = vec![false; (map.width * map.height) as usize];
    let mut regions = Vec::new();

    for y in 0..map.height {
        for x in 0..map.width {
            let index = map.index(x, y);
            if map.get_tile(x, y) == TileType::Floor && !visited[index] {
                let region = flood_fill(map, x, y, &mut visited);
                regions.push(region);
            }
        }
    }

    regions
}

fn flood_fill(map: &Map, start_x: i32, start_y: i32, visited: &mut [bool]) -> Vec<usize> {
    let mut region = Vec::new();
    let mut queue = VecDeque::new();

    queue.push_back((start_x, start_y));

    while let Some((x, y)) = queue.pop_front() {
        if !map.in_bounds(x, y) {
            continue;
        }

        let index = map.index(x, y);
        if visited[index] || map.get_tile(x, y) != TileType::Floor {
            continue;
        }

        visited[index] = true;
        region.push(index);

        queue.push_back((x - 1, y));
        queue.push_back((x + 1, y));
        queue.push_back((x, y - 1));
        queue.push_back((x, y + 1));
    }

    region
}

fn create_fallback_cave(map: &mut Map) {
    let center_x = map.width / 2;
    let center_y = map.height / 2;
    let radius = (map.width.min(map.height) / 4) as f32;

    for y in 0..map.height {
        for x in 0..map.width {
            let dx = (x - center_x) as f32;
            let dy = (y - center_y) as f32;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < radius {
                map.set_tile(x, y, TileType::Floor);
            }
        }
    }
}

fn expand_cave(map: &mut Map, rng: &mut StdRng, amount: usize) {
    let mut expanded = 0;
    let mut attempts = 0;
    let max_attempts = amount * 100;

    while expanded < amount && attempts < max_attempts {
        attempts += 1;

        let x = rng.random_range(1..map.width - 1);
        let y = rng.random_range(1..map.height - 1);

        if map.get_tile(x, y) == TileType::Wall {
            let has_floor_neighbor = [(-1, 0), (1, 0), (0, -1), (0, 1)]
                .iter()
                .any(|(dx, dy)| map.get_tile(x + dx, y + dy) == TileType::Floor);

            if has_floor_neighbor {
                map.set_tile(x, y, TileType::Floor);
                expanded += 1;
            }
        }
    }
}

fn ensure_border_walls(map: &mut Map) {
    for x in 0..map.width {
        map.set_tile(x, 0, TileType::Wall);
        map.set_tile(x, map.height - 1, TileType::Wall);
    }
    for y in 0..map.height {
        map.set_tile(0, y, TileType::Wall);
        map.set_tile(map.width - 1, y, TileType::Wall);
    }
}

fn place_stairs(map: &mut Map, rng: &mut StdRng) {
    let floor_tiles: Vec<(i32, i32)> = (0..map.height)
        .flat_map(|y| (0..map.width).map(move |x| (x, y)))
        .filter(|(x, y)| map.get_tile(*x, *y) == TileType::Floor)
        .collect();

    if let Some(&(x, y)) = floor_tiles.choose(rng) {
        map.set_tile(x, y, TileType::StairsDown);
    }
}

pub fn find_random_floor(map: &Map, rng: &mut StdRng) -> Option<(i32, i32)> {
    let floor_tiles: Vec<(i32, i32)> = (0..map.height)
        .flat_map(|y| (0..map.width).map(move |x| (x, y)))
        .filter(|(x, y)| map.get_tile(*x, *y) == TileType::Floor)
        .collect();

    floor_tiles.choose(rng).copied()
}

pub fn find_random_floor_away_from(
    map: &Map,
    rng: &mut StdRng,
    avoid_x: i32,
    avoid_y: i32,
    min_distance: i32,
) -> Option<(i32, i32)> {
    let floor_tiles: Vec<(i32, i32)> = (0..map.height)
        .flat_map(|y| (0..map.width).map(move |x| (x, y)))
        .filter(|(x, y)| {
            if map.get_tile(*x, *y) != TileType::Floor {
                return false;
            }
            let dx = (*x - avoid_x).abs();
            let dy = (*y - avoid_y).abs();
            dx + dy >= min_distance
        })
        .collect();

    floor_tiles.choose(rng).copied()
}
