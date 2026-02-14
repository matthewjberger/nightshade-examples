use nightshade::prelude::*;
use rand::Rng;

use crate::city::CHUNK_SIZE;
use crate::descriptors::ChunkData;

const DOCK_DEPTH: f32 = 20.0;
const DOCK_HEIGHT: f32 = 0.4;
const DOCK_Y: f32 = 0.3;
const SUPPORT_RADIUS: f32 = 0.4;
const SUPPORT_HEIGHT: f32 = 3.5;
const SUPPORT_SPACING: f32 = 12.0;
const DETAIL_SPACING: f32 = 12.0;

const BOAT_WATER_Y: f32 = -1.8;
const BOAT_SCALE: Vec3 = Vec3::new(2.0, 2.0, 2.0);

const DOCK_CRANE_TOWER_HEIGHT_MIN: f32 = 15.0;
const DOCK_CRANE_TOWER_HEIGHT_MAX: f32 = 20.0;
const DOCK_CRANE_TOWER_RADIUS: f32 = 0.8;
const DOCK_CRANE_ARM_LENGTH: f32 = 12.0;
const DOCK_CRANE_ARM_THICKNESS: f32 = 0.6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeDirection {
    North,
    South,
    East,
    West,
}

pub fn describe_dock_base(data: &mut ChunkData, chunk_coords: (i32, i32), edge: EdgeDirection) {
    describe_dock_platform(data, chunk_coords, edge);
}

pub fn describe_dock_buildings(
    data: &mut ChunkData,
    chunk_coords: (i32, i32),
    edge: EdgeDirection,
    rng: &mut impl Rng,
) {
    describe_dock_supports(data, chunk_coords, edge);
    describe_dock_cranes(data, chunk_coords, edge, rng);
}

pub fn describe_dock_detail(
    data: &mut ChunkData,
    chunk_coords: (i32, i32),
    edge: EdgeDirection,
    rng: &mut impl Rng,
) {
    describe_dock_details(data, chunk_coords, edge, rng);
    describe_boats(data, chunk_coords, edge, rng);
}

pub fn describe_bridge(
    data: &mut ChunkData,
    corner_chunk: (i32, i32),
    edge_a: EdgeDirection,
    edge_b: EdgeDirection,
) {
    let outer_a = dock_outer_position(corner_chunk, edge_a);
    let outer_b = dock_outer_position(corner_chunk, edge_b);

    let bridge_center_x = (outer_a.x + outer_b.x) / 2.0;
    let bridge_center_z = (outer_a.z + outer_b.z) / 2.0;

    let dx = outer_b.x - outer_a.x;
    let dz = outer_b.z - outer_a.z;
    let bridge_length = (dx * dx + dz * dz).sqrt();

    let angle = dz.atan2(dx);
    let rotation = nalgebra_glm::quat_angle_axis(angle, &Vec3::y());

    data.instance(
        "Cube",
        Vec3::new(bridge_center_x, DOCK_Y, bridge_center_z),
        Vec3::new(bridge_length, 0.3, 3.0),
        "BridgeConcrete",
        Some(rotation),
    );

    let railing_offset = 1.3;
    let railing_height = 1.0;
    for side in [-1.0f32, 1.0] {
        let offset_x = -dz / bridge_length * railing_offset * side;
        let offset_z = dx / bridge_length * railing_offset * side;

        data.instance(
            "Cube",
            Vec3::new(
                bridge_center_x + offset_x,
                DOCK_Y + DOCK_HEIGHT / 2.0 + railing_height / 2.0,
                bridge_center_z + offset_z,
            ),
            Vec3::new(bridge_length, railing_height, 0.1),
            "BridgeMetal",
            Some(rotation),
        );
    }
}

fn dock_outer_position(chunk_coords: (i32, i32), edge: EdgeDirection) -> Vec3 {
    let chunk_base_x = chunk_coords.0 as f32 * CHUNK_SIZE;
    let chunk_base_z = chunk_coords.1 as f32 * CHUNK_SIZE;
    let chunk_center_x = chunk_base_x + CHUNK_SIZE / 2.0;
    let chunk_center_z = chunk_base_z + CHUNK_SIZE / 2.0;

    match edge {
        EdgeDirection::East => Vec3::new(
            chunk_base_x + CHUNK_SIZE + DOCK_DEPTH,
            DOCK_Y,
            chunk_center_z,
        ),
        EdgeDirection::West => Vec3::new(chunk_base_x - DOCK_DEPTH, DOCK_Y, chunk_center_z),
        EdgeDirection::North => Vec3::new(chunk_center_x, DOCK_Y, chunk_base_z - DOCK_DEPTH),
        EdgeDirection::South => Vec3::new(
            chunk_center_x,
            DOCK_Y,
            chunk_base_z + CHUNK_SIZE + DOCK_DEPTH,
        ),
    }
}

