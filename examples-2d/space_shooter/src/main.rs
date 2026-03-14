use image::GenericImageView;
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::transform::commands::mark_local_transform_dirty;
use nightshade::prelude::*;

const PLAYER_SPEED: f32 = 400.0;
const PLAYER_FOCUSED_SPEED: f32 = 160.0;
const PLAYER_FIRE_COOLDOWN: f32 = 0.10;
const PLAYER_BULLET_SPEED: f32 = 900.0;
const PLAYER_BULLET_LIFETIME: f32 = 1.5;
const PLAYER_HITBOX_RADIUS: f32 = 14.0;
const GRAZE_RADIUS: f32 = 40.0;
const ENEMY_BULLET_RADIUS: f32 = 6.0;
const INVINCIBILITY_DURATION: f32 = 2.5;
const BOMB_DURATION: f32 = 0.8;
const BOMB_INVINCIBILITY: f32 = 3.0;
const POWERUP_LIFETIME: f32 = 8.0;
const POWERUP_DROP_CHANCE: f32 = 0.25;
const WAVE_INTRO_DURATION: f32 = 2.5;
const WAVE_CLEAR_DURATION: f32 = 2.0;
const BOSS_INTRO_DURATION: f32 = 3.0;
const METEOR_COUNT: usize = 8;
const STAR_COUNT: usize = 40;
const BG_TILE_COUNT: i32 = 5;
const PLAYER_EDGE_MARGIN: f32 = 40.0;
const EXTRA_LIFE_SCORE_INTERVAL: u64 = 500_000;
const SCREEN_SHAKE_DECAY: f32 = 10.0;
const GRAZE_FLASH_DURATION: f32 = 0.6;
const ENEMY_BULLET_BASE_SPEED: f32 = 280.0;
const DESPAWN_MARGIN: f32 = 100.0;
const CAMERA_Z: f32 = 1303.0;

const PLAYER_MAX_TILT: f32 = 0.4;
const PLAYER_TILT_SPEED: f32 = 8.0;

const BARREL_ROLL_DURATION: f32 = 0.4;
const BARREL_ROLL_INVINCIBILITY: f32 = 0.5;
const DOUBLE_TAP_WINDOW: f32 = 0.3;
const GAMEPAD_DEADZONE: f32 = 0.15;

const SLOT_BG: u32 = 0;
const SLOT_PLAYER: u32 = 1;
const SLOT_LASER_BLUE: u32 = 2;
const SLOT_LASER_RED: u32 = 3;
const SLOT_ENEMY_BLACK: u32 = 4;
const SLOT_ENEMY_BLUE: u32 = 5;
const SLOT_ENEMY_RED: u32 = 6;
const SLOT_UFO_BLUE: u32 = 7;
const SLOT_UFO_RED: u32 = 8;
const SLOT_METEOR: u32 = 9;
const SLOT_STAR: u32 = 10;
const SLOT_FIRE0: u32 = 11;
const SLOT_FIRE1: u32 = 12;
const SLOT_FIRE2: u32 = 13;
const SLOT_FIRE3: u32 = 14;
const SLOT_POWERUP_POWER: u32 = 15;
const SLOT_POWERUP_BOMB: u32 = 16;

const LAYER_BG: f32 = 0.0;
const LAYER_STARS: f32 = 1.0;
const LAYER_METEORS: f32 = 2.0;
const LAYER_ENEMY_BULLETS: f32 = 3.0;
const LAYER_POWERUPS: f32 = 4.0;
const LAYER_ENEMIES: f32 = 5.0;
const LAYER_PLAYER_BULLETS: f32 = 6.0;
const LAYER_PLAYER: f32 = 7.0;
const LAYER_HITBOX: f32 = 7.5;
const LAYER_EXPLOSIONS: f32 = 8.0;

struct TextureEntry {
    slot: u32,
    bytes: &'static [u8],
}

fn load_textures(world: &mut World) -> (Vec<(f32, f32)>, Vec<nalgebra_glm::Vec2>) {
    let entries = [
        TextureEntry {
            slot: SLOT_BG,
            bytes: include_bytes!("../assets/bg_blue.png"),
        },
        TextureEntry {
            slot: SLOT_PLAYER,
            bytes: include_bytes!("../assets/playerShip1_blue.png"),
        },
        TextureEntry {
            slot: SLOT_LASER_BLUE,
            bytes: include_bytes!("../assets/laserBlue01.png"),
        },
        TextureEntry {
            slot: SLOT_LASER_RED,
            bytes: include_bytes!("../assets/laserRed01.png"),
        },
        TextureEntry {
            slot: SLOT_ENEMY_BLACK,
            bytes: include_bytes!("../assets/enemyBlack1.png"),
        },
        TextureEntry {
            slot: SLOT_ENEMY_BLUE,
            bytes: include_bytes!("../assets/enemyBlue2.png"),
        },
        TextureEntry {
            slot: SLOT_ENEMY_RED,
            bytes: include_bytes!("../assets/enemyRed3.png"),
        },
        TextureEntry {
            slot: SLOT_UFO_BLUE,
            bytes: include_bytes!("../assets/ufo_blue.png"),
        },
        TextureEntry {
            slot: SLOT_UFO_RED,
            bytes: include_bytes!("../assets/ufo_red.png"),
        },
        TextureEntry {
            slot: SLOT_METEOR,
            bytes: include_bytes!("../assets/meteor_brown1.png"),
        },
        TextureEntry {
            slot: SLOT_STAR,
            bytes: include_bytes!("../assets/star_large.png"),
        },
        TextureEntry {
            slot: SLOT_FIRE0,
            bytes: include_bytes!("../assets/fire00.png"),
        },
        TextureEntry {
            slot: SLOT_FIRE1,
            bytes: include_bytes!("../assets/fire01.png"),
        },
        TextureEntry {
            slot: SLOT_FIRE2,
            bytes: include_bytes!("../assets/fire02.png"),
        },
        TextureEntry {
            slot: SLOT_FIRE3,
            bytes: include_bytes!("../assets/fire03.png"),
        },
        TextureEntry {
            slot: SLOT_POWERUP_POWER,
            bytes: include_bytes!("../assets/powerupGreen_bolt.png"),
        },
        TextureEntry {
            slot: SLOT_POWERUP_BOMB,
            bytes: include_bytes!("../assets/powerupBlue_shield.png"),
        },
    ];

    let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
    let mut uv_max_table = vec![nalgebra_glm::Vec2::new(1.0, 1.0); 128];
    let mut pixel_sizes = vec![(0.0_f32, 0.0_f32); 128];

    for entry in &entries {
        let image = image::load_from_memory(entry.bytes).expect("failed to decode image");
        let (width, height) = image.dimensions();
        let rgba = image.to_rgba8().into_raw();

        world
            .resources
            .command_queue
            .push(WorldCommand::UploadSpriteTexture {
                slot: entry.slot,
                rgba_data: rgba,
                width,
                height,
            });

        pixel_sizes[entry.slot as usize] = (width as f32, height as f32);

        let half_texel_x = 0.5 / atlas_slot_size.0 as f32;
        let half_texel_y = 0.5 / atlas_slot_size.1 as f32;
        uv_max_table[entry.slot as usize] = nalgebra_glm::Vec2::new(
            width as f32 / atlas_slot_size.0 as f32 - half_texel_x,
            height as f32 / atlas_slot_size.1 as f32 - half_texel_y,
        );
    }

    (pixel_sizes, uv_max_table)
}

fn uv_for_slot(
    uv_max_table: &[nalgebra_glm::Vec2],
    slot: u32,
) -> (nalgebra_glm::Vec2, nalgebra_glm::Vec2) {
    let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
    let half_texel = nalgebra_glm::Vec2::new(
        0.5 / atlas_slot_size.0 as f32,
        0.5 / atlas_slot_size.1 as f32,
    );
    (half_texel, uv_max_table[slot as usize])
}

fn entity_position_2d(world: &World, entity: freecs::Entity) -> nalgebra_glm::Vec2 {
    world
        .sprite2d
        .get_sprite(entity)
        .map(|sprite| sprite.position)
        .unwrap_or_default()
}

fn set_entity_position_2d(world: &mut World, entity: freecs::Entity, position: nalgebra_glm::Vec2) {
    if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
        sprite.position = position;
    }
}

fn translate_entity_2d(world: &mut World, entity: freecs::Entity, delta: nalgebra_glm::Vec2) {
    if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
        sprite.position += delta;
    }
}

fn texture_size_from_list(sizes: &[(f32, f32)], slot: u32) -> nalgebra_glm::Vec2 {
    if (slot as usize) < sizes.len() {
        let (width, height) = sizes[slot as usize];
        nalgebra_glm::Vec2::new(width, height)
    } else {
        nalgebra_glm::Vec2::new(64.0, 64.0)
    }
}

fn update_hud_entity(world: &mut World, entity: Option<freecs::Entity>, text: &str) {
    if let Some(entity) = entity {
        let text_index = world.core.get_text(entity).map(|hud| hud.text_index);
        if let Some(text_index) = text_index {
            world.resources.text_cache.set_text(text_index, text);
            if let Some(text_component) = world.core.get_text_mut(entity) {
                text_component.dirty = true;
            }
        }
    }
}

fn rotate_vec2(direction: nalgebra_glm::Vec2, angle_radians: f32) -> nalgebra_glm::Vec2 {
    let cos = angle_radians.cos();
    let sin = angle_radians.sin();
    nalgebra_glm::Vec2::new(
        direction.x * cos - direction.y * sin,
        direction.x * sin + direction.y * cos,
    )
}

fn circle_overlap(
    pos_a: nalgebra_glm::Vec2,
    radius_a: f32,
    pos_b: nalgebra_glm::Vec2,
    radius_b: f32,
) -> bool {
    let difference = pos_a - pos_b;
    let distance_squared = difference.x * difference.x + difference.y * difference.y;
    let radii_sum = radius_a + radius_b;
    distance_squared < radii_sum * radii_sum
}

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    WaveIntro,
    Playing,
    WaveClear,
    BossIntro,
    Bombing,
    GameOver,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum EnemyKind {
    Black,
    Blue,
    Red,
    UfoBlue,
    UfoRed,
}

#[derive(Clone, Copy, PartialEq)]
enum PowerupKind {
    Power,
    Bomb,
}

#[derive(Clone, Copy, PartialEq)]
enum BulletColor {
    Red,
    Blue,
}

#[derive(Clone)]
enum PatternShape {
    Radial {
        bullet_count: u32,
        angular_offset: f32,
        rotation_speed: f32,
    },
    Spiral {
        arm_count: u32,
        bullets_per_arm: u32,
        angular_velocity: f32,
        bullet_spacing: f32,
    },
    Aimed {
        bullet_count: u32,
        spread_angle: f32,
    },
    Ring {
        bullet_count: u32,
        ring_count: u32,
        ring_speed_increment: f32,
        ring_delay: f32,
    },
    Sine {
        bullet_count: u32,
        wave_amplitude: f32,
        wave_frequency: f32,
        base_direction: f32,
    },
}

