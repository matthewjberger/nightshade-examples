use nightshade::prelude::*;
use rand::Rng;

use crate::building::{BuildingSpec, BuildingType};
use crate::descriptors::ChunkData;

pub const WALL_THICKNESS: f32 = 0.3;
pub const DOOR_WIDTH: f32 = 2.0;
pub const DOOR_HEIGHT: f32 = 2.5;
const INTERIOR_FLOOR_Y: f32 = 0.1;
const CEILING_THICKNESS: f32 = 0.2;

const DOOR_SURFACE_OFFSET: f32 = 0.12;

pub fn door_face_for_building(spec: &BuildingSpec) -> u32 {
    let hash =
        ((spec.x * 73.856) as i32).wrapping_mul(31) ^ ((spec.z * 137.294) as i32).wrapping_mul(17);
    hash.unsigned_abs() % 4
}

pub fn building_has_interior(spec: &BuildingSpec) -> bool {
    matches!(
        spec.building_type,
        BuildingType::Warehouse | BuildingType::House | BuildingType::LowRiseOffice
    ) && spec.height < 8.0
        && spec.width >= 4.0
        && spec.depth >= 4.0
}

pub fn describe_interior_shell(data: &mut ChunkData, spec: &BuildingSpec) {
    let half_width = spec.width / 2.0;
    let half_depth = spec.depth / 2.0;
    let wall_height = spec.height;

    let interior_width = spec.width - WALL_THICKNESS * 2.0;
    let interior_depth = spec.depth - WALL_THICKNESS * 2.0;

    data.mesh(
        "Cube",
        Vec3::new(spec.x, INTERIOR_FLOOR_Y / 2.0, spec.z),
        Vec3::new(interior_width, INTERIOR_FLOOR_Y, interior_depth),
        "ConcreteLight",
    );

    data.mesh(
        "Cube",
        Vec3::new(spec.x, wall_height - CEILING_THICKNESS / 2.0, spec.z),
        Vec3::new(spec.width, CEILING_THICKNESS, spec.depth),
        "ConcreteMedium",
    );

    let door_face = door_face_for_building(spec);

    describe_wall_with_optional_door(
        data,
        door_face == 0,
        Vec3::new(
            spec.x,
            wall_height / 2.0,
            spec.z + half_depth - WALL_THICKNESS / 2.0,
        ),
        spec.width,
        wall_height,
        true,
    );

    describe_wall_with_optional_door(
        data,
        door_face == 1,
        Vec3::new(
            spec.x,
            wall_height / 2.0,
            spec.z - half_depth + WALL_THICKNESS / 2.0,
        ),
        spec.width,
        wall_height,
        true,
    );

    describe_wall_with_optional_door(
        data,
        door_face == 2,
        Vec3::new(
            spec.x + half_width - WALL_THICKNESS / 2.0,
            wall_height / 2.0,
            spec.z,
        ),
        spec.depth,
        wall_height,
        false,
    );

    describe_wall_with_optional_door(
        data,
        door_face == 3,
        Vec3::new(
            spec.x - half_width + WALL_THICKNESS / 2.0,
            wall_height / 2.0,
            spec.z,
        ),
        spec.depth,
        wall_height,
        false,
    );
}

pub fn describe_interior_furnishings(
    data: &mut ChunkData,
    spec: &BuildingSpec,
    rng: &mut impl Rng,
) {
    data.light(
        Vec3::new(spec.x, spec.height - 0.5, spec.z),
        Vec3::new(1.0, 0.9, 0.7),
        8.0,
        spec.width.max(spec.depth) * 1.5,
    );

    describe_furniture(data, spec, rng);
}

