use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};

use crate::building::{BuildingSpec, BuildingType};

pub const CHUNK_SIZE: f32 = 64.0;
const BLOCK_SIZE: f32 = 16.0;
const ROAD_WIDTH: f32 = 4.0;
const SIDEWALK_WIDTH: f32 = 1.5;
const BUILDING_MARGIN: f32 = 1.0;

pub struct StreetlightPosition {
    pub x: f32,
    pub z: f32,
}

pub struct CityChunkLayout {
    pub buildings: Vec<BuildingSpec>,
    pub road_segments: Vec<RoadSegment>,
    pub streetlight_positions: Vec<StreetlightPosition>,
    pub intersection_positions: Vec<IntersectionPosition>,
}

pub struct RoadSegment {
    pub x: f32,
    pub z: f32,
    pub width: f32,
    pub depth: f32,
    pub is_sidewalk: bool,
}

pub struct IntersectionPosition {
    pub x: f32,
    pub z: f32,
}

enum BlockSubdivision {
    Single,
    SplitTwo,
    Grid2x2,
}

fn pick_subdivision(height_influence: f32, rng: &mut impl Rng) -> BlockSubdivision {
    let roll: f32 = rng.random_range(0.0..1.0);
    if height_influence > 0.7 {
        BlockSubdivision::Single
    } else if height_influence > 0.4 {
        if roll < 0.5 {
            BlockSubdivision::Single
        } else {
            BlockSubdivision::SplitTwo
        }
    } else if roll < 0.3 {
        BlockSubdivision::Single
    } else if roll < 0.7 {
        BlockSubdivision::SplitTwo
    } else {
        BlockSubdivision::Grid2x2
    }
}

fn generate_block_buildings(
    buildable_x_start: f32,
    buildable_z_start: f32,
    buildable_width: f32,
    buildable_depth: f32,
    height_influence: f32,
    rng: &mut impl Rng,
) -> Vec<BuildingSpec> {
    let subdivision = pick_subdivision(height_influence, rng);
    let gap = 0.5;

    match subdivision {
        BlockSubdivision::Single => generate_single_building(
            buildable_x_start,
            buildable_z_start,
            buildable_width,
            buildable_depth,
            height_influence,
            rng,
        ),
        BlockSubdivision::SplitTwo => {
            let mut result = Vec::new();
            if buildable_width >= buildable_depth {
                let parcel_width = (buildable_width - gap) / 2.0;
                for parcel_index in 0..2 {
                    let px = buildable_x_start + parcel_index as f32 * (parcel_width + gap);
                    result.extend(generate_single_building(
                        px,
                        buildable_z_start,
                        parcel_width,
                        buildable_depth,
                        height_influence,
                        rng,
                    ));
                }
            } else {
                let parcel_depth = (buildable_depth - gap) / 2.0;
                for parcel_index in 0..2 {
                    let pz = buildable_z_start + parcel_index as f32 * (parcel_depth + gap);
                    result.extend(generate_single_building(
                        buildable_x_start,
                        pz,
                        buildable_width,
                        parcel_depth,
                        height_influence,
                        rng,
                    ));
                }
            }
            result
        }
        BlockSubdivision::Grid2x2 => {
            let parcel_width = (buildable_width - gap) / 2.0;
            let parcel_depth = (buildable_depth - gap) / 2.0;
            let mut result = Vec::new();
            for grid_x in 0..2 {
                for grid_z in 0..2 {
                    let px = buildable_x_start + grid_x as f32 * (parcel_width + gap);
                    let pz = buildable_z_start + grid_z as f32 * (parcel_depth + gap);
                    result.extend(generate_single_building_small(
                        px,
                        pz,
                        parcel_width,
                        parcel_depth,
                        rng,
                    ));
                }
            }
            result
        }
    }
}