#[derive(Clone)]
struct BulletPattern {
    shape: PatternShape,
    bullet_speed: f32,
    fire_rate: f32,
    duration: f32,
    color: BulletColor,
}

struct PatternEmitter {
    pattern: BulletPattern,
    elapsed: f32,
    fire_accumulator: f32,
    ring_index: u32,
    ring_timer: f32,
    spiral_angle: f32,
    rotation_angle: f32,
    finished: bool,
}

impl PatternEmitter {
    fn new(pattern: BulletPattern) -> Self {
        Self {
            pattern,
            elapsed: 0.0,
            fire_accumulator: 0.0,
            ring_index: 0,
            ring_timer: 0.0,
            spiral_angle: 0.0,
            rotation_angle: 0.0,
            finished: false,
        }
    }

    fn reset(&mut self) {
        self.elapsed = 0.0;
        self.fire_accumulator = 0.0;
        self.ring_index = 0;
        self.ring_timer = 0.0;
        self.spiral_angle = 0.0;
        self.rotation_angle = 0.0;
        self.finished = false;
    }
}

fn evaluate_pattern_salvo(
    emitter: &mut PatternEmitter,
    delta_time: f32,
    origin: nalgebra_glm::Vec2,
    player_position: nalgebra_glm::Vec2,
) -> Vec<(nalgebra_glm::Vec2, nalgebra_glm::Vec2, BulletColor)> {
    if emitter.finished {
        return Vec::new();
    }

    emitter.elapsed += delta_time;
    if emitter.pattern.duration > 0.0 && emitter.elapsed >= emitter.pattern.duration {
        emitter.finished = true;
        return Vec::new();
    }

    let mut results = Vec::new();
    let speed = emitter.pattern.bullet_speed;
    let color = emitter.pattern.color;

    match &emitter.pattern.shape {
        PatternShape::Radial {
            bullet_count,
            angular_offset,
            rotation_speed,
        } => {
            let bullet_count = *bullet_count;
            let angular_offset = *angular_offset;
            let rotation_speed = *rotation_speed;

            emitter.rotation_angle += rotation_speed * delta_time;
            emitter.fire_accumulator += delta_time;

            if emitter.fire_accumulator >= emitter.pattern.fire_rate {
                emitter.fire_accumulator -= emitter.pattern.fire_rate;

                let angle_step = std::f32::consts::TAU / bullet_count as f32;
                for bullet_index in 0..bullet_count {
                    let angle =
                        angle_step * bullet_index as f32 + angular_offset + emitter.rotation_angle;
                    let direction = nalgebra_glm::Vec2::new(angle.cos(), angle.sin());
                    results.push((origin, direction * speed, color));
                }
            }
        }
        PatternShape::Spiral {
            arm_count,
            bullets_per_arm,
            angular_velocity,
            bullet_spacing,
        } => {
            let arm_count = *arm_count;
            let angular_velocity = *angular_velocity;
            let bullet_spacing = *bullet_spacing;
            let _bullets_per_arm = *bullets_per_arm;

            emitter.spiral_angle += angular_velocity * delta_time;
            emitter.fire_accumulator += delta_time;

            if emitter.fire_accumulator >= bullet_spacing {
                emitter.fire_accumulator -= bullet_spacing;

                let arm_step = std::f32::consts::TAU / arm_count as f32;
                for arm_index in 0..arm_count {
                    let angle = emitter.spiral_angle + arm_step * arm_index as f32;
                    let direction = nalgebra_glm::Vec2::new(angle.cos(), angle.sin());
                    results.push((origin, direction * speed, color));
                }
            }
        }
        PatternShape::Aimed {
            bullet_count,
            spread_angle,
        } => {
            let bullet_count = *bullet_count;
            let spread_angle = *spread_angle;

            emitter.fire_accumulator += delta_time;

            if emitter.fire_accumulator >= emitter.pattern.fire_rate {
                emitter.fire_accumulator -= emitter.pattern.fire_rate;

                let to_player = player_position - origin;
                let length = nalgebra_glm::length(&to_player);
                let base_direction = if length > 1.0 {
                    to_player / length
                } else {
                    nalgebra_glm::Vec2::new(0.0, -1.0)
                };

                if bullet_count == 1 {
                    results.push((origin, base_direction * speed, color));
                } else {
                    let half_spread = spread_angle * 0.5;
                    let angle_step = spread_angle / (bullet_count - 1).max(1) as f32;
                    for bullet_index in 0..bullet_count {
                        let offset = -half_spread + angle_step * bullet_index as f32;
                        let direction = rotate_vec2(base_direction, offset);
                        results.push((origin, direction * speed, color));
                    }
                }
            }
        }
        PatternShape::Ring {
            bullet_count,
            ring_count,
            ring_speed_increment,
            ring_delay,
        } => {
            let bullet_count = *bullet_count;
            let ring_count = *ring_count;
            let ring_speed_increment = *ring_speed_increment;
            let ring_delay = *ring_delay;

            if emitter.ring_index >= ring_count {
                emitter.finished = true;
                return results;
            }

            emitter.ring_timer += delta_time;

            if emitter.ring_timer >= ring_delay {
                emitter.ring_timer -= ring_delay;

                let ring_speed = speed + ring_speed_increment * emitter.ring_index as f32;
                let angle_step = std::f32::consts::TAU / bullet_count as f32;
                let offset = if emitter.ring_index.is_multiple_of(2) {
                    0.0
                } else {
                    angle_step * 0.5
                };

                for bullet_index in 0..bullet_count {
                    let angle = angle_step * bullet_index as f32 + offset;
                    let direction = nalgebra_glm::Vec2::new(angle.cos(), angle.sin());
                    results.push((origin, direction * ring_speed, color));
                }

                emitter.ring_index += 1;
            }
        }
        PatternShape::Sine {
            bullet_count,
            wave_amplitude,
            wave_frequency,
            base_direction,
        } => {
            let bullet_count = *bullet_count;
            let wave_amplitude = *wave_amplitude;
            let wave_frequency = *wave_frequency;
            let base_direction_angle = *base_direction;

            emitter.fire_accumulator += delta_time;

            if emitter.fire_accumulator >= emitter.pattern.fire_rate {
                emitter.fire_accumulator -= emitter.pattern.fire_rate;

                let base_dir =
                    nalgebra_glm::Vec2::new(base_direction_angle.cos(), base_direction_angle.sin());

                let spread = std::f32::consts::TAU / bullet_count as f32;
                for bullet_index in 0..bullet_count {
                    let phase = bullet_index as f32 * spread;
                    let sine_offset =
                        (emitter.elapsed * wave_frequency + phase).sin() * wave_amplitude;
                    let perpendicular = nalgebra_glm::Vec2::new(-base_dir.y, base_dir.x);
                    let direction = base_dir + perpendicular * sine_offset;
                    let length = nalgebra_glm::length(&direction);
                    let normalized = if length > 0.001 {
                        direction / length
                    } else {
                        base_dir
                    };
                    results.push((origin, normalized * speed, color));
                }
            }
        }
    }

    results
}

#[derive(Clone, Copy, PartialEq)]
enum EnemyBehavior {
    DriftDown {
        speed: f32,
    },
    SwoopToY {
        target_y: f32,
        speed: f32,
    },
    HoverAtY {
        target_y: f32,
        speed: f32,
        sway_amplitude: f32,
        sway_frequency: f32,
    },
    CircleAroundPoint {
        center_x: f32,
        center_y: f32,
        radius: f32,
        angular_speed: f32,
    },
}

struct WaveSpawnEntry {
    kind: EnemyKind,
    position: nalgebra_glm::Vec2,
    behavior: EnemyBehavior,
    patterns: Vec<BulletPattern>,
    health: i32,
    delay: f32,
}

struct WaveDefinition {
    spawns: Vec<WaveSpawnEntry>,
}

fn generate_wave(
    wave_number: u32,
    play_area_half_width: f32,
    play_area_half_height: f32,
) -> WaveDefinition {
    use rand::Rng;
    let mut rng = rand::rng();

    let is_boss_wave = wave_number.is_multiple_of(10) && wave_number > 0;
    let is_miniboss_wave = wave_number.is_multiple_of(5) && !is_boss_wave && wave_number > 0;

    if is_boss_wave {
        return generate_boss_wave(wave_number, play_area_half_height);
    }

    if is_miniboss_wave {
        return generate_miniboss_wave(wave_number, play_area_half_height);
    }

    let enemy_count = (8 + wave_number * 3).min(60);
    let speed_scale = 1.0 + (wave_number as f32 - 1.0) * 0.08;
    let bullet_speed = ENEMY_BULLET_BASE_SPEED * speed_scale;
    let extra_bullets = wave_number;

    let mut spawns = Vec::new();
    let mut accumulated_delay = 0.0;

    for enemy_index in 0..enemy_count {
        let spawn_x = rng.random_range(-play_area_half_width * 0.8..play_area_half_width * 0.8);
        let spawn_y = play_area_half_height + 80.0;
        let position = nalgebra_glm::Vec2::new(spawn_x, spawn_y);

        let kind = pick_enemy_kind_for_wave(wave_number, &mut rng);
        let health = match kind {
            EnemyKind::Black => 1 + (wave_number / 4) as i32,
            EnemyKind::Blue => 3 + (wave_number / 3) as i32,
            EnemyKind::Red => 5 + (wave_number / 2) as i32,
            EnemyKind::UfoBlue => 15,
            EnemyKind::UfoRed => 40,
        };

        let behavior = match kind {
            EnemyKind::Black => {
                if rng.random_bool(0.5) {
                    EnemyBehavior::DriftDown {
                        speed: 80.0 + rng.random_range(0.0..40.0),
                    }
                } else {
                    let target_y = rng.random_range(0.0..play_area_half_height * 0.6);
                    EnemyBehavior::SwoopToY {
                        target_y,
                        speed: 200.0,
                    }
                }
            }
            EnemyKind::Blue => EnemyBehavior::HoverAtY {
                target_y: rng.random_range(100.0..play_area_half_height * 0.7),
                speed: 180.0,
                sway_amplitude: rng.random_range(60.0..150.0),
                sway_frequency: rng.random_range(0.8..2.0),
            },
            EnemyKind::Red => EnemyBehavior::HoverAtY {
                target_y: rng.random_range(150.0..play_area_half_height * 0.5),
                speed: 150.0,
                sway_amplitude: rng.random_range(40.0..100.0),
                sway_frequency: rng.random_range(0.5..1.5),
            },
            _ => EnemyBehavior::DriftDown { speed: 60.0 },
        };

        let patterns = make_patterns_for_kind(kind, bullet_speed, extra_bullets);

        let delay = if enemy_index == 0 {
            0.0
        } else {
            let base_interval = (0.8 - wave_number as f32 * 0.03).max(0.1);
            accumulated_delay += base_interval + rng.random_range(0.0..0.15);
            accumulated_delay
        };

        spawns.push(WaveSpawnEntry {
            kind,
            position,
            behavior,
            patterns,
            health,
            delay,
        });
    }

    WaveDefinition { spawns }
}

