use nightshade::prelude::*;
use rand::Rng;

use crate::descriptors::ChunkData;

#[derive(Debug, Clone, Copy)]
pub enum BuildingType {
    Skyscraper,
    OfficeTower,
    LowRiseOffice,
    ApartmentBlock,
    House,
    Warehouse,
    Park,
}

pub struct BuildingSpec {
    pub building_type: BuildingType,
    pub x: f32,
    pub z: f32,
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub body_material: &'static str,
    pub roof_material: &'static str,
}

const FLOOR_HEIGHT: f32 = 3.0;
const WINDOW_BAND_HEIGHT: f32 = 1.0;
const WINDOW_SURFACE_OFFSET: f32 = 0.08;
const WINDOW_LIT_PROBABILITY: f32 = 0.6;
const MAX_WINDOW_FLOORS: i32 = 8;
const WINDOW_SKIP_THRESHOLD: f32 = 24.0;

const NEON_MATERIALS: &[&str] = &["NeonRed", "NeonBlue", "NeonPink"];

const SHOP_NAMES: &[&str] = &[
    "CAFE", "BAR", "HOTEL", "DINER", "PIZZA", "SUSHI", "OPEN", "CLUB", "GYM", "SPA", "NEWS",
    "BOOKS", "MUSIC", "JAZZ", "LOANS", "WINE", "TAXI", "NAILS", "SALON", "PUB",
];

const SHOPFRONT_HEIGHT: f32 = 2.0;
const SHOPFRONT_Y: f32 = 1.5;
const SHOPFRONT_OFFSET: f32 = 0.10;

const BALCONY_WIDTH: f32 = 2.0;
const BALCONY_HEIGHT: f32 = 0.15;
const BALCONY_DEPTH: f32 = 0.8;
const RAILING_HEIGHT: f32 = 0.6;
const RAILING_THICKNESS: f32 = 0.05;

const BILLBOARD_MATERIALS: &[&str] = &["BillboardWhite", "BillboardYellow"];
const BILLBOARD_PROBABILITY: f32 = 0.15;
const SCREEN_BILLBOARD_PROBABILITY: f32 = 0.20;

const PROXY_SCALE: f32 = 0.90;
const PROXY_HEIGHT_SCALE: f32 = 0.95;

pub fn proxy_material_position_scale(spec: &BuildingSpec) -> (&'static str, Vec3, Vec3) {
    match spec.building_type {
        BuildingType::Park => (
            "ParkGreen",
            Vec3::new(spec.x, 0.05, spec.z),
            Vec3::new(spec.width * PROXY_SCALE, 0.1, spec.depth * PROXY_SCALE),
        ),
        _ => (
            spec.body_material,
            Vec3::new(spec.x, spec.height * PROXY_HEIGHT_SCALE / 2.0, spec.z),
            Vec3::new(
                spec.width * PROXY_SCALE,
                spec.height * PROXY_HEIGHT_SCALE,
                spec.depth * PROXY_SCALE,
            ),
        ),
    }
}

pub fn describe_building_body(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    match spec.building_type {
        BuildingType::Park => describe_park(data, spec, rng),
        BuildingType::House => describe_house_body(data, spec),
        BuildingType::Skyscraper => describe_skyscraper_body(data, spec, rng),
        _ => describe_generic_building_body(data, spec, rng),
    }
}

pub fn describe_building_detail(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    match spec.building_type {
        BuildingType::Park | BuildingType::House | BuildingType::Warehouse => {}
        BuildingType::Skyscraper => describe_skyscraper_detail(data, spec, rng),
        _ => describe_generic_building_detail(data, spec, rng),
    }
}

