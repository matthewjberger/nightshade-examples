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
        BuildingType::Warehouse
            | BuildingType::House
            | BuildingType::LowRiseOffice
            | BuildingType::ApartmentBlock
    ) && spec.height < 12.0
        && spec.width >= 3.5
        && spec.depth >= 3.5
}

pub fn building_is_enterable(spec: &BuildingSpec) -> bool {
    matches!(
        spec.building_type,
        BuildingType::Warehouse | BuildingType::House | BuildingType::ApartmentBlock
    ) && spec.height < 12.0
        && spec.width >= 5.0
        && spec.depth >= 5.0
}

pub fn describe_interior_shell(data: &mut ChunkData, spec: &BuildingSpec) {
    let half_width = spec.width / 2.0;
    let half_depth = spec.depth / 2.0;
    let wall_height = spec.height;

    let interior_width = spec.width - WALL_THICKNESS * 2.0;
    let interior_depth = spec.depth - WALL_THICKNESS * 2.0;

    data.instance(
        "Cube",
        Vec3::new(spec.x, INTERIOR_FLOOR_Y / 2.0, spec.z),
        Vec3::new(interior_width, INTERIOR_FLOOR_Y, interior_depth),
        "ConcreteLight",
        None,
    );

    data.instance(
        "Cube",
        Vec3::new(spec.x, wall_height - CEILING_THICKNESS / 2.0, spec.z),
        Vec3::new(spec.width, CEILING_THICKNESS, spec.depth),
        "ConcreteMedium",
        None,
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
        data.instance(
            "Cube",
            center,
            Vec3::new(sx, wall_height, sz),
            "ConcreteMedium",
            None,
        );
        return;
    }

    let half_length = wall_length / 2.0;
    let left_section_length = (half_length - DOOR_WIDTH / 2.0).max(0.5);
    let right_section_length = left_section_length;
    let above_door_height = (wall_height - DOOR_HEIGHT).max(0.1);

    if is_z_facing {
        let left_center_x = center.x - half_length + left_section_length / 2.0;
        data.instance(
            "Cube",
            Vec3::new(left_center_x, center.y, center.z),
            Vec3::new(left_section_length, wall_height, WALL_THICKNESS),
            "ConcreteMedium",
            None,
        );

        let right_center_x = center.x + half_length - right_section_length / 2.0;
        data.instance(
            "Cube",
            Vec3::new(right_center_x, center.y, center.z),
            Vec3::new(right_section_length, wall_height, WALL_THICKNESS),
            "ConcreteMedium",
            None,
        );

        if above_door_height > 0.1 {
            data.instance(
                "Cube",
                Vec3::new(center.x, DOOR_HEIGHT + above_door_height / 2.0, center.z),
                Vec3::new(DOOR_WIDTH, above_door_height, WALL_THICKNESS),
                "ConcreteMedium",
                None,
            );
        }
    } else {
        let left_center_z = center.z - half_length + left_section_length / 2.0;
        data.instance(
            "Cube",
            Vec3::new(center.x, center.y, left_center_z),
            Vec3::new(WALL_THICKNESS, wall_height, left_section_length),
            "ConcreteMedium",
            None,
        );

        let right_center_z = center.z + half_length - right_section_length / 2.0;
        data.instance(
            "Cube",
            Vec3::new(center.x, center.y, right_center_z),
            Vec3::new(WALL_THICKNESS, wall_height, right_section_length),
            "ConcreteMedium",
            None,
        );

        if above_door_height > 0.1 {
            data.instance(
                "Cube",
                Vec3::new(center.x, DOOR_HEIGHT + above_door_height / 2.0, center.z),
                Vec3::new(WALL_THICKNESS, above_door_height, DOOR_WIDTH),
                "ConcreteMedium",
                None,
            );
        }
    }
}