fn generate_miniboss_wave(wave_number: u32, play_area_half_height: f32) -> WaveDefinition {
    use rand::Rng;
    let mut rng = rand::rng();

    let speed_scale = 1.0 + (wave_number as f32 - 1.0) * 0.08;
    let bullet_speed = ENEMY_BULLET_BASE_SPEED * speed_scale;
    let extra_bullets = wave_number;

    let mut spawns = Vec::new();

    let escort_count = 4 + (wave_number / 5).min(4);
    for escort_index in 0..escort_count {
        let side = if escort_index % 2 == 0 { -1.0 } else { 1.0 };
        let x_offset = side * (120.0 + (escort_index / 2) as f32 * 80.0);
        let escort_kind = if escort_index < 2 {
            EnemyKind::Red
        } else {
            EnemyKind::Blue
        };
        spawns.push(WaveSpawnEntry {
            kind: escort_kind,
            position: nalgebra_glm::Vec2::new(x_offset, play_area_half_height + 80.0),
            behavior: EnemyBehavior::HoverAtY {
                target_y: rng.random_range(150.0..300.0),
                speed: 180.0,
                sway_amplitude: 100.0,
                sway_frequency: 1.5,
            },
            patterns: make_patterns_for_kind(escort_kind, bullet_speed, extra_bullets),
            health: 4 + (wave_number / 3) as i32,
            delay: escort_index as f32 * 0.4,
        });
    }

    let miniboss_patterns = vec![
        BulletPattern {
            shape: PatternShape::Ring {
                bullet_count: 20 + extra_bullets,
                ring_count: 4,
                ring_speed_increment: 35.0,
                ring_delay: 0.3,
            },
            bullet_speed,
            fire_rate: 0.0,
            duration: 0.0,
            color: BulletColor::Red,
        },
        BulletPattern {
            shape: PatternShape::Aimed {
                bullet_count: 4 + extra_bullets / 2,
                spread_angle: 0.5,
            },
            bullet_speed: bullet_speed * 1.4,
            fire_rate: 0.4,
            duration: 0.0,
            color: BulletColor::Blue,
        },
        BulletPattern {
            shape: PatternShape::Spiral {
                arm_count: 3 + extra_bullets / 4,
                bullets_per_arm: 12,
                angular_velocity: 2.0,
                bullet_spacing: 0.1,
            },
            bullet_speed: bullet_speed * 0.9,
            fire_rate: 0.1,
            duration: 0.0,
            color: BulletColor::Red,
        },
    ];

    spawns.push(WaveSpawnEntry {
        kind: EnemyKind::UfoBlue,
        position: nalgebra_glm::Vec2::new(0.0, play_area_half_height + 80.0),
        behavior: EnemyBehavior::HoverAtY {
            target_y: 280.0,
            speed: 140.0,
            sway_amplitude: 150.0,
            sway_frequency: 0.8,
        },
        patterns: miniboss_patterns,
        health: 20 + (wave_number as i32 / 2),
        delay: 1.5,
    });

    WaveDefinition { spawns }
}

fn generate_boss_wave(wave_number: u32, play_area_half_height: f32) -> WaveDefinition {
    let speed_scale = 1.0 + (wave_number as f32 - 1.0) * 0.08;
    let bullet_speed = ENEMY_BULLET_BASE_SPEED * speed_scale;
    let extra_bullets = wave_number;

    let boss_patterns = vec![
        BulletPattern {
            shape: PatternShape::Spiral {
                arm_count: 5 + extra_bullets / 3,
                bullets_per_arm: 25,
                angular_velocity: 3.0,
                bullet_spacing: 0.06,
            },
            bullet_speed,
            fire_rate: 0.06,
            duration: 0.0,
            color: BulletColor::Red,
        },
        BulletPattern {
            shape: PatternShape::Aimed {
                bullet_count: 6 + extra_bullets / 2,
                spread_angle: 0.7,
            },
            bullet_speed: bullet_speed * 1.6,
            fire_rate: 0.25,
            duration: 0.0,
            color: BulletColor::Blue,
        },
        BulletPattern {
            shape: PatternShape::Radial {
                bullet_count: 32 + extra_bullets,
                angular_offset: 0.0,
                rotation_speed: 1.2,
            },
            bullet_speed: bullet_speed * 0.85,
            fire_rate: 0.5,
            duration: 0.0,
            color: BulletColor::Red,
        },
        BulletPattern {
            shape: PatternShape::Ring {
                bullet_count: 24 + extra_bullets,
                ring_count: 6,
                ring_speed_increment: 30.0,
                ring_delay: 0.25,
            },
            bullet_speed,
            fire_rate: 0.0,
            duration: 0.0,
            color: BulletColor::Blue,
        },
        BulletPattern {
            shape: PatternShape::Sine {
                bullet_count: 8 + extra_bullets / 2,
                wave_amplitude: 0.5,
                wave_frequency: 4.0,
                base_direction: -std::f32::consts::FRAC_PI_2,
            },
            bullet_speed: bullet_speed * 0.75,
            fire_rate: 0.3,
            duration: 0.0,
            color: BulletColor::Red,
        },
    ];

    let mut spawns = Vec::new();

    use rand::Rng;
    let mut rng = rand::rng();
    let escort_count = 2 + (wave_number / 10).min(4);
    for escort_index in 0..escort_count {
        let side = if escort_index % 2 == 0 { -1.0 } else { 1.0 };
        let x_offset = side * (200.0 + (escort_index / 2) as f32 * 100.0);
        spawns.push(WaveSpawnEntry {
            kind: EnemyKind::Red,
            position: nalgebra_glm::Vec2::new(x_offset, play_area_half_height + 80.0),
            behavior: EnemyBehavior::HoverAtY {
                target_y: rng.random_range(180.0..320.0),
                speed: 160.0,
                sway_amplitude: 90.0,
                sway_frequency: 1.3,
            },
            patterns: make_patterns_for_kind(EnemyKind::Red, bullet_speed, extra_bullets),
            health: 5 + (wave_number / 3) as i32,
            delay: escort_index as f32 * 0.6,
        });
    }

    spawns.push(WaveSpawnEntry {
        kind: EnemyKind::UfoRed,
        position: nalgebra_glm::Vec2::new(0.0, play_area_half_height + 80.0),
        behavior: EnemyBehavior::CircleAroundPoint {
            center_x: 0.0,
            center_y: 280.0,
            radius: 150.0,
            angular_speed: 0.6,
        },
        patterns: boss_patterns,
        health: 50 + (wave_number as i32 * 3),
        delay: 1.0,
    });

    WaveDefinition { spawns }
}

fn pick_enemy_kind_for_wave(wave_number: u32, rng: &mut impl rand::Rng) -> EnemyKind {
    let roll: f32 = rng.random_range(0.0..1.0);

    if wave_number >= 3 && roll < 0.25 {
        EnemyKind::Red
    } else if wave_number >= 2 && roll < 0.50 {
        EnemyKind::Blue
    } else {
        EnemyKind::Black
    }
}

fn make_patterns_for_kind(
    kind: EnemyKind,
    bullet_speed: f32,
    extra_bullets: u32,
) -> Vec<BulletPattern> {
    match kind {
        EnemyKind::Black => vec![BulletPattern {
            shape: PatternShape::Aimed {
                bullet_count: 2 + extra_bullets / 3,
                spread_angle: 0.25,
            },
            bullet_speed: bullet_speed * 1.3,
            fire_rate: 1.0,
            duration: 0.0,
            color: BulletColor::Red,
        }],
        EnemyKind::Blue => vec![
            BulletPattern {
                shape: PatternShape::Radial {
                    bullet_count: 12 + extra_bullets / 2,
                    angular_offset: 0.0,
                    rotation_speed: 0.8,
                },
                bullet_speed,
                fire_rate: 0.7,
                duration: 0.0,
                color: BulletColor::Red,
            },
            BulletPattern {
                shape: PatternShape::Aimed {
                    bullet_count: 2 + extra_bullets / 4,
                    spread_angle: 0.3,
                },
                bullet_speed: bullet_speed * 1.2,
                fire_rate: 1.2,
                duration: 0.0,
                color: BulletColor::Blue,
            },
        ],
        EnemyKind::Red => vec![
            BulletPattern {
                shape: PatternShape::Spiral {
                    arm_count: 4 + extra_bullets / 4,
                    bullets_per_arm: 15,
                    angular_velocity: 2.5,
                    bullet_spacing: 0.08,
                },
                bullet_speed,
                fire_rate: 0.08,
                duration: 0.0,
                color: BulletColor::Blue,
            },
            BulletPattern {
                shape: PatternShape::Radial {
                    bullet_count: 10 + extra_bullets / 3,
                    angular_offset: 0.0,
                    rotation_speed: 1.0,
                },
                bullet_speed: bullet_speed * 0.9,
                fire_rate: 0.9,
                duration: 0.0,
                color: BulletColor::Red,
            },
        ],
        _ => Vec::new(),
    }
}

struct PlayerBullet {
    entity: freecs::Entity,
    velocity: nalgebra_glm::Vec2,
    lifetime: f32,
}

struct EnemyBullet {
    entity: freecs::Entity,
    velocity: nalgebra_glm::Vec2,
    position: nalgebra_glm::Vec2,
    radius: f32,
    grazed: bool,
}

struct Enemy {
    entity: freecs::Entity,
    kind: EnemyKind,
    health: i32,
    max_health: i32,
    behavior: EnemyBehavior,
    emitters: Vec<PatternEmitter>,
    elapsed: f32,
    phase: u32,
}

struct Meteor {
    entity: freecs::Entity,
    velocity: nalgebra_glm::Vec2,
}

struct Explosion {
    entity: freecs::Entity,
    lifetime: f32,
}

struct GrazeFlash {
    entity: freecs::Entity,
    lifetime: f32,
}

struct Powerup {
    entity: freecs::Entity,
    kind: PowerupKind,
    lifetime: f32,
    bob_phase: f32,
    base_y: f32,
}

struct BulletHell {
    camera_entity: Option<freecs::Entity>,
    player_entity: Option<freecs::Entity>,
    hitbox_entity: Option<freecs::Entity>,
    engine_exhaust_entity: Option<freecs::Entity>,
    player_position: nalgebra_glm::Vec2,
    power_level: u32,
    fire_cooldown: f32,
    invincible_timer: f32,
    focused: bool,

    player_tilt_angle: f32,

    barrel_roll_active: bool,
    barrel_roll_timer: f32,
    barrel_roll_direction: f32,
    game_time: f32,
    last_left_tap_time: f32,
    last_right_tap_time: f32,

    player_bullets: Vec<PlayerBullet>,
    enemy_bullets: Vec<EnemyBullet>,
    enemies: Vec<Enemy>,
    meteors: Vec<Meteor>,
    explosions: Vec<Explosion>,
    graze_flashes: Vec<GrazeFlash>,
    powerups: Vec<Powerup>,

    score: u64,
    multiplier: f32,
    graze_count: u64,
    next_extra_life_score: u64,

