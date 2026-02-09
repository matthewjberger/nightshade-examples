use image::GenericImageView;
use nightshade::ecs::transform::commands::mark_local_transform_dirty;
use nightshade::prelude::*;
use rand::Rng;

const GRAVITY: f32 = 800.0;
const FLAP_STRENGTH: f32 = 350.0;
const SCROLL_SPEED: f32 = 200.0;
const ROCK_SPACING: f32 = 300.0;
const GAP_SIZE: f32 = 200.0;
const GAP_MIN_Y: f32 = -100.0;
const GAP_MAX_Y: f32 = 200.0;
const GROUND_Y: f32 = -240.0;
const GROUND_HEIGHT: f32 = 71.0;
const PLANE_X: f32 = -200.0;
const ANGLE_FACTOR: f32 = 0.08;
const MAX_ANGLE: f32 = 0.5;
const MIN_ANGLE: f32 = -0.8;
const ROCK_WIDTH: f32 = 108.0;
const ROCK_HEIGHT: f32 = 239.0;
const PLANE_WIDTH: f32 = 88.0;
const PLANE_HEIGHT: f32 = 73.0;
const PLANE_HITBOX_W: f32 = 60.0;
const PLANE_HITBOX_H: f32 = 40.0;
const BG_WIDTH: f32 = 800.0;
const BG_HEIGHT: f32 = 480.0;
const GROUND_TILE_WIDTH: f32 = 808.0;
const STAR_RADIUS: f32 = 18.0;
const SMOKE_LIFETIME: f32 = 0.5;
const SMOKE_SPAWN_INTERVAL: f32 = 0.05;
const ANIM_FRAME_DURATION: f32 = 0.08;

const SLOT_BACKGROUND: u32 = 0;
const SLOT_GROUND: u32 = 1;
const SLOT_PLANE1: u32 = 2;
const SLOT_PLANE2: u32 = 3;
const SLOT_PLANE3: u32 = 4;
const SLOT_ROCK_BOTTOM: u32 = 5;
const SLOT_ROCK_TOP: u32 = 6;
const SLOT_STAR: u32 = 7;
const SLOT_SMOKE: u32 = 8;

const LAYER_BG: f32 = 0.0;
const LAYER_ROCKS: f32 = 1.0;
const LAYER_STARS: f32 = 1.5;
const LAYER_GROUND: f32 = 2.0;
const LAYER_SMOKE: f32 = 2.5;
const LAYER_PLANE: f32 = 3.0;

struct TextureEntry {
    slot: u32,
    bytes: &'static [u8],
}

fn load_textures(world: &mut World) -> Vec<Vec2> {
    let entries = [
        TextureEntry {
            slot: SLOT_BACKGROUND,
            bytes: include_bytes!("../assets/background.png"),
        },
        TextureEntry {
            slot: SLOT_GROUND,
            bytes: include_bytes!("../assets/ground.png"),
        },
        TextureEntry {
            slot: SLOT_PLANE1,
            bytes: include_bytes!("../assets/plane1.png"),
        },
        TextureEntry {
            slot: SLOT_PLANE2,
            bytes: include_bytes!("../assets/plane2.png"),
        },
        TextureEntry {
            slot: SLOT_PLANE3,
            bytes: include_bytes!("../assets/plane3.png"),
        },
        TextureEntry {
            slot: SLOT_ROCK_BOTTOM,
            bytes: include_bytes!("../assets/rock_bottom.png"),
        },
        TextureEntry {
            slot: SLOT_ROCK_TOP,
            bytes: include_bytes!("../assets/rock_top.png"),
        },
        TextureEntry {
            slot: SLOT_STAR,
            bytes: include_bytes!("../assets/star.png"),
        },
        TextureEntry {
            slot: SLOT_SMOKE,
            bytes: include_bytes!("../assets/smoke.png"),
        },
    ];

    let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
    let mut uv_max_table = vec![Vec2::new(1.0, 1.0); 128];

    for entry in &entries {
        let img = image::load_from_memory(entry.bytes).expect("Failed to decode image");
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8().into_raw();

        world
            .resources
            .command_queue
            .push(WorldCommand::UploadSpriteTexture {
                slot: entry.slot,
                rgba_data: rgba,
                width,
                height,
            });

        let half_texel_x = 0.5 / atlas_slot_size.0 as f32;
        let half_texel_y = 0.5 / atlas_slot_size.1 as f32;
        uv_max_table[entry.slot as usize] = Vec2::new(
            width as f32 / atlas_slot_size.0 as f32 - half_texel_x,
            height as f32 / atlas_slot_size.1 as f32 - half_texel_y,
        );
    }

    uv_max_table
}

