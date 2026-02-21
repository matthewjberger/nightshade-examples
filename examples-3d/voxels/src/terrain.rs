use std::collections::HashMap;

use nightshade::prelude::*;
use noise::{NoiseFn, Perlin};

pub const CHUNK_SIZE: usize = 32;
const WATER_LEVEL: i32 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VoxelType {
    Water = 1,
    Sand = 2,
    Grass = 3,
    Dirt = 4,
    Stone = 5,
    Snow = 6,
}

impl VoxelType {
    pub const ALL_SOLID: &[VoxelType] = &[
        VoxelType::Water,
        VoxelType::Sand,
        VoxelType::Grass,
        VoxelType::Dirt,
        VoxelType::Stone,
        VoxelType::Snow,
    ];

    pub fn color(self) -> [f32; 4] {
        match self {
            VoxelType::Water => [0.1, 0.5, 0.85, 0.8],
            VoxelType::Sand => [0.95, 0.85, 0.55, 1.0],
            VoxelType::Grass => [0.2, 0.8, 0.2, 1.0],
            VoxelType::Dirt => [0.6, 0.4, 0.2, 1.0],
            VoxelType::Stone => [0.5, 0.5, 0.55, 1.0],
            VoxelType::Snow => [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn from_height(world_y: i32) -> Self {
        match world_y {
            y if y < 0 => VoxelType::Water,
            y if y < 8 => VoxelType::Sand,
            y if y < 20 => VoxelType::Grass,
            y if y < 30 => VoxelType::Dirt,
            y if y < 45 => VoxelType::Stone,
            _ => VoxelType::Snow,
        }
    }

    fn band_bottom(self) -> i32 {
        match self {
            VoxelType::Water => i32::MIN,
            VoxelType::Sand => 0,
            VoxelType::Grass => 8,
            VoxelType::Dirt => 20,
            VoxelType::Stone => 30,
            VoxelType::Snow => 45,
        }
    }

    fn band_top(self) -> i32 {
        match self {
            VoxelType::Water => -1,
            VoxelType::Sand => 7,
            VoxelType::Grass => 19,
            VoxelType::Dirt => 29,
            VoxelType::Stone => 44,
            VoxelType::Snow => i32::MAX,
        }
    }

    pub fn material_name(self) -> &'static str {
        match self {
            VoxelType::Water => "Voxel_Water",
            VoxelType::Sand => "Voxel_Sand",
            VoxelType::Grass => "Voxel_Grass",
            VoxelType::Dirt => "Voxel_Dirt",
            VoxelType::Stone => "Voxel_Stone",
            VoxelType::Snow => "Voxel_Snow",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SurfaceCell {
    pub voxel_type: VoxelType,
    pub height: i32,
}

pub struct ChunkTerrainData {
    pub surface: Vec<SurfaceCell>,
    pub cliff_bottom: Vec<i32>,
    pub cliff_top: Vec<i32>,
}

pub struct ChunkInstanceData {
    pub instances_by_type: HashMap<VoxelType, Vec<InstanceTransform>>,
}

impl ChunkInstanceData {
    pub fn total_instances(&self) -> usize {
        self.instances_by_type.values().map(|v| v.len()).sum()
    }
}

pub fn generate_terrain_data(
    noise: &Perlin,
    chunk_x: i32,
    chunk_z: i32,
    noise_scale: f64,
    noise_octaves: usize,
) -> ChunkTerrainData {
    let padded_size = CHUNK_SIZE + 2;
    let mut height_map = vec![0i32; padded_size * padded_size];

    let world_base_x = chunk_x as f64 * CHUNK_SIZE as f64;
    let world_base_z = chunk_z as f64 * CHUNK_SIZE as f64;

    for padded_z in 0..padded_size {
        for padded_x in 0..padded_size {
            let world_x = world_base_x + (padded_x as f64 - 1.0);
            let world_z = world_base_z + (padded_z as f64 - 1.0);
            let height = sample_terrain_height(noise, world_x, world_z, noise_scale, noise_octaves);
            height_map[padded_x + padded_z * padded_size] = height;
        }
    }

    let mut surface = vec![
        SurfaceCell {
            voxel_type: VoxelType::Water,
            height: WATER_LEVEL - 1,
        };
        CHUNK_SIZE * CHUNK_SIZE
    ];
    let mut cliff_bottom = vec![0i32; CHUNK_SIZE * CHUNK_SIZE];
    let mut cliff_top = vec![0i32; CHUNK_SIZE * CHUNK_SIZE];

    for local_z in 0..CHUNK_SIZE {
        for local_x in 0..CHUNK_SIZE {
            let padded_x = local_x + 1;
            let padded_z = local_z + 1;
            let index = local_x + local_z * CHUNK_SIZE;

            let height = height_map[padded_x + padded_z * padded_size];

            if height < WATER_LEVEL {
                surface[index] = SurfaceCell {
                    voxel_type: VoxelType::Water,
                    height: WATER_LEVEL - 1,
                };
            } else {
                surface[index] = SurfaceCell {
                    voxel_type: VoxelType::from_height(height),
                    height,
                };

                let height_left = height_map[(padded_x - 1) + padded_z * padded_size];
                let height_right = height_map[(padded_x + 1) + padded_z * padded_size];
                let height_back = height_map[padded_x + (padded_z - 1) * padded_size];
                let height_front = height_map[padded_x + (padded_z + 1) * padded_size];

                let min_neighbor = height_left
                    .min(height_right)
                    .min(height_back)
                    .min(height_front);

                if min_neighbor < height {
                    cliff_bottom[index] = min_neighbor.max(WATER_LEVEL) + 1;
                    cliff_top[index] = height;
                }
            }
        }
    }

    ChunkTerrainData {
        surface,
        cliff_bottom,
        cliff_top,
    }
}

pub fn build_detailed_instances(
    data: &ChunkTerrainData,
    chunk_x: i32,
    chunk_z: i32,
) -> ChunkInstanceData {
    let mut instances_by_type: HashMap<VoxelType, Vec<InstanceTransform>> = HashMap::new();

    for local_z in 0..CHUNK_SIZE {
        for local_x in 0..CHUNK_SIZE {
            let index = local_x + local_z * CHUNK_SIZE;
            let cell = data.surface[index];
            let world_x = (chunk_x * CHUNK_SIZE as i32 + local_x as i32) as f32;
            let world_z = (chunk_z * CHUNK_SIZE as i32 + local_z as i32) as f32;

            instances_by_type.entry(cell.voxel_type).or_default().push(
                InstanceTransform::from_translation_scale(
                    Vec3::new(world_x + 0.5, cell.height as f32 + 0.5, world_z + 0.5),
                    Vec3::new(1.0, 1.0, 1.0),
                ),
            );

            let bottom = data.cliff_bottom[index];
            let top = data.cliff_top[index];
            if bottom < top {
                push_cliff_columns(&mut instances_by_type, world_x, world_z, bottom, top);
            }
        }
    }

    ChunkInstanceData { instances_by_type }
}

pub fn build_merged_instances(
    data: &ChunkTerrainData,
    chunk_x: i32,
    chunk_z: i32,
) -> ChunkInstanceData {
    let mut instances_by_type: HashMap<VoxelType, Vec<InstanceTransform>> = HashMap::new();

    greedy_mesh_surface(&data.surface, chunk_x, chunk_z, &mut instances_by_type);
    greedy_mesh_cliffs(
        &data.cliff_bottom,
        &data.cliff_top,
        chunk_x,
        chunk_z,
        &mut instances_by_type,
    );

    ChunkInstanceData { instances_by_type }
}

fn push_cliff_columns(
    instances_by_type: &mut HashMap<VoxelType, Vec<InstanceTransform>>,
    world_x: f32,
    world_z: f32,
    bottom: i32,
    top: i32,
) {
    let mut y = bottom;
    while y <= top {
        let voxel_type = VoxelType::from_height(y);
        let run_start = y;
        while y <= top && VoxelType::from_height(y) == voxel_type {
            y += 1;
        }
        let run_height = (y - run_start) as f32;
        instances_by_type.entry(voxel_type).or_default().push(
            InstanceTransform::from_translation_scale(
                Vec3::new(
                    world_x + 0.5,
                    run_start as f32 + run_height * 0.5,
                    world_z + 0.5,
                ),
                Vec3::new(1.0, run_height, 1.0),
            ),
        );
    }
}

fn greedy_mesh_surface(
    surface: &[SurfaceCell],
    chunk_x: i32,
    chunk_z: i32,
    instances_by_type: &mut HashMap<VoxelType, Vec<InstanceTransform>>,
) {
    let mut visited = [false; CHUNK_SIZE * CHUNK_SIZE];

    for start_z in 0..CHUNK_SIZE {
        for start_x in 0..CHUNK_SIZE {
            let start_index = start_x + start_z * CHUNK_SIZE;
            if visited[start_index] {
                continue;
            }

            let cell = surface[start_index];
            visited[start_index] = true;

            let mut width = 1usize;
            while start_x + width < CHUNK_SIZE {
                let neighbor_index = (start_x + width) + start_z * CHUNK_SIZE;
                if visited[neighbor_index] || surface[neighbor_index] != cell {
                    break;
                }
                visited[neighbor_index] = true;
                width += 1;
            }

            let mut depth = 1usize;
            'extend_depth: while start_z + depth < CHUNK_SIZE {
                for dx in 0..width {
                    let neighbor_index = (start_x + dx) + (start_z + depth) * CHUNK_SIZE;
                    if visited[neighbor_index] || surface[neighbor_index] != cell {
                        break 'extend_depth;
                    }
                }
                for dx in 0..width {
                    visited[(start_x + dx) + (start_z + depth) * CHUNK_SIZE] = true;
                }
                depth += 1;
            }

            let world_x = (chunk_x * CHUNK_SIZE as i32 + start_x as i32) as f32;
            let world_z = (chunk_z * CHUNK_SIZE as i32 + start_z as i32) as f32;

            instances_by_type.entry(cell.voxel_type).or_default().push(
                InstanceTransform::from_translation_scale(
                    Vec3::new(
                        world_x + width as f32 * 0.5,
                        cell.height as f32 + 0.5,
                        world_z + depth as f32 * 0.5,
                    ),
                    Vec3::new(width as f32, 1.0, depth as f32),
                ),
            );
        }
    }
}

fn greedy_mesh_cliffs(
    cliff_bottom: &[i32],
    cliff_top: &[i32],
    chunk_x: i32,
    chunk_z: i32,
    instances_by_type: &mut HashMap<VoxelType, Vec<InstanceTransform>>,
) {
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    for index in 0..CHUNK_SIZE * CHUNK_SIZE {
        if cliff_top[index] > cliff_bottom[index] {
            min_y = min_y.min(cliff_bottom[index]);
            max_y = max_y.max(cliff_top[index]);
        }
    }

    if min_y > max_y {
        return;
    }

    for voxel_type in VoxelType::ALL_SOLID {
        let band_lo = voxel_type.band_bottom().max(min_y);
        let band_hi = voxel_type.band_top().min(max_y);

        if band_lo > band_hi {
            continue;
        }

        let mut visited = [false; CHUNK_SIZE * CHUNK_SIZE];
        let mut cell_extent = [(0i32, 0i32); CHUNK_SIZE * CHUNK_SIZE];
        let mut has_any = false;

        for index in 0..CHUNK_SIZE * CHUNK_SIZE {
            let bottom = cliff_bottom[index];
            let top = cliff_top[index];
            if bottom >= top {
                continue;
            }
            let clamped_bottom = bottom.max(band_lo);
            let clamped_top = top.min(band_hi);
            if clamped_bottom > clamped_top {
                continue;
            }
            cell_extent[index] = (clamped_bottom, clamped_top);
            has_any = true;
        }

        if !has_any {
            continue;
        }

        for start_z in 0..CHUNK_SIZE {
            for start_x in 0..CHUNK_SIZE {
                let start_index = start_x + start_z * CHUNK_SIZE;
                if visited[start_index] || cell_extent[start_index] == (0, 0) {
                    continue;
                }

                let extent = cell_extent[start_index];
                visited[start_index] = true;

                let mut width = 1usize;
                while start_x + width < CHUNK_SIZE {
                    let neighbor_index = (start_x + width) + start_z * CHUNK_SIZE;
                    if visited[neighbor_index] || cell_extent[neighbor_index] != extent {
                        break;
                    }
                    visited[neighbor_index] = true;
                    width += 1;
                }

                let mut depth = 1usize;
                'extend_depth: while start_z + depth < CHUNK_SIZE {
                    for dx in 0..width {
                        let neighbor_index = (start_x + dx) + (start_z + depth) * CHUNK_SIZE;
                        if visited[neighbor_index] || cell_extent[neighbor_index] != extent {
                            break 'extend_depth;
                        }
                    }
                    for dx in 0..width {
                        visited[(start_x + dx) + (start_z + depth) * CHUNK_SIZE] = true;
                    }
                    depth += 1;
                }

                let run_height = (extent.1 - extent.0 + 1) as f32;
                let world_x = (chunk_x * CHUNK_SIZE as i32 + start_x as i32) as f32;
                let world_z = (chunk_z * CHUNK_SIZE as i32 + start_z as i32) as f32;

                instances_by_type.entry(*voxel_type).or_default().push(
                    InstanceTransform::from_translation_scale(
                        Vec3::new(
                            world_x + width as f32 * 0.5,
                            extent.0 as f32 + run_height * 0.5,
                            world_z + depth as f32 * 0.5,
                        ),
                        Vec3::new(width as f32, run_height, depth as f32),
                    ),
                );
            }
        }
    }
}

fn sample_terrain_height(
    noise: &Perlin,
    world_x: f64,
    world_z: f64,
    scale: f64,
    octaves: usize,
) -> i32 {
    let mut height = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        let sample_x = world_x * scale * frequency;
        let sample_z = world_z * scale * frequency;
        let noise_value = noise.get([sample_x, sample_z]);
        height += noise_value * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    height /= max_value;

    let base_height = 12.0;
    let mountain_height = 35.0;

    (base_height + height * mountain_height) as i32
}
