pub struct TerrainConfig {
    pub chunk_size: f32,
    pub view_distance: u32,
    pub patches_per_chunk_side: u32,
    pub max_tessellation: u32,
    pub min_tessellation: u32,
    pub height_scale: f32,
    pub noise_frequency: f32,
    pub noise_octaves: u32,
    pub lod_distances: [f32; 5],
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            chunk_size: 64.0,
            view_distance: 8,
            patches_per_chunk_side: 8,
            max_tessellation: 16,
            min_tessellation: 1,
            height_scale: 50.0,
            noise_frequency: 0.01,
            noise_octaves: 6,
            lod_distances: [50.0, 150.0, 400.0, 800.0, 1600.0],
        }
    }
}

impl TerrainConfig {
    pub fn patch_size(&self) -> f32 {
        self.chunk_size / self.patches_per_chunk_side as f32
    }

    pub fn patches_per_chunk(&self) -> u32 {
        self.patches_per_chunk_side * self.patches_per_chunk_side
    }

    pub fn total_visible_chunks(&self) -> u32 {
        let side = self.view_distance * 2 + 1;
        side * side
    }

    pub fn max_patches(&self) -> u32 {
        self.total_visible_chunks() * self.patches_per_chunk()
    }
}