fn describe_park(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    data.mesh(
        "Cube",
        Vec3::new(spec.x, 0.05, spec.z),
        Vec3::new(spec.width, 0.1, spec.depth),
        "ParkGreen",
    );

    let tree_count = rng.random_range(2u32..5);
    for _ in 0..tree_count {
        let tree_x = spec.x + rng.random_range(-spec.width * 0.3..spec.width * 0.3);
        let tree_z = spec.z + rng.random_range(-spec.depth * 0.3..spec.depth * 0.3);
        let trunk_height = rng.random_range(2.0..4.0);
        let is_conifer = rng.random_range(0.0f32..1.0) < 0.5;

        data.mesh(
            "Cylinder",
            Vec3::new(tree_x, trunk_height / 2.0, tree_z),
            Vec3::new(0.3, trunk_height, 0.3),
            "TreeTrunk",
        );

        if is_conifer {
            let cone_height = rng.random_range(2.5..4.0);
            let cone_radius = rng.random_range(0.8..1.2);
            data.mesh(
                "Cone",
                Vec3::new(tree_x, trunk_height + cone_height / 2.0, tree_z),
                Vec3::new(cone_radius * 2.0, cone_height, cone_radius * 2.0),
                "ParkDarkGreen",
            );
        } else {
            let foliage_radius = rng.random_range(1.0..2.0);
            data.mesh(
                "Sphere",
                Vec3::new(tree_x, trunk_height + foliage_radius * 0.5, tree_z),
                Vec3::new(
                    foliage_radius * 2.0,
                    foliage_radius * 2.0,
                    foliage_radius * 2.0,
                ),
                "ParkDarkGreen",
            );
        }
    }

    let bench_count = rng.random_range(1u32..3);
    for _ in 0..bench_count {
        let edge_side = rng.random_range(0u32..4);
        let (bench_x, bench_z) = match edge_side {
            0 => (
                spec.x + rng.random_range(-spec.width * 0.3..spec.width * 0.3),
                spec.z + spec.depth * 0.4,
            ),
            1 => (
                spec.x + rng.random_range(-spec.width * 0.3..spec.width * 0.3),
                spec.z - spec.depth * 0.4,
            ),
            2 => (
                spec.x + spec.width * 0.4,
                spec.z + rng.random_range(-spec.depth * 0.3..spec.depth * 0.3),
            ),
            _ => (
                spec.x - spec.width * 0.4,
                spec.z + rng.random_range(-spec.depth * 0.3..spec.depth * 0.3),
            ),
        };

        data.mesh(
            "Cube",
            Vec3::new(bench_x, 0.25, bench_z),
            Vec3::new(1.5, 0.5, 0.5),
            "DockWood",
        );
    }

    let flower_count = rng.random_range(1u32..3);
    for _ in 0..flower_count {
        let flower_x = spec.x + rng.random_range(-spec.width * 0.25..spec.width * 0.25);
        let flower_z = spec.z + rng.random_range(-spec.depth * 0.25..spec.depth * 0.25);
        let flower_material = if rng.random_range(0.0f32..1.0) < 0.5 {
            "FlowerRed"
        } else {
            "FlowerYellow"
        };

        data.mesh(
            "Cube",
            Vec3::new(flower_x, 0.08, flower_z),
            Vec3::new(rng.random_range(1.0..2.0), 0.15, rng.random_range(1.0..2.0)),
            flower_material,
        );
    }

    data.campfire(Vec3::new(spec.x, 0.3, spec.z));
}

fn describe_house_body(data: &mut ChunkData, spec: &BuildingSpec) {
    data.mesh_shadow(
        "Cube",
        Vec3::new(spec.x, spec.height / 2.0, spec.z),
        Vec3::new(spec.width, spec.height, spec.depth),
        spec.body_material,
    );

    let roof_height = spec.height * 0.4;
    data.mesh(
        "Cone",
        Vec3::new(spec.x, spec.height + roof_height / 2.0, spec.z),
        Vec3::new(spec.width * 1.1, roof_height, spec.depth * 1.1),
        spec.roof_material,
    );
}