fn dock_platform_position_and_size(chunk_coords: (i32, i32), edge: EdgeDirection) -> (Vec3, Vec3) {
    let chunk_base_x = chunk_coords.0 as f32 * CHUNK_SIZE;
    let chunk_base_z = chunk_coords.1 as f32 * CHUNK_SIZE;
    let chunk_center_x = chunk_base_x + CHUNK_SIZE / 2.0;
    let chunk_center_z = chunk_base_z + CHUNK_SIZE / 2.0;

    match edge {
        EdgeDirection::East => (
            Vec3::new(
                chunk_base_x + CHUNK_SIZE + DOCK_DEPTH / 2.0,
                DOCK_Y,
                chunk_center_z,
            ),
            Vec3::new(DOCK_DEPTH, DOCK_HEIGHT, CHUNK_SIZE),
        ),
        EdgeDirection::West => (
            Vec3::new(chunk_base_x - DOCK_DEPTH / 2.0, DOCK_Y, chunk_center_z),
            Vec3::new(DOCK_DEPTH, DOCK_HEIGHT, CHUNK_SIZE),
        ),
        EdgeDirection::North => (
            Vec3::new(chunk_center_x, DOCK_Y, chunk_base_z - DOCK_DEPTH / 2.0),
            Vec3::new(CHUNK_SIZE, DOCK_HEIGHT, DOCK_DEPTH),
        ),
        EdgeDirection::South => (
            Vec3::new(
                chunk_center_x,
                DOCK_Y,
                chunk_base_z + CHUNK_SIZE + DOCK_DEPTH / 2.0,
            ),
            Vec3::new(CHUNK_SIZE, DOCK_HEIGHT, DOCK_DEPTH),
        ),
    }
}

fn describe_dock_platform(data: &mut ChunkData, chunk_coords: (i32, i32), edge: EdgeDirection) {
    let (position, scale) = dock_platform_position_and_size(chunk_coords, edge);
    data.instance("Cube", position, scale, "DockConcrete", None);
}

fn describe_dock_supports(data: &mut ChunkData, chunk_coords: (i32, i32), edge: EdgeDirection) {
    let (platform_pos, platform_scale) = dock_platform_position_and_size(chunk_coords, edge);

    let support_y = DOCK_Y - DOCK_HEIGHT / 2.0 - SUPPORT_HEIGHT / 2.0;

    let (along_axis_min, along_axis_max, cross_pos) = match edge {
        EdgeDirection::East | EdgeDirection::West => {
            let z_min = platform_pos.z - platform_scale.z / 2.0 + SUPPORT_SPACING / 2.0;
            let z_max = platform_pos.z + platform_scale.z / 2.0 - SUPPORT_SPACING / 2.0;
            (z_min, z_max, platform_pos.x)
        }
        EdgeDirection::North | EdgeDirection::South => {
            let x_min = platform_pos.x - platform_scale.x / 2.0 + SUPPORT_SPACING / 2.0;
            let x_max = platform_pos.x + platform_scale.x / 2.0 - SUPPORT_SPACING / 2.0;
            (x_min, x_max, platform_pos.z)
        }
    };

    let support_count = ((along_axis_max - along_axis_min) / SUPPORT_SPACING).floor() as i32 + 1;

    for index in 0..support_count {
        let along_pos = along_axis_min + index as f32 * SUPPORT_SPACING;

        let position = match edge {
            EdgeDirection::East | EdgeDirection::West => Vec3::new(cross_pos, support_y, along_pos),
            EdgeDirection::North | EdgeDirection::South => {
                Vec3::new(along_pos, support_y, cross_pos)
            }
        };

        data.instance(
            "Cylinder",
            position,
            Vec3::new(SUPPORT_RADIUS * 2.0, SUPPORT_HEIGHT, SUPPORT_RADIUS * 2.0),
            "DockMetal",
            None,
        );
    }
}