fn uv_for_slot(uv_max_table: &[Vec2], slot: u32) -> (Vec2, Vec2) {
    let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
    let half_texel = Vec2::new(
        0.5 / atlas_slot_size.0 as f32,
        0.5 / atlas_slot_size.1 as f32,
    );
    (half_texel, uv_max_table[slot as usize])
}

fn spawn_textured_sprite(
    world: &mut World,
    position: Vec3,
    size: Vec2,
    texture_slot: u32,
    uv_max_table: &[Vec2],
) -> Entity {
    let entity = spawn_sprite(world, position, size);
    let (uv_min, uv_max) = uv_for_slot(uv_max_table, texture_slot);
    if let Some(sprite) = world.get_sprite_mut(entity) {
        sprite.texture_index = texture_slot;
        sprite.texture_index2 = texture_slot;
        sprite.uv_min = uv_min;
        sprite.uv_max = uv_max;
    }
    entity
}

fn set_sprite_texture(world: &mut World, entity: Entity, slot: u32, uv_max_table: &[Vec2]) {
    let (uv_min, uv_max) = uv_for_slot(uv_max_table, slot);
    if let Some(sprite) = world.get_sprite_mut(entity) {
        sprite.texture_index = slot;
        sprite.texture_index2 = slot;
        sprite.uv_min = uv_min;
        sprite.uv_max = uv_max;
    }
}

struct RockPair {
    bottom_entity: Entity,
    top_entity: Entity,
    x: f32,
    gap_y: f32,
    gap_size: f32,
    scored: bool,
}

struct Star {
    entity: Entity,
    x: f32,
    y: f32,
    collected: bool,
}

struct Smoke {
    entity: Entity,
    lifetime: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum GamePhase {
    WaitingToStart,
    Playing,
    GameOver,
}

struct TappyPlane {
    camera_entity: Option<Entity>,
    uv_max_table: Vec<Vec2>,
    initialized: bool,

    plane_entity: Option<Entity>,
    plane_y: f32,
    plane_velocity: f32,
    plane_angle: f32,
    plane_anim_timer: f32,
    plane_anim_frame: u32,

    scroll_x: f32,
    next_rock_x: f32,
    rocks: Vec<RockPair>,
    stars: Vec<Star>,
    smokes: Vec<Smoke>,
    smoke_timer: f32,

    background_entities: Vec<Entity>,
    ground_entities: Vec<Entity>,

    score: u32,
    best_score: u32,
    phase: GamePhase,