    lives: u32,
    bombs: u32,
    wave_number: u32,
    game_state: GameState,
    previous_state: GameState,
    state_timer: f32,

    wave_definition: Option<WaveDefinition>,
    wave_spawn_index: usize,
    wave_elapsed: f32,

    bomb_timer: f32,

    shake_timer: f32,
    shake_intensity: f32,

    score_hud: Option<freecs::Entity>,
    lives_bombs_hud: Option<freecs::Entity>,
    power_hud: Option<freecs::Entity>,
    wave_hud: Option<freecs::Entity>,
    fps_hud: Option<freecs::Entity>,
    graze_hud: Option<freecs::Entity>,
    multiplier_hud: Option<freecs::Entity>,
    center_text_hud: Option<freecs::Entity>,
    center_text_timer: f32,

    texture_sizes: Vec<(f32, f32)>,
    uv_max_table: Vec<nalgebra_glm::Vec2>,
    left_was_pressed: bool,
    right_was_pressed: bool,
    play_area_half_width: f32,
    play_area_half_height: f32,
}

fn visible_half_extents(viewport_width: u32, viewport_height: u32) -> (f32, f32) {
    let aspect_ratio = viewport_width as f32 / viewport_height.max(1) as f32;
    let half_height = CAMERA_Z * (45.0_f32.to_radians() / 2.0).tan();
    let half_width = half_height * aspect_ratio;
    (half_width, half_height)
}

impl Default for BulletHell {
    fn default() -> Self {
        Self {
            camera_entity: None,
            player_entity: None,
            hitbox_entity: None,
            engine_exhaust_entity: None,
            player_position: nalgebra_glm::Vec2::zeros(),
            power_level: 0,
            fire_cooldown: 0.0,
            invincible_timer: 0.0,
            focused: false,

            player_tilt_angle: 0.0,

            barrel_roll_active: false,
            barrel_roll_timer: 0.0,
            barrel_roll_direction: 1.0,
            game_time: 0.0,
            last_left_tap_time: -1.0,
            last_right_tap_time: -1.0,

            player_bullets: Vec::new(),
            enemy_bullets: Vec::new(),
            enemies: Vec::new(),
            meteors: Vec::new(),
            explosions: Vec::new(),
            graze_flashes: Vec::new(),
            powerups: Vec::new(),

            score: 0,
            multiplier: 1.0,
            graze_count: 0,
            next_extra_life_score: EXTRA_LIFE_SCORE_INTERVAL,

            lives: 3,
            bombs: 3,
            wave_number: 1,
            game_state: GameState::WaveIntro,
            previous_state: GameState::WaveIntro,
            state_timer: WAVE_INTRO_DURATION,

            wave_definition: None,
            wave_spawn_index: 0,
            wave_elapsed: 0.0,

            bomb_timer: 0.0,

            shake_timer: 0.0,
            shake_intensity: 0.0,

            score_hud: None,
            lives_bombs_hud: None,
            power_hud: None,
            wave_hud: None,
            fps_hud: None,
            graze_hud: None,
            multiplier_hud: None,
            center_text_hud: None,
            center_text_timer: WAVE_INTRO_DURATION,

            texture_sizes: Vec::new(),
            uv_max_table: Vec::new(),
            left_was_pressed: false,
            right_was_pressed: false,
            play_area_half_width: 0.0,
            play_area_half_height: 0.0,
        }
    }
}