fn describe_dock_details(
    data: &mut ChunkData,
    chunk_coords: (i32, i32),
    edge: EdgeDirection,
    rng: &mut impl Rng,
) {
    let (platform_pos, platform_scale) = dock_platform_position_and_size(chunk_coords, edge);

    let top_y = DOCK_Y + DOCK_HEIGHT / 2.0;

    let (long_min, long_max, is_x_long) = match edge {
        EdgeDirection::East | EdgeDirection::West => (
            platform_pos.z - platform_scale.z / 2.0 + DETAIL_SPACING / 2.0,
            platform_pos.z + platform_scale.z / 2.0 - DETAIL_SPACING / 2.0,
            false,
        ),
        EdgeDirection::North | EdgeDirection::South => (
            platform_pos.x - platform_scale.x / 2.0 + DETAIL_SPACING / 2.0,
            platform_pos.x + platform_scale.x / 2.0 - DETAIL_SPACING / 2.0,
            true,
        ),
    };

    let cell_count = ((long_max - long_min) / DETAIL_SPACING).floor() as i32 + 1;

    for cell_index in 0..cell_count {
        let long_pos = long_min + cell_index as f32 * DETAIL_SPACING;
        let cross_jitter = rng.random_range(-3.0f32..3.0);

        let (x, z) = if is_x_long {
            let x = long_pos + rng.random_range(-2.0f32..2.0);
            let z = platform_pos.z + cross_jitter;
            (x, z)
        } else {
            let x = platform_pos.x + cross_jitter;
            let z = long_pos + rng.random_range(-2.0f32..2.0);
            (x, z)
        };

        let detail_type = rng.random_range(0u32..3);
        match detail_type {
            0 => {
                let bollard_height = 0.8;
                data.instance(
                    "Cylinder",
                    Vec3::new(x, top_y + bollard_height / 2.0, z),
                    Vec3::new(0.3, bollard_height, 0.3),
                    "DockMetal",
                    None,
                );
            }
            1 => {
                let cargo_model = if rng.random_range(0.0f32..1.0) < 0.5 {
                    crate::kenney::CARGO_CONTAINER_A
                } else {
                    crate::kenney::CARGO_CONTAINER_B
                };
                let cargo_scale = rng.random_range(1.5..2.5);
                data.instance(
                    cargo_model,
                    Vec3::new(x, top_y, z),
                    Vec3::new(cargo_scale, cargo_scale, cargo_scale),
                    crate::kenney::MAT_WATERCRAFT,
                    None,
                );
            }
            _ => {
                let warehouse_width = rng.random_range(3.0..5.0);
                let warehouse_depth = rng.random_range(4.0..6.0);
                let warehouse_height = rng.random_range(3.0..5.0);
                data.instance(
                    "Cube",
                    Vec3::new(x, top_y + warehouse_height / 2.0, z),
                    Vec3::new(warehouse_width, warehouse_height, warehouse_depth),
                    "DockConcrete",
                    None,
                );

                data.instance(
                    "Cube",
                    Vec3::new(x, top_y + warehouse_height + 0.1, z),
                    Vec3::new(warehouse_width + 0.4, 0.2, warehouse_depth + 0.4),
                    "RooftopMetal",
                    None,
                );

                data.instance(
                    crate::kenney::CARGO_PILE,
                    Vec3::new(
                        x + rng.random_range(-2.0f32..2.0),
                        top_y,
                        z + rng.random_range(-2.0f32..2.0),
                    ),
                    Vec3::new(1.5, 1.5, 1.5),
                    crate::kenney::MAT_WATERCRAFT,
                    None,
                );
            }
        }
    }
}