fn skyscraper_sections(spec: &BuildingSpec) -> Vec<(f32, f32, f32, f32)> {
    if spec.height > 30.0 {
        let section_count = if spec.height > 45.0 { 3 } else { 2 };
        let (base_frac, mid_frac, top_frac) = if section_count == 3 {
            (0.60, 0.25, 0.15)
        } else {
            (0.65, 0.35, 0.0)
        };

        let mut sections = Vec::new();
        let base_height = spec.height * base_frac;
        sections.push((0.0, base_height, spec.width, spec.depth));

        let mid_height = spec.height * mid_frac;
        sections.push((
            base_height,
            mid_height,
            spec.width * 0.70,
            spec.depth * 0.70,
        ));

        if section_count == 3 {
            let top_height = spec.height * top_frac;
            sections.push((
                base_height + mid_height,
                top_height,
                spec.width * 0.50,
                spec.depth * 0.50,
            ));
        }

        sections
    } else {
        vec![(0.0, spec.height, spec.width, spec.depth)]
    }
}

fn describe_skyscraper_body(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    let sections = skyscraper_sections(spec);

    for &(y_base, section_height, width, depth) in &sections {
        data.mesh_shadow(
            "Cube",
            Vec3::new(spec.x, y_base + section_height / 2.0, spec.z),
            Vec3::new(width, section_height, depth),
            spec.body_material,
        );
    }

    for &(y_base, section_height, width, depth) in &sections {
        describe_window_strips_at(
            data,
            &WindowStripParams {
                center_x: spec.x,
                center_z: spec.z,
                y_base,
                y_top: y_base + section_height,
                width,
                depth,
            },
            rng,
        );
    }

    let &(_, _, top_width, top_depth) = sections.last().unwrap();

    let top_choice = rng.random_range(0u32..3);
    match top_choice {
        0 => {
            let sphere_radius = top_width.min(top_depth) * 0.4;
            data.mesh(
                "Sphere",
                Vec3::new(spec.x, spec.height + sphere_radius * 0.5, spec.z),
                Vec3::new(
                    sphere_radius * 2.0,
                    sphere_radius * 2.0,
                    sphere_radius * 2.0,
                ),
                spec.body_material,
            );
        }
        1 => {
            let cyl_height = rng.random_range(2.0..5.0);
            let cyl_radius = top_width.min(top_depth) * 0.25;
            data.mesh(
                "Cylinder",
                Vec3::new(spec.x, spec.height + cyl_height / 2.0, spec.z),
                Vec3::new(cyl_radius * 2.0, cyl_height, cyl_radius * 2.0),
                spec.body_material,
            );
        }
        _ => {}
    }

    let antenna_height = rng.random_range(3.0..8.0);
    data.mesh(
        "Cylinder",
        Vec3::new(spec.x, spec.height + antenna_height / 2.0, spec.z),
        Vec3::new(0.15, antenna_height, 0.15),
        "Antenna",
    );

    describe_rooftop_detail(data, spec, rng);
}

fn describe_skyscraper_detail(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    describe_shopfront(data, spec);

    if rng.random_range(0.0f32..1.0) < 0.20 {
        describe_neon_sign(data, spec, rng);
    }

    if rng.random_range(0.0f32..1.0) < SCREEN_BILLBOARD_PROBABILITY {
        describe_screen_billboard(data, spec, rng);
    }
}

fn describe_generic_building_body(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    data.mesh_shadow(
        "Cube",
        Vec3::new(spec.x, spec.height / 2.0, spec.z),
        Vec3::new(spec.width, spec.height, spec.depth),
        spec.body_material,
    );

    if !matches!(spec.building_type, BuildingType::Warehouse) {
        describe_window_strips(data, spec, 0.0, spec.height, spec.width, spec.depth, rng);
    }

    describe_rooftop_detail(data, spec, rng);
}

fn describe_generic_building_detail(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    if matches!(
        spec.building_type,
        BuildingType::OfficeTower | BuildingType::LowRiseOffice | BuildingType::ApartmentBlock
    ) {
        describe_shopfront(data, spec);
    }

    if matches!(spec.building_type, BuildingType::ApartmentBlock) {
        describe_balconies(data, spec, rng);
    }

    if matches!(
        spec.building_type,
        BuildingType::OfficeTower | BuildingType::LowRiseOffice
    ) && rng.random_range(0.0f32..1.0) < BILLBOARD_PROBABILITY
    {
        describe_billboard(data, spec, rng);
    }

    if rng.random_range(0.0f32..1.0) < 0.20 {
        describe_neon_sign(data, spec, rng);
    }

    if matches!(spec.building_type, BuildingType::OfficeTower)
        && rng.random_range(0.0f32..1.0) < SCREEN_BILLBOARD_PROBABILITY * 0.5
    {
        describe_screen_billboard(data, spec, rng);
    }
}