impl BulletHell {
    fn spawn_game_sprite(
        world: &mut World,
        position: nalgebra_glm::Vec2,
        size: nalgebra_glm::Vec2,
        texture_slot: u32,
        z_layer: f32,
        uv_max_table: &[nalgebra_glm::Vec2],
    ) -> freecs::Entity {
        let (uv_min, uv_max) = uv_for_slot(uv_max_table, texture_slot);
        let entity = spawn_sprite(world, position, size);
        if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
            sprite.texture_index = texture_slot;
            sprite.texture_index2 = texture_slot;
            sprite.uv_min = uv_min;
            sprite.uv_max = uv_max;
            sprite.depth = z_layer;
        }
        entity
    }

    fn texture_size(&self, slot: u32) -> nalgebra_glm::Vec2 {
        texture_size_from_list(&self.texture_sizes, slot)
    }

    fn spawn_background(&self, world: &mut World) {
        let bg_size = self.texture_size(SLOT_BG);
        for row in -BG_TILE_COUNT..=BG_TILE_COUNT {
            for column in -BG_TILE_COUNT..=BG_TILE_COUNT {
                let position =
                    nalgebra_glm::Vec2::new(column as f32 * bg_size.x, row as f32 * bg_size.y);
                Self::spawn_game_sprite(
                    world,
                    position,
                    bg_size,
                    SLOT_BG,
                    LAYER_BG,
                    &self.uv_max_table,
                );
            }
        }
    }

    fn spawn_star_field(&self, world: &mut World) {
        use rand::Rng;
        let mut rng = rand::rng();
        let spread_x = self.play_area_half_width + 200.0;
        let spread_y = self.play_area_half_height + 200.0;

        for _ in 0..STAR_COUNT {
            let position = nalgebra_glm::Vec2::new(
                rng.random_range(-spread_x..spread_x),
                rng.random_range(-spread_y..spread_y),
            );
            let star_size = self.texture_size(SLOT_STAR);
            let scale = rng.random_range(0.3..1.0);
            let entity = Self::spawn_game_sprite(
                world,
                position,
                nalgebra_glm::Vec2::new(star_size.x * scale, star_size.y * scale),
                SLOT_STAR,
                LAYER_STARS,
                &self.uv_max_table,
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.color[3] = rng.random_range(0.2..0.6);
            }
        }
    }

    fn spawn_meteors(&mut self, world: &mut World) {
        use rand::Rng;
        let mut rng = rand::rng();

        for _ in 0..METEOR_COUNT {
            let position = nalgebra_glm::Vec2::new(
                rng.random_range(-self.play_area_half_width..self.play_area_half_width),
                rng.random_range(-self.play_area_half_height..self.play_area_half_height),
            );
            let meteor_size = self.texture_size(SLOT_METEOR);
            let scale = rng.random_range(0.5..1.2);
            let entity = Self::spawn_game_sprite(
                world,
                position,
                meteor_size * scale,
                SLOT_METEOR,
                LAYER_METEORS,
                &self.uv_max_table,
            );
            let velocity = nalgebra_glm::Vec2::new(
                rng.random_range(-15.0..15.0),
                rng.random_range(-30.0..-10.0),
            );
            self.meteors.push(Meteor { entity, velocity });
        }
    }

    fn spawn_player(&mut self, world: &mut World) {
        let player_size = self.texture_size(SLOT_PLAYER);
        let entity = Self::spawn_game_sprite(
            world,
            self.player_position,
            player_size,
            SLOT_PLAYER,
            LAYER_PLAYER,
            &self.uv_max_table,
        );
        self.player_entity = Some(entity);

        let hitbox_size = nalgebra_glm::Vec2::new(8.0, 8.0);
        let hitbox_entity = Self::spawn_game_sprite(
            world,
            self.player_position,
            hitbox_size,
            SLOT_STAR,
            LAYER_HITBOX,
            &self.uv_max_table,
        );
        if let Some(sprite) = world.sprite2d.get_sprite_mut(hitbox_entity) {
            sprite.color = [1.0, 0.3, 0.3, 0.0];
        }
        self.hitbox_entity = Some(hitbox_entity);

        let exhaust_entity = world.spawn();
        let exhaust_y = self.player_position.y - player_size.y * 0.45;
        world.sprite2d.set_sprite_particle_emitter(
            exhaust_entity,
            SpriteParticleEmitter::fire_trail(self.player_position.x, exhaust_y)
                .with_depth(LAYER_PLAYER - 0.1)
                .with_spawn_rate(120.0)
                .with_max_particles(256)
                .with_velocity(
                    nalgebra_glm::Vec2::new(-15.0, -80.0),
                    nalgebra_glm::Vec2::new(15.0, -30.0),
                )
                .with_gravity(nalgebra_glm::Vec2::new(0.0, -50.0))
                .with_size(
                    nalgebra_glm::Vec2::new(6.0, 6.0),
                    nalgebra_glm::Vec2::new(2.0, 2.0),
                )
                .with_lifetime(0.15, 0.4)
                .with_color(ColorRange2D::new(
                    [0.4, 0.7, 1.0, 0.9],
                    [0.2, 0.3, 1.0, 0.0],
                )),
        );
        self.engine_exhaust_entity = Some(exhaust_entity);
    }

    fn spawn_hud(&mut self, world: &mut World) {
        let hud_props = TextProperties {
            font_size: 22.0,
            color: nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0),
            ..Default::default()
        };

        self.score_hud = Some(spawn_ui_text_with_properties(
            world,
            format!("Score: {}", self.score),
            nalgebra_glm::Vec2::zeros(),
            hud_props.clone(),
        ));

        self.lives_bombs_hud = Some(spawn_ui_text_with_properties(
            world,
            format!("Lives: {}  Bombs: {}", self.lives, self.bombs),
            nalgebra_glm::Vec2::zeros(),
            hud_props.clone(),
        ));

        self.power_hud = Some(spawn_ui_text_with_properties(
            world,
            format!("Power: {}", self.power_level),
            nalgebra_glm::Vec2::zeros(),
            hud_props.clone(),
        ));

        self.wave_hud = Some(spawn_ui_text_with_properties(
            world,
            format!("Wave: {}", self.wave_number),
            nalgebra_glm::Vec2::zeros(),
            hud_props.clone(),
        ));

        self.fps_hud = Some(spawn_ui_text_with_properties(
            world,
            "FPS: 0",
            nalgebra_glm::Vec2::zeros(),
            hud_props.clone(),
        ));

        self.graze_hud = Some(spawn_ui_text_with_properties(
            world,
            format!("Graze: {}", self.graze_count),
            nalgebra_glm::Vec2::zeros(),
            hud_props.clone(),
        ));

        self.multiplier_hud = Some(spawn_ui_text_with_properties(
            world,
            format!("x{:.2}", self.multiplier),
            nalgebra_glm::Vec2::zeros(),
            hud_props,
        ));

        let center_props = TextProperties {
            font_size: 48.0,
            color: nalgebra_glm::Vec4::new(1.0, 1.0, 0.0, 1.0),
            alignment: TextAlignment::Center,
            ..Default::default()
        };

        self.center_text_hud = Some(spawn_ui_text_with_properties(
            world,
            format!("WAVE {}", self.wave_number),
            nalgebra_glm::Vec2::zeros(),
            center_props,
        ));
    }

    fn update_hud_text(&self, world: &mut World) {
        update_hud_entity(world, self.score_hud, &format!("Score: {}", self.score));
        update_hud_entity(
            world,
            self.lives_bombs_hud,
            &format!("Lives: {}  Bombs: {}", self.lives, self.bombs),
        );
        update_hud_entity(
            world,
            self.power_hud,
            &format!("Power: {}", self.power_level),
        );
        update_hud_entity(world, self.wave_hud, &format!("Wave: {}", self.wave_number));

        let fps = world.resources.window.timing.frames_per_second;
        update_hud_entity(world, self.fps_hud, &format!("FPS: {}", fps as u32));
        update_hud_entity(
            world,
            self.graze_hud,
            &format!("Graze: {}", self.graze_count),
        );
        update_hud_entity(
            world,
            self.multiplier_hud,
            &format!("x{:.2}", self.multiplier),
        );
    }

    fn set_center_text(&self, world: &mut World, text: &str) {
        update_hud_entity(world, self.center_text_hud, text);
    }

    fn hide_center_text(&self, world: &mut World) {
        self.set_center_text(world, "");
    }

    fn announce(&mut self, world: &mut World, text: &str, duration: f32) {
        self.set_center_text(world, text);
        self.center_text_timer = duration;
    }

    fn player_input_system(&mut self, world: &mut World) {
        if self.game_state == GameState::GameOver {
            return;
        }

        let delta_time = world.resources.window.timing.delta_time;

        let keyboard = &world.resources.input.keyboard;

        let mut left =
            keyboard.is_key_pressed(KeyCode::KeyA) || keyboard.is_key_pressed(KeyCode::ArrowLeft);
        let mut right =
            keyboard.is_key_pressed(KeyCode::KeyD) || keyboard.is_key_pressed(KeyCode::ArrowRight);

        let left_just_pressed = left && !self.left_was_pressed;
        let right_just_pressed = right && !self.right_was_pressed;
        self.left_was_pressed = left;
        self.right_was_pressed = right;
        let mut up =
            keyboard.is_key_pressed(KeyCode::KeyW) || keyboard.is_key_pressed(KeyCode::ArrowUp);
        let mut down =
            keyboard.is_key_pressed(KeyCode::KeyS) || keyboard.is_key_pressed(KeyCode::ArrowDown);

        self.focused = keyboard.is_key_pressed(KeyCode::ShiftLeft)
            || keyboard.is_key_pressed(KeyCode::ShiftRight);

        if let Some(gamepad) = query_active_gamepad(world) {
            let stick_x = gamepad.value(gilrs::Axis::LeftStickX);
            let stick_y = gamepad.value(gilrs::Axis::LeftStickY);
            if stick_x < -GAMEPAD_DEADZONE {
                left = true;
            }
            if stick_x > GAMEPAD_DEADZONE {
                right = true;
            }
            if stick_y > GAMEPAD_DEADZONE {
                up = true;
            }
            if stick_y < -GAMEPAD_DEADZONE {
                down = true;
            }
            if gamepad.is_pressed(gilrs::Button::LeftTrigger2) {
                self.focused = true;
            }
        }

        if !self.barrel_roll_active {
            if left_just_pressed {
                if self.game_time - self.last_left_tap_time < DOUBLE_TAP_WINDOW {
                    self.barrel_roll_active = true;
                    self.barrel_roll_timer = BARREL_ROLL_DURATION;
                    self.barrel_roll_direction = 1.0;
                    self.invincible_timer = self.invincible_timer.max(BARREL_ROLL_INVINCIBILITY);
                    self.last_left_tap_time = -1.0;
                } else {
                    self.last_left_tap_time = self.game_time;
                }
            }
            if right_just_pressed {
                if self.game_time - self.last_right_tap_time < DOUBLE_TAP_WINDOW {
                    self.barrel_roll_active = true;
                    self.barrel_roll_timer = BARREL_ROLL_DURATION;
                    self.barrel_roll_direction = -1.0;
                    self.invincible_timer = self.invincible_timer.max(BARREL_ROLL_INVINCIBILITY);
                    self.last_right_tap_time = -1.0;
                } else {
                    self.last_right_tap_time = self.game_time;
                }
            }
        }

        if let Some(gamepad) = query_active_gamepad(world)
            && !self.barrel_roll_active
        {
            if gamepad.is_pressed(gilrs::Button::LeftTrigger) {
                self.barrel_roll_active = true;
                self.barrel_roll_timer = BARREL_ROLL_DURATION;
                self.barrel_roll_direction = 1.0;
                self.invincible_timer = self.invincible_timer.max(BARREL_ROLL_INVINCIBILITY);
            } else if gamepad.is_pressed(gilrs::Button::RightTrigger) {
                self.barrel_roll_active = true;
                self.barrel_roll_timer = BARREL_ROLL_DURATION;
                self.barrel_roll_direction = -1.0;
                self.invincible_timer = self.invincible_timer.max(BARREL_ROLL_INVINCIBILITY);
            }
        }

        if self.barrel_roll_active {
            self.barrel_roll_timer -= delta_time;
            if self.barrel_roll_timer <= 0.0 {
                self.barrel_roll_active = false;
                self.barrel_roll_timer = 0.0;
            }
        }

        let speed = if self.focused {
            PLAYER_FOCUSED_SPEED
        } else {
            PLAYER_SPEED
        };

        let mut movement = nalgebra_glm::Vec2::new(0.0, 0.0);
        if left {
            movement.x -= 1.0;
        }
        if right {
            movement.x += 1.0;
        }
        if up {
            movement.y += 1.0;
        }
        if down {
            movement.y -= 1.0;
        }

        let length = nalgebra_glm::length(&movement);
        if length > 0.0 {
            movement /= length;
            self.player_position += movement * speed * delta_time;
        }

        let clamp_width = self.play_area_half_width - PLAYER_EDGE_MARGIN;
        let clamp_height = self.play_area_half_height - PLAYER_EDGE_MARGIN;
        self.player_position.x = self.player_position.x.clamp(-clamp_width, clamp_width);
        self.player_position.y = self.player_position.y.clamp(-clamp_height, clamp_height);

        let y_rotation = if self.barrel_roll_active {
            let progress = 1.0 - (self.barrel_roll_timer / BARREL_ROLL_DURATION);
            self.barrel_roll_direction * progress * std::f32::consts::TAU
        } else {
            let target_tilt = if left && !right {
                PLAYER_MAX_TILT
            } else if right && !left {
                -PLAYER_MAX_TILT
            } else {
                0.0
            };
            let difference = target_tilt - self.player_tilt_angle;
            self.player_tilt_angle += difference * (PLAYER_TILT_SPEED * delta_time).min(1.0);
            self.player_tilt_angle
        };

        if let Some(entity) = self.player_entity {
            set_entity_position_2d(world, entity, self.player_position);

            if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.rotation = y_rotation;
            }

            if self.invincible_timer > 0.0 {
                let blink = ((self.invincible_timer * 10.0).sin() + 1.0) * 0.5;
                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.color[3] = 0.3 + blink * 0.7;
                }
            } else if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.color[3] = 1.0;
            }
        }

        if let Some(hitbox_entity) = self.hitbox_entity {
            set_entity_position_2d(world, hitbox_entity, self.player_position);

            if let Some(sprite) = world.sprite2d.get_sprite_mut(hitbox_entity) {
                sprite.color[3] = if self.focused { 0.9 } else { 0.0 };
            }
        }

        if let Some(exhaust_entity) = self.engine_exhaust_entity {
            let player_size = self.texture_size(SLOT_PLAYER);
            let exhaust_position = nalgebra_glm::Vec2::new(
                self.player_position.x,
                self.player_position.y - player_size.y * 0.45,
            );
            if let Some(emitter) = world
                .sprite2d
                .get_sprite_particle_emitter_mut(exhaust_entity)
            {
                emitter.anchor = exhaust_position;
            }
        }
    }

    fn player_shooting_system(&mut self, world: &mut World) {
        if self.game_state != GameState::Playing && self.game_state != GameState::Bombing {
            return;
        }

        let delta_time = world.resources.window.timing.delta_time;
        self.fire_cooldown -= delta_time;

        let mut shooting = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Space)
            || world
                .resources
                .input
                .mouse
                .state
                .contains(MouseState::LEFT_CLICKED);

        if let Some(gamepad) = query_active_gamepad(world)
            && (gamepad.is_pressed(gilrs::Button::South)
                || gamepad.is_pressed(gilrs::Button::RightTrigger2))
        {
            shooting = true;
        }

        if !shooting || self.fire_cooldown > 0.0 {
            return;
        }

        self.fire_cooldown = PLAYER_FIRE_COOLDOWN;

        let bullet_size = self.texture_size(SLOT_LASER_BLUE);
        let base_y = self.player_position.y + self.texture_size(SLOT_PLAYER).y * 0.5;

        let shots = self.compute_shot_offsets();

        for (offset_x, angle) in shots {
            let spawn_position = nalgebra_glm::Vec2::new(self.player_position.x + offset_x, base_y);

            let velocity = if self.focused {
                nalgebra_glm::Vec2::new(0.0, PLAYER_BULLET_SPEED)
            } else {
                rotate_vec2(nalgebra_glm::Vec2::new(0.0, PLAYER_BULLET_SPEED), angle)
            };

            let entity = Self::spawn_game_sprite(
                world,
                spawn_position,
                bullet_size,
                SLOT_LASER_BLUE,
                LAYER_PLAYER_BULLETS,
                &self.uv_max_table,
            );
            if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.blend_mode = SpriteBlendMode::Additive;
            }

            self.player_bullets.push(PlayerBullet {
                entity,
                velocity,
                lifetime: PLAYER_BULLET_LIFETIME,
            });
        }
    }

    fn compute_shot_offsets(&self) -> Vec<(f32, f32)> {
        match self.power_level {
            0 => vec![(0.0, 0.0)],
            1 => vec![(-8.0, 0.0), (8.0, 0.0)],
            2 => vec![(0.0, 0.0), (-14.0, -0.12), (14.0, 0.12)],
            3 => vec![(-6.0, 0.0), (6.0, 0.0), (-16.0, -0.15), (16.0, 0.15)],
            _ => vec![
                (0.0, 0.0),
                (-8.0, 0.0),
                (8.0, 0.0),
                (-18.0, -0.18),
                (18.0, 0.18),
            ],
        }
    }

    fn player_bullet_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;
        let mut to_remove = Vec::new();

        for (index, bullet) in self.player_bullets.iter_mut().enumerate() {
            bullet.lifetime -= delta_time;
            if bullet.lifetime <= 0.0 {
                to_remove.push(index);
                continue;
            }

            translate_entity_2d(world, bullet.entity, bullet.velocity * delta_time);

            let position = entity_position_2d(world, bullet.entity);
            if position.y > self.play_area_half_height + DESPAWN_MARGIN
                || position.y < -self.play_area_half_height - DESPAWN_MARGIN
                || position.x > self.play_area_half_width + DESPAWN_MARGIN
                || position.x < -self.play_area_half_width - DESPAWN_MARGIN
            {
                to_remove.push(index);
            }
        }

        to_remove.sort_unstable();
        to_remove.dedup();
        for index in to_remove.into_iter().rev() {
            let bullet = self.player_bullets.remove(index);
            despawn_entities_with_cache_cleanup(world, &[bullet.entity]);
        }
    }

    fn bomb_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;

        if self.game_state == GameState::Playing {
            let mut bomb_pressed = world
                .resources
                .input
                .keyboard
                .frame_keys
                .iter()
                .any(|(key, pressed)| *key == KeyCode::KeyX && *pressed);

            if let Some(gamepad) = query_active_gamepad(world)
                && (gamepad.is_pressed(gilrs::Button::West)
                    || gamepad.is_pressed(gilrs::Button::East))
            {
                bomb_pressed = true;
            }

            if bomb_pressed && self.bombs > 0 {
                self.bombs -= 1;
                self.bomb_timer = BOMB_DURATION;
                self.invincible_timer = BOMB_INVINCIBILITY;
                self.previous_state = self.game_state;
                self.game_state = GameState::Bombing;

                for bullet in self.enemy_bullets.drain(..) {
                    despawn_entities_with_cache_cleanup(world, &[bullet.entity]);
                }

                for enemy in &mut self.enemies {
                    enemy.health -= 5;
                }
            }
        }

        if self.game_state == GameState::Bombing {
            self.bomb_timer -= delta_time;
            if self.bomb_timer <= 0.0 {
                self.game_state = self.previous_state;
            }
        }
    }

    fn wave_state_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;

        if self.game_state != GameState::GameOver
            && self.center_text_timer > 0.0
            && self.center_text_timer < f32::MAX
        {
            self.center_text_timer -= delta_time;
            if self.center_text_timer <= 0.0 {
                self.center_text_timer = 0.0;
                self.hide_center_text(world);
            }
        }

        self.invincible_timer = (self.invincible_timer - delta_time).max(0.0);

        match self.game_state {
            GameState::WaveIntro => {
                self.state_timer -= delta_time;
                if self.state_timer <= 0.0 {
                    self.game_state = GameState::Playing;
                    self.wave_definition = Some(generate_wave(
                        self.wave_number,
                        self.play_area_half_width,
                        self.play_area_half_height,
                    ));
                    self.wave_spawn_index = 0;
                    self.wave_elapsed = 0.0;
                }
            }
            GameState::BossIntro => {
                self.state_timer -= delta_time;
                if self.state_timer <= 0.0 {
                    self.game_state = GameState::Playing;
                    self.wave_definition = Some(generate_wave(
                        self.wave_number,
                        self.play_area_half_width,
                        self.play_area_half_height,
                    ));
                    self.wave_spawn_index = 0;
                    self.wave_elapsed = 0.0;
                }
            }
            GameState::Playing => {
                let all_spawned = self
                    .wave_definition
                    .as_ref()
                    .map(|wave_definition| self.wave_spawn_index >= wave_definition.spawns.len())
                    .unwrap_or(true);
                let all_dead = self.enemies.is_empty();

                if all_spawned && all_dead {
                    let bonus = 5000 * self.wave_number as u64;
                    self.score += bonus;

                    self.game_state = GameState::WaveClear;
                    self.state_timer = WAVE_CLEAR_DURATION;
                    self.announce(
                        world,
                        &format!("WAVE {} CLEAR!\n+{} BONUS", self.wave_number, bonus),
                        WAVE_CLEAR_DURATION,
                    );
                }
            }
            GameState::WaveClear => {
                self.state_timer -= delta_time;
                if self.state_timer <= 0.0 {
                    self.wave_number += 1;

                    let is_boss = self.wave_number.is_multiple_of(10);
                    let is_miniboss = self.wave_number.is_multiple_of(5) && !is_boss;

                    if is_boss {
                        self.game_state = GameState::BossIntro;
                        self.state_timer = BOSS_INTRO_DURATION;
                        self.announce(
                            world,
                            &format!("WARNING\nBOSS APPROACHING\nWAVE {}", self.wave_number),
                            BOSS_INTRO_DURATION,
                        );
                    } else if is_miniboss {
                        self.game_state = GameState::WaveIntro;
                        self.state_timer = WAVE_INTRO_DURATION;
                        self.announce(
                            world,
                            &format!("MINI-BOSS\nWAVE {}", self.wave_number),
                            WAVE_INTRO_DURATION,
                        );
                    } else {
                        self.game_state = GameState::WaveIntro;
                        self.state_timer = WAVE_INTRO_DURATION;
                        self.announce(
                            world,
                            &format!("WAVE {}", self.wave_number),
                            WAVE_INTRO_DURATION,
                        );
                    }
                }
            }
            GameState::Bombing => {}
            GameState::GameOver => {}
        }
    }

    fn enemy_spawn_system(&mut self, world: &mut World) {
        if self.game_state != GameState::Playing && self.game_state != GameState::Bombing {
            return;
        }

        let delta_time = world.resources.window.timing.delta_time;
        self.wave_elapsed += delta_time;

        let wave_definition = match &self.wave_definition {
            Some(wave_definition) => wave_definition,
            None => return,
        };

        while self.wave_spawn_index < wave_definition.spawns.len() {
            let spawn_entry = &wave_definition.spawns[self.wave_spawn_index];
            if self.wave_elapsed < spawn_entry.delay {
                break;
            }

            let (slot, flip) = match spawn_entry.kind {
                EnemyKind::Black => (SLOT_ENEMY_BLACK, true),
                EnemyKind::Blue => (SLOT_ENEMY_BLUE, true),
                EnemyKind::Red => (SLOT_ENEMY_RED, true),
                EnemyKind::UfoBlue => (SLOT_UFO_BLUE, false),
                EnemyKind::UfoRed => (SLOT_UFO_RED, false),
            };

            let enemy_size = self.texture_size(slot);
            let entity = Self::spawn_game_sprite(
                world,
                spawn_entry.position,
                enemy_size,
                slot,
                LAYER_ENEMIES,
                &self.uv_max_table,
            );

            if flip && let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.flip_y = true;
            }

            let emitters = spawn_entry
                .patterns
                .iter()
                .map(|pattern| PatternEmitter::new(pattern.clone()))
                .collect();

            self.enemies.push(Enemy {
                entity,
                kind: spawn_entry.kind,
                health: spawn_entry.health,
                max_health: spawn_entry.health,
                behavior: spawn_entry.behavior,
                emitters,
                elapsed: 0.0,
                phase: 0,
            });

            self.wave_spawn_index += 1;
        }
    }

    fn enemy_movement_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;

        for enemy in &mut self.enemies {
            enemy.elapsed += delta_time;
            let enemy_pos = entity_position_2d(world, enemy.entity);

            let movement = match enemy.behavior {
                EnemyBehavior::DriftDown { speed } => {
                    nalgebra_glm::Vec2::new(0.0, -speed * delta_time)
                }
                EnemyBehavior::SwoopToY { target_y, speed } => {
                    if enemy_pos.y > target_y {
                        nalgebra_glm::Vec2::new(0.0, -speed * delta_time)
                    } else {
                        nalgebra_glm::Vec2::zeros()
                    }
                }
                EnemyBehavior::HoverAtY {
                    target_y,
                    speed,
                    sway_amplitude,
                    sway_frequency,
                } => {
                    let vertical = if enemy_pos.y > target_y + 5.0 {
                        -speed * delta_time
                    } else if enemy_pos.y < target_y - 5.0 {
                        speed * delta_time
                    } else {
                        0.0
                    };
                    let horizontal =
                        (enemy.elapsed * sway_frequency).cos() * sway_amplitude * delta_time;
                    nalgebra_glm::Vec2::new(horizontal, vertical)
                }
                EnemyBehavior::CircleAroundPoint {
                    center_x,
                    center_y,
                    radius,
                    angular_speed,
                } => {
                    let angle = enemy.elapsed * angular_speed;
                    let target = nalgebra_glm::Vec2::new(
                        center_x + angle.cos() * radius,
                        center_y + angle.sin() * radius,
                    );
                    let to_target = target - enemy_pos;
                    let distance = nalgebra_glm::length(&to_target);
                    if distance > 1.0 {
                        (to_target / distance) * 200.0 * delta_time
                    } else {
                        nalgebra_glm::Vec2::zeros()
                    }
                }
            };

            translate_entity_2d(world, enemy.entity, movement);
        }
    }

    fn enemy_pattern_system(&mut self, world: &mut World) {
        if self.game_state != GameState::Playing {
            return;
        }

        let delta_time = world.resources.window.timing.delta_time;
        let player_position = self.player_position;
        let mut new_bullets = Vec::new();

        for enemy in &mut self.enemies {
            let enemy_pos = entity_position_2d(world, enemy.entity);

            if enemy_pos.y > self.play_area_half_height + 50.0
                || enemy_pos.y < -self.play_area_half_height - 50.0
            {
                continue;
            }

            if enemy.kind == EnemyKind::UfoRed {
                let health_ratio = enemy.health as f32 / enemy.max_health.max(1) as f32;
                let new_phase = if health_ratio > 0.6 {
                    0
                } else if health_ratio > 0.3 {
                    1
                } else {
                    2
                };

                if new_phase != enemy.phase {
                    enemy.phase = new_phase;
                    for emitter in &mut enemy.emitters {
                        emitter.reset();
                    }
                }
            }

            let active_emitter_indices: Vec<usize> = match enemy.kind {
                EnemyKind::UfoRed => match enemy.phase {
                    0 => vec![0, 1],
                    1 => vec![2, 3, 4],
                    _ => (0..enemy.emitters.len()).collect(),
                },
                _ => (0..enemy.emitters.len()).collect(),
            };

            for emitter_index in active_emitter_indices {
                if emitter_index < enemy.emitters.len() {
                    let salvo = evaluate_pattern_salvo(
                        &mut enemy.emitters[emitter_index],
                        delta_time,
                        enemy_pos,
                        player_position,
                    );
                    new_bullets.extend(salvo);
                }
            }
        }

        for (position, velocity, color) in new_bullets {
            let slot = match color {
                BulletColor::Red => SLOT_LASER_RED,
                BulletColor::Blue => SLOT_LASER_BLUE,
            };
            let bullet_size = nalgebra_glm::Vec2::new(16.0, 16.0);
            let entity = Self::spawn_game_sprite(
                world,
                position,
                bullet_size,
                slot,
                LAYER_ENEMY_BULLETS,
                &self.uv_max_table,
            );

            if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.blend_mode = SpriteBlendMode::Additive;
                if color == BulletColor::Blue {
                    sprite.color = [0.3, 0.6, 1.0, 1.0];
                }
            }

            self.enemy_bullets.push(EnemyBullet {
                entity,
                velocity,
                position,
                radius: ENEMY_BULLET_RADIUS,
                grazed: false,
            });
        }
    }

    fn enemy_bullet_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;
        let mut to_remove = Vec::new();

        for (index, bullet) in self.enemy_bullets.iter_mut().enumerate() {
            bullet.position += bullet.velocity * delta_time;
            set_entity_position_2d(world, bullet.entity, bullet.position);

            if bullet.position.x > self.play_area_half_width + DESPAWN_MARGIN
                || bullet.position.x < -self.play_area_half_width - DESPAWN_MARGIN
                || bullet.position.y > self.play_area_half_height + DESPAWN_MARGIN
                || bullet.position.y < -self.play_area_half_height - DESPAWN_MARGIN
            {
                to_remove.push(index);
            }
        }

        for index in to_remove.into_iter().rev() {
            let bullet = self.enemy_bullets.remove(index);
            despawn_entities_with_cache_cleanup(world, &[bullet.entity]);
        }
    }

    fn collision_player_bullets_vs_enemies(&mut self, world: &mut World) {
        if self.game_state != GameState::Playing && self.game_state != GameState::Bombing {
            return;
        }

        let mut bullets_to_remove = Vec::new();
        let mut enemies_to_remove = Vec::new();
        let mut spawn_explosions = Vec::new();
        let mut spawn_powerups_list = Vec::new();
        let mut score_gain = 0u64;

        let bullet_radius = 6.0;
        let enemy_radii: std::collections::HashMap<EnemyKind, f32> = [
            (EnemyKind::Black, 20.0),
            (EnemyKind::Blue, 25.0),
            (EnemyKind::Red, 25.0),
            (EnemyKind::UfoBlue, 35.0),
            (EnemyKind::UfoRed, 40.0),
        ]
        .into_iter()
        .collect();

        for (bullet_index, bullet) in self.player_bullets.iter().enumerate() {
            let bullet_pos = entity_position_2d(world, bullet.entity);

            for (enemy_index, enemy) in self.enemies.iter_mut().enumerate() {
                if enemies_to_remove.contains(&enemy_index) {
                    continue;
                }

                let enemy_pos = entity_position_2d(world, enemy.entity);
                let enemy_radius = enemy_radii.get(&enemy.kind).copied().unwrap_or(20.0);

                if circle_overlap(bullet_pos, bullet_radius, enemy_pos, enemy_radius) {
                    bullets_to_remove.push(bullet_index);
                    enemy.health -= 1;

                    if enemy.health <= 0 {
                        enemies_to_remove.push(enemy_index);
                        spawn_explosions.push(enemy_pos);

                        let points: u64 = match enemy.kind {
                            EnemyKind::Black => 100,
                            EnemyKind::Blue => 200,
                            EnemyKind::Red => 500,
                            EnemyKind::UfoBlue => 1500,
                            EnemyKind::UfoRed => 5000,
                        };
                        score_gain += points;

                        let mut rng = rand::rng();
                        let roll: f32 = rand::Rng::random_range(&mut rng, 0.0..1.0);
                        if roll < POWERUP_DROP_CHANCE {
                            let kind = if rand::Rng::random_bool(&mut rng, 0.6) {
                                PowerupKind::Power
                            } else {
                                PowerupKind::Bomb
                            };
                            spawn_powerups_list.push((enemy_pos, kind));
                        }
                    }
                    break;
                }
            }
        }

        self.score += score_gain;

        bullets_to_remove.sort_unstable();
        bullets_to_remove.dedup();
        for index in bullets_to_remove.into_iter().rev() {
            let bullet = self.player_bullets.remove(index);
            despawn_entities_with_cache_cleanup(world, &[bullet.entity]);
        }

        enemies_to_remove.sort_unstable();
        enemies_to_remove.dedup();
        for index in enemies_to_remove.into_iter().rev() {
            let enemy = self.enemies.remove(index);
            despawn_entities_with_cache_cleanup(world, &[enemy.entity]);
        }

        for position in spawn_explosions {
            self.spawn_explosion(world, position);
        }

        for (position, kind) in spawn_powerups_list {
            self.spawn_powerup(world, position, kind);
        }
    }

    fn collision_enemy_bullets_vs_player(&mut self, world: &mut World) {
        if self.game_state != GameState::Playing {
            return;
        }
        if self.invincible_timer > 0.0 {
            return;
        }

        let player_pos = self.player_position;
        let mut bullets_to_remove = Vec::new();
        let mut hit = false;

        for (index, bullet) in self.enemy_bullets.iter().enumerate() {
            if circle_overlap(
                player_pos,
                PLAYER_HITBOX_RADIUS,
                bullet.position,
                bullet.radius,
            ) {
                hit = true;
                bullets_to_remove.push(index);
                break;
            }
        }

        for index in bullets_to_remove.into_iter().rev() {
            let bullet = self.enemy_bullets.remove(index);
            despawn_entities_with_cache_cleanup(world, &[bullet.entity]);
        }

        if hit {
            self.take_damage(world);
        }
    }

    fn graze_system(&mut self, world: &mut World) {
        if self.game_state != GameState::Playing {
            return;
        }
        if self.invincible_timer > 0.0 {
            return;
        }

        let player_pos = self.player_position;
        let mut flash_positions = Vec::new();

        for bullet in &mut self.enemy_bullets {
            if bullet.grazed {
                continue;
            }

            let within_graze =
                circle_overlap(player_pos, GRAZE_RADIUS, bullet.position, bullet.radius);
            let within_hit = circle_overlap(
                player_pos,
                PLAYER_HITBOX_RADIUS,
                bullet.position,
                bullet.radius,
            );

            if within_graze && !within_hit {
                bullet.grazed = true;
                self.graze_count += 1;
                self.multiplier = (self.multiplier + 0.01).min(5.0);
                self.score += (100.0 * self.multiplier) as u64;
                flash_positions.push(bullet.position);
            }
        }

        for position in flash_positions {
            self.spawn_graze_flash(world, position);
        }
    }

    fn collision_enemy_vs_player(&mut self, world: &mut World) {
        if self.game_state != GameState::Playing {
            return;
        }
        if self.invincible_timer > 0.0 {
            return;
        }

        let player_pos = self.player_position;
        let mut hit = false;
        let mut enemies_to_remove = Vec::new();

        let enemy_radii: std::collections::HashMap<EnemyKind, f32> = [
            (EnemyKind::Black, 20.0),
            (EnemyKind::Blue, 25.0),
            (EnemyKind::Red, 25.0),
            (EnemyKind::UfoBlue, 35.0),
            (EnemyKind::UfoRed, 40.0),
        ]
        .into_iter()
        .collect();

        for (index, enemy) in self.enemies.iter().enumerate() {
            let enemy_pos = entity_position_2d(world, enemy.entity);
            let enemy_radius = enemy_radii.get(&enemy.kind).copied().unwrap_or(20.0);

            if circle_overlap(player_pos, PLAYER_HITBOX_RADIUS, enemy_pos, enemy_radius) {
                hit = true;
                enemies_to_remove.push(index);
                self.spawn_explosion(world, enemy_pos);
                break;
            }
        }

        for index in enemies_to_remove.into_iter().rev() {
            let enemy = self.enemies.remove(index);
            despawn_entities_with_cache_cleanup(world, &[enemy.entity]);
        }

        if hit {
            self.take_damage(world);
        }
    }

    fn take_damage(&mut self, world: &mut World) {
        self.lives = self.lives.saturating_sub(1);
        self.invincible_timer = INVINCIBILITY_DURATION;
        self.multiplier = 1.0;
        self.spawn_explosion(world, self.player_position);

        self.shake_timer = 0.3;
        self.shake_intensity = 8.0;

        if self.bombs < 3 {
            self.bombs = 3;
        }

        if self.lives == 0 {
            self.game_state = GameState::GameOver;
            let text = format!(
                "GAME OVER\nScore: {}\nWave: {}\nGraze: {}\nPress R or Start to restart",
                self.score, self.wave_number, self.graze_count
            );
            self.announce(world, &text, f32::MAX);
        } else {
            let text = format!("{} LIVES LEFT", self.lives);
            self.announce(world, &text, 1.5);
        }
    }

    fn spawn_explosion(&mut self, world: &mut World, position: nalgebra_glm::Vec2) {
        let entity = world.spawn();
        world.sprite2d.set_sprite_particle_emitter(
            entity,
            SpriteParticleEmitter::explosion(position.x, position.y).with_depth(LAYER_EXPLOSIONS),
        );
        self.explosions.push(Explosion {
            entity,
            lifetime: 1.0,
        });

        self.shake_timer = self.shake_timer.max(0.15);
        self.shake_intensity = self.shake_intensity.max(4.0);
    }

    fn spawn_graze_flash(&mut self, world: &mut World, position: nalgebra_glm::Vec2) {
        let entity = world.spawn();
        world.sprite2d.set_sprite_particle_emitter(
            entity,
            SpriteParticleEmitter::sparks(position.x, position.y)
                .with_depth(LAYER_EXPLOSIONS)
                .with_color(ColorRange2D::new(
                    [0.0, 1.0, 1.0, 1.0],
                    [0.0, 0.5, 1.0, 0.0],
                ))
                .with_size(
                    nalgebra_glm::Vec2::new(4.0, 4.0),
                    nalgebra_glm::Vec2::new(1.0, 1.0),
                ),
        );
        self.graze_flashes.push(GrazeFlash {
            entity,
            lifetime: GRAZE_FLASH_DURATION,
        });
    }

    fn explosion_cleanup_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;
        let mut to_remove = Vec::new();

        for (index, explosion) in self.explosions.iter_mut().enumerate() {
            explosion.lifetime -= delta_time;
            if explosion.lifetime <= 0.0 {
                to_remove.push(index);
            }
        }

        for index in to_remove.into_iter().rev() {
            let explosion = self.explosions.remove(index);
            despawn_entities_with_cache_cleanup(world, &[explosion.entity]);
        }
    }

    fn graze_flash_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;
        let mut to_remove = Vec::new();

        for (index, flash) in self.graze_flashes.iter_mut().enumerate() {
            flash.lifetime -= delta_time;
            if flash.lifetime <= 0.0 {
                to_remove.push(index);
            }
        }

        for index in to_remove.into_iter().rev() {
            let flash = self.graze_flashes.remove(index);
            despawn_entities_with_cache_cleanup(world, &[flash.entity]);
        }
    }

    fn spawn_powerup(
        &mut self,
        world: &mut World,
        position: nalgebra_glm::Vec2,
        kind: PowerupKind,
    ) {
        let slot = match kind {
            PowerupKind::Power => SLOT_POWERUP_POWER,
            PowerupKind::Bomb => SLOT_POWERUP_BOMB,
        };
        let powerup_size = self.texture_size(slot);
        let entity = Self::spawn_game_sprite(
            world,
            position,
            powerup_size,
            slot,
            LAYER_POWERUPS,
            &self.uv_max_table,
        );

        self.powerups.push(Powerup {
            entity,
            kind,
            lifetime: POWERUP_LIFETIME,
            bob_phase: 0.0,
            base_y: position.y,
        });
    }

    fn powerup_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;
        let player_pos = self.player_position;
        let mut to_remove = Vec::new();
        let mut collected = Vec::new();

        for (index, powerup) in self.powerups.iter_mut().enumerate() {
            powerup.lifetime -= delta_time;
            if powerup.lifetime <= 0.0 {
                to_remove.push(index);
                continue;
            }

            powerup.bob_phase += delta_time * 3.0;

            if let Some(sprite) = world.sprite2d.get_sprite_mut(powerup.entity) {
                sprite.position.y = powerup.base_y + powerup.bob_phase.sin() * 10.0;
            }

            if powerup.lifetime < 3.0
                && let Some(sprite) = world.sprite2d.get_sprite_mut(powerup.entity)
            {
                let blink = ((powerup.lifetime * 8.0).sin() + 1.0) * 0.5;
                sprite.color[3] = 0.3 + blink * 0.7;
            }

            let powerup_pos = entity_position_2d(world, powerup.entity);
            if circle_overlap(player_pos, 30.0, powerup_pos, 15.0) {
                collected.push((index, powerup.kind));
            }
        }

        for &(index, kind) in collected.iter().rev() {
            match kind {
                PowerupKind::Power => {
                    self.power_level = (self.power_level + 1).min(4);
                }
                PowerupKind::Bomb => {
                    self.bombs = (self.bombs + 1).min(5);
                }
            }
            to_remove.push(index);
        }

        to_remove.sort_unstable();
        to_remove.dedup();
        for index in to_remove.into_iter().rev() {
            let powerup = self.powerups.remove(index);
            despawn_entities_with_cache_cleanup(world, &[powerup.entity]);
        }
    }

    fn screen_shake_system(&mut self, world: &mut World) {
        if self.shake_timer <= 0.0 {
            if let Some(camera_entity) = self.camera_entity {
                if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
                    transform.translation.x = 0.0;
                    transform.translation.y = 0.0;
                    transform.translation.z = CAMERA_Z;
                }
                mark_local_transform_dirty(world, camera_entity);
            }
            return;
        }

        let delta_time = world.resources.window.timing.delta_time;
        self.shake_timer -= delta_time;
        self.shake_intensity *= (1.0 - SCREEN_SHAKE_DECAY * delta_time).max(0.0);

        if let Some(camera_entity) = self.camera_entity {
            let (offset_x, offset_y) = if self.shake_intensity > 0.001 {
                use rand::Rng;
                let mut rng = rand::rng();
                (
                    rng.random_range(-self.shake_intensity..self.shake_intensity),
                    rng.random_range(-self.shake_intensity..self.shake_intensity),
                )
            } else {
                (0.0, 0.0)
            };

            if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
                transform.translation.x = offset_x;
                transform.translation.y = offset_y;
                transform.translation.z = CAMERA_Z;
            }
            mark_local_transform_dirty(world, camera_entity);
        }
    }

    fn meteor_system(&mut self, world: &mut World) {
        use rand::Rng;
        let delta_time = world.resources.window.timing.delta_time;

        for meteor in &mut self.meteors {
            translate_entity_2d(world, meteor.entity, meteor.velocity * delta_time);

            let meteor_pos = entity_position_2d(world, meteor.entity);
            let out_of_bounds = meteor_pos.x.abs() > self.play_area_half_width + 200.0
                || meteor_pos.y.abs() > self.play_area_half_height + 200.0;

            if out_of_bounds {
                let mut rng = rand::rng();
                let new_pos = nalgebra_glm::Vec2::new(
                    rng.random_range(-self.play_area_half_width..self.play_area_half_width),
                    self.play_area_half_height + 100.0,
                );
                set_entity_position_2d(world, meteor.entity, new_pos);
                meteor.velocity = nalgebra_glm::Vec2::new(
                    rng.random_range(-15.0..15.0),
                    rng.random_range(-30.0..-10.0),
                );
            }
        }
    }

    fn despawn_offscreen_enemies(&mut self, world: &mut World) {
        let mut to_remove = Vec::new();

        for (index, enemy) in self.enemies.iter().enumerate() {
            let enemy_pos = entity_position_2d(world, enemy.entity);
            if enemy_pos.y < -self.play_area_half_height - 200.0 {
                to_remove.push(index);
            }
        }

        for index in to_remove.into_iter().rev() {
            let enemy = self.enemies.remove(index);
            despawn_entities_with_cache_cleanup(world, &[enemy.entity]);
        }
    }

    fn extra_life_system(&mut self, world: &mut World) {
        if self.score >= self.next_extra_life_score {
            self.lives += 1;
            self.next_extra_life_score += EXTRA_LIFE_SCORE_INTERVAL;
            self.announce(world, "EXTRA LIFE!", 1.5);
        }
    }

    fn restart_system(&mut self, world: &mut World) {
        if self.game_state != GameState::GameOver {
            return;
        }

        let mut restart = world.resources.input.keyboard.is_key_pressed(KeyCode::KeyR);
        if let Some(gamepad) = query_active_gamepad(world)
            && gamepad.is_pressed(gilrs::Button::Start)
        {
            restart = true;
        }

        if restart {
            for bullet in self.player_bullets.drain(..) {
                despawn_entities_with_cache_cleanup(world, &[bullet.entity]);
            }
            for bullet in self.enemy_bullets.drain(..) {
                despawn_entities_with_cache_cleanup(world, &[bullet.entity]);
            }
            for enemy in self.enemies.drain(..) {
                despawn_entities_with_cache_cleanup(world, &[enemy.entity]);
            }
            for explosion in self.explosions.drain(..) {
                despawn_entities_with_cache_cleanup(world, &[explosion.entity]);
            }
            for flash in self.graze_flashes.drain(..) {
                despawn_entities_with_cache_cleanup(world, &[flash.entity]);
            }
            for powerup in self.powerups.drain(..) {
                despawn_entities_with_cache_cleanup(world, &[powerup.entity]);
            }

            self.player_position = nalgebra_glm::Vec2::new(0.0, -self.play_area_half_height * 0.7);
            if let Some(entity) = self.player_entity {
                set_entity_position_2d(world, entity, self.player_position);
                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.color[3] = 1.0;
                }
            }

            if let Some(exhaust_entity) = self.engine_exhaust_entity {
                let player_size = self.texture_size(SLOT_PLAYER);
                let exhaust_position = nalgebra_glm::Vec2::new(
                    self.player_position.x,
                    self.player_position.y - player_size.y * 0.45,
                );
                if let Some(emitter) = world
                    .sprite2d
                    .get_sprite_particle_emitter_mut(exhaust_entity)
                {
                    emitter.anchor = exhaust_position;
                }
            }

            if let Some(camera_entity) = self.camera_entity {
                if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
                    transform.translation.x = 0.0;
                    transform.translation.y = 0.0;
                    transform.translation.z = CAMERA_Z;
                }
                mark_local_transform_dirty(world, camera_entity);
            }

            self.player_tilt_angle = 0.0;
            self.barrel_roll_active = false;
            self.barrel_roll_timer = 0.0;
            self.barrel_roll_direction = 1.0;
            self.last_left_tap_time = -1.0;
            self.last_right_tap_time = -1.0;

            self.score = 0;
            self.multiplier = 1.0;
            self.graze_count = 0;
            self.next_extra_life_score = EXTRA_LIFE_SCORE_INTERVAL;
            self.lives = 3;
            self.bombs = 3;
            self.wave_number = 1;
            self.power_level = 0;
            self.fire_cooldown = 0.0;
            self.invincible_timer = 0.0;
            self.focused = false;
            self.bomb_timer = 0.0;
            self.shake_timer = 0.0;
            self.shake_intensity = 0.0;
            self.wave_definition = None;
            self.wave_spawn_index = 0;
            self.wave_elapsed = 0.0;
            self.game_state = GameState::WaveIntro;
            self.previous_state = GameState::WaveIntro;
            self.state_timer = WAVE_INTRO_DURATION;
            self.announce(
                world,
                &format!("WAVE {}", self.wave_number),
                WAVE_INTRO_DURATION,
            );
        }
    }

    fn escape_exit_system(&self, world: &mut World) {
        let mut should_exit = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Escape);
        if let Some(gamepad) = query_active_gamepad(world)
            && gamepad.is_pressed(gilrs::Button::Select)
        {
            should_exit = true;
        }
        if should_exit {
            std::process::exit(0);
        }
    }
}