fn generate_single_building(
    area_x: f32,
    area_z: f32,
    area_width: f32,
    area_depth: f32,
    height_influence: f32,
    rng: &mut impl Rng,
) -> Vec<BuildingSpec> {
    if area_width < 3.0 || area_depth < 3.0 {
        return Vec::new();
    }

    let (building_type, base_height, width_range, depth_range) =
        pick_building_type(height_influence, rng);

    let bw = rng
        .random_range(width_range.0..width_range.1)
        .min(area_width - BUILDING_MARGIN * 2.0);
    let bd = rng
        .random_range(depth_range.0..depth_range.1)
        .min(area_depth - BUILDING_MARGIN * 2.0);

    let height_variation = rng.random_range(0.8..1.2);
    let height = base_height * height_variation;

    let center_x = area_x + area_width / 2.0;
    let center_z = area_z + area_depth / 2.0;

    let (body_material, roof_material) = pick_materials(building_type, rng);

    vec![BuildingSpec {
        building_type,
        x: center_x,
        z: center_z,
        width: bw,
        depth: bd,
        height,
        body_material,
        roof_material,
    }]
}

fn generate_single_building_small(
    area_x: f32,
    area_z: f32,
    area_width: f32,
    area_depth: f32,
    rng: &mut impl Rng,
) -> Vec<BuildingSpec> {
    if area_width < 2.5 || area_depth < 2.5 {
        return Vec::new();
    }

    let roll: f32 = rng.random_range(0.0..1.0);
    let (building_type, base_height, width_range, depth_range): (
        BuildingType,
        f32,
        (f32, f32),
        (f32, f32),
    ) = if roll < 0.6 {
        (
            BuildingType::House,
            rng.random_range(3.0..5.0),
            (3.0, 5.0),
            (3.0, 5.0),
        )
    } else {
        (
            BuildingType::LowRiseOffice,
            rng.random_range(4.0..7.0),
            (3.0, 6.0),
            (3.0, 6.0),
        )
    };

    let bw = rng
        .random_range(width_range.0..width_range.1)
        .min(area_width - BUILDING_MARGIN * 2.0);
    let bd = rng
        .random_range(depth_range.0..depth_range.1)
        .min(area_depth - BUILDING_MARGIN * 2.0);

    let height_variation = rng.random_range(0.8..1.2);
    let height = base_height * height_variation;

    let center_x = area_x + area_width / 2.0;
    let center_z = area_z + area_depth / 2.0;

    let (body_material, roof_material) = pick_materials(building_type, rng);

    vec![BuildingSpec {
        building_type,
        x: center_x,
        z: center_z,
        width: bw,
        depth: bd,
        height,
        body_material,
        roof_material,
    }]
}

pub fn generate_chunk_layout(chunk_x: i32, chunk_z: i32, noise: &Perlin) -> CityChunkLayout {
    let seed = (chunk_x as u64).wrapping_mul(73856093) ^ (chunk_z as u64).wrapping_mul(19349663);
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let chunk_base_x = chunk_x as f32 * CHUNK_SIZE;
    let chunk_base_z = chunk_z as f32 * CHUNK_SIZE;

    let height_influence = sample_district_noise(noise, chunk_x, chunk_z);

    let mut buildings = Vec::new();
    let road_segments = generate_road_grid(chunk_base_x, chunk_base_z);

    let blocks_per_side = (CHUNK_SIZE / BLOCK_SIZE) as i32;

    for block_ix in 0..blocks_per_side {
        for block_iz in 0..blocks_per_side {
            let block_base_x = chunk_base_x + block_ix as f32 * BLOCK_SIZE;
            let block_base_z = chunk_base_z + block_iz as f32 * BLOCK_SIZE;

            let buildable_x_start = block_base_x + ROAD_WIDTH / 2.0 + SIDEWALK_WIDTH;
            let buildable_z_start = block_base_z + ROAD_WIDTH / 2.0 + SIDEWALK_WIDTH;
            let buildable_width = BLOCK_SIZE - ROAD_WIDTH - SIDEWALK_WIDTH * 2.0;
            let buildable_depth = BLOCK_SIZE - ROAD_WIDTH - SIDEWALK_WIDTH * 2.0;

            if buildable_width < 3.0 || buildable_depth < 3.0 {
                continue;
            }

            buildings.extend(generate_block_buildings(
                buildable_x_start,
                buildable_z_start,
                buildable_width,
                buildable_depth,
                height_influence,
                &mut rng,
            ));
        }
    }

    let streetlight_positions =
        generate_streetlight_positions(chunk_base_x, chunk_base_z, blocks_per_side);

    let mut intersection_positions = Vec::new();
    for block_ix in 1..blocks_per_side {
        for block_iz in 1..blocks_per_side {
            intersection_positions.push(IntersectionPosition {
                x: chunk_base_x + block_ix as f32 * BLOCK_SIZE,
                z: chunk_base_z + block_iz as f32 * BLOCK_SIZE,
            });
        }
    }

    CityChunkLayout {
        buildings,
        road_segments,
        streetlight_positions,
        intersection_positions,
    }
}

