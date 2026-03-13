use std::collections::HashMap;

use nightshade::prelude::*;
use noise::Perlin;

use crate::terrain::{self, CHUNK_SIZE, ChunkTerrainData, VoxelType};

const LOAD_RADIUS: i32 = 24;
const UNLOAD_RADIUS: i32 = 26;
const DETAIL_RADIUS: i32 = 5;
const MAX_PREGEN_PER_FRAME: usize = 8;
const MAX_LOAD_PER_FRAME: usize = 6;
const MAX_LOD_SWAPS_PER_FRAME: usize = 4;
const MAX_TOTAL_INSTANCES: usize = 500_000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LodLevel {
    Detailed,
    Merged,
}

pub struct ChunkStreamer {
    noise: Perlin,
    noise_scale: f64,
    noise_octaves: usize,
    terrain_cache: HashMap<(i32, i32), ChunkTerrainData>,
    loaded_chunks: HashMap<(i32, i32), LodLevel>,
    material_entities: HashMap<VoxelType, Entity>,
    last_camera_chunk: (i32, i32),
    cached_instance_count: usize,
    instances_dirty: bool,
    pending_work: bool,
    scratch_to_unload: Vec<(i32, i32)>,
    scratch_wanted: Vec<(i32, i32)>,
    scratch_stale_terrain: Vec<(i32, i32)>,
    scratch_lod_swaps: Vec<(i32, i32)>,
}

impl ChunkStreamer {
    pub fn new(seed: u32, noise_scale: f64, noise_octaves: usize) -> Self {
        Self {
            noise: Perlin::new(seed),
            noise_scale,
            noise_octaves,
            terrain_cache: HashMap::new(),
            loaded_chunks: HashMap::new(),
            material_entities: HashMap::new(),
            last_camera_chunk: (i32::MAX, i32::MAX),
            cached_instance_count: 0,
            instances_dirty: false,
            pending_work: false,
            scratch_to_unload: Vec::new(),
            scratch_wanted: Vec::new(),
            scratch_stale_terrain: Vec::new(),
            scratch_lod_swaps: Vec::new(),
        }
    }

    pub fn initialize(&mut self, world: &mut World) {
        for voxel_type in VoxelType::ALL_SOLID {
            let entity = spawn_instanced_mesh_with_material(
                world,
                "Cube",
                Vec::new(),
                voxel_type.material_name(),
            );
            world.core.remove_casts_shadow(entity);
            self.material_entities.insert(*voxel_type, entity);
        }
    }

    pub fn loaded_chunk_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    pub fn instance_count(&self) -> usize {
        self.cached_instance_count
    }

