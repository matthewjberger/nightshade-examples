use crate::ecs::{FovMap, Map, TileType};

pub const FOV_RADIUS: i32 = 8;

struct CastLightParams<'a> {
    fov_map: &'a mut FovMap,
    map: &'a Map,
    origin_x: i32,
    origin_y: i32,
    radius: i32,
    row: i32,
    start_slope: f32,
    end_slope: f32,
    octant: u8,
}

pub fn compute_fov(fov_map: &mut FovMap, map: &Map, player_x: i32, player_y: i32) {
    fov_map.clear_visible();

    fov_map.set_visible(player_x, player_y, true);

    for octant in 0..8 {
        cast_light(CastLightParams {
            fov_map,
            map,
            origin_x: player_x,
            origin_y: player_y,
            radius: FOV_RADIUS,
            row: 1,
            start_slope: 1.0,
            end_slope: 0.0,
            octant,
        });
    }
}

fn cast_light(params: CastLightParams) {
    let CastLightParams {
        fov_map,
        map,
        origin_x,
        origin_y,
        radius,
        row,
        mut start_slope,
        end_slope,
        octant,
    } = params;
    if start_slope < end_slope {
        return;
    }

    let mut next_start_slope = start_slope;

    for current_row in row..=radius {
        let mut blocked = false;

        let dy = -current_row;
        let mut dx = dy;

        while dx <= 0 {
            let left_slope = (dx as f32 - 0.5) / (dy as f32 + 0.5);
            let right_slope = (dx as f32 + 0.5) / (dy as f32 - 0.5);

            if start_slope < right_slope {
                dx += 1;
                continue;
            }
            if end_slope > left_slope {
                break;
            }

            let (actual_x, actual_y) = transform_octant(origin_x, origin_y, dx, dy, octant);

            let distance_squared = dx * dx + dy * dy;
            let radius_squared = radius * radius;

            if distance_squared <= radius_squared {
                fov_map.set_visible(actual_x, actual_y, true);
            }

            if blocked {
                if is_blocking(map, actual_x, actual_y) {
                    next_start_slope = right_slope;
                    dx += 1;
                    continue;
                } else {
                    blocked = false;
                    start_slope = next_start_slope;
                }
            } else if is_blocking(map, actual_x, actual_y) && current_row < radius {
                blocked = true;
                cast_light(CastLightParams {
                    fov_map,
                    map,
                    origin_x,
                    origin_y,
                    radius,
                    row: current_row + 1,
                    start_slope,
                    end_slope: left_slope,
                    octant,
                });
                next_start_slope = right_slope;
            }

            dx += 1;
        }

        if blocked {
            break;
        }
    }
}

fn transform_octant(origin_x: i32, origin_y: i32, dx: i32, dy: i32, octant: u8) -> (i32, i32) {
    match octant {
        0 => (origin_x + dx, origin_y + dy),
        1 => (origin_x + dy, origin_y + dx),
        2 => (origin_x - dy, origin_y + dx),
        3 => (origin_x - dx, origin_y + dy),
        4 => (origin_x - dx, origin_y - dy),
        5 => (origin_x - dy, origin_y - dx),
        6 => (origin_x + dy, origin_y - dx),
        7 => (origin_x + dx, origin_y - dy),
        _ => (origin_x, origin_y),
    }
}

fn is_blocking(map: &Map, x: i32, y: i32) -> bool {
    if !map.in_bounds(x, y) {
        return true;
    }
    map.get_tile(x, y) == TileType::Wall
}