fn describe_furniture(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    use crate::kenney;

    let interior_half_w = (spec.width - WALL_THICKNESS * 2.0) / 2.0 - 0.3;
    let interior_half_d = (spec.depth - WALL_THICKNESS * 2.0) / 2.0 - 0.3;

    const FURN_SCALE: Vec3 = Vec3::new(2.0, 2.0, 2.0);
    const SMALL_SCALE: Vec3 = Vec3::new(1.8, 1.8, 1.8);

    match spec.building_type {
        BuildingType::Warehouse => {
            let shelf_count = rng.random_range(2u32..4);
            for shelf_index in 0..shelf_count {
                let offset_x = rng.random_range(-interior_half_w * 0.7..interior_half_w * 0.7);
                let offset_z = -interior_half_d
                    + (shelf_index as f32 + 0.5) * (interior_half_d * 2.0 / shelf_count as f32);
                data.instance(
                    kenney::BOOKCASE_OPEN,
                    Vec3::new(spec.x + offset_x, 0.0, spec.z + offset_z),
                    FURN_SCALE,
                    kenney::MAT_FURNITURE,
                    None,
                );
            }

            let crate_count = rng.random_range(2u32..5);
            for _ in 0..crate_count {
                let offset_x = rng.random_range(-interior_half_w * 0.6..interior_half_w * 0.6);
                let offset_z = rng.random_range(-interior_half_d * 0.6..interior_half_d * 0.6);
                let box_model = if rng.random_range(0.0f32..1.0) < 0.5 {
                    kenney::BOX_CLOSED
                } else {
                    kenney::BOX_OPEN
                };
                data.instance(
                    box_model,
                    Vec3::new(spec.x + offset_x, 0.0, spec.z + offset_z),
                    SMALL_SCALE,
                    kenney::MAT_FURNITURE,
                    None,
                );
            }
        }
        BuildingType::House => {
            data.instance(
                kenney::TABLE_ROUND,
                Vec3::new(spec.x, 0.0, spec.z),
                FURN_SCALE,
                kenney::MAT_FURNITURE,
                None,
            );

            let chair_offset_x = rng.random_range(-0.8f32..0.8);
            data.instance(
                kenney::CHAIR_CUSHION,
                Vec3::new(spec.x + chair_offset_x, 0.0, spec.z + 0.8),
                SMALL_SCALE,
                kenney::MAT_FURNITURE,
                None,
            );

            data.instance(
                kenney::BOOKCASE_OPEN,
                Vec3::new(
                    spec.x + interior_half_w * 0.7,
                    0.0,
                    spec.z - interior_half_d * 0.7,
                ),
                FURN_SCALE,
                kenney::MAT_FURNITURE,
                None,
            );

            data.instance(
                kenney::BED_SINGLE,
                Vec3::new(
                    spec.x - interior_half_w * 0.6,
                    0.0,
                    spec.z - interior_half_d * 0.5,
                ),
                FURN_SCALE,
                kenney::MAT_FURNITURE,
                None,
            );

            data.instance(
                kenney::LAMP_CEILING,
                Vec3::new(spec.x, spec.height - CEILING_THICKNESS - 0.1, spec.z),
                SMALL_SCALE,
                kenney::MAT_FURNITURE,
                None,
            );
        }
        BuildingType::LowRiseOffice => {
            let desk_count = rng.random_range(1u32..3);
            for desk_index in 0..desk_count {
                let spacing = interior_half_w * 2.0 / (desk_count as f32 + 1.0);
                let offset_x = -interior_half_w + spacing * (desk_index as f32 + 1.0);

                data.instance(
                    kenney::DESK,
                    Vec3::new(spec.x + offset_x, 0.0, spec.z),
                    FURN_SCALE,
                    kenney::MAT_FURNITURE,
                    None,
                );

                data.instance(
                    kenney::CHAIR_DESK,
                    Vec3::new(spec.x + offset_x, 0.0, spec.z + 0.6),
                    SMALL_SCALE,
                    kenney::MAT_FURNITURE,
                    None,
                );
            }

            data.instance(
                kenney::BOOKCASE_CLOSED,
                Vec3::new(
                    spec.x - interior_half_w * 0.8,
                    0.0,
                    spec.z - interior_half_d * 0.8,
                ),
                FURN_SCALE,
                kenney::MAT_FURNITURE,
                None,
            );
        }
        BuildingType::ApartmentBlock => {
            data.instance(
                kenney::SOFA,
                Vec3::new(
                    spec.x - interior_half_w * 0.6,
                    0.0,
                    spec.z - interior_half_d * 0.7,
                ),
                FURN_SCALE,
                kenney::MAT_FURNITURE,
                None,
            );

            data.instance(
                kenney::TABLE_COFFEE,
                Vec3::new(spec.x + interior_half_w * 0.3, 0.0, spec.z),
                FURN_SCALE,
                kenney::MAT_FURNITURE,
                None,
            );

            data.instance(
                kenney::BOOKCASE_OPEN,
                Vec3::new(
                    spec.x + interior_half_w * 0.6,
                    0.0,
                    spec.z + interior_half_d * 0.7,
                ),
                FURN_SCALE,
                kenney::MAT_FURNITURE,
                None,
            );

            let rug_size = rng.random_range(1.0..1.8);
            data.instance(
                "Cube",
                Vec3::new(spec.x, 0.02, spec.z),
                Vec3::new(rug_size, 0.02, rug_size * 0.7),
                "DockWood",
                None,
            );

            data.instance(
                kenney::FRIDGE,
                Vec3::new(
                    spec.x + interior_half_w * 0.8,
                    0.0,
                    spec.z - interior_half_d * 0.8,
                ),
                SMALL_SCALE,
                kenney::MAT_FURNITURE,
                None,
            );

            data.instance(
                kenney::STOVE,
                Vec3::new(
                    spec.x + interior_half_w * 0.8,
                    0.0,
                    spec.z - interior_half_d * 0.4,
                ),
                SMALL_SCALE,
                kenney::MAT_FURNITURE,
                None,
            );
        }
        _ => {}
    }
}