    pub fn update(&mut self, world: &mut World, camera_pos: Vec3, camera_forward: Vec3) {
        let camera_chunk = (
            (camera_pos.x / CHUNK_SIZE as f32).floor() as i32,
            (camera_pos.z / CHUNK_SIZE as f32).floor() as i32,
        );

        let camera_changed = camera_chunk != self.last_camera_chunk;
        if camera_changed {
            self.last_camera_chunk = camera_chunk;
        }

        if !camera_changed && !self.pending_work {
            return;
        }

        self.pending_work = false;

        let camera_forward_xz = Vec3::new(camera_forward.x, 0.0, camera_forward.z);
        let forward_len = nalgebra_glm::length(&camera_forward_xz);
        let camera_forward_normalized = if forward_len > 0.001 {
            camera_forward_xz / forward_len
        } else {
            Vec3::new(0.0, 0.0, -1.0)
        };

        self.scratch_to_unload.clear();
        for &coords in self.loaded_chunks.keys() {
            if chebyshev_distance(coords, camera_chunk) > UNLOAD_RADIUS {
                self.scratch_to_unload.push(coords);
            }
        }
        for index in 0..self.scratch_to_unload.len() {
            self.loaded_chunks.remove(&self.scratch_to_unload[index]);
            self.instances_dirty = true;
        }

        self.scratch_stale_terrain.clear();
        for &coords in self.terrain_cache.keys() {
            if chebyshev_distance(coords, camera_chunk) > UNLOAD_RADIUS {
                self.scratch_stale_terrain.push(coords);
            }
        }
        for index in 0..self.scratch_stale_terrain.len() {
            self.terrain_cache
                .remove(&self.scratch_stale_terrain[index]);
        }

        self.scratch_lod_swaps.clear();
        for (&coords, &current_lod) in &self.loaded_chunks {
            let distance = chebyshev_distance(coords, camera_chunk);
            let desired = if distance <= DETAIL_RADIUS {
                LodLevel::Detailed
            } else {
                LodLevel::Merged
            };
            if current_lod != desired {
                self.scratch_lod_swaps.push(coords);
            }
        }
        self.scratch_lod_swaps
            .sort_by_key(|coords| chebyshev_distance(*coords, camera_chunk));
        let swap_count = self.scratch_lod_swaps.len().min(MAX_LOD_SWAPS_PER_FRAME);
        for index in 0..swap_count {
            let coords = self.scratch_lod_swaps[index];
            let distance = chebyshev_distance(coords, camera_chunk);
            let desired = if distance <= DETAIL_RADIUS {
                LodLevel::Detailed
            } else {
                LodLevel::Merged
            };
            self.loaded_chunks.insert(coords, desired);
            self.instances_dirty = true;
        }
        if swap_count < self.scratch_lod_swaps.len() {
            self.pending_work = true;
        }

        self.scratch_wanted.clear();
        for chunk_x in (camera_chunk.0 - LOAD_RADIUS)..=(camera_chunk.0 + LOAD_RADIUS) {
            for chunk_z in (camera_chunk.1 - LOAD_RADIUS)..=(camera_chunk.1 + LOAD_RADIUS) {
                let coords = (chunk_x, chunk_z);
                if chebyshev_distance(coords, camera_chunk) > LOAD_RADIUS {
                    continue;
                }
                if self.loaded_chunks.contains_key(&coords) {
                    continue;
                }
                self.scratch_wanted.push(coords);
            }
        }

        self.scratch_wanted.sort_by(|a, b| {
            let score_a = load_priority(*a, camera_chunk, &camera_forward_normalized);
            let score_b = load_priority(*b, camera_chunk, &camera_forward_normalized);
            score_a.partial_cmp(&score_b).unwrap()
        });

        let mut pregen_budget = MAX_PREGEN_PER_FRAME;
        let mut load_budget = MAX_LOAD_PER_FRAME;

        for wanted_index in 0..self.scratch_wanted.len() {
            let coords = self.scratch_wanted[wanted_index];

            if self.terrain_cache.contains_key(&coords) {
                if load_budget > 0 {
                    let distance = chebyshev_distance(coords, camera_chunk);
                    let lod = if distance <= DETAIL_RADIUS {
                        LodLevel::Detailed
                    } else {
                        LodLevel::Merged
                    };
                    self.loaded_chunks.insert(coords, lod);
                    self.instances_dirty = true;
                    load_budget -= 1;
                } else {
                    self.pending_work = true;
                }
                continue;
            }

            if pregen_budget > 0 {
                let terrain_data = terrain::generate_terrain_data(
                    &self.noise,
                    coords.0,
                    coords.1,
                    self.noise_scale,
                    self.noise_octaves,
                );
                self.terrain_cache.insert(coords, terrain_data);
                pregen_budget -= 1;
                self.pending_work = true;
            } else {
                self.pending_work = true;
            }
        }

        if self.instances_dirty {
            self.rebuild_instances(world);
            self.instances_dirty = false;
        }
    }

    fn rebuild_instances(&mut self, world: &mut World) {
        let mut all_instances: HashMap<VoxelType, Vec<InstanceTransform>> = HashMap::new();
        for voxel_type in VoxelType::ALL_SOLID {
            all_instances.insert(*voxel_type, Vec::new());
        }

        let mut total_count = 0usize;

        let mut sorted_chunks: Vec<((i32, i32), LodLevel)> = self
            .loaded_chunks
            .iter()
            .map(|(&coords, &lod)| (coords, lod))
            .collect();
        sorted_chunks
            .sort_by_key(|(coords, _)| chebyshev_distance(*coords, self.last_camera_chunk));

        for (coords, lod) in &sorted_chunks {
            let terrain_data = match self.terrain_cache.get(coords) {
                Some(data) => data,
                None => continue,
            };

            let instance_data = match lod {
                LodLevel::Detailed => {
                    terrain::build_detailed_instances(terrain_data, coords.0, coords.1)
                }
                LodLevel::Merged => {
                    terrain::build_merged_instances(terrain_data, coords.0, coords.1)
                }
            };

            let chunk_count = instance_data.total_instances();
            if total_count + chunk_count > MAX_TOTAL_INSTANCES {
                continue;
            }
            total_count += chunk_count;

            for (voxel_type, instances) in instance_data.instances_by_type {
                all_instances
                    .entry(voxel_type)
                    .or_default()
                    .extend(instances);
            }
        }

        for (voxel_type, instances) in &all_instances {
            if let Some(&entity) = self.material_entities.get(voxel_type)
                && let Some(instanced_mesh) = world.core.get_instanced_mesh_mut(entity)
            {
                instanced_mesh.set_instances(instances.clone());
            }
        }

        self.cached_instance_count = total_count;
        world
            .resources
            .mesh_render_state
            .mark_instanced_meshes_changed();
    }
}

fn chebyshev_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

fn load_priority(chunk: (i32, i32), camera_chunk: (i32, i32), camera_forward_xz: &Vec3) -> f32 {
    let distance = chebyshev_distance(chunk, camera_chunk) as f32;
    let dx = (chunk.0 - camera_chunk.0) as f32;
    let dz = (chunk.1 - camera_chunk.1) as f32;
    let len = (dx * dx + dz * dz).sqrt();
    let facing_bonus = if len > 0.001 {
        (dx * camera_forward_xz.x + dz * camera_forward_xz.z) / len
    } else {
        0.0
    };
    distance - facing_bonus * 2.0
}