impl State for BulletHell {
    fn title(&self) -> &str {
        "Bullet Hell"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.clear_color = [0.02, 0.02, 0.08, 1.0];
        world.resources.graphics.atmosphere = Atmosphere::Space;
        world.resources.graphics.show_grid = false;

        let (pixel_sizes, uv_max_table) = load_textures(world);
        self.texture_sizes = pixel_sizes;
        self.uv_max_table = uv_max_table;

        let camera = world.spawn_entities(
            nightshade::ecs::world::NAME
                | nightshade::ecs::world::LOCAL_TRANSFORM
                | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
                | nightshade::ecs::world::GLOBAL_TRANSFORM
                | nightshade::ecs::world::CAMERA,
            1,
        )[0];
        world
            .core
            .set_name(camera, Name("BulletHellCamera".to_string()));
        world.core.set_local_transform(
            camera,
            LocalTransform {
                translation: nalgebra_glm::Vec3::new(0.0, 0.0, CAMERA_Z),
                rotation: Quat::identity(),
                scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world
            .core
            .set_local_transform_dirty(camera, LocalTransformDirty);
        world
            .core
            .set_global_transform(camera, GlobalTransform::default());
        if let Some(camera_component) = world.core.get_camera_mut(camera) {
            *camera_component = Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 45.0_f32.to_radians(),
                    z_far: Some(2000.0),
                    z_near: 0.1,
                }),
                smoothing: None,
            };
        }
        world.resources.active_camera = Some(camera);
        self.camera_entity = Some(camera);

