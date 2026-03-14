use nightshade::ecs::text::components::TextProperties;
use nightshade::ecs::world::commands::WorldCommand;
use nightshade::prelude::*;
use nightshade::render::{
    boolean_intersect, boolean_subtract, boolean_union, generate_blurred_texture,
    generate_circle_texture_with_aa,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

const SCENE_COUNT: usize = 20;

const SCENE_NAMES: &[&str] = &[
    "Matrix Rain",
    "Starfield Warp",
    "Spiral Galaxy",
    "Neon Pulse",
    "Lightning Storm",
    "Fireworks",
    "Kaleidoscope",
    "Ripple Pond",
    "Particle Storm",
    "DNA Helix",
    "Confetti",
    "Solar System",
    "CRT Matrix",
    "Underwater Ripples",
    "Glitch Storm",
    "Plasma Galaxy",
    "Boolean Dance",
    "Stencil Windows",
    "Shadow Theater",
    "Path Stars",
];

const POST_PROCESS_SHADER: &str = include_str!("../shaders/post_process.wgsl");

const HELIX_NUCLEOTIDE_COUNT: usize = 30;
const CONFETTI_COUNT: usize = 250;
const PLANET_COUNT: usize = 7;

fn hash(seed: u32) -> u32 {
    let mut x = seed;
    x ^= x >> 16;
    x = x.wrapping_mul(0x45d9f3b);
    x ^= x >> 16;
    x = x.wrapping_mul(0x45d9f3b);
    x ^= x >> 16;
    x
}

fn hash_f32(seed: u32) -> f32 {
    (hash(seed) & 0xFFFF) as f32 / 65535.0
}

fn hash_range(seed: u32, min: f32, max: f32) -> f32 {
    min + hash_f32(seed) * (max - min)
}

fn hue_to_rgb(hue: f32) -> (f32, f32, f32) {
    let hue = hue.fract();
    let segment = hue * 6.0;
    let fraction = segment.fract();
    match segment as u32 {
        0 => (1.0, fraction, 0.0),
        1 => (1.0 - fraction, 1.0, 0.0),
        2 => (0.0, 1.0, fraction),
        3 => (0.0, 1.0 - fraction, 1.0),
        4 => (fraction, 0.0, 1.0),
        _ => (1.0, 0.0, 1.0 - fraction),
    }
}

struct MatrixColumnState {
    entity_start: usize,
    trail_length: usize,
    cursor_y: f32,
    speed: f32,
}

struct StarState {
    angle: f32,
    distance: f32,
    speed: f32,
}

struct GalaxyParticleState {
    radius: f32,
    angle: f32,
    angular_speed: f32,
}

struct NeonBlobState {
    phase_x: f32,
    phase_y: f32,
    frequency_x: f32,
    frequency_y: f32,
    orbit_x: f32,
    orbit_y: f32,
    color_phase: f32,
    size: f32,
}

struct RainDropState {
    position_x: f32,
    position_y: f32,
    speed: f32,
}

struct KaleidoscopeItemState {
    orbit_radius: f32,
    orbit_speed: f32,
    base_angle: f32,
    color_phase: f32,
}

struct WaveSourceState {
    position: Vec2,
    ring_ages: Vec<f32>,
}

struct ConfettiState {
    velocity_x: f32,
    velocity_y: f32,
    angular_velocity: f32,
}

struct PlanetState {
    orbit_radius: f32,
    angle: f32,
    speed: f32,
}

struct MoonState {
    orbit_radius: f32,
    angle: f32,
    speed: f32,
    planet_index: usize,
}

struct BooleanShapeState {
    orbit_radius: f32,
    orbit_speed: f32,
    angle: f32,
}

struct StencilMaskState {
    orbit_radius: f32,
    orbit_speed: f32,
    angle: f32,
}

struct ShadowItemState {
    velocity: Vec2,
    size: f32,
}

struct GfxShowcase {
    camera_entity: Option<Entity>,
    current_scene: usize,
    time: f32,
    scene_entities: Vec<Entity>,
    scene_emitters: Vec<Entity>,
    effect_mode: Arc<AtomicU32>,

    matrix_columns: Vec<MatrixColumnState>,
    stars: Vec<StarState>,
    galaxy_particles: Vec<GalaxyParticleState>,
    neon_blobs: Vec<NeonBlobState>,
    rain_drops: Vec<RainDropState>,
    lightning_timer: f32,
    lightning_bolt_entities: Vec<Entity>,
    lightning_bolt_age: f32,
    flash_entity: Option<Entity>,
    flash_alpha: f32,
    firework_timer: f32,
    kaleidoscope_items: Vec<KaleidoscopeItemState>,
    wave_sources: Vec<WaveSourceState>,
    storm_timer: f32,
    confetti: Vec<ConfettiState>,
    planets: Vec<PlanetState>,
    moons: Vec<MoonState>,

    boolean_shapes: Vec<BooleanShapeState>,
    boolean_entity_start: usize,
    stencil_masks: Vec<StencilMaskState>,
    stencil_stripe_count: usize,
    shadow_items: Vec<ShadowItemState>,
    shadow_entity_start: usize,
    path_star_count: usize,
}

impl Default for GfxShowcase {
    fn default() -> Self {
        Self {
            camera_entity: None,
            current_scene: 0,
            time: 0.0,
            scene_entities: Vec::new(),
            scene_emitters: Vec::new(),
            effect_mode: Arc::new(AtomicU32::new(0)),
            matrix_columns: Vec::new(),
            stars: Vec::new(),
            galaxy_particles: Vec::new(),
            neon_blobs: Vec::new(),
            rain_drops: Vec::new(),
            lightning_timer: 2.0,
            lightning_bolt_entities: Vec::new(),
            lightning_bolt_age: 0.0,
            flash_entity: None,
            flash_alpha: 0.0,
            firework_timer: 1.0,
            kaleidoscope_items: Vec::new(),
            wave_sources: Vec::new(),
            storm_timer: 0.2,
            confetti: Vec::new(),
            planets: Vec::new(),
            moons: Vec::new(),
            boolean_shapes: Vec::new(),
            boolean_entity_start: 0,
            stencil_masks: Vec::new(),
            stencil_stripe_count: 0,
            shadow_items: Vec::new(),
            shadow_entity_start: 0,
            path_star_count: 0,
        }
    }
}

impl GfxShowcase {
    fn clear_scene(&mut self, world: &mut World) {
        if !self.lightning_bolt_entities.is_empty() {
            world.despawn_entities(&self.lightning_bolt_entities);
            self.lightning_bolt_entities.clear();
        }
        if !self.scene_entities.is_empty() {
            world.despawn_entities(&self.scene_entities);
            self.scene_entities.clear();
        }
        if !self.scene_emitters.is_empty() {
            world.despawn_entities(&self.scene_emitters);
            self.scene_emitters.clear();
        }

        self.matrix_columns.clear();
        self.stars.clear();
        self.galaxy_particles.clear();
        self.neon_blobs.clear();
        self.rain_drops.clear();
        self.lightning_timer = 2.0;
        self.lightning_bolt_age = 0.0;
        self.flash_entity = None;
        self.flash_alpha = 0.0;
        self.firework_timer = 1.0;
        self.kaleidoscope_items.clear();
        self.wave_sources.clear();
        self.storm_timer = 0.2;
        self.confetti.clear();
        self.planets.clear();
        self.moons.clear();
        self.boolean_shapes.clear();
        self.boolean_entity_start = 0;
        self.stencil_masks.clear();
        self.stencil_stripe_count = 0;
        self.shadow_items.clear();
        self.shadow_entity_start = 0;
        self.path_star_count = 0;
    }

    fn switch_scene(&mut self, world: &mut World, scene: usize) {
        self.clear_scene(world);
        self.current_scene = scene;

        let mode = match scene {
            12 => 1,
            13 => 2,
            14 => 3,
            15 => 4,
            _ => 0,
        };
        self.effect_mode.store(mode, Ordering::Relaxed);

        self.build_current_scene(world);
    }

    fn build_current_scene(&mut self, world: &mut World) {
        match self.current_scene {
            0 | 12 => self.build_matrix_rain(world),
            1 => self.build_starfield(world),
            2 | 15 => self.build_galaxy(world),
            3 => self.build_neon_pulse(world),
            4 | 14 => self.build_lightning_storm(world),
            5 => self.build_fireworks(world),
            6 => self.build_kaleidoscope(world),
            7 | 13 => self.build_ripple_pond(world),
            8 => self.build_particle_storm(world),
            9 => self.build_dna_helix(world),
            10 => self.build_confetti(world),
            11 => self.build_solar_system(world),
            16 => self.build_boolean_dance(world),
            17 => self.build_stencil_windows(world),
            18 => self.build_shadow_theater(world),
            19 => self.build_path_stars(world),
            _ => {}
        }
    }

    fn update_current_scene(&mut self, world: &mut World, delta_time: f32) {
        match self.current_scene {
            0 | 12 => self.update_matrix_rain(world, delta_time),
            1 => self.update_starfield(world, delta_time),
            2 | 15 => self.update_galaxy(world, delta_time),
            3 => self.update_neon_pulse(world),
            4 | 14 => self.update_lightning_storm(world, delta_time),
            5 => self.update_fireworks(world, delta_time),
            6 => self.update_kaleidoscope(world),
            7 | 13 => self.update_ripple_pond(world, delta_time),
            8 => self.update_particle_storm(world, delta_time),
            9 => self.update_dna_helix(world),
            10 => self.update_confetti(world, delta_time),
            11 => self.update_solar_system(world, delta_time),
            16 => self.update_boolean_dance(world, delta_time),
            17 => self.update_stencil_windows(world, delta_time),
            18 => self.update_shadow_theater(world, delta_time),
            19 => self.update_path_stars(world),
            _ => {}
        }
    }

    // ======================== Scene 0: Matrix Rain ========================

    fn build_matrix_rain(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.0, 0.015, 0.0, 1.0];

        let column_count = 45;
        let column_spacing = 960.0 / column_count as f32;
        let start_x = -480.0 + column_spacing / 2.0;
        let trail_spacing = 18.0;

        for column_index in 0..column_count {
            let position_x = start_x + column_index as f32 * column_spacing;
            let trail_length = hash_range(column_index as u32, 8.0, 22.0) as usize;
            let speed = hash_range(column_index as u32 + 100, 80.0, 280.0);
            let total_height = trail_length as f32 * trail_spacing;
            let cursor_y = hash_range(column_index as u32 + 200, -270.0, 270.0 + total_height);

            let entity_start = self.scene_entities.len();

            let head = spawn_rect(
                world,
                Vec2::new(position_x, cursor_y),
                Vec2::new(6.0, 10.0),
                [0.7, 1.0, 0.7, 1.0],
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(head) {
                sprite.depth = 5.0;
            }
            self.scene_entities.push(head);

            for trail_index in 0..trail_length {
                let trail_y = cursor_y + (trail_index + 1) as f32 * trail_spacing;
                let parameter = trail_index as f32 / trail_length as f32;
                let alpha = (1.0 - parameter).powf(1.2) * 0.8;
                let green = 0.3 + 0.5 * (1.0 - parameter);

                let entity = spawn_rect(
                    world,
                    Vec2::new(position_x, trail_y),
                    Vec2::new(6.0, 10.0),
                    [0.0, green, 0.0, alpha],
                );
                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.depth = 5.0;
                }
                self.scene_entities.push(entity);
            }

            self.matrix_columns.push(MatrixColumnState {
                entity_start,
                trail_length,
                cursor_y,
                speed,
            });
        }
    }

    fn update_matrix_rain(&mut self, world: &mut World, delta_time: f32) {
        let trail_spacing = 18.0;

        for column_index in 0..self.matrix_columns.len() {
            let column = &mut self.matrix_columns[column_index];
            column.cursor_y -= column.speed * delta_time;

            let total_trail_height = column.trail_length as f32 * trail_spacing;
            if column.cursor_y + total_trail_height < -270.0 {
                column.cursor_y = 270.0
                    + total_trail_height
                    + hash_range(
                        hash((self.time * 100.0) as u32 + column_index as u32),
                        0.0,
                        200.0,
                    );
                column.speed = hash_range(
                    hash((self.time * 50.0) as u32 + column_index as u32 + 500),
                    80.0,
                    280.0,
                );
            }

            let head_entity = self.scene_entities[column.entity_start];
            if let Some(sprite) = world.sprite2d.get_sprite_mut(head_entity) {
                sprite.position.y = column.cursor_y;
                let flicker = 0.8 + 0.2 * (self.time * 15.0 + column_index as f32 * 3.7).sin();
                sprite.color = [0.6 * flicker, 1.0 * flicker, 0.6 * flicker, 1.0];
            }

            for trail_index in 0..column.trail_length {
                let entity = self.scene_entities[column.entity_start + 1 + trail_index];
                let trail_y = column.cursor_y + (trail_index + 1) as f32 * trail_spacing;
                let parameter = trail_index as f32 / column.trail_length as f32;
                let alpha = (1.0 - parameter).powf(1.2) * 0.8;
                let green = 0.3 + 0.5 * (1.0 - parameter);

                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.position.y = trail_y;
                    sprite.color = [0.0, green, 0.0, alpha];
                }
            }
        }
    }

    // ======================== Scene 1: Starfield Warp ========================

    fn build_starfield(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.0, 0.0, 0.02, 1.0];

        let star_count = 350;

        for index in 0..star_count {
            let angle = hash_range(index as u32, 0.0, std::f32::consts::TAU);
            let distance = hash_range(index as u32 + 1000, 5.0, 500.0);
            let speed = hash_range(index as u32 + 2000, 40.0, 140.0);

            let position = Vec2::new(angle.cos() * distance, angle.sin() * distance);
            let size = 1.0 + distance * 0.008;

            let entity = spawn_circle(world, position, size, [1.0, 1.0, 1.0, 0.8]);
            if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.depth = 5.0;
                sprite.blend_mode = SpriteBlendMode::Additive;
            }
            self.scene_entities.push(entity);
            self.stars.push(StarState {
                angle,
                distance,
                speed,
            });
        }
    }

    fn update_starfield(&mut self, world: &mut World, delta_time: f32) {
        for (index, star) in self.stars.iter_mut().enumerate() {
            star.distance += star.speed * (1.0 + star.distance * 0.004) * delta_time;

            if star.distance > 550.0 {
                star.distance =
                    hash_range(hash((self.time * 1000.0) as u32 + index as u32), 2.0, 15.0);
                star.angle = hash_range(
                    hash(index as u32 + (self.time * 100.0) as u32 + 777),
                    0.0,
                    std::f32::consts::TAU,
                );
                star.speed = hash_range(
                    hash(index as u32 + (self.time * 100.0) as u32 + 888),
                    40.0,
                    140.0,
                );
            }

            let position = Vec2::new(
                star.angle.cos() * star.distance,
                star.angle.sin() * star.distance,
            );
            let size = 1.0 + star.distance * 0.012;
            let alpha = (star.distance / 80.0).min(1.0);
            let stretch = 1.0 + star.distance * 0.003;

            if let Some(sprite) = world.sprite2d.get_sprite_mut(self.scene_entities[index]) {
                sprite.position = position;
                sprite.size = Vec2::new(size * 2.0, size * 2.0 * stretch);
                sprite.color = [0.9, 0.95, 1.0, alpha];
                sprite.rotation = star.angle + std::f32::consts::FRAC_PI_2;
            }
        }
    }

    // ======================== Scene 2: Spiral Galaxy ========================

    fn build_galaxy(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.02, 0.01, 0.04, 1.0];

        let particle_count = 500;
        let arm_count = 2;
        let twist = 3.0;

        for index in 0..particle_count {
            let arm = index % arm_count;
            let arm_offset = arm as f32 * std::f32::consts::TAU / arm_count as f32;

            let radius = hash_range(index as u32, 8.0, 240.0);
            let angular_speed = 0.4 / (radius.sqrt() + 1.0);
            let scatter = hash_range(index as u32 + 3000, -0.4, 0.4);
            let base_angle = arm_offset + twist * (radius / 240.0).powf(0.7) + scatter;

            let tilt = 0.45;
            let position = Vec2::new(radius * base_angle.cos(), radius * base_angle.sin() * tilt);

            let normalized_radius = radius / 240.0;
            let (red, green, blue) = if normalized_radius < 0.25 {
                (0.8 + normalized_radius, 0.85 + normalized_radius * 0.5, 1.0)
            } else {
                let outer = (normalized_radius - 0.25) / 0.75;
                (0.9 + outer * 0.1, 0.7 - outer * 0.4, 0.9 - outer * 0.6)
            };

            let size = hash_range(index as u32 + 4000, 1.5, 4.0);
            let alpha = hash_range(index as u32 + 5000, 0.3, 0.7);

            let entity = spawn_circle(world, position, size, [red, green, blue, alpha]);
            if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.depth = 5.0;
                sprite.blend_mode = SpriteBlendMode::Additive;
            }
            self.scene_entities.push(entity);
            self.galaxy_particles.push(GalaxyParticleState {
                radius,
                angle: base_angle,
                angular_speed,
            });
        }

        let core = spawn_shape(
            world,
            SpriteShape::SoftCircle,
            Vec2::zeros(),
            Vec2::new(100.0, 55.0),
            [0.9, 0.85, 1.0, 0.3],
        );
        if let Some(sprite) = world.sprite2d.get_sprite_mut(core) {
            sprite.depth = 4.0;
            sprite.blend_mode = SpriteBlendMode::Additive;
        }
        self.scene_entities.push(core);
    }

    fn update_galaxy(&mut self, world: &mut World, delta_time: f32) {
        let tilt = 0.45;

        for (index, particle) in self.galaxy_particles.iter_mut().enumerate() {
            particle.angle += particle.angular_speed * delta_time;

            let position = Vec2::new(
                particle.radius * particle.angle.cos(),
                particle.radius * particle.angle.sin() * tilt,
            );

            if let Some(sprite) = world.sprite2d.get_sprite_mut(self.scene_entities[index]) {
                sprite.position = position;
            }
        }

        let core_index = self.galaxy_particles.len();
        if core_index < self.scene_entities.len() {
            let pulse = 0.9 + 0.1 * (self.time * 0.5).sin();
            if let Some(sprite) = world
                .sprite2d
                .get_sprite_mut(self.scene_entities[core_index])
            {
                sprite.size = Vec2::new(100.0 * pulse, 55.0 * pulse);
                sprite.color[3] = 0.25 + 0.1 * (self.time * 0.3).sin();
            }
        }
    }

    // ======================== Scene 3: Neon Pulse ========================

    fn build_neon_pulse(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.03, 0.01, 0.05, 1.0];

        let blob_count = 14;

        for index in 0..blob_count {
            let size = hash_range(index as u32, 80.0, 180.0);
            let hue = hash_range(index as u32 + 300, 0.0, 1.0);
            let (red, green, blue) = hue_to_rgb(hue);

            let entity = spawn_shape(
                world,
                SpriteShape::SoftCircle,
                Vec2::zeros(),
                Vec2::new(size, size),
                [red, green, blue, 0.2],
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.depth = 5.0;
                sprite.blend_mode = SpriteBlendMode::Additive;
            }
            self.scene_entities.push(entity);

            self.neon_blobs.push(NeonBlobState {
                phase_x: hash_range(index as u32 + 400, 0.0, std::f32::consts::TAU),
                phase_y: hash_range(index as u32 + 500, 0.0, std::f32::consts::TAU),
                frequency_x: hash_range(index as u32 + 600, 0.12, 0.45),
                frequency_y: hash_range(index as u32 + 700, 0.12, 0.45),
                orbit_x: hash_range(index as u32 + 800, 100.0, 300.0),
                orbit_y: hash_range(index as u32 + 900, 60.0, 180.0),
                color_phase: hue,
                size,
            });
        }
    }

    fn update_neon_pulse(&mut self, world: &mut World) {
        for (index, blob) in self.neon_blobs.iter().enumerate() {
            let position_x = (self.time * blob.frequency_x + blob.phase_x).sin() * blob.orbit_x;
            let position_y = (self.time * blob.frequency_y + blob.phase_y).sin() * blob.orbit_y;

            let hue = (blob.color_phase + self.time * 0.04).fract();
            let (red, green, blue) = hue_to_rgb(hue);

            let pulse = 0.85 + 0.15 * (self.time * 0.3 + index as f32 * 0.5).sin();
            let size = blob.size * pulse;

            if let Some(sprite) = world.sprite2d.get_sprite_mut(self.scene_entities[index]) {
                sprite.position = Vec2::new(position_x, position_y);
                sprite.size = Vec2::new(size, size);
                sprite.color = [red, green, blue, 0.2];
            }
        }
    }

    // ======================== Scene 4: Lightning Storm ========================

    fn build_lightning_storm(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.015, 0.015, 0.03, 1.0];

        let rain_count = 200;
        for index in 0..rain_count {
            let position_x = hash_range(index as u32, -480.0, 480.0);
            let position_y = hash_range(index as u32 + 100, -270.0, 270.0);
            let speed = hash_range(index as u32 + 200, 250.0, 550.0);
            let length = hash_range(index as u32 + 300, 5.0, 14.0);

            let entity = spawn_line(
                world,
                Vec2::new(position_x, position_y),
                Vec2::new(position_x - 1.5, position_y - length),
                0.5,
                [0.3, 0.35, 0.5, 0.25],
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.depth = 3.0;
            }
            self.scene_entities.push(entity);
            self.rain_drops.push(RainDropState {
                position_x,
                position_y,
                speed,
            });
        }

        let flash = spawn_shape(
            world,
            SpriteShape::SoftCircle,
            Vec2::new(0.0, 80.0),
            Vec2::new(900.0, 700.0),
            [0.8, 0.85, 1.0, 0.0],
        );
        if let Some(sprite) = world.sprite2d.get_sprite_mut(flash) {
            sprite.depth = 1.0;
            sprite.blend_mode = SpriteBlendMode::Additive;
        }
        self.flash_entity = Some(flash);
        self.scene_entities.push(flash);

        self.lightning_timer = hash_range(42, 0.5, 2.5);
        self.flash_alpha = 0.0;
    }

    fn spawn_lightning_bolt(&mut self, world: &mut World) {
        let start_x = hash_range((self.time * 100.0) as u32, -300.0, 300.0);
        let start_y = 260.0;
        let end_y = hash_range((self.time * 100.0) as u32 + 1, -150.0, 50.0);
        let segments = 14;

        let mut current_x = start_x;
        let mut current_y = start_y;

        for segment in 0..segments {
            let next_y = start_y - (start_y - end_y) * (segment + 1) as f32 / segments as f32;
            let jitter = hash_range(
                (self.time * 1000.0) as u32 + segment as u32 * 7,
                -35.0,
                35.0,
            );
            let next_x = current_x + jitter;

            let thickness = 3.0 - segment as f32 * 0.15;

            let entity = spawn_line(
                world,
                Vec2::new(current_x, current_y),
                Vec2::new(next_x, next_y),
                thickness.max(1.0),
                [0.7, 0.8, 1.0, 1.0],
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.depth = 10.0;
                sprite.blend_mode = SpriteBlendMode::Additive;
            }
            self.lightning_bolt_entities.push(entity);

            if hash_f32((self.time * 1000.0) as u32 + segment as u32 * 13 + 500) < 0.3 {
                let branch_angle = hash_range(
                    (self.time * 1000.0) as u32 + segment as u32 * 17 + 600,
                    -1.2,
                    1.2,
                );
                let branch_length = hash_range(
                    (self.time * 1000.0) as u32 + segment as u32 * 19 + 700,
                    20.0,
                    70.0,
                );
                let branch_end_x = next_x + branch_angle * branch_length;
                let branch_end_y = next_y - branch_length * 0.4;

                let branch = spawn_line(
                    world,
                    Vec2::new(next_x, next_y),
                    Vec2::new(branch_end_x, branch_end_y),
                    1.5,
                    [0.5, 0.6, 0.9, 0.8],
                );
                if let Some(sprite) = world.sprite2d.get_sprite_mut(branch) {
                    sprite.depth = 10.0;
                    sprite.blend_mode = SpriteBlendMode::Additive;
                }
                self.lightning_bolt_entities.push(branch);
            }

            current_x = next_x;
            current_y = next_y;
        }

        self.lightning_bolt_age = 0.0;
        self.flash_alpha = 0.7;

        if let Some(entity) = self.flash_entity
            && let Some(sprite) = world.sprite2d.get_sprite_mut(entity)
        {
            sprite.position.x = start_x;
        }
    }

    fn update_lightning_storm(&mut self, world: &mut World, delta_time: f32) {
        for (index, drop) in self.rain_drops.iter_mut().enumerate() {
            drop.position_y -= drop.speed * delta_time;
            drop.position_x -= drop.speed * 0.04 * delta_time;

            if drop.position_y < -280.0 {
                drop.position_y = 280.0;
                drop.position_x = hash_range(
                    hash((self.time * 100.0) as u32 + index as u32 + 999),
                    -480.0,
                    480.0,
                );
            }

            if let Some(sprite) = world.sprite2d.get_sprite_mut(self.scene_entities[index]) {
                sprite.position = Vec2::new(drop.position_x, drop.position_y);
            }
        }

        self.lightning_timer -= delta_time;
        if self.lightning_timer <= 0.0 {
            self.spawn_lightning_bolt(world);
            self.lightning_timer = hash_range((self.time * 100.0) as u32 + 9999, 0.4, 2.5);
        }

        if !self.lightning_bolt_entities.is_empty() {
            self.lightning_bolt_age += delta_time;
            let bolt_alpha = (1.0 - self.lightning_bolt_age / 0.25).max(0.0);

            for &entity in &self.lightning_bolt_entities {
                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.color[3] = bolt_alpha;
                }
            }

            if bolt_alpha <= 0.0 {
                world.despawn_entities(&self.lightning_bolt_entities);
                self.lightning_bolt_entities.clear();
            }
        }

        if self.flash_alpha > 0.0 {
            self.flash_alpha = (self.flash_alpha - delta_time * 3.5).max(0.0);
            if let Some(entity) = self.flash_entity
                && let Some(sprite) = world.sprite2d.get_sprite_mut(entity)
            {
                sprite.color[3] = self.flash_alpha;
            }
        }
    }

    // ======================== Scene 5: Fireworks ========================

    fn build_fireworks(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.01, 0.01, 0.03, 1.0];
        self.firework_timer = 0.3;
    }

    fn update_fireworks(&mut self, world: &mut World, delta_time: f32) {
        self.firework_timer -= delta_time;

        if self.firework_timer <= 0.0 {
            let position_x = hash_range((self.time * 100.0) as u32, -350.0, 350.0);
            let position_y = hash_range((self.time * 100.0) as u32 + 1, -80.0, 180.0);

            let hue = hash_f32((self.time * 100.0) as u32 + 2);
            let (red, green, blue) = hue_to_rgb(hue);

            let (texture_index, uv_min, uv_max) = shape_texture_info(SpriteShape::SoftCircle);

            let mut emitter = SpriteParticleEmitter::explosion(position_x, position_y)
                .with_texture(texture_index)
                .with_uv(uv_min, uv_max)
                .with_depth(10.0)
                .with_color(ColorRange2D::new(
                    [red, green, blue, 1.0],
                    [red * 0.3, green * 0.3, blue * 0.3, 0.0],
                ));
            emitter.max_particles = 128;
            emitter.burst_count = 96;

            let entity = world.spawn();
            world.sprite2d.set_sprite_particle_emitter(entity, emitter);
            self.scene_emitters.push(entity);

            if hash_f32((self.time * 100.0) as u32 + 3) < 0.4 {
                let secondary_hue = hash_f32((self.time * 100.0) as u32 + 4);
                let (red_secondary, green_secondary, blue_secondary) = hue_to_rgb(secondary_hue);
                let offset_x = hash_range((self.time * 100.0) as u32 + 5, -60.0, 60.0);
                let offset_y = hash_range((self.time * 100.0) as u32 + 6, -40.0, 40.0);

                let mut secondary_emitter =
                    SpriteParticleEmitter::explosion(position_x + offset_x, position_y + offset_y)
                        .with_texture(texture_index)
                        .with_uv(uv_min, uv_max)
                        .with_depth(10.0)
                        .with_color(ColorRange2D::new(
                            [red_secondary, green_secondary, blue_secondary, 1.0],
                            [
                                red_secondary * 0.3,
                                green_secondary * 0.3,
                                blue_secondary * 0.3,
                                0.0,
                            ],
                        ));
                secondary_emitter.max_particles = 80;
                secondary_emitter.burst_count = 64;

                let secondary_entity = world.spawn();
                world
                    .sprite2d
                    .set_sprite_particle_emitter(secondary_entity, secondary_emitter);
                self.scene_emitters.push(secondary_entity);
            }

            if hash_f32((self.time * 100.0) as u32 + 7) < 0.25 {
                self.firework_timer = hash_range((self.time * 100.0) as u32 + 8, 0.05, 0.2);
            } else {
                self.firework_timer = hash_range((self.time * 100.0) as u32 + 8, 0.4, 1.8);
            }
        }

        while self.scene_emitters.len() > 25 {
            let entity = self.scene_emitters.remove(0);
            world.despawn_entities(&[entity]);
        }
    }

    // ======================== Scene 6: Kaleidoscope ========================

    fn build_kaleidoscope(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.03, 0.02, 0.06, 1.0];

        let primary_count = 20;
        let symmetry = 6;
        let shapes = [
            SpriteShape::Triangle,
            SpriteShape::Circle,
            SpriteShape::Ring,
            SpriteShape::Rect,
            SpriteShape::Capsule,
        ];

        for primary_index in 0..primary_count {
            let orbit_radius = hash_range(primary_index as u32, 30.0, 220.0);
            let orbit_speed = hash_range(primary_index as u32 + 100, 0.08, 0.5);
            let base_angle = hash_range(primary_index as u32 + 200, 0.0, std::f32::consts::TAU);
            let size_value = hash_range(primary_index as u32 + 300, 6.0, 22.0);
            let shape_index = hash(primary_index as u32 + 400) as usize % shapes.len();
            let color_phase = hash_range(primary_index as u32 + 500, 0.0, 1.0);

            let size = if shapes[shape_index] == SpriteShape::Capsule {
                Vec2::new(size_value * 1.8, size_value * 0.7)
            } else {
                Vec2::new(size_value, size_value)
            };

            let direction = if primary_index % 2 == 0 { 1.0 } else { -1.0 };

            for mirror in 0..symmetry {
                let mirror_angle = mirror as f32 * std::f32::consts::TAU / symmetry as f32;
                let angle = base_angle + mirror_angle;
                let position = Vec2::new(orbit_radius * angle.cos(), orbit_radius * angle.sin());

                let hue = (color_phase + mirror as f32 / symmetry as f32 * 0.15).fract();
                let (red, green, blue) = hue_to_rgb(hue);

                let entity = spawn_shape(
                    world,
                    shapes[shape_index],
                    position,
                    size,
                    [red, green, blue, 0.65],
                );
                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.depth = 5.0;
                    if shapes[shape_index] == SpriteShape::SoftCircle {
                        sprite.blend_mode = SpriteBlendMode::Additive;
                    }
                }
                self.scene_entities.push(entity);
            }

            self.kaleidoscope_items.push(KaleidoscopeItemState {
                orbit_radius,
                orbit_speed: orbit_speed * direction,
                base_angle,
                color_phase,
            });
        }
    }

    fn update_kaleidoscope(&mut self, world: &mut World) {
        let symmetry = 6;

        for (primary_index, item) in self.kaleidoscope_items.iter().enumerate() {
            let current_angle = item.base_angle + self.time * item.orbit_speed;

            for mirror in 0..symmetry {
                let entity_index = primary_index * symmetry + mirror;
                let mirror_angle = mirror as f32 * std::f32::consts::TAU / symmetry as f32;
                let angle = current_angle + mirror_angle;
                let position = Vec2::new(
                    item.orbit_radius * angle.cos(),
                    item.orbit_radius * angle.sin(),
                );

                let hue =
                    (item.color_phase + self.time * 0.06 + mirror as f32 / symmetry as f32 * 0.15)
                        .fract();
                let (red, green, blue) = hue_to_rgb(hue);

                if let Some(sprite) = world
                    .sprite2d
                    .get_sprite_mut(self.scene_entities[entity_index])
                {
                    sprite.position = position;
                    sprite.color = [red, green, blue, 0.65];
                    sprite.rotation = angle;
                }
            }
        }
    }

    // ======================== Scene 7: Ripple Pond ========================

    fn build_ripple_pond(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.02, 0.02, 0.06, 1.0];

        let source_positions = [
            Vec2::new(-160.0, 80.0),
            Vec2::new(130.0, -70.0),
            Vec2::new(-90.0, -130.0),
            Vec2::new(200.0, 110.0),
            Vec2::new(0.0, 0.0),
        ];

        let rings_per_source = 18;
        let max_age = rings_per_source as f32 * 0.4;

        for (source_index, &position) in source_positions.iter().enumerate() {
            let mut ring_ages = Vec::new();

            for ring_index in 0..rings_per_source {
                let age = ring_index as f32 * 0.4;
                let radius = (age * 50.0).max(3.0);
                let alpha = (1.0 - age / max_age).max(0.0) * 0.4;
                let hue = (source_index as f32 / source_positions.len() as f32
                    + ring_index as f32 * 0.02)
                    .fract();
                let (red, green, blue) = hue_to_rgb(hue);

                let entity = spawn_ring(world, position, radius, [red, green, blue, alpha]);
                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.depth = 5.0;
                    sprite.blend_mode = SpriteBlendMode::Additive;
                }
                self.scene_entities.push(entity);
                ring_ages.push(age);
            }

            self.wave_sources.push(WaveSourceState {
                position,
                ring_ages,
            });
        }
    }

    fn update_ripple_pond(&mut self, world: &mut World, delta_time: f32) {
        let rings_per_source = 18;
        let max_age = 7.2;
        let expansion_speed = 50.0;
        let source_count = self.wave_sources.len() as f32;

        for (source_index, source) in self.wave_sources.iter_mut().enumerate() {
            for (ring_index, age) in source.ring_ages.iter_mut().enumerate() {
                *age += delta_time;

                if *age > max_age {
                    *age -= max_age;
                }

                let radius = (*age * expansion_speed).max(3.0);
                let alpha = (1.0 - *age / max_age).max(0.0) * 0.35;
                let hue = (source_index as f32 / source_count
                    + ring_index as f32 * 0.02
                    + self.time * 0.02)
                    .fract();
                let (red, green, blue) = hue_to_rgb(hue);

                let entity_index = source_index * rings_per_source + ring_index;
                if let Some(sprite) = world
                    .sprite2d
                    .get_sprite_mut(self.scene_entities[entity_index])
                {
                    sprite.position = source.position;
                    sprite.size = Vec2::new(radius * 2.0, radius * 2.0);
                    sprite.color = [red, green, blue, alpha];
                }
            }
        }
    }

    // ======================== Scene 8: Particle Storm ========================

    fn build_particle_storm(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.02, 0.01, 0.01, 1.0];
        self.storm_timer = 0.0;
    }

    fn update_particle_storm(&mut self, world: &mut World, delta_time: f32) {
        self.storm_timer -= delta_time;

        if self.storm_timer <= 0.0 {
            let seed = (self.time * 100.0) as u32;
            let position_x = hash_range(seed, -400.0, 400.0);
            let position_y = hash_range(seed + 1, -200.0, 200.0);

            let (texture_index, uv_min, uv_max) = shape_texture_info(SpriteShape::SoftCircle);
            let emitter_type = hash(seed + 2) % 5;

            let emitter = match emitter_type {
                0 => {
                    let hue = hash_f32(seed + 3);
                    let (red, green, blue) = hue_to_rgb(hue);
                    let mut emitter = SpriteParticleEmitter::explosion(position_x, position_y)
                        .with_texture(texture_index)
                        .with_uv(uv_min, uv_max)
                        .with_depth(8.0)
                        .with_color(ColorRange2D::new(
                            [red, green, blue, 1.0],
                            [red * 0.2, green * 0.2, blue * 0.2, 0.0],
                        ));
                    emitter.max_particles = 80;
                    emitter.burst_count = 60;
                    emitter
                }
                1 => {
                    let mut emitter = SpriteParticleEmitter::fire_trail(position_x, position_y)
                        .with_texture(texture_index)
                        .with_uv(uv_min, uv_max)
                        .with_depth(8.0);
                    emitter.max_particles = 100;
                    emitter
                }
                2 => {
                    let mut emitter = SpriteParticleEmitter::sparks(position_x, position_y)
                        .with_texture(texture_index)
                        .with_uv(uv_min, uv_max)
                        .with_depth(8.0);
                    emitter.max_particles = 60;
                    emitter.burst_count = 40;
                    emitter
                }
                3 => {
                    let mut emitter = SpriteParticleEmitter::smoke(position_x, position_y)
                        .with_texture(texture_index)
                        .with_uv(uv_min, uv_max)
                        .with_depth(6.0);
                    emitter.max_particles = 40;
                    emitter
                }
                _ => {
                    let hue = hash_f32(seed + 4);
                    let (red, green, blue) = hue_to_rgb(hue);
                    let mut emitter = SpriteParticleEmitter::explosion(position_x, position_y)
                        .with_texture(texture_index)
                        .with_uv(uv_min, uv_max)
                        .with_depth(10.0)
                        .with_color(ColorRange2D::new(
                            [1.0, 1.0, 1.0, 1.0],
                            [red, green, blue, 0.0],
                        ));
                    emitter.max_particles = 150;
                    emitter.burst_count = 120;
                    emitter
                }
            };

            let entity = world.spawn();
            world.sprite2d.set_sprite_particle_emitter(entity, emitter);
            self.scene_emitters.push(entity);

            self.storm_timer = hash_range(seed + 5, 0.08, 0.35);
        }

        while self.scene_emitters.len() > 40 {
            let entity = self.scene_emitters.remove(0);
            world.despawn_entities(&[entity]);
        }
    }

    // ======================== Scene 9: DNA Helix ========================

    fn build_dna_helix(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.01, 0.02, 0.04, 1.0];

        let helix_radius = 90.0;
        let y_min = -250.0;
        let y_max = 250.0;
        let y_spacing = (y_max - y_min) / (HELIX_NUCLEOTIDE_COUNT - 1) as f32;
        let pitch = 180.0;
        let base_size = 8.0;

        for nucleotide_index in 0..HELIX_NUCLEOTIDE_COUNT {
            let axis_y = y_min + nucleotide_index as f32 * y_spacing;
            let phase = std::f32::consts::TAU * axis_y / pitch;

            let strand1_x = helix_radius * phase.cos();
            let depth1 = phase.sin();
            let size1 = base_size * (1.0 + depth1 * 0.4);
            let alpha1 = 0.5 + 0.5 * (depth1 * 0.5 + 0.5);

            let strand1 = spawn_circle(
                world,
                Vec2::new(strand1_x, axis_y),
                size1,
                [0.2, 0.5, 1.0, alpha1],
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(strand1) {
                sprite.depth = 5.0 + depth1;
                sprite.blend_mode = SpriteBlendMode::Additive;
            }
            self.scene_entities.push(strand1);
        }

        for nucleotide_index in 0..HELIX_NUCLEOTIDE_COUNT {
            let axis_y = y_min + nucleotide_index as f32 * y_spacing;
            let phase = std::f32::consts::TAU * axis_y / pitch + std::f32::consts::PI;

            let strand2_x = helix_radius * phase.cos();
            let depth2 = phase.sin();
            let size2 = base_size * (1.0 + depth2 * 0.4);
            let alpha2 = 0.5 + 0.5 * (depth2 * 0.5 + 0.5);

            let strand2 = spawn_circle(
                world,
                Vec2::new(strand2_x, axis_y),
                size2,
                [1.0, 0.3, 0.3, alpha2],
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(strand2) {
                sprite.depth = 5.0 + depth2;
                sprite.blend_mode = SpriteBlendMode::Additive;
            }
            self.scene_entities.push(strand2);
        }

        for nucleotide_index in 0..HELIX_NUCLEOTIDE_COUNT {
            let axis_y = y_min + nucleotide_index as f32 * y_spacing;
            let phase = std::f32::consts::TAU * axis_y / pitch;
            let strand1_x = helix_radius * phase.cos();
            let strand2_x = helix_radius * (phase + std::f32::consts::PI).cos();

            let rung = spawn_line(
                world,
                Vec2::new(strand1_x, axis_y),
                Vec2::new(strand2_x, axis_y),
                1.5,
                [0.4, 0.4, 0.6, 0.4],
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(rung) {
                sprite.depth = 4.5;
            }
            self.scene_entities.push(rung);
        }

        for nucleotide_index in 0..HELIX_NUCLEOTIDE_COUNT - 1 {
            let axis_y1 = y_min + nucleotide_index as f32 * y_spacing;
            let axis_y2 = y_min + (nucleotide_index + 1) as f32 * y_spacing;
            let phase1 = std::f32::consts::TAU * axis_y1 / pitch;
            let phase2 = std::f32::consts::TAU * axis_y2 / pitch;

            let backbone1 = spawn_line(
                world,
                Vec2::new(helix_radius * phase1.cos(), axis_y1),
                Vec2::new(helix_radius * phase2.cos(), axis_y2),
                1.0,
                [0.15, 0.35, 0.8, 0.5],
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(backbone1) {
                sprite.depth = 5.5;
                sprite.blend_mode = SpriteBlendMode::Additive;
            }
            self.scene_entities.push(backbone1);
        }

        for nucleotide_index in 0..HELIX_NUCLEOTIDE_COUNT - 1 {
            let axis_y1 = y_min + nucleotide_index as f32 * y_spacing;
            let axis_y2 = y_min + (nucleotide_index + 1) as f32 * y_spacing;
            let phase1 = std::f32::consts::TAU * axis_y1 / pitch + std::f32::consts::PI;
            let phase2 = std::f32::consts::TAU * axis_y2 / pitch + std::f32::consts::PI;

            let backbone2 = spawn_line(
                world,
                Vec2::new(helix_radius * phase1.cos(), axis_y1),
                Vec2::new(helix_radius * phase2.cos(), axis_y2),
                1.0,
                [0.8, 0.2, 0.2, 0.5],
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(backbone2) {
                sprite.depth = 5.5;
                sprite.blend_mode = SpriteBlendMode::Additive;
            }
            self.scene_entities.push(backbone2);
        }
    }

    fn update_dna_helix(&mut self, world: &mut World) {
        let helix_radius = 90.0;
        let y_min = -250.0;
        let y_max = 250.0;
        let y_spacing = (y_max - y_min) / (HELIX_NUCLEOTIDE_COUNT - 1) as f32;
        let pitch = 180.0;
        let base_size = 8.0;
        let rotation = self.time * 0.8;

        let strand1_start = 0;
        let strand2_start = HELIX_NUCLEOTIDE_COUNT;
        let rung_start = HELIX_NUCLEOTIDE_COUNT * 2;
        let backbone1_start = HELIX_NUCLEOTIDE_COUNT * 3;
        let backbone2_start = HELIX_NUCLEOTIDE_COUNT * 3 + (HELIX_NUCLEOTIDE_COUNT - 1);

        for nucleotide_index in 0..HELIX_NUCLEOTIDE_COUNT {
            let axis_y = y_min + nucleotide_index as f32 * y_spacing;
            let phase = std::f32::consts::TAU * axis_y / pitch + rotation;

            let strand1_x = helix_radius * phase.cos();
            let depth1 = phase.sin();
            let size1 = base_size * (1.0 + depth1 * 0.4);
            let alpha1 = 0.5 + 0.5 * (depth1 * 0.5 + 0.5);
            let color_shift = (self.time * 0.3 + nucleotide_index as f32 * 0.05).sin() * 0.15;

            if let Some(sprite) = world
                .sprite2d
                .get_sprite_mut(self.scene_entities[strand1_start + nucleotide_index])
            {
                sprite.position = Vec2::new(strand1_x, axis_y);
                sprite.size = Vec2::new(size1 * 2.0, size1 * 2.0);
                sprite.color = [0.2 + color_shift, 0.5, 1.0, alpha1];
                sprite.depth = 5.0 + depth1;
            }

            let phase2 = phase + std::f32::consts::PI;
            let strand2_x = helix_radius * phase2.cos();
            let depth2 = phase2.sin();
            let size2 = base_size * (1.0 + depth2 * 0.4);
            let alpha2 = 0.5 + 0.5 * (depth2 * 0.5 + 0.5);

            if let Some(sprite) = world
                .sprite2d
                .get_sprite_mut(self.scene_entities[strand2_start + nucleotide_index])
            {
                sprite.position = Vec2::new(strand2_x, axis_y);
                sprite.size = Vec2::new(size2 * 2.0, size2 * 2.0);
                sprite.color = [1.0, 0.3 + color_shift, 0.3, alpha2];
                sprite.depth = 5.0 + depth2;
            }

            if let Some(sprite) = world
                .sprite2d
                .get_sprite_mut(self.scene_entities[rung_start + nucleotide_index])
            {
                sprite.position = Vec2::new((strand1_x + strand2_x) / 2.0, axis_y);
                let rung_length = (strand2_x - strand1_x).abs();
                sprite.size = Vec2::new(rung_length, 1.5);
                sprite.rotation = 0.0;
                sprite.color[3] = 0.3 + 0.2 * ((depth1 + depth2) / 2.0).abs();
            }
        }

        for nucleotide_index in 0..HELIX_NUCLEOTIDE_COUNT - 1 {
            let axis_y1 = y_min + nucleotide_index as f32 * y_spacing;
            let axis_y2 = y_min + (nucleotide_index + 1) as f32 * y_spacing;
            let phase1 = std::f32::consts::TAU * axis_y1 / pitch + rotation;
            let phase2 = std::f32::consts::TAU * axis_y2 / pitch + rotation;

            let x1 = helix_radius * phase1.cos();
            let x2 = helix_radius * phase2.cos();
            let mid_x = (x1 + x2) / 2.0;
            let mid_y = (axis_y1 + axis_y2) / 2.0;
            let dx = x2 - x1;
            let dy = axis_y2 - axis_y1;
            let length = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            if let Some(sprite) = world
                .sprite2d
                .get_sprite_mut(self.scene_entities[backbone1_start + nucleotide_index])
            {
                sprite.position = Vec2::new(mid_x, mid_y);
                sprite.size = Vec2::new(length, 1.0);
                sprite.rotation = angle;
            }

            let phase1_b = phase1 + std::f32::consts::PI;
            let phase2_b = phase2 + std::f32::consts::PI;
            let x1_b = helix_radius * phase1_b.cos();
            let x2_b = helix_radius * phase2_b.cos();
            let mid_x_b = (x1_b + x2_b) / 2.0;
            let mid_y_b = (axis_y1 + axis_y2) / 2.0;
            let dx_b = x2_b - x1_b;
            let dy_b = axis_y2 - axis_y1;
            let length_b = (dx_b * dx_b + dy_b * dy_b).sqrt();
            let angle_b = dy_b.atan2(dx_b);

            if let Some(sprite) = world
                .sprite2d
                .get_sprite_mut(self.scene_entities[backbone2_start + nucleotide_index])
            {
                sprite.position = Vec2::new(mid_x_b, mid_y_b);
                sprite.size = Vec2::new(length_b, 1.0);
                sprite.rotation = angle_b;
            }
        }
    }

    // ======================== Scene 10: Confetti ========================

    fn build_confetti(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.02, 0.02, 0.04, 1.0];

        let shapes = [
            SpriteShape::Rect,
            SpriteShape::Triangle,
            SpriteShape::Circle,
            SpriteShape::Capsule,
        ];

        for index in 0..CONFETTI_COUNT {
            let seed = index as u32;
            let shape = shapes[hash(seed + 100) as usize % shapes.len()];
            let hue = hash_f32(seed + 200);
            let (red, green, blue) = hue_to_rgb(hue);
            let saturation = hash_range(seed + 250, 0.7, 1.0);
            let color = [
                red * saturation + (1.0 - saturation),
                green * saturation + (1.0 - saturation),
                blue * saturation + (1.0 - saturation),
                0.9,
            ];

            let position_x = hash_range(seed + 300, -400.0, 400.0);
            let position_y = hash_range(seed + 400, -300.0, 400.0);
            let size_value = hash_range(seed + 500, 4.0, 14.0);
            let size = if shape == SpriteShape::Capsule {
                Vec2::new(size_value * 2.0, size_value * 0.6)
            } else {
                Vec2::new(size_value, size_value)
            };

            let entity = spawn_shape(world, shape, Vec2::new(position_x, position_y), size, color);
            if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.depth = 5.0;
                sprite.rotation = hash_range(seed + 600, 0.0, std::f32::consts::TAU);
            }
            self.scene_entities.push(entity);

            self.confetti.push(ConfettiState {
                velocity_x: hash_range(seed + 700, -80.0, 80.0),
                velocity_y: hash_range(seed + 800, 200.0, 500.0),
                angular_velocity: hash_range(seed + 900, -5.0, 5.0),
            });
        }
    }

    fn update_confetti(&mut self, world: &mut World, delta_time: f32) {
        let gravity = 350.0;

        for (index, confetti) in self.confetti.iter_mut().enumerate() {
            confetti.velocity_y -= gravity * delta_time;
            confetti.velocity_x *= 1.0 - 0.3 * delta_time;

            if let Some(sprite) = world.sprite2d.get_sprite_mut(self.scene_entities[index]) {
                sprite.position.x += confetti.velocity_x * delta_time;
                sprite.position.y += confetti.velocity_y * delta_time;
                sprite.rotation += confetti.angular_velocity * delta_time;

                if sprite.position.y < -300.0 {
                    let seed = hash((self.time * 1000.0) as u32 + index as u32);
                    sprite.position.x = hash_range(seed, -200.0, 200.0);
                    sprite.position.y = -280.0;
                    confetti.velocity_x = hash_range(seed + 1, -120.0, 120.0);
                    confetti.velocity_y = hash_range(seed + 2, 350.0, 650.0);
                    confetti.angular_velocity = hash_range(seed + 3, -6.0, 6.0);

                    let hue = hash_f32(seed + 4);
                    let (red, green, blue) = hue_to_rgb(hue);
                    sprite.color = [red, green, blue, 0.9];
                }
            }
        }
    }

    // ======================== Scene 11: Solar System ========================

    fn build_solar_system(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.005, 0.005, 0.015, 1.0];

        let sun = spawn_shape(
            world,
            SpriteShape::SoftCircle,
            Vec2::zeros(),
            Vec2::new(50.0, 50.0),
            [1.0, 0.85, 0.3, 0.9],
        );
        if let Some(sprite) = world.sprite2d.get_sprite_mut(sun) {
            sprite.depth = 3.0;
            sprite.blend_mode = SpriteBlendMode::Additive;
        }
        self.scene_entities.push(sun);

        let sun_glow = spawn_shape(
            world,
            SpriteShape::SoftCircle,
            Vec2::zeros(),
            Vec2::new(90.0, 90.0),
            [1.0, 0.7, 0.2, 0.15],
        );
        if let Some(sprite) = world.sprite2d.get_sprite_mut(sun_glow) {
            sprite.depth = 2.0;
            sprite.blend_mode = SpriteBlendMode::Additive;
        }
        self.scene_entities.push(sun_glow);

        let planet_configs: [(f32, f32, f32, [f32; 4]); PLANET_COUNT] = [
            (55.0, 1.2, 5.0, [0.6, 0.6, 0.7, 1.0]),
            (85.0, 0.8, 7.0, [0.9, 0.6, 0.2, 1.0]),
            (120.0, 0.55, 9.0, [0.2, 0.5, 0.9, 1.0]),
            (165.0, 0.38, 8.0, [0.9, 0.35, 0.2, 1.0]),
            (215.0, 0.25, 16.0, [0.85, 0.75, 0.5, 1.0]),
            (275.0, 0.16, 13.0, [0.5, 0.7, 0.4, 1.0]),
            (340.0, 0.10, 10.0, [0.45, 0.55, 0.85, 1.0]),
        ];

        for (planet_index, &(orbit_radius, _, _, _)) in planet_configs.iter().enumerate() {
            let ring = spawn_ring(world, Vec2::zeros(), orbit_radius, [0.15, 0.15, 0.2, 0.15]);
            if let Some(sprite) = world.sprite2d.get_sprite_mut(ring) {
                sprite.depth = 1.0;
            }
            self.scene_entities.push(ring);

            let angle = hash_range(planet_index as u32 + 7000, 0.0, std::f32::consts::TAU);
            let position = Vec2::new(orbit_radius * angle.cos(), orbit_radius * angle.sin());

            let planet = spawn_circle(
                world,
                position,
                planet_configs[planet_index].2 / 2.0,
                planet_configs[planet_index].3,
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(planet) {
                sprite.depth = 6.0;
            }
            self.scene_entities.push(planet);

            self.planets.push(PlanetState {
                orbit_radius,
                angle,
                speed: planet_configs[planet_index].1,
            });
        }

        let moon_configs = [
            (3, 16.0, 2.5, 3.0),
            (4, 22.0, 1.8, 3.5),
            (4, 28.0, 1.2, 2.5),
            (5, 20.0, 2.0, 2.0),
        ];

        for (moon_index, &(planet_index, moon_orbit, moon_speed, moon_size)) in
            moon_configs.iter().enumerate()
        {
            let moon_angle = hash_range(moon_index as u32 + 8000, 0.0, std::f32::consts::TAU);
            let moon = spawn_circle(world, Vec2::zeros(), moon_size, [0.7, 0.7, 0.75, 0.9]);
            if let Some(sprite) = world.sprite2d.get_sprite_mut(moon) {
                sprite.depth = 7.0;
            }
            self.scene_entities.push(moon);

            self.moons.push(MoonState {
                orbit_radius: moon_orbit,
                angle: moon_angle,
                speed: moon_speed,
                planet_index,
            });
        }
    }

    fn update_solar_system(&mut self, world: &mut World, delta_time: f32) {
        let sun_pulse = 0.9 + 0.1 * (self.time * 0.8).sin();
        if let Some(sprite) = world.sprite2d.get_sprite_mut(self.scene_entities[0]) {
            sprite.size = Vec2::new(50.0 * sun_pulse, 50.0 * sun_pulse);
        }
        let glow_pulse = 0.9 + 0.1 * (self.time * 0.6 + 1.0).sin();
        if let Some(sprite) = world.sprite2d.get_sprite_mut(self.scene_entities[1]) {
            sprite.size = Vec2::new(90.0 * glow_pulse, 90.0 * glow_pulse);
        }

        let orbit_ring_start = 2;
        let planet_stride = 2;

        for (planet_index, planet) in self.planets.iter_mut().enumerate() {
            planet.angle += planet.speed * delta_time;

            let position = Vec2::new(
                planet.orbit_radius * planet.angle.cos(),
                planet.orbit_radius * planet.angle.sin(),
            );

            let entity_index = orbit_ring_start + planet_index * planet_stride + 1;
            if let Some(sprite) = world
                .sprite2d
                .get_sprite_mut(self.scene_entities[entity_index])
            {
                sprite.position = position;
            }
        }

        let moon_entity_start = orbit_ring_start + PLANET_COUNT * planet_stride;
        for (moon_index, moon) in self.moons.iter_mut().enumerate() {
            moon.angle += moon.speed * delta_time;

            let parent = &self.planets[moon.planet_index];
            let parent_position = Vec2::new(
                parent.orbit_radius * parent.angle.cos(),
                parent.orbit_radius * parent.angle.sin(),
            );
            let moon_position = parent_position
                + Vec2::new(
                    moon.orbit_radius * moon.angle.cos(),
                    moon.orbit_radius * moon.angle.sin(),
                );

            if let Some(sprite) = world
                .sprite2d
                .get_sprite_mut(self.scene_entities[moon_entity_start + moon_index])
            {
                sprite.position = moon_position;
            }
        }
    }
    // ======================== Scene 16: Boolean Dance ========================

    fn build_boolean_dance(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.04, 0.02, 0.06, 1.0];

        let texture_size = 128_u32;
        let circle_a = generate_circle_texture_with_aa(texture_size, 1.0);
        let circle_b = generate_circle_texture_with_aa(texture_size, 1.0);

        let shift = (texture_size as f32 * 0.25) as usize;
        let mut shifted_b = vec![0u8; (texture_size * texture_size * 4) as usize];
        for row in 0..texture_size as usize {
            for col in shift..texture_size as usize {
                let source_col = col - shift;
                let destination_offset = (row * texture_size as usize + col) * 4;
                let source_offset = (row * texture_size as usize + source_col) * 4;
                shifted_b[destination_offset..destination_offset + 4]
                    .copy_from_slice(&circle_b[source_offset..source_offset + 4]);
            }
        }

        let union_data = boolean_union(&circle_a, &shifted_b, texture_size, texture_size);
        let subtract_data = boolean_subtract(&circle_a, &shifted_b, texture_size, texture_size);
        let intersect_data = boolean_intersect(&circle_a, &shifted_b, texture_size, texture_size);

        let slots = [
            allocate_sprite_slot(world),
            allocate_sprite_slot(world),
            allocate_sprite_slot(world),
        ];
        let textures = [union_data, subtract_data, intersect_data];

        for (index, slot) in slots.iter().enumerate() {
            world.queue_command(WorldCommand::UploadSpriteTexture {
                slot: *slot,
                rgba_data: textures[index].clone(),
                width: texture_size,
                height: texture_size,
            });
        }

        let uv_max_coord = texture_size as f32 / 512.0;
        let half_texel = 0.5 / 512.0;
        let uv_min = Vec2::new(half_texel, half_texel);
        let uv_max = Vec2::new(uv_max_coord - half_texel, uv_max_coord - half_texel);

        let colors: [[f32; 4]; 3] = [
            [0.3, 0.6, 1.0, 0.9],
            [1.0, 0.3, 0.4, 0.9],
            [0.3, 1.0, 0.5, 0.9],
        ];

        self.boolean_entity_start = self.scene_entities.len();

        for group_index in 0..3 {
            let orbit_radius = 100.0 + group_index as f32 * 50.0;
            let orbit_speed = 0.3 + group_index as f32 * 0.15;
            let angle = group_index as f32 * std::f32::consts::TAU / 3.0;

            for (type_index, &slot) in slots.iter().enumerate() {
                let sub_angle = angle + type_index as f32 * 0.3;
                let sub_radius = 30.0;
                let position_x = orbit_radius * sub_angle.cos()
                    + sub_radius
                        * (sub_angle + type_index as f32 * std::f32::consts::TAU / 3.0).cos();
                let position_y = orbit_radius * sub_angle.sin()
                    + sub_radius
                        * (sub_angle + type_index as f32 * std::f32::consts::TAU / 3.0).sin();

                let size = 40.0 + type_index as f32 * 8.0;
                let entity = spawn_sprite(
                    world,
                    Vec2::new(position_x, position_y),
                    Vec2::new(size, size),
                );
                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.texture_index = slot;
                    sprite.texture_index2 = slot;
                    sprite.uv_min = uv_min;
                    sprite.uv_max = uv_max;
                    sprite.color = colors[type_index];
                    sprite.depth = 5.0;
                }
                self.scene_entities.push(entity);
            }

            self.boolean_shapes.push(BooleanShapeState {
                orbit_radius,
                orbit_speed,
                angle,
            });
        }

        let label = spawn_ui_text_with_properties(
            world,
            "Union / Subtract / Intersect",
            Vec2::new(-130.0, 240.0),
            TextProperties {
                font_size: 13.0,
                color: Vec4::new(0.6, 0.6, 0.7, 0.8),
                ..Default::default()
            },
        );
        self.scene_entities.push(label);
    }

    fn update_boolean_dance(&mut self, world: &mut World, delta_time: f32) {
        for (group_index, shape) in self.boolean_shapes.iter_mut().enumerate() {
            shape.angle += shape.orbit_speed * delta_time;

            for type_index in 0..3 {
                let entity_index = self.boolean_entity_start + group_index * 3 + type_index;
                let sub_angle = shape.angle + type_index as f32 * std::f32::consts::TAU / 3.0;
                let position_x = shape.orbit_radius * shape.angle.cos() + 30.0 * sub_angle.cos();
                let position_y = shape.orbit_radius * shape.angle.sin() + 30.0 * sub_angle.sin();

                let pulse = 0.85 + 0.15 * (self.time * 1.5 + type_index as f32 * 0.8).sin();
                let base_size = 40.0 + type_index as f32 * 8.0;

                if let Some(sprite) = world
                    .sprite2d
                    .get_sprite_mut(self.scene_entities[entity_index])
                {
                    sprite.position = Vec2::new(position_x, position_y);
                    sprite.rotation = shape.angle * 0.5;
                    sprite.size = Vec2::new(base_size * pulse, base_size * pulse);

                    let hue =
                        (self.time * 0.08 + group_index as f32 * 0.33 + type_index as f32 * 0.1)
                            .fract();
                    let (red, green, blue) = hue_to_rgb(hue);
                    sprite.color = [red, green, blue, 0.9];
                }
            }
        }
    }

    // ======================== Scene 17: Stencil Windows ========================

    fn build_stencil_windows(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.02, 0.02, 0.04, 1.0];

        let stripe_count = 40;
        self.stencil_stripe_count = stripe_count;
        let stripe_width = 24.0;
        let total_width = stripe_count as f32 * stripe_width;
        let start_x = -total_width / 2.0 + stripe_width / 2.0;

        for index in 0..stripe_count {
            let hue = index as f32 / stripe_count as f32;
            let (red, green, blue) = hue_to_rgb(hue);
            let entity = spawn_rect(
                world,
                Vec2::new(start_x + index as f32 * stripe_width, 0.0),
                Vec2::new(stripe_width - 1.0, 540.0),
                [red, green, blue, 1.0],
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.depth = 4.0;
                sprite.stencil_mode = SpriteStencilMode::Test;
            }
            self.scene_entities.push(entity);
        }

        let mask_count = 5;
        for index in 0..mask_count {
            let orbit_radius = 60.0 + index as f32 * 40.0;
            let orbit_speed = 0.4 - index as f32 * 0.06;
            let angle = index as f32 * std::f32::consts::TAU / mask_count as f32;

            let mask_size = 80.0 + index as f32 * 20.0;
            let mask_entity =
                spawn_circle(world, Vec2::zeros(), mask_size / 2.0, [1.0, 1.0, 1.0, 1.0]);
            if let Some(sprite) = world.sprite2d.get_sprite_mut(mask_entity) {
                sprite.depth = 3.0;
                sprite.stencil_mode = SpriteStencilMode::Write;
            }
            self.scene_entities.push(mask_entity);

            let border = spawn_ring(world, Vec2::zeros(), mask_size / 2.0, [0.5, 0.5, 0.6, 0.4]);
            if let Some(sprite) = world.sprite2d.get_sprite_mut(border) {
                sprite.depth = 6.0;
            }
            self.scene_entities.push(border);

            self.stencil_masks.push(StencilMaskState {
                orbit_radius,
                orbit_speed,
                angle,
            });
        }

        let label = spawn_ui_text_with_properties(
            world,
            "GPU Stencil Masking",
            Vec2::new(-100.0, 240.0),
            TextProperties {
                font_size: 13.0,
                color: Vec4::new(0.6, 0.6, 0.7, 0.8),
                ..Default::default()
            },
        );
        self.scene_entities.push(label);
    }

    fn update_stencil_windows(&mut self, world: &mut World, delta_time: f32) {
        for index in 0..self.stencil_stripe_count {
            let hue = (index as f32 / self.stencil_stripe_count as f32 + self.time * 0.05).fract();
            let (red, green, blue) = hue_to_rgb(hue);
            if let Some(sprite) = world.sprite2d.get_sprite_mut(self.scene_entities[index]) {
                sprite.color = [red, green, blue, 1.0];
            }
        }

        for (index, mask) in self.stencil_masks.iter_mut().enumerate() {
            mask.angle += mask.orbit_speed * delta_time;

            let position = Vec2::new(
                mask.orbit_radius * mask.angle.cos(),
                mask.orbit_radius * mask.angle.sin(),
            );

            let mask_entity_index = self.stencil_stripe_count + index * 2;
            let border_entity_index = mask_entity_index + 1;

            if let Some(sprite) = world
                .sprite2d
                .get_sprite_mut(self.scene_entities[mask_entity_index])
            {
                sprite.position = position;
            }
            if let Some(sprite) = world
                .sprite2d
                .get_sprite_mut(self.scene_entities[border_entity_index])
            {
                sprite.position = position;
            }
        }
    }

    // ======================== Scene 18: Shadow Theater ========================

    fn build_shadow_theater(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.06, 0.06, 0.08, 1.0];

        let texture_size = 128_u32;
        let source_data = generate_circle_texture_with_aa(texture_size, 1.0);
        let blurred_data = generate_blurred_texture(&source_data, texture_size, texture_size, 10);

        let source_slot = allocate_sprite_slot(world);
        let shadow_slot = allocate_sprite_slot(world);

        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: source_slot,
            rgba_data: source_data,
            width: texture_size,
            height: texture_size,
        });
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: shadow_slot,
            rgba_data: blurred_data,
            width: texture_size,
            height: texture_size,
        });

        let uv_max_coord = texture_size as f32 / 512.0;
        let half_texel = 0.5 / 512.0;
        let uv_min = Vec2::new(half_texel, half_texel);
        let uv_max = Vec2::new(uv_max_coord - half_texel, uv_max_coord - half_texel);

        let item_count = 12;
        self.shadow_entity_start = self.scene_entities.len();

        for index in 0..item_count {
            let seed = index as u32;
            let position_x = hash_range(seed + 100, -350.0, 350.0);
            let position_y = hash_range(seed + 200, -180.0, 180.0);
            let size = hash_range(seed + 300, 30.0, 70.0);
            let hue = hash_f32(seed + 400);
            let (red, green, blue) = hue_to_rgb(hue);
            let velocity_x = hash_range(seed + 500, -80.0, 80.0);
            let velocity_y = hash_range(seed + 600, -60.0, 60.0);

            let shadow_entity = spawn_sprite(
                world,
                Vec2::new(position_x + 6.0, position_y - 6.0),
                Vec2::new(size * 1.3, size * 1.3),
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(shadow_entity) {
                sprite.texture_index = shadow_slot;
                sprite.texture_index2 = shadow_slot;
                sprite.uv_min = uv_min;
                sprite.uv_max = uv_max;
                sprite.color = [0.0, 0.0, 0.0, 0.5];
                sprite.depth = 3.0;
            }
            self.scene_entities.push(shadow_entity);

            let shape_entity = spawn_sprite(
                world,
                Vec2::new(position_x, position_y),
                Vec2::new(size, size),
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(shape_entity) {
                sprite.texture_index = source_slot;
                sprite.texture_index2 = source_slot;
                sprite.uv_min = uv_min;
                sprite.uv_max = uv_max;
                sprite.color = [red, green, blue, 1.0];
                sprite.depth = 5.0;
            }
            self.scene_entities.push(shape_entity);

            self.shadow_items.push(ShadowItemState {
                velocity: Vec2::new(velocity_x, velocity_y),
                size,
            });
        }

        let glow_source = generate_circle_texture_with_aa(texture_size, 1.0);
        let glow_blurred = generate_blurred_texture(&glow_source, texture_size, texture_size, 14);
        let glow_slot = allocate_sprite_slot(world);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: glow_slot,
            rgba_data: glow_blurred,
            width: texture_size,
            height: texture_size,
        });

        for index in 0..4 {
            let glow_x = hash_range(index as u32 + 700, -200.0, 200.0);
            let glow_y = hash_range(index as u32 + 800, -100.0, 100.0);
            let glow_entity =
                spawn_sprite(world, Vec2::new(glow_x, glow_y), Vec2::new(120.0, 120.0));
            if let Some(sprite) = world.sprite2d.get_sprite_mut(glow_entity) {
                sprite.texture_index = glow_slot;
                sprite.texture_index2 = glow_slot;
                sprite.uv_min = uv_min;
                sprite.uv_max = uv_max;
                let hue = hash_f32(index as u32 + 900);
                let (red, green, blue) = hue_to_rgb(hue);
                sprite.color = [red, green, blue, 0.3];
                sprite.depth = 2.0;
                sprite.blend_mode = SpriteBlendMode::Additive;
            }
            self.scene_entities.push(glow_entity);
        }

        let label = spawn_ui_text_with_properties(
            world,
            "CPU Blur Shadows + Glow",
            Vec2::new(-120.0, 240.0),
            TextProperties {
                font_size: 13.0,
                color: Vec4::new(0.6, 0.6, 0.7, 0.8),
                ..Default::default()
            },
        );
        self.scene_entities.push(label);
    }

    fn update_shadow_theater(&mut self, world: &mut World, delta_time: f32) {
        for (index, item) in self.shadow_items.iter_mut().enumerate() {
            let shadow_index = self.shadow_entity_start + index * 2;
            let shape_index = shadow_index + 1;

            let mut shape_position = Vec2::zeros();
            let mut shape_size = Vec2::zeros();

            if let Some(sprite) = world
                .sprite2d
                .get_sprite_mut(self.scene_entities[shape_index])
            {
                sprite.position += item
                    .velocity
                    .component_mul(&Vec2::new(delta_time, delta_time));

                if sprite.position.x > 400.0 {
                    item.velocity.x = -item.velocity.x.abs();
                } else if sprite.position.x < -400.0 {
                    item.velocity.x = item.velocity.x.abs();
                }
                if sprite.position.y > 220.0 {
                    item.velocity.y = -item.velocity.y.abs();
                } else if sprite.position.y < -220.0 {
                    item.velocity.y = item.velocity.y.abs();
                }

                let pulse = 0.9 + 0.1 * (self.time * 2.0 + index as f32 * 0.5).sin();
                sprite.size = Vec2::new(item.size * pulse, item.size * pulse);

                let hue = (hash_f32(index as u32 + 400) + self.time * 0.03).fract();
                let (red, green, blue) = hue_to_rgb(hue);
                sprite.color = [red, green, blue, 1.0];

                shape_position = sprite.position;
                shape_size = sprite.size;
            }

            if let Some(shadow) = world
                .sprite2d
                .get_sprite_mut(self.scene_entities[shadow_index])
            {
                shadow.position = shape_position + Vec2::new(6.0, -6.0);
                shadow.size = shape_size * 1.3;
            }
        }

        let glow_start = self.shadow_entity_start + self.shadow_items.len() * 2;
        for glow_index in 0..4 {
            let entity_index = glow_start + glow_index;
            if entity_index < self.scene_entities.len()
                && let Some(sprite) = world
                    .sprite2d
                    .get_sprite_mut(self.scene_entities[entity_index])
            {
                let pulse = 0.6 + 0.4 * (self.time * 0.8 + glow_index as f32 * 1.2).sin();
                sprite.color[3] = 0.3 * pulse;
                let size = 120.0 + 30.0 * (self.time * 0.5 + glow_index as f32).sin();
                sprite.size = Vec2::new(size, size);
            }
        }
    }

    // ======================== Scene 19: Path Stars ========================

    fn build_path_stars(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.02, 0.01, 0.04, 1.0];

        let star_count = 20;
        self.path_star_count = star_count;

        for index in 0..star_count {
            let seed = index as u32;
            let position_x = hash_range(seed + 100, -380.0, 380.0);
            let position_y = hash_range(seed + 200, -200.0, 200.0);
            let point_count = 3 + (hash(seed + 300) % 4) as usize;
            let size = hash_range(seed + 400, 20.0, 60.0);
            let hue = hash_f32(seed + 500);
            let (red, green, blue) = hue_to_rgb(hue);

            let points: Vec<Vec2> = (0..point_count * 2)
                .map(|vertex_index| {
                    let angle = vertex_index as f32 * std::f32::consts::TAU
                        / (point_count * 2) as f32
                        - std::f32::consts::FRAC_PI_2;
                    let radius = if vertex_index % 2 == 0 { 0.48 } else { 0.2 };
                    Vec2::new(0.5 + radius * angle.cos(), 0.5 + radius * angle.sin())
                })
                .collect();

            let entity = spawn_filled_path(
                world,
                &points,
                [red, green, blue, 0.9],
                Vec2::new(position_x, position_y),
                Vec2::new(size, size),
                5.0,
            );
            self.scene_entities.push(entity);
        }

        for index in 0..8 {
            let seed = (index + star_count) as u32;
            let position_x = hash_range(seed + 100, -300.0, 300.0);
            let position_y = hash_range(seed + 200, -150.0, 150.0);
            let sides = 5 + (hash(seed + 300) % 4) as usize;
            let size = hash_range(seed + 400, 25.0, 50.0);
            let hue = hash_f32(seed + 500);
            let (red, green, blue) = hue_to_rgb(hue);

            let points: Vec<Vec2> = (0..sides)
                .map(|vertex_index| {
                    let angle = vertex_index as f32 * std::f32::consts::TAU / sides as f32;
                    Vec2::new(0.5 + 0.45 * angle.cos(), 0.5 + 0.45 * angle.sin())
                })
                .collect();

            let entity = spawn_filled_path(
                world,
                &points,
                [red, green, blue, 0.7],
                Vec2::new(position_x, position_y),
                Vec2::new(size, size),
                4.0,
            );
            self.scene_entities.push(entity);
        }

        let label = spawn_ui_text_with_properties(
            world,
            "Filled Path Shapes",
            Vec2::new(-95.0, 240.0),
            TextProperties {
                font_size: 13.0,
                color: Vec4::new(0.6, 0.6, 0.7, 0.8),
                ..Default::default()
            },
        );
        self.scene_entities.push(label);
    }

    fn update_path_stars(&mut self, world: &mut World) {
        let total_entities = self.path_star_count + 8;
        for index in 0..total_entities {
            if let Some(sprite) = world.sprite2d.get_sprite_mut(self.scene_entities[index]) {
                let speed = if index < self.path_star_count {
                    0.3 + (index as f32 * 0.1)
                } else {
                    0.2 + ((index - self.path_star_count) as f32 * 0.08)
                };
                sprite.rotation = self.time * speed;

                let pulse = 0.9 + 0.1 * (self.time * 1.5 + index as f32 * 0.4).sin();
                sprite.scale = Vec2::new(pulse, pulse);

                let hue = (hash_f32(index as u32 + 500) + self.time * 0.04).fract();
                let (red, green, blue) = hue_to_rgb(hue);
                let alpha = if index < self.path_star_count {
                    0.9
                } else {
                    0.7
                };
                sprite.color = [red, green, blue, alpha];
            }
        }
    }
}