    score_hud: Option<Entity>,
    message_hud: Option<Entity>,
    best_hud: Option<Entity>,
}

impl Default for TappyPlane {
    fn default() -> Self {
        Self {
            camera_entity: None,
            uv_max_table: Vec::new(),
            initialized: false,
            plane_entity: None,
            plane_y: 0.0,
            plane_velocity: 0.0,
            plane_angle: 0.0,
            plane_anim_timer: 0.0,
            plane_anim_frame: 0,
            scroll_x: 0.0,
            next_rock_x: 400.0,
            rocks: Vec::new(),
            stars: Vec::new(),
            smokes: Vec::new(),
            smoke_timer: 0.0,
            background_entities: Vec::new(),
            ground_entities: Vec::new(),
            score: 0,
            best_score: 0,
            phase: GamePhase::WaitingToStart,
            score_hud: None,
            message_hud: None,
            best_hud: None,
        }
    }
}

impl TappyPlane {
    fn setup_scene(&mut self, world: &mut World) {
        for index in 0..4 {
            let entity = spawn_textured_sprite(
                world,
                Vec3::new((index as f32 - 1.0) * BG_WIDTH, 0.0, LAYER_BG),
                Vec2::new(BG_WIDTH, BG_HEIGHT),
                SLOT_BACKGROUND,
                &self.uv_max_table,
            );
            self.background_entities.push(entity);
        }

        for index in 0..5 {
            let entity = spawn_textured_sprite(
                world,
                Vec3::new(
                    (index as f32 - 1.0) * GROUND_TILE_WIDTH,
                    GROUND_Y,
                    LAYER_GROUND,
                ),
                Vec2::new(GROUND_TILE_WIDTH, GROUND_HEIGHT),
                SLOT_GROUND,
                &self.uv_max_table,
            );
            self.ground_entities.push(entity);
        }

        let plane_entity = spawn_textured_sprite(
            world,
            Vec3::new(PLANE_X, self.plane_y, LAYER_PLANE),
            Vec2::new(PLANE_WIDTH, PLANE_HEIGHT),
            SLOT_PLANE1,
            &self.uv_max_table,
        );
        self.plane_entity = Some(plane_entity);
    }

    fn input_system(&mut self, world: &mut World) {
        let space_pressed = world
            .resources
            .input
            .keyboard
            .frame_keys
            .iter()
            .any(|(key, pressed)| *key == KeyCode::Space && *pressed);
        let mouse_clicked = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_JUST_PRESSED);

        let flap = space_pressed || mouse_clicked;

        match self.phase {
            GamePhase::WaitingToStart => {
                if flap {
                    self.phase = GamePhase::Playing;
                    self.plane_velocity = FLAP_STRENGTH;
                }
            }
            GamePhase::Playing => {
                if flap {
                    self.plane_velocity = FLAP_STRENGTH;
                }
            }
            GamePhase::GameOver => {
                let restart =
                    world
                        .resources
                        .input
                        .keyboard
                        .frame_keys
                        .iter()
                        .any(|(key, pressed)| {
                            (*key == KeyCode::Space || *key == KeyCode::KeyR) && *pressed
                        });
                if restart {
                    self.restart(world);
                }
            }
        }
    }

    fn physics_system(&mut self, delta_time: f32) {
        if self.phase != GamePhase::Playing {
            return;
        }

        self.plane_velocity -= GRAVITY * delta_time;
        self.plane_y += self.plane_velocity * delta_time;

        self.plane_angle = (self.plane_velocity * ANGLE_FACTOR).clamp(MIN_ANGLE, MAX_ANGLE);
    }

    fn scroll_system(&mut self, world: &mut World, delta_time: f32) {
        if self.phase != GamePhase::Playing {
            return;
        }

        self.scroll_x += SCROLL_SPEED * delta_time;

        if let Some(camera_entity) = self.camera_entity {
            if let Some(transform) = world.get_local_transform_mut(camera_entity) {
                transform.translation.x = self.scroll_x;
            }
            mark_local_transform_dirty(world, camera_entity);
        }
    }

    fn rock_generation_system(&mut self, world: &mut World) {
        if self.phase != GamePhase::Playing {
            return;
        }

        let camera_right = self.scroll_x + 900.0;

        while self.next_rock_x < camera_right {
            let mut rng = rand::rng();
            let gap_y = rng.random_range(GAP_MIN_Y..GAP_MAX_Y);
            let current_gap = (GAP_SIZE - self.score as f32 * 2.0).max(140.0);
            let half_gap = current_gap / 2.0;

            let bottom_y = gap_y - half_gap - ROCK_HEIGHT / 2.0;
            let top_y = gap_y + half_gap + ROCK_HEIGHT / 2.0;

            let bottom_entity = spawn_textured_sprite(
                world,
                Vec3::new(self.next_rock_x, bottom_y, LAYER_ROCKS),
                Vec2::new(ROCK_WIDTH, ROCK_HEIGHT),
                SLOT_ROCK_BOTTOM,
                &self.uv_max_table,
            );

            let top_entity = spawn_textured_sprite(
                world,
                Vec3::new(self.next_rock_x, top_y, LAYER_ROCKS),
                Vec2::new(ROCK_WIDTH, ROCK_HEIGHT),
                SLOT_ROCK_TOP,
                &self.uv_max_table,
            );

            self.rocks.push(RockPair {
                bottom_entity,
                top_entity,
                x: self.next_rock_x,
                gap_y,
                gap_size: current_gap,
                scored: false,
            });

            if rng.random_range(0..3) == 0 {
                let star_entity = spawn_textured_sprite(
                    world,
                    Vec3::new(self.next_rock_x, gap_y, LAYER_STARS),
                    Vec2::new(36.0, 34.0),
                    SLOT_STAR,
                    &self.uv_max_table,
                );
                self.stars.push(Star {
                    entity: star_entity,
                    x: self.next_rock_x,
                    y: gap_y,
                    collected: false,
                });
            }

            self.next_rock_x += ROCK_SPACING;
        }
    }

