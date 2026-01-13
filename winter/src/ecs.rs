use nightshade::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MovementState {
    #[default]
    Idle,
    Walking,
    Running,
}

#[derive(Default)]
pub struct GameWorld {
    pub resources: GameResources,
}

#[derive(Default)]
pub struct GameResources {
    pub terrain_config: TerrainConfig,
    pub fox_entity: Option<freecs::Entity>,
    pub controller_entity: Option<freecs::Entity>,
    pub camera_entity: Option<freecs::Entity>,
    pub campfire_light: Option<freecs::Entity>,
    pub string_lights_entity: Option<freecs::Entity>,
    pub movement_state: MovementState,
    pub fox_rotation: f32,
    pub current_animation: Option<usize>,
    pub was_moving: bool,
    pub animation_indices: AnimationIndices,
    pub footprint_emitter: Option<freecs::Entity>,
    pub head_bone_entity: Option<freecs::Entity>,
    pub santa_hat_entity: Option<freecs::Entity>,
    pub smoothed_fox_position: Vec3,
    pub fox_position_initialized: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TerrainConfig {
    pub width: f32,
    pub depth: f32,
    pub resolution_x: u32,
    pub resolution_z: u32,
    pub height_scale: f32,
    pub noise: NoiseConfig,
    pub uv_scale: [f32; 2],
}

#[derive(Debug, Clone, Default)]
pub struct NoiseConfig {
    pub seed: u32,
    pub frequency: f64,
    pub octaves: usize,
    pub lacunarity: f64,
    pub persistence: f64,
}

#[derive(Debug, Clone, Default)]
pub struct AnimationIndices {
    pub survey: Option<usize>,
    pub walk: Option<usize>,
    pub run: Option<usize>,
}

impl TerrainConfig {
    pub fn to_nightshade_config(&self) -> nightshade::ecs::terrain::TerrainConfig {
        nightshade::ecs::terrain::TerrainConfig {
            width: self.width,
            depth: self.depth,
            resolution_x: self.resolution_x,
            resolution_z: self.resolution_z,
            height_scale: self.height_scale,
            noise: nightshade::ecs::terrain::NoiseConfig {
                seed: self.noise.seed,
                frequency: self.noise.frequency,
                octaves: self.noise.octaves,
                lacunarity: self.noise.lacunarity,
                persistence: self.noise.persistence,
                noise_type: nightshade::ecs::terrain::NoiseType::Perlin,
            },
            uv_scale: self.uv_scale,
        }
    }
}

pub fn default_terrain_config() -> TerrainConfig {
    TerrainConfig {
        width: 200.0,
        depth: 200.0,
        resolution_x: 128,
        resolution_z: 128,
        height_scale: 6.0,
        noise: NoiseConfig {
            seed: 42,
            frequency: 0.015,
            octaves: 4,
            lacunarity: 2.0,
            persistence: 0.45,
        },
        uv_scale: [20.0, 20.0],
    }
}