fn describe_window_strips(
    data: &mut ChunkData,
    spec: &BuildingSpec,
    y_base: f32,
    y_top: f32,
    width: f32,
    depth: f32,
    rng: &mut impl Rng,
) {
    describe_window_strips_at(
        data,
        &WindowStripParams {
            center_x: spec.x,
            center_z: spec.z,
            y_base,
            y_top,
            width,
            depth,
        },
        rng,
    );
}

struct WindowStripParams {
    center_x: f32,
    center_z: f32,
    y_base: f32,
    y_top: f32,
    width: f32,
    depth: f32,
}

fn describe_window_strips_at(data: &mut ChunkData, params: &WindowStripParams, rng: &mut impl Rng) {
    let WindowStripParams {
        center_x,
        center_z,
        y_base,
        y_top,
        width,
        depth,
    } = *params;
    let section_height = y_top - y_base;
    let total_floors = ((section_height - FLOOR_HEIGHT) / FLOOR_HEIGHT).floor() as i32;

    if total_floors <= 0 {
        return;
    }

    let floor_step = if section_height > WINDOW_SKIP_THRESHOLD {
        2
    } else {
        1
    };
    let floor_count = (total_floors / floor_step).min(MAX_WINDOW_FLOORS);

    let half_width = width / 2.0;
    let half_depth = depth / 2.0;

    struct FaceParams {
        x: f32,
        z: f32,
        scale_x: f32,
        scale_z: f32,
    }

    let faces = [
        FaceParams {
            x: center_x + half_width + WINDOW_SURFACE_OFFSET,
            z: center_z,
            scale_x: WINDOW_SURFACE_OFFSET,
            scale_z: depth * 0.85,
        },
        FaceParams {
            x: center_x - half_width - WINDOW_SURFACE_OFFSET,
            z: center_z,
            scale_x: WINDOW_SURFACE_OFFSET,
            scale_z: depth * 0.85,
        },
        FaceParams {
            x: center_x,
            z: center_z + half_depth + WINDOW_SURFACE_OFFSET,
            scale_x: width * 0.85,
            scale_z: WINDOW_SURFACE_OFFSET,
        },
        FaceParams {
            x: center_x,
            z: center_z - half_depth - WINDOW_SURFACE_OFFSET,
            scale_x: width * 0.85,
            scale_z: WINDOW_SURFACE_OFFSET,
        },
    ];

    for window_index in 0..floor_count {
        let floor_number = 1 + window_index * floor_step;
        let floor_y = y_base + floor_number as f32 * FLOOR_HEIGHT + FLOOR_HEIGHT / 2.0;
        if floor_y + WINDOW_BAND_HEIGHT / 2.0 > y_top {
            break;
        }

        for face in &faces {
            let is_lit = rng.random_range(0.0f32..1.0) < WINDOW_LIT_PROBABILITY;
            let material = if is_lit { "WindowLit" } else { "WindowDark" };

            data.instance(
                "Cube",
                Vec3::new(face.x, floor_y, face.z),
                Vec3::new(face.scale_x, WINDOW_BAND_HEIGHT, face.scale_z),
                material,
                None,
            );
        }
    }
}