    fn rock_cleanup_system(&mut self, world: &mut World) {
        let camera_left = self.scroll_x - 800.0;
        let mut to_despawn = Vec::new();

        self.rocks.retain(|rock| {
            if rock.x < camera_left {
                to_despawn.push(rock.bottom_entity);
                to_despawn.push(rock.top_entity);
                false
            } else {
                true
            }
        });

        self.stars.retain(|star| {
            if star.x < camera_left || star.collected {
                to_despawn.push(star.entity);
                false
            } else {
                true
            }
        });

        if !to_despawn.is_empty() {
            world.despawn_entities(&to_despawn);
        }
    }

    fn star_system(&mut self) {
        let plane_world_x = self.scroll_x + PLANE_X;
        for star in &mut self.stars {
            if star.collected {
                continue;
            }
            let distance_x = (plane_world_x - star.x).abs();
            let distance_y = (self.plane_y - star.y).abs();
            if distance_x < STAR_RADIUS + PLANE_HITBOX_W / 2.0
                && distance_y < STAR_RADIUS + PLANE_HITBOX_H / 2.0
            {
                star.collected = true;
                self.score += 5;
            }
        }
    }

    fn ground_system(&mut self, world: &mut World) {
        for (index, entity) in self.ground_entities.iter().enumerate() {
            let base_x = (index as f32 - 1.0) * GROUND_TILE_WIDTH;
            let mut tile_x = base_x;

            while tile_x < self.scroll_x - GROUND_TILE_WIDTH * 2.0 {
                tile_x += self.ground_entities.len() as f32 * GROUND_TILE_WIDTH;
            }

            if let Some(transform) = world.get_local_transform_mut(*entity) {
                transform.translation.x = tile_x;
            }
            mark_local_transform_dirty(world, *entity);
        }
    }

    fn background_system(&mut self, world: &mut World) {
        let parallax_x = self.scroll_x * 0.3;
        for (index, entity) in self.background_entities.iter().enumerate() {
            let base_x = (index as f32 - 1.0) * BG_WIDTH;
            let mut tile_x = base_x + parallax_x;

            let total_width = self.background_entities.len() as f32 * BG_WIDTH;
            while tile_x < self.scroll_x - BG_WIDTH * 2.0 {
                tile_x += total_width;
            }

            if let Some(transform) = world.get_local_transform_mut(*entity) {
                transform.translation.x = tile_x;
            }
            mark_local_transform_dirty(world, *entity);
        }
    }

    fn collision_system(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }

        let plane_world_x = self.scroll_x + PLANE_X;
        let half_w = PLANE_HITBOX_W / 2.0;
        let half_h = PLANE_HITBOX_H / 2.0;

        if self.plane_y - half_h < GROUND_Y + GROUND_HEIGHT / 2.0 {
            self.phase = GamePhase::GameOver;
            if self.score > self.best_score {
                self.best_score = self.score;
            }
            return;
        }

        if self.plane_y + half_h > 350.0 {
            self.plane_y = 350.0 - half_h;
            self.plane_velocity = 0.0;
        }

