use crate::config::TerrainConfig;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn from_world_position(world_x: f32, world_z: f32, chunk_size: f32) -> Self {
        Self {
            x: (world_x / chunk_size).floor() as i32,
            z: (world_z / chunk_size).floor() as i32,
        }
    }

    pub fn world_origin(&self, chunk_size: f32) -> (f32, f32) {
        (self.x as f32 * chunk_size, self.z as f32 * chunk_size)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PatchInput {
    pub world_x: f32,
    pub world_z: f32,
    pub patch_size: f32,
    pub _padding: f32,
}

pub struct ChunkManager {
    active_chunks: HashMap<ChunkCoord, ()>,
    last_camera_chunk: Option<ChunkCoord>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            active_chunks: HashMap::new(),
            last_camera_chunk: None,
        }
    }

    pub fn update(&mut self, camera_x: f32, camera_z: f32, config: &TerrainConfig) -> bool {
        let camera_chunk = ChunkCoord::from_world_position(camera_x, camera_z, config.chunk_size);

        if self.last_camera_chunk == Some(camera_chunk) && !self.active_chunks.is_empty() {
            return false;
        }

        self.last_camera_chunk = Some(camera_chunk);

        let view_dist = config.view_distance as i32;
        let mut new_chunks = HashMap::new();

        for dz in -view_dist..=view_dist {
            for dx in -view_dist..=view_dist {
                let coord = ChunkCoord {
                    x: camera_chunk.x + dx,
                    z: camera_chunk.z + dz,
                };
                new_chunks.insert(coord, ());
            }
        }

        self.active_chunks = new_chunks;
        true
    }

    pub fn generate_patches(&self, config: &TerrainConfig) -> Vec<PatchInput> {
        let patch_size = config.patch_size();
        let patches_per_side = config.patches_per_chunk_side;
        let mut patches =
            Vec::with_capacity(self.active_chunks.len() * config.patches_per_chunk() as usize);

        let mut sorted_chunks: Vec<_> = self.active_chunks.keys().collect();
        sorted_chunks.sort_by(|a, b| a.z.cmp(&b.z).then_with(|| a.x.cmp(&b.x)));

        for coord in sorted_chunks {
            let (chunk_origin_x, chunk_origin_z) = coord.world_origin(config.chunk_size);

            for pz in 0..patches_per_side {
                for px in 0..patches_per_side {
                    let world_x = chunk_origin_x + px as f32 * patch_size;
                    let world_z = chunk_origin_z + pz as f32 * patch_size;

                    patches.push(PatchInput {
                        world_x,
                        world_z,
                        patch_size,
                        _padding: 0.0,
                    });
                }
            }
        }

        patches
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}