fn describe_neon_sign(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    let face_choice = rng.random_range(0u32..4);
    let sign_y = FLOOR_HEIGHT;
    let offset: f32 = 0.15;

    let half_width = spec.width / 2.0;
    let half_depth = spec.depth / 2.0;

    let (x, z, rotation) = match face_choice {
        0 => (
            spec.x + half_width + offset,
            spec.z,
            nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::y()),
        ),
        1 => (
            spec.x - half_width - offset,
            spec.z,
            nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::y()),
        ),
        2 => (
            spec.x,
            spec.z + half_depth + offset,
            nalgebra_glm::quat_identity(),
        ),
        _ => (
            spec.x,
            spec.z - half_depth - offset,
            nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &Vec3::y()),
        ),
    };

    let material = NEON_MATERIALS[rng.random_range(0..NEON_MATERIALS.len())];
    let text = SHOP_NAMES[rng.random_range(0..SHOP_NAMES.len())];

    data.neon_sign(text, Vec3::new(x, sign_y, z), material, 0.8, rotation);

    data.light(
        Vec3::new(x, sign_y, z),
        match material {
            "NeonRed" => Vec3::new(1.0, 0.15, 0.1),
            "NeonBlue" => Vec3::new(0.1, 0.3, 1.0),
            _ => Vec3::new(1.0, 0.2, 0.6),
        },
        3.0,
        8.0,
    );

    if rng.random_range(0.0f32..1.0) < 0.20 {
        data.sparks(Vec3::new(x, sign_y + 0.3, z));
    }
}

fn describe_shopfront(data: &mut ChunkData, spec: &BuildingSpec) {
    let half_width = spec.width / 2.0;
    let half_depth = spec.depth / 2.0;

    let faces: [(f32, f32, f32, f32); 4] = [
        (
            spec.x + half_width + SHOPFRONT_OFFSET,
            spec.z,
            SHOPFRONT_OFFSET,
            spec.depth * 0.90,
        ),
        (
            spec.x - half_width - SHOPFRONT_OFFSET,
            spec.z,
            SHOPFRONT_OFFSET,
            spec.depth * 0.90,
        ),
        (
            spec.x,
            spec.z + half_depth + SHOPFRONT_OFFSET,
            spec.width * 0.90,
            SHOPFRONT_OFFSET,
        ),
        (
            spec.x,
            spec.z - half_depth - SHOPFRONT_OFFSET,
            spec.width * 0.90,
            SHOPFRONT_OFFSET,
        ),
    ];

    for (x, z, sx, sz) in faces {
        data.instance(
            "Cube",
            Vec3::new(x, SHOPFRONT_Y, z),
            Vec3::new(sx, SHOPFRONT_HEIGHT, sz),
            "ShopfrontLit",
            None,
        );
    }
}

fn describe_balconies(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    let total_floors = (spec.height / FLOOR_HEIGHT).floor() as i32;
    let half_width = spec.width / 2.0;
    let half_depth = spec.depth / 2.0;

    let face_normals: [(f32, f32, f32, f32); 2] = [
        (spec.x + half_width, spec.z, 1.0, 0.0),
        (spec.x, spec.z + half_depth, 0.0, 1.0),
    ];

    for (base_x, base_z, normal_x, normal_z) in face_normals {
        let balconies_per_face = rng.random_range(2u32..5).min(total_floors as u32 / 2);
        for balcony_index in 0..balconies_per_face {
            let floor = (balcony_index as i32 * 2 + 2).min(total_floors - 1);
            let balcony_y = floor as f32 * FLOOR_HEIGHT;

            let balcony_x = base_x + normal_x * BALCONY_DEPTH / 2.0;
            let balcony_z = base_z + normal_z * BALCONY_DEPTH / 2.0;

            let (sx, sz) = if normal_x.abs() > 0.5 {
                (BALCONY_DEPTH, BALCONY_WIDTH)
            } else {
                (BALCONY_WIDTH, BALCONY_DEPTH)
            };

            data.instance(
                "Cube",
                Vec3::new(balcony_x, balcony_y, balcony_z),
                Vec3::new(sx, BALCONY_HEIGHT, sz),
                "ConcreteMedium",
                None,
            );

            let railing_x = base_x + normal_x * BALCONY_DEPTH;
            let railing_z = base_z + normal_z * BALCONY_DEPTH;

            let (rsx, rsz) = if normal_x.abs() > 0.5 {
                (RAILING_THICKNESS, BALCONY_WIDTH)
            } else {
                (BALCONY_WIDTH, RAILING_THICKNESS)
            };

            data.instance(
                "Cube",
                Vec3::new(railing_x, balcony_y + RAILING_HEIGHT / 2.0, railing_z),
                Vec3::new(rsx, RAILING_HEIGHT, rsz),
                "DockMetal",
                None,
            );
        }
    }
}