        if let Some(window_handle) = &world.resources.window.handle {
            let size = window_handle.inner_size();
            let (half_width, half_height) = visible_half_extents(size.width, size.height);
            self.play_area_half_width = half_width;
            self.play_area_half_height = half_height;
        }
        self.player_position = nalgebra_glm::Vec2::new(0.0, -self.play_area_half_height * 0.7);

        self.spawn_background(world);
        self.spawn_star_field(world);
        self.spawn_meteors(world);
        self.spawn_player(world);
        self.spawn_hud(world);
        self.announce(
            world,
            &format!("WAVE {}", self.wave_number),
            WAVE_INTRO_DURATION,
        );
    }

    fn run_systems(&mut self, world: &mut World) {
        if let Some((width, height)) = world.resources.window.cached_viewport_size {
            let (half_width, half_height) = visible_half_extents(width, height);
            self.play_area_half_width = half_width;
            self.play_area_half_height = half_height;
        }

        self.game_time += world.resources.window.timing.delta_time;

        self.escape_exit_system(world);
        self.player_input_system(world);
        self.player_shooting_system(world);
        self.player_bullet_system(world);
        self.bomb_system(world);
        self.wave_state_system(world);
        self.enemy_spawn_system(world);
        self.enemy_movement_system(world);
        self.enemy_pattern_system(world);
        self.enemy_bullet_system(world);
        self.collision_player_bullets_vs_enemies(world);
        self.collision_enemy_bullets_vs_player(world);
        self.graze_system(world);
        self.collision_enemy_vs_player(world);
        self.powerup_system(world);
        self.explosion_cleanup_system(world);
        self.graze_flash_system(world);
        self.screen_shake_system(world);
        self.meteor_system(world);
        self.despawn_offscreen_enemies(world);
        self.extra_life_system(world);
        self.update_hud_text(world);
        self.restart_system(world);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    nightshade::run::launch(BulletHell::default())
}