fn describe_dock_cranes(
    data: &mut ChunkData,
    chunk_coords: (i32, i32),
    edge: EdgeDirection,
    rng: &mut impl Rng,
) {
    let (platform_pos, platform_scale) = dock_platform_position_and_size(chunk_coords, edge);
    let top_y = DOCK_Y + DOCK_HEIGHT / 2.0;

    let crane_count = rng.random_range(1u32..3);

    let (along_min, along_max, is_x_along) = match edge {
        EdgeDirection::East | EdgeDirection::West => (
            platform_pos.z - platform_scale.z / 2.0 + 10.0,
            platform_pos.z + platform_scale.z / 2.0 - 10.0,
            false,
        ),
        EdgeDirection::North | EdgeDirection::South => (
            platform_pos.x - platform_scale.x / 2.0 + 10.0,
            platform_pos.x + platform_scale.x / 2.0 - 10.0,
            true,
        ),
    };

    let spacing = (along_max - along_min) / crane_count as f32;

    for crane_index in 0..crane_count {
        let along_pos = along_min + (crane_index as f32 + 0.5) * spacing;
        let tower_height =
            rng.random_range(DOCK_CRANE_TOWER_HEIGHT_MIN..DOCK_CRANE_TOWER_HEIGHT_MAX);

        let (crane_x, crane_z) = if is_x_along {
            (along_pos, platform_pos.z)
        } else {
            (platform_pos.x, along_pos)
        };

        data.instance(
            "Cube",
            Vec3::new(crane_x, top_y + 1.0, crane_z),
            Vec3::new(2.5, 2.0, 2.5),
            "CraneMetal",
            None,
        );

        data.instance(
            "Cylinder",
            Vec3::new(crane_x, top_y + tower_height / 2.0, crane_z),
            Vec3::new(
                DOCK_CRANE_TOWER_RADIUS * 2.0,
                tower_height,
                DOCK_CRANE_TOWER_RADIUS * 2.0,
            ),
            "CraneMetal",
            None,
        );

        let arm_offset_x = if is_x_along {
            0.0
        } else {
            DOCK_CRANE_ARM_LENGTH / 2.0 - 1.0
        };
        let arm_offset_z = if is_x_along {
            DOCK_CRANE_ARM_LENGTH / 2.0 - 1.0
        } else {
            0.0
        };

        data.instance(
            "Cube",
            Vec3::new(
                crane_x + arm_offset_x,
                top_y + tower_height,
                crane_z + arm_offset_z,
            ),
            Vec3::new(
                if is_x_along {
                    DOCK_CRANE_ARM_THICKNESS
                } else {
                    DOCK_CRANE_ARM_LENGTH
                },
                DOCK_CRANE_ARM_THICKNESS,
                if is_x_along {
                    DOCK_CRANE_ARM_LENGTH
                } else {
                    DOCK_CRANE_ARM_THICKNESS
                },
            ),
            "CraneMetal",
            None,
        );

        let cable_x = crane_x + arm_offset_x * 1.5;
        let cable_z = crane_z + arm_offset_z * 1.5;
        let cable_height = 8.0;

        data.instance(
            "Cylinder",
            Vec3::new(cable_x, top_y + tower_height - cable_height / 2.0, cable_z),
            Vec3::new(0.06, cable_height, 0.06),
            "DockMetal",
            None,
        );

        let counterweight_x = crane_x - arm_offset_x * 0.5;
        let counterweight_z = crane_z - arm_offset_z * 0.5;

        data.instance(
            "Cube",
            Vec3::new(counterweight_x, top_y + tower_height, counterweight_z),
            Vec3::new(1.5, 1.5, 1.5),
            "CraneMetal",
            None,
        );
    }
}

fn describe_boats(
    data: &mut ChunkData,
    chunk_coords: (i32, i32),
    edge: EdgeDirection,
    rng: &mut impl Rng,
) {
    use crate::kenney;

    let (platform_pos, platform_scale) = dock_platform_position_and_size(chunk_coords, edge);

    let boat_count = rng.random_range(2u32..5);
    let boat_offset = 6.0;

    let (along_min, along_max, outer_pos, is_x_along) = match edge {
        EdgeDirection::East => (
            platform_pos.z - platform_scale.z / 2.0 + 5.0,
            platform_pos.z + platform_scale.z / 2.0 - 5.0,
            platform_pos.x + platform_scale.x / 2.0 + boat_offset,
            false,
        ),
        EdgeDirection::West => (
            platform_pos.z - platform_scale.z / 2.0 + 5.0,
            platform_pos.z + platform_scale.z / 2.0 - 5.0,
            platform_pos.x - platform_scale.x / 2.0 - boat_offset,
            false,
        ),
        EdgeDirection::North => (
            platform_pos.x - platform_scale.x / 2.0 + 5.0,
            platform_pos.x + platform_scale.x / 2.0 - 5.0,
            platform_pos.z - platform_scale.z / 2.0 - boat_offset,
            true,
        ),
        EdgeDirection::South => (
            platform_pos.x - platform_scale.x / 2.0 + 5.0,
            platform_pos.x + platform_scale.x / 2.0 - 5.0,
            platform_pos.z + platform_scale.z / 2.0 + boat_offset,
            true,
        ),
    };

    let spacing = (along_max - along_min) / boat_count as f32;

    for boat_index in 0..boat_count {
        let along_pos =
            along_min + (boat_index as f32 + 0.5) * spacing + rng.random_range(-1.0f32..1.0);
        let rotation_jitter = rng.random_range(-0.15f32..0.15);

        let (boat_x, boat_z) = if is_x_along {
            (along_pos, outer_pos + rng.random_range(-1.0f32..1.0))
        } else {
            (outer_pos + rng.random_range(-1.0f32..1.0), along_pos)
        };

        let model = kenney::BOAT_MODELS[rng.random_range(0..kenney::BOAT_MODELS.len())];
        let rotation = nalgebra_glm::quat_angle_axis(rotation_jitter, &Vec3::y());

        data.mesh(
            model,
            Vec3::new(boat_x, BOAT_WATER_Y, boat_z),
            BOAT_SCALE,
            kenney::MAT_WATERCRAFT,
            Some(rotation),
        );
    }
}