// ======================== Post-Process Pass ========================

struct PostProcessPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    cached_bind_group: Option<wgpu::BindGroup>,
    effect_mode: Arc<AtomicU32>,
}

impl PostProcessPass {
    fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        effect_mode: Arc<AtomicU32>,
    ) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Post Process Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(POST_PROCESS_SHADER)),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post Process Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post Process Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Post Process Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Post Process Uniforms"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Post Process Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            uniform_buffer,
            cached_bind_group: None,
            effect_mode,
        }
    }
}

impl PassNode<World> for PostProcessPass {
    fn name(&self) -> &str {
        "post_process_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["input"]
    }

    fn writes(&self) -> Vec<&str> {
        vec!["output"]
    }

    fn invalidate_bind_groups(&mut self) {
        self.cached_bind_group = None;
    }

    fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, world: &World) {
        let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
        let mode = self.effect_mode.load(Ordering::Relaxed);
        let (width, height) = world
            .resources
            .window
            .cached_viewport_size
            .unwrap_or((1920, 1080));

        let mut data = [0u8; 16];
        data[0..4].copy_from_slice(&time.to_le_bytes());
        data[4..8].copy_from_slice(&mode.to_le_bytes());
        data[8..12].copy_from_slice(&(width as f32).to_le_bytes());
        data[12..16].copy_from_slice(&(height as f32).to_le_bytes());
        queue.write_buffer(&self.uniform_buffer, 0, &data);
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, World>,
    ) -> Result<
        Vec<nightshade::render::wgpu::rendergraph::SubGraphRunCommand<'r>>,
        nightshade::render::wgpu::rendergraph::RenderGraphError,
    > {
        if self.cached_bind_group.is_none() {
            let input_view = context.get_texture_view("input")?;

            self.cached_bind_group = Some(context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Post Process Bind Group"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(input_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.uniform_buffer.as_entire_binding(),
                        },
                    ],
                },
            ));
        }

        let (color_view, color_load_op, color_store_op) = context.get_color_attachment("output")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Post Process Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: color_load_op,
                        store: color_store_op,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.cached_bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..1);
        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}