fn generate_streetlight_positions(
    chunk_base_x: f32,
    chunk_base_z: f32,
    blocks_per_side: i32,
) -> Vec<StreetlightPosition> {
    let mut positions = Vec::new();
    let sidewalk_offset = ROAD_WIDTH / 2.0 + 0.8;

    for block_ix in 0..blocks_per_side {
        for block_iz in 0..blocks_per_side {
            let intersection_x = chunk_base_x + block_ix as f32 * BLOCK_SIZE;
            let intersection_z = chunk_base_z + block_iz as f32 * BLOCK_SIZE;

            let corner = ((block_ix * 7 + block_iz * 13) as u32) % 4;
            let (dx, dz) = match corner {
                0 => (sidewalk_offset, sidewalk_offset),
                1 => (sidewalk_offset, -sidewalk_offset),
                2 => (-sidewalk_offset, sidewalk_offset),
                _ => (-sidewalk_offset, -sidewalk_offset),
            };

            positions.push(StreetlightPosition {
                x: intersection_x + dx,
                z: intersection_z + dz,
            });
        }
    }

    positions
}

fn generate_road_grid(chunk_base_x: f32, chunk_base_z: f32) -> Vec<RoadSegment> {
    let mut segments = Vec::new();
    let blocks_per_side = (CHUNK_SIZE / BLOCK_SIZE) as i32;

    for block_index in 0..blocks_per_side {
        let offset = block_index as f32 * BLOCK_SIZE;

        segments.push(RoadSegment {
            x: chunk_base_x + offset,
            z: chunk_base_z + CHUNK_SIZE / 2.0,
            width: ROAD_WIDTH,
            depth: CHUNK_SIZE,
            is_sidewalk: false,
        });

        segments.push(RoadSegment {
            x: chunk_base_x + CHUNK_SIZE / 2.0,
            z: chunk_base_z + offset,
            width: CHUNK_SIZE,
            depth: ROAD_WIDTH,
            is_sidewalk: false,
        });
    }

    for block_ix in 0..blocks_per_side {
        for block_iz in 0..blocks_per_side {
            let block_base_x = chunk_base_x + block_ix as f32 * BLOCK_SIZE;
            let block_base_z = chunk_base_z + block_iz as f32 * BLOCK_SIZE;

            let buildable_x_start = block_base_x + ROAD_WIDTH / 2.0 + SIDEWALK_WIDTH;
            let buildable_z_start = block_base_z + ROAD_WIDTH / 2.0 + SIDEWALK_WIDTH;
            let buildable_width = BLOCK_SIZE - ROAD_WIDTH - SIDEWALK_WIDTH * 2.0;
            let buildable_depth = BLOCK_SIZE - ROAD_WIDTH - SIDEWALK_WIDTH * 2.0;

            if buildable_width < 1.0 || buildable_depth < 1.0 {
                continue;
            }

            segments.push(RoadSegment {
                x: buildable_x_start + buildable_width / 2.0,
                z: block_base_z + ROAD_WIDTH / 2.0 + SIDEWALK_WIDTH / 2.0,
                width: buildable_width,
                depth: SIDEWALK_WIDTH,
                is_sidewalk: true,
            });
            segments.push(RoadSegment {
                x: block_base_x + ROAD_WIDTH / 2.0 + SIDEWALK_WIDTH / 2.0,
                z: buildable_z_start + buildable_depth / 2.0,
                width: SIDEWALK_WIDTH,
                depth: buildable_depth,
                is_sidewalk: true,
            });
        }
    }

    segments
}