fn describe_billboard(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    let face = rng.random_range(0u32..4);
    let sign_y = spec.height * 0.6;
    let billboard_width = 4.0f32.min(spec.width * 0.7);
    let billboard_height = 3.0f32.min(spec.height * 0.15);
    let billboard_thickness = 0.1;
    let offset = 0.15;

    let half_width = spec.width / 2.0;
    let half_depth = spec.depth / 2.0;

    let (x, z, sx, sz) = match face {
        0 => (
            spec.x + half_width + offset,
            spec.z,
            billboard_thickness,
            billboard_width.min(spec.depth * 0.8),
        ),
        1 => (
            spec.x - half_width - offset,
            spec.z,
            billboard_thickness,
            billboard_width.min(spec.depth * 0.8),
        ),
        2 => (
            spec.x,
            spec.z + half_depth + offset,
            billboard_width.min(spec.width * 0.8),
            billboard_thickness,
        ),
        _ => (
            spec.x,
            spec.z - half_depth - offset,
            billboard_width.min(spec.width * 0.8),
            billboard_thickness,
        ),
    };

    let material = BILLBOARD_MATERIALS[rng.random_range(0..BILLBOARD_MATERIALS.len())];

    data.mesh(
        "Cube",
        Vec3::new(x, sign_y, z),
        Vec3::new(sx, billboard_height, sz),
        material,
    );
}

fn describe_rooftop_detail(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    let detail_x = spec.x + rng.random_range(-spec.width * 0.25..spec.width * 0.25);
    let detail_z = spec.z + rng.random_range(-spec.depth * 0.25..spec.depth * 0.25);

    let detail_type = rng.random_range(0u32..3);
    match detail_type {
        0 => {
            let ac_size = rng.random_range(0.5..1.2);
            data.mesh(
                "Cube",
                Vec3::new(detail_x, spec.height + ac_size / 2.0, detail_z),
                Vec3::new(ac_size * 1.5, ac_size, ac_size),
                "RooftopMetal",
            );
        }
        1 => {
            let tank_height = rng.random_range(1.5..3.0);
            let tank_radius = rng.random_range(0.4..0.8);
            data.mesh(
                "Cylinder",
                Vec3::new(detail_x, spec.height + tank_height / 2.0, detail_z),
                Vec3::new(tank_radius * 2.0, tank_height, tank_radius * 2.0),
                "RooftopGrey",
            );
        }
        _ => {
            let antenna_height = rng.random_range(2.0..5.0);
            data.mesh(
                "Cylinder",
                Vec3::new(detail_x, spec.height + antenna_height / 2.0, detail_z),
                Vec3::new(0.1, antenna_height, 0.1),
                "Antenna",
            );
        }
    }
}

fn describe_screen_billboard(data: &mut ChunkData, spec: &BuildingSpec, rng: &mut impl Rng) {
    let face = rng.random_range(0u32..4);
    let sign_y = spec.height * 0.65;
    let screen_half_width = 2.0_f32.min(spec.width * 0.3);
    let screen_half_height = 1.2_f32.min(spec.height * 0.06);
    let offset = 0.15;

    let half_width = spec.width / 2.0;
    let half_depth = spec.depth / 2.0;

    let vertical =
        nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::new(1.0, 0.0, 0.0));

    let (x, z, rotation) = match face {
        0 => (
            spec.x + half_width + offset,
            spec.z,
            nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::y()) * vertical,
        ),
        1 => (
            spec.x - half_width - offset,
            spec.z,
            nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::y()) * vertical,
        ),
        2 => (spec.x, spec.z + half_depth + offset, vertical),
        _ => (
            spec.x,
            spec.z - half_depth - offset,
            nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &Vec3::y()) * vertical,
        ),
    };

    let material_index = rng.random_range(0..crate::billboard::SCREEN_MATERIALS.len());
    let material = crate::billboard::SCREEN_MATERIALS[material_index];

    data.mesh_rotated(
        "Plane",
        Vec3::new(x, sign_y, z),
        Vec3::new(screen_half_width, 1.0, screen_half_height),
        material,
        rotation,
    );
}