pub fn describe_exterior_door(data: &mut ChunkData, spec: &BuildingSpec) {
    let half_width = spec.width / 2.0;
    let half_depth = spec.depth / 2.0;
    let door_face = door_face_for_building(spec);

    let door_y = DOOR_HEIGHT / 2.0;

    let (door_x, door_z, door_sx, door_sz) = match door_face {
        0 => (
            spec.x,
            spec.z + half_depth + DOOR_SURFACE_OFFSET,
            DOOR_WIDTH,
            DOOR_SURFACE_OFFSET,
        ),
        1 => (
            spec.x,
            spec.z - half_depth - DOOR_SURFACE_OFFSET,
            DOOR_WIDTH,
            DOOR_SURFACE_OFFSET,
        ),
        2 => (
            spec.x + half_width + DOOR_SURFACE_OFFSET,
            spec.z,
            DOOR_SURFACE_OFFSET,
            DOOR_WIDTH,
        ),
        _ => (
            spec.x - half_width - DOOR_SURFACE_OFFSET,
            spec.z,
            DOOR_SURFACE_OFFSET,
            DOOR_WIDTH,
        ),
    };

    data.mesh(
        "Cube",
        Vec3::new(door_x, door_y, door_z),
        Vec3::new(door_sx, DOOR_HEIGHT, door_sz),
        "DockWood",
    );

    let window_height = DOOR_HEIGHT - 0.3;
    let window_width = 0.6;
    let window_y = window_height / 2.0;
    let gap = 0.3;
    let door_half = DOOR_WIDTH / 2.0;

    match door_face {
        0 | 1 => {
            let left_x = spec.x - door_half - gap - window_width / 2.0;
            let right_x = spec.x + door_half + gap + window_width / 2.0;
            let face_z = if door_face == 0 {
                spec.z + half_depth + DOOR_SURFACE_OFFSET
            } else {
                spec.z - half_depth - DOOR_SURFACE_OFFSET
            };
            data.instance(
                "Cube",
                Vec3::new(left_x, window_y, face_z),
                Vec3::new(window_width, window_height, DOOR_SURFACE_OFFSET),
                "WindowLit",
                None,
            );
            data.instance(
                "Cube",
                Vec3::new(right_x, window_y, face_z),
                Vec3::new(window_width, window_height, DOOR_SURFACE_OFFSET),
                "WindowLit",
                None,
            );
        }
        _ => {
            let left_z = spec.z - door_half - gap - window_width / 2.0;
            let right_z = spec.z + door_half + gap + window_width / 2.0;
            let face_x = if door_face == 2 {
                spec.x + half_width + DOOR_SURFACE_OFFSET
            } else {
                spec.x - half_width - DOOR_SURFACE_OFFSET
            };
            data.instance(
                "Cube",
                Vec3::new(face_x, window_y, left_z),
                Vec3::new(DOOR_SURFACE_OFFSET, window_height, window_width),
                "WindowLit",
                None,
            );
            data.instance(
                "Cube",
                Vec3::new(face_x, window_y, right_z),
                Vec3::new(DOOR_SURFACE_OFFSET, window_height, window_width),
                "WindowLit",
                None,
            );
        }
    }
}

fn describe_wall_with_optional_door(
    data: &mut ChunkData,
    has_door: bool,
    center: Vec3,
    wall_length: f32,
    wall_height: f32,
    is_z_facing: bool,
) {
    if !has_door {
        let (sx, sz) = if is_z_facing {
            (wall_length, WALL_THICKNESS)
        } else {
            (WALL_THICKNESS, wall_length)
        };
        data.mesh_shadow(
            "Cube",
            center,
            Vec3::new(sx, wall_height, sz),
            "ConcreteMedium",
        );
        return;
    }

    let half_length = wall_length / 2.0;
    let left_section_length = (half_length - DOOR_WIDTH / 2.0).max(0.5);
    let right_section_length = left_section_length;
    let above_door_height = (wall_height - DOOR_HEIGHT).max(0.1);

    if is_z_facing {
        let left_center_x = center.x - half_length + left_section_length / 2.0;
        data.mesh_shadow(
            "Cube",
            Vec3::new(left_center_x, center.y, center.z),
            Vec3::new(left_section_length, wall_height, WALL_THICKNESS),
            "ConcreteMedium",
        );

        let right_center_x = center.x + half_length - right_section_length / 2.0;
        data.mesh_shadow(
            "Cube",
            Vec3::new(right_center_x, center.y, center.z),
            Vec3::new(right_section_length, wall_height, WALL_THICKNESS),
            "ConcreteMedium",
        );

        if above_door_height > 0.1 {
            data.mesh_shadow(
                "Cube",
                Vec3::new(center.x, DOOR_HEIGHT + above_door_height / 2.0, center.z),
                Vec3::new(DOOR_WIDTH, above_door_height, WALL_THICKNESS),
                "ConcreteMedium",
            );
        }
    } else {
        let left_center_z = center.z - half_length + left_section_length / 2.0;
        data.mesh_shadow(
            "Cube",
            Vec3::new(center.x, center.y, left_center_z),
            Vec3::new(WALL_THICKNESS, wall_height, left_section_length),
            "ConcreteMedium",
        );

        let right_center_z = center.z + half_length - right_section_length / 2.0;
        data.mesh_shadow(
            "Cube",
            Vec3::new(center.x, center.y, right_center_z),
            Vec3::new(WALL_THICKNESS, wall_height, right_section_length),
            "ConcreteMedium",
        );

        if above_door_height > 0.1 {
            data.mesh_shadow(
                "Cube",
                Vec3::new(center.x, DOOR_HEIGHT + above_door_height / 2.0, center.z),
                Vec3::new(WALL_THICKNESS, above_door_height, DOOR_WIDTH),
                "ConcreteMedium",
            );
        }
    }
}