fn sample_district_noise(noise: &Perlin, chunk_x: i32, chunk_z: i32) -> f32 {
    let scale = 0.05;
    let value = noise.get([chunk_x as f64 * scale, chunk_z as f64 * scale]);
    ((value + 1.0) / 2.0).clamp(0.0, 1.0) as f32
}

fn pick_building_type(
    height_influence: f32,
    rng: &mut impl Rng,
) -> (BuildingType, f32, (f32, f32), (f32, f32)) {
    let roll: f32 = rng.random_range(0.0..1.0);

    if height_influence > 0.7 {
        if roll < 0.4 {
            (
                BuildingType::Skyscraper,
                rng.random_range(25.0..60.0),
                (5.0, 10.0),
                (5.0, 10.0),
            )
        } else if roll < 0.75 {
            (
                BuildingType::OfficeTower,
                rng.random_range(15.0..35.0),
                (6.0, 12.0),
                (6.0, 12.0),
            )
        } else {
            (
                BuildingType::ApartmentBlock,
                rng.random_range(10.0..20.0),
                (6.0, 14.0),
                (5.0, 8.0),
            )
        }
    } else if height_influence > 0.4 {
        if roll < 0.3 {
            (
                BuildingType::OfficeTower,
                rng.random_range(10.0..25.0),
                (6.0, 12.0),
                (6.0, 12.0),
            )
        } else if roll < 0.55 {
            (
                BuildingType::ApartmentBlock,
                rng.random_range(8.0..18.0),
                (6.0, 14.0),
                (5.0, 8.0),
            )
        } else if roll < 0.8 {
            (
                BuildingType::LowRiseOffice,
                rng.random_range(5.0..12.0),
                (5.0, 10.0),
                (5.0, 10.0),
            )
        } else {
            (BuildingType::Park, 0.1, (8.0, 12.0), (8.0, 12.0))
        }
    } else if roll < 0.35 {
        (
            BuildingType::House,
            rng.random_range(3.0..5.0),
            (4.0, 7.0),
            (4.0, 7.0),
        )
    } else if roll < 0.55 {
        (
            BuildingType::LowRiseOffice,
            rng.random_range(4.0..8.0),
            (5.0, 10.0),
            (5.0, 10.0),
        )
    } else if roll < 0.75 {
        (
            BuildingType::Warehouse,
            rng.random_range(4.0..7.0),
            (8.0, 14.0),
            (8.0, 14.0),
        )
    } else if roll < 0.9 {
        (
            BuildingType::ApartmentBlock,
            rng.random_range(6.0..12.0),
            (6.0, 12.0),
            (5.0, 8.0),
        )
    } else {
        (BuildingType::Park, 0.1, (8.0, 12.0), (8.0, 12.0))
    }
}

fn pick_materials(building_type: BuildingType, rng: &mut impl Rng) -> (&'static str, &'static str) {
    match building_type {
        BuildingType::Skyscraper => {
            let bodies = ["GlassBlue", "GlassTeal", "GlassDark"];
            (bodies[rng.random_range(0..bodies.len())], "RooftopMetal")
        }
        BuildingType::OfficeTower => {
            let bodies = ["ConcreteMedium", "ConcreteLight", "ModernWhite"];
            (bodies[rng.random_range(0..bodies.len())], "RooftopGrey")
        }
        BuildingType::LowRiseOffice => {
            let bodies = ["ConcreteLight", "BrickTan", "ConcreteMedium"];
            (bodies[rng.random_range(0..bodies.len())], "RooftopGrey")
        }
        BuildingType::ApartmentBlock => {
            let bodies = ["BrickRed", "BrickBrown", "BrickTan"];
            (bodies[rng.random_range(0..bodies.len())], "RooftopGrey")
        }
        BuildingType::House => {
            let bodies = ["BrickTan", "ModernCream", "ConcreteLight"];
            (bodies[rng.random_range(0..bodies.len())], "RooftopRed")
        }
        BuildingType::Warehouse => {
            let bodies = ["ConcreteDark", "RooftopMetal", "ConcreteMedium"];
            (bodies[rng.random_range(0..bodies.len())], "RooftopMetal")
        }
        BuildingType::Park => ("ParkGreen", "ParkGreen"),
    }
}