        for rock in &self.rocks {
            let half_gap = rock.gap_size / 2.0;
            let rock_half_w = ROCK_WIDTH / 2.0;

            if (plane_world_x + half_w > rock.x - rock_half_w)
                && (plane_world_x - half_w < rock.x + rock_half_w)
                && (self.plane_y - half_h < rock.gap_y - half_gap
                    || self.plane_y + half_h > rock.gap_y + half_gap)
            {
                self.phase = GamePhase::GameOver;
                if self.score > self.best_score {
                    self.best_score = self.score;
                }
                return;
            }
        }
    }

    fn score_system(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }

        let plane_world_x = self.scroll_x + PLANE_X;
        for rock in &mut self.rocks {
            if !rock.scored && rock.x < plane_world_x {
                rock.scored = true;
                self.score += 1;
            }
        }
    }

    fn animation_system(&mut self, world: &mut World, delta_time: f32) {
        self.plane_anim_timer += delta_time;
        if self.plane_anim_timer >= ANIM_FRAME_DURATION {
            self.plane_anim_timer -= ANIM_FRAME_DURATION;
            self.plane_anim_frame = (self.plane_anim_frame + 1) % 3;
        }

        if let Some(plane_entity) = self.plane_entity {
            let slot = match self.plane_anim_frame {
                0 => SLOT_PLANE1,
                1 => SLOT_PLANE2,
                _ => SLOT_PLANE3,
            };
            set_sprite_texture(world, plane_entity, slot, &self.uv_max_table);
        }
    }

    fn smoke_system(&mut self, world: &mut World, delta_time: f32) {
        if self.phase == GamePhase::Playing {
            self.smoke_timer += delta_time;
            if self.smoke_timer >= SMOKE_SPAWN_INTERVAL {
                self.smoke_timer -= SMOKE_SPAWN_INTERVAL;

                let smoke_x = self.scroll_x + PLANE_X - PLANE_WIDTH / 2.0;
                let smoke_y = self.plane_y;

                let entity = spawn_textured_sprite(
                    world,
                    Vec3::new(smoke_x, smoke_y, LAYER_SMOKE),
                    Vec2::new(16.0, 16.0),
                    SLOT_SMOKE,
                    &self.uv_max_table,
                );

                self.smokes.push(Smoke {
                    entity,
                    lifetime: SMOKE_LIFETIME,
                });
            }
        }

        let mut to_despawn = Vec::new();
        for smoke in &mut self.smokes {
            smoke.lifetime -= delta_time;
            let alpha = (smoke.lifetime / SMOKE_LIFETIME).max(0.0);
            let scale = 1.0 + (1.0 - alpha) * 0.5;

            if let Some(sprite) = world.get_sprite_mut(smoke.entity) {
                sprite.color = [1.0, 1.0, 1.0, alpha];
            }
            if let Some(transform) = world.get_local_transform_mut(smoke.entity) {
                transform.scale = Vec3::new(scale, scale, 1.0);
            }
            mark_local_transform_dirty(world, smoke.entity);

            if smoke.lifetime <= 0.0 {
                to_despawn.push(smoke.entity);
            }
        }

        if !to_despawn.is_empty() {
            world.despawn_entities(&to_despawn);
            self.smokes.retain(|smoke| smoke.lifetime > 0.0);
        }
    }

    fn render_sync(&mut self, world: &mut World) {
        if let Some(plane_entity) = self.plane_entity {
            if let Some(transform) = world.get_local_transform_mut(plane_entity) {
                transform.translation.x = self.scroll_x + PLANE_X;
                transform.translation.y = self.plane_y;
                transform.rotation = nalgebra_glm::quat_angle_axis(self.plane_angle, &Vec3::z());
            }
            mark_local_transform_dirty(world, plane_entity);
        }
    }

    fn update_hud(&self, world: &mut World) {
        if let Some(score_entity) = self.score_hud {
            let text_index = world.get_hud_text(score_entity).map(|text| text.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("{}", self.score));
                if let Some(hud_text) = world.get_hud_text_mut(score_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(message_entity) = self.message_hud {
            let text_index = world
                .get_hud_text(message_entity)
                .map(|text| text.text_index);
            if let Some(text_index) = text_index {
                let message = match self.phase {
                    GamePhase::WaitingToStart => "TAP SPACE TO START".to_string(),
                    GamePhase::Playing => String::new(),
                    GamePhase::GameOver => {
                        format!("GAME OVER - Score: {} - Press SPACE", self.score)
                    }
                };
                world.resources.text_cache.set_text(text_index, message);
                if let Some(hud_text) = world.get_hud_text_mut(message_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(best_entity) = self.best_hud {
            let text_index = world.get_hud_text(best_entity).map(|text| text.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("Best: {}", self.best_score));
                if let Some(hud_text) = world.get_hud_text_mut(best_entity) {
                    hud_text.dirty = true;
                }
            }
        }
    }

    fn restart(&mut self, world: &mut World) {
        let mut to_despawn = Vec::new();
        for rock in &self.rocks {
            to_despawn.push(rock.bottom_entity);
            to_despawn.push(rock.top_entity);
        }
        for star in &self.stars {
            to_despawn.push(star.entity);
        }
        for smoke in &self.smokes {
            to_despawn.push(smoke.entity);
        }
        if !to_despawn.is_empty() {
            world.despawn_entities(&to_despawn);
        }

        self.rocks.clear();
        self.stars.clear();
        self.smokes.clear();
        self.plane_y = 0.0;
        self.plane_velocity = 0.0;
        self.plane_angle = 0.0;
        self.scroll_x = 0.0;
        self.next_rock_x = 400.0;
        self.score = 0;
        self.smoke_timer = 0.0;
        self.phase = GamePhase::WaitingToStart;

        if let Some(camera_entity) = self.camera_entity {
            if let Some(transform) = world.get_local_transform_mut(camera_entity) {
                transform.translation.x = 0.0;
            }
            mark_local_transform_dirty(world, camera_entity);
        }
    }
}

impl State for TappyPlane {
    fn title(&self) -> &str {
        "Tappy Plane"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.clear_color = [0.4, 0.7, 1.0, 1.0];

        let camera = spawn_ortho_camera(world, Vec2::new(0.0, 0.0));
        self.camera_entity = Some(camera);

        if let Some(camera_data) = world.get_camera_mut(camera)
            && let Projection::Orthographic(ref mut ortho) = camera_data.projection
        {
            ortho.x_mag = 320.0;
            ortho.y_mag = 270.0;
        }

        self.uv_max_table = load_textures(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        if !self.initialized {
            self.initialized = true;
            self.setup_scene(world);

            self.score_hud = Some(spawn_hud_text_with_properties(
                world,
                "0",
                HudAnchor::TopLeft,
                Vec2::new(20.0, 20.0),
                TextProperties {
                    font_size: 48.0,
                    color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                    ..Default::default()
                },
            ));

            self.message_hud = Some(spawn_hud_text_with_properties(
                world,
                "TAP SPACE TO START",
                HudAnchor::Center,
                Vec2::new(0.0, 0.0),
                TextProperties {
                    font_size: 36.0,
                    color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                    alignment: TextAlignment::Center,
                    ..Default::default()
                },
            ));

            self.best_hud = Some(spawn_hud_text_with_properties(
                world,
                "Best: 0",
                HudAnchor::TopRight,
                Vec2::new(-10.0, 20.0),
                TextProperties {
                    font_size: 24.0,
                    color: Vec4::new(1.0, 1.0, 0.5, 1.0),
                    ..Default::default()
                },
            ));
        }

        let delta_time = world.resources.window.timing.delta_time;

        escape_key_exit_system(world);
        self.input_system(world);
        self.physics_system(delta_time);
        self.scroll_system(world, delta_time);
        self.rock_generation_system(world);
        self.rock_cleanup_system(world);
        self.star_system();
        self.ground_system(world);
        self.background_system(world);
        self.collision_system();
        self.score_system();
        self.animation_system(world, delta_time);
        self.smoke_system(world, delta_time);
        self.render_sync(world);
        self.update_hud(world);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(TappyPlane::default())
}