fn describe_furniture(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    let interior_half_w = (spec.width - WALL_THICKNESS * 2.0) / 2.0 - 0.3;
    let interior_half_d = (spec.depth - WALL_THICKNESS * 2.0) / 2.0 - 0.3;

    match spec.building_type {
        BuildingType::Warehouse => {
            let shelf_count = rng.random_range(2u32..5);
            for shelf_index in 0..shelf_count {
                let offset_x = rng.random_range(-interior_half_w * 0.8..interior_half_w * 0.8);
                let offset_z = -interior_half_d
                    + (shelf_index as f32 + 0.5) * (interior_half_d * 2.0 / shelf_count as f32);
                let shelf_height = rng.random_range(1.5..2.5);
                data.mesh(
                    "Cube",
                    Vec3::new(spec.x + offset_x, shelf_height / 2.0, spec.z + offset_z),
                    Vec3::new(rng.random_range(1.5..3.0), shelf_height, 0.4),
                    "DockWood",
                );
            }

            let crate_count = rng.random_range(1u32..4);
            for _ in 0..crate_count {
                let crate_size = rng.random_range(0.4..0.8);
                let offset_x = rng.random_range(-interior_half_w * 0.6..interior_half_w * 0.6);
                let offset_z = rng.random_range(-interior_half_d * 0.6..interior_half_d * 0.6);
                data.mesh(
                    "Cube",
                    Vec3::new(spec.x + offset_x, crate_size / 2.0, spec.z + offset_z),
                    Vec3::new(crate_size, crate_size, crate_size),
                    "DockWood",
                );
            }
        }
        BuildingType::House => {
            let table_width = rng.random_range(1.0..1.5);
            let table_depth = rng.random_range(0.6..1.0);
            data.mesh(
                "Cube",
                Vec3::new(spec.x, 0.4, spec.z),
                Vec3::new(table_width, 0.8, table_depth),
                "DockWood",
            );

            let chair_offset_x = rng.random_range(-0.8..0.8);
            data.mesh(
                "Cube",
                Vec3::new(
                    spec.x + chair_offset_x,
                    0.25,
                    spec.z + table_depth / 2.0 + 0.3,
                ),
                Vec3::new(0.4, 0.5, 0.4),
                "DockWood",
            );

            data.mesh(
                "Cube",
                Vec3::new(
                    spec.x + interior_half_w * 0.7,
                    0.5,
                    spec.z - interior_half_d * 0.7,
                ),
                Vec3::new(1.2, 1.0, 0.4),
                "ConcreteMedium",
            );
        }
        BuildingType::LowRiseOffice => {
            let desk_count = rng.random_range(1u32..3);
            for desk_index in 0..desk_count {
                let spacing = interior_half_w * 2.0 / (desk_count as f32 + 1.0);
                let offset_x = -interior_half_w + spacing * (desk_index as f32 + 1.0);
                data.mesh(
                    "Cube",
                    Vec3::new(spec.x + offset_x, 0.38, spec.z),
                    Vec3::new(1.2, 0.75, 0.6),
                    "ConcreteMedium",
                );

                data.mesh(
                    "Cube",
                    Vec3::new(spec.x + offset_x, 0.25, spec.z + 0.5),
                    Vec3::new(0.4, 0.5, 0.4),
                    "DockWood",
                );
            }

            data.mesh(
                "Cube",
                Vec3::new(
                    spec.x - interior_half_w * 0.8,
                    1.0,
                    spec.z - interior_half_d * 0.8,
                ),
                Vec3::new(0.3, 2.0, 1.5),
                "ConcreteMedium",
            );
        }
        _ => {}
    }
}