// ======================== State Implementation ========================

impl State for GfxShowcase {
    fn title(&self) -> &str {
        "2D Effects Showcase"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;

        let camera = spawn_ortho_camera(world, Vec2::zeros());
        self.camera_entity = Some(camera);

        if let Some(camera_data) = world.core.get_camera_mut(camera)
            && let Projection::Orthographic(ref mut ortho) = camera_data.projection
        {
            ortho.x_mag = 480.0;
            ortho.y_mag = 270.0;
        }

        self.build_current_scene(world);
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let pass = PostProcessPass::new(device, surface_format, self.effect_mode.clone());

        graph
            .pass(Box::new(pass))
            .read("input", resources.scene_color)
            .write("output", resources.compute_output);

        let fxaa_output = graph
            .add_color_texture("fxaa_output")
            .format(surface_format)
            .size(
                resources.surface_width.max(1),
                resources.surface_height.max(1),
            )
            .transient();

        let fxaa_pass = passes::FxaaPass::new(device, surface_format);
        graph
            .pass(Box::new(fxaa_pass))
            .read("input", resources.compute_output)
            .write("output", fxaa_output);

        let swapchain_blit_pass =
            passes::BlitPass::new(device, surface_format).with_name("default_swapchain_blit");
        graph
            .pass(Box::new(swapchain_blit_pass))
            .read("input", fxaa_output)
            .write("output", resources.swapchain);
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;
        self.time += delta_time;

        escape_key_exit_system(world);

        let pressed_right = world
            .resources
            .input
            .keyboard
            .just_pressed(KeyCode::ArrowRight);
        let pressed_left = world
            .resources
            .input
            .keyboard
            .just_pressed(KeyCode::ArrowLeft);
        let pressed_digits: [bool; 8] = [
            world.resources.input.keyboard.just_pressed(KeyCode::Digit1),
            world.resources.input.keyboard.just_pressed(KeyCode::Digit2),
            world.resources.input.keyboard.just_pressed(KeyCode::Digit3),
            world.resources.input.keyboard.just_pressed(KeyCode::Digit4),
            world.resources.input.keyboard.just_pressed(KeyCode::Digit5),
            world.resources.input.keyboard.just_pressed(KeyCode::Digit6),
            world.resources.input.keyboard.just_pressed(KeyCode::Digit7),
            world.resources.input.keyboard.just_pressed(KeyCode::Digit8),
        ];

        if pressed_right {
            self.switch_scene(world, (self.current_scene + 1) % SCENE_COUNT);
        } else if pressed_left {
            let scene = if self.current_scene == 0 {
                SCENE_COUNT - 1
            } else {
                self.current_scene - 1
            };
            self.switch_scene(world, scene);
        } else {
            for (digit, &pressed) in pressed_digits.iter().enumerate() {
                if pressed && digit != self.current_scene {
                    self.switch_scene(world, digit);
                    break;
                }
            }
        }

        self.update_current_scene(world, delta_time);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("2D Effects Showcase")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                let fps = world.resources.window.timing.frames_per_second;
                ui.label(format!("FPS: {fps:.0}"));
                ui.separator();

                let shader_indicator = match self.current_scene {
                    12..=15 => " [Shader]",
                    _ => "",
                };
                ui.label(format!(
                    "Scene {}/{}: {}{}",
                    self.current_scene + 1,
                    SCENE_COUNT,
                    SCENE_NAMES[self.current_scene],
                    shader_indicator,
                ));

                ui.separator();
                ui.label("Left/Right arrows to switch scenes");
                ui.label("Keys 1-8 for first 8 scenes");
                ui.separator();

                egui::CollapsingHeader::new("All Scenes").show(ui, |ui| {
                    for (index, name) in SCENE_NAMES.iter().enumerate() {
                        let prefix = if index == self.current_scene {
                            ">"
                        } else {
                            " "
                        };
                        let shader_tag = if (12..=15).contains(&index) {
                            " [Shader]"
                        } else {
                            ""
                        };
                        ui.label(format!("{prefix} {}: {name}{shader_tag}", index + 1));
                    }
                });
            });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(GfxShowcase::default())
}
