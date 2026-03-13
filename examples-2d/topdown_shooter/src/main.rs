use image::GenericImageView;
use nightshade::ecs::transform::commands::mark_local_transform_dirty;
use nightshade::prelude::*;
use rand::Rng;

const TILE_SIZE: f32 = 64.0;
const PLAYER_SPEED: f32 = 200.0;
const BULLET_SPEED: f32 = 600.0;
const BULLET_LIFETIME: f32 = 2.0;
const FIRE_COOLDOWN: f32 = 0.15;
const ZOMBIE_BASE_SPEED: f32 = 80.0;
const ZOMBIE_RADIUS: f32 = 20.0;
const PLAYER_RADIUS: f32 = 18.0;
const BULLET_RADIUS: f32 = 4.0;
const SPAWN_MARGIN: f32 = 100.0;

const SLOT_FLOOR_GRASS: u32 = 0;
const SLOT_FLOOR_WOOD: u32 = 1;
const SLOT_WALL: u32 = 2;
const SLOT_PLAYER: u32 = 3;
const SLOT_ZOMBIE: u32 = 4;
const SLOT_BULLET: u32 = 5;

const LAYER_FLOOR: f32 = 0.0;
const LAYER_ENTITIES: f32 = 1.0;
const LAYER_BULLETS: f32 = 2.0;
const LAYER_PLAYER: f32 = 3.0;

const ARENA_WIDTH: usize = 20;
const ARENA_HEIGHT: usize = 15;

const ARENA_DATA: &str = "\
WWWWWWWWWWWWWWWWWWWW\
W..................W\
W..................W\
W...WWWW...........W\
W...W..............W\
W..................W\
W..................W\
W......WWW.........W\
W..................W\
W..................W\
W..........WW..W...W\
W..........WW..W...W\
W..................W\
W..................W\
WWWWWWWWWWWWWWWWWWWW";

struct TextureEntry {
    slot: u32,
    bytes: &'static [u8],
}

fn load_textures(world: &mut World) -> Vec<Vec2> {
    let entries = [
        TextureEntry {
            slot: SLOT_FLOOR_GRASS,
            bytes: include_bytes!("../assets/floor_grass.png"),
        },
        TextureEntry {
            slot: SLOT_FLOOR_WOOD,
            bytes: include_bytes!("../assets/floor_wood.png"),
        },
        TextureEntry {
            slot: SLOT_WALL,
            bytes: include_bytes!("../assets/wall.png"),
        },
        TextureEntry {
            slot: SLOT_PLAYER,
            bytes: include_bytes!("../assets/player.png"),
        },
        TextureEntry {
            slot: SLOT_ZOMBIE,
            bytes: include_bytes!("../assets/zombie.png"),
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

    let bullet_size = 8u32;
    let mut bullet_data = vec![0u8; (bullet_size * bullet_size * 4) as usize];
    for pixel_y in 0..bullet_size {
        for pixel_x in 0..bullet_size {
            let center_x = bullet_size as f32 / 2.0 - 0.5;
            let center_y = bullet_size as f32 / 2.0 - 0.5;
            let distance =
                ((pixel_x as f32 - center_x).powi(2) + (pixel_y as f32 - center_y).powi(2)).sqrt();
            let index = ((pixel_y * bullet_size + pixel_x) * 4) as usize;
            if distance < bullet_size as f32 / 2.0 {
                bullet_data[index] = 255;
                bullet_data[index + 1] = 255;
                bullet_data[index + 2] = 100;
                bullet_data[index + 3] = 255;
            }
        }
    }
    world
        .resources
        .command_queue
        .push(WorldCommand::UploadSpriteTexture {
            slot: SLOT_BULLET,
            rgba_data: bullet_data,
            width: bullet_size,
            height: bullet_size,
        });
    let half_texel_x = 0.5 / atlas_slot_size.0 as f32;
    let half_texel_y = 0.5 / atlas_slot_size.1 as f32;
    uv_max_table[SLOT_BULLET as usize] = Vec2::new(
        bullet_size as f32 / atlas_slot_size.0 as f32 - half_texel_x,
        bullet_size as f32 / atlas_slot_size.1 as f32 - half_texel_y,
    );

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
    position: Vec2,
    depth: f32,
    size: Vec2,
    texture_slot: u32,
    uv_max_table: &[Vec2],
) -> Entity {
    let entity = spawn_sprite(world, position, size);
    let (uv_min, uv_max) = uv_for_slot(uv_max_table, texture_slot);
    if let Some(sprite) = world.core.get_sprite_mut(entity) {
        sprite.depth = depth;
        sprite.texture_index = texture_slot;
        sprite.texture_index2 = texture_slot;
        sprite.uv_min = uv_min;
        sprite.uv_max = uv_max;
    }
    entity
}

struct Bullet {
    entity: Entity,
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    lifetime: f32,
}

struct Zombie {
    entity: Entity,
    x: f32,
    y: f32,
    health: i32,
    angle_offset: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum GamePhase {
    Playing,
    GameOver,
}

struct TopdownShooter {
    camera_entity: Option<Entity>,
    uv_max_table: Vec<Vec2>,
    initialized: bool,

    player_entity: Option<Entity>,
    player_x: f32,
    player_y: f32,
    player_angle: f32,
    fire_cooldown: f32,

    bullets: Vec<Bullet>,
    zombies: Vec<Zombie>,
    solid_tiles: Vec<bool>,
    tile_entities: Vec<Entity>,

    wave: u32,
    kills: u32,
    score: u32,
    zombies_to_spawn: u32,
    spawn_timer: f32,
    spawn_interval: f32,
    phase: GamePhase,

    score_hud: Option<Entity>,
    wave_hud: Option<Entity>,
    message_hud: Option<Entity>,
}

impl Default for TopdownShooter {
    fn default() -> Self {
        Self {
            camera_entity: None,
            uv_max_table: Vec::new(),
            initialized: false,
            player_entity: None,
            player_x: 0.0,
            player_y: 0.0,
            player_angle: 0.0,
            fire_cooldown: 0.0,
            bullets: Vec::new(),
            zombies: Vec::new(),
            solid_tiles: Vec::new(),
            tile_entities: Vec::new(),
            wave: 1,
            kills: 0,
            score: 0,
            zombies_to_spawn: 5,
            spawn_timer: 0.0,
            spawn_interval: 1.0,
            phase: GamePhase::Playing,
            score_hud: None,
            wave_hud: None,
            message_hud: None,
        }
    }
}

impl TopdownShooter {
    fn grid_to_world(col: usize, row: usize) -> (f32, f32) {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let y = (ARENA_HEIGHT - 1 - row) as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        (x, y)
    }

    fn world_to_grid(x: f32, y: f32) -> (i32, i32) {
        let col = (x / TILE_SIZE).floor() as i32;
        let row = ARENA_HEIGHT as i32 - 1 - (y / TILE_SIZE).floor() as i32;
        (col, row)
    }

    fn is_solid(&self, col: i32, row: i32) -> bool {
        if col < 0 || row < 0 || col >= ARENA_WIDTH as i32 || row >= ARENA_HEIGHT as i32 {
            return true;
        }
        self.solid_tiles[row as usize * ARENA_WIDTH + col as usize]
    }

    fn is_solid_at_world(&self, x: f32, y: f32) -> bool {
        let (col, row) = Self::world_to_grid(x, y);
        self.is_solid(col, row)
    }

    fn build_arena(&mut self, world: &mut World) {
        self.solid_tiles = vec![false; ARENA_WIDTH * ARENA_HEIGHT];
        let chars: Vec<char> = ARENA_DATA.chars().collect();

        for row in 0..ARENA_HEIGHT {
            for col in 0..ARENA_WIDTH {
                let index = row * ARENA_WIDTH + col;
                if index >= chars.len() {
                    continue;
                }
                let character = chars[index];
                let (world_x, world_y) = Self::grid_to_world(col, row);

                match character {
                    'W' => {
                        self.solid_tiles[index] = true;
                        let entity = spawn_textured_sprite(
                            world,
                            Vec2::new(world_x, world_y),
                            LAYER_FLOOR + 0.5,
                            Vec2::new(TILE_SIZE, TILE_SIZE),
                            SLOT_WALL,
                            &self.uv_max_table,
                        );
                        self.tile_entities.push(entity);
                    }
                    _ => {
                        let entity = spawn_textured_sprite(
                            world,
                            Vec2::new(world_x, world_y),
                            LAYER_FLOOR,
                            Vec2::new(TILE_SIZE, TILE_SIZE),
                            SLOT_FLOOR_GRASS,
                            &self.uv_max_table,
                        );
                        self.tile_entities.push(entity);
                    }
                }
            }
        }

        let (spawn_x, spawn_y) = Self::grid_to_world(ARENA_WIDTH / 2, ARENA_HEIGHT / 2);
        self.player_x = spawn_x;
        self.player_y = spawn_y;

        let player_entity = spawn_textured_sprite(
            world,
            Vec2::new(self.player_x, self.player_y),
            LAYER_PLAYER,
            Vec2::new(50.0, 43.0),
            SLOT_PLAYER,
            &self.uv_max_table,
        );
        self.player_entity = Some(player_entity);
    }

    fn input_system(&mut self, world: &mut World, delta_time: f32) {
        if self.phase == GamePhase::GameOver {
            return;
        }

        let keyboard = &world.resources.input.keyboard;
        let mut move_x = 0.0_f32;
        let mut move_y = 0.0_f32;

        if keyboard.is_key_pressed(KeyCode::KeyA) || keyboard.is_key_pressed(KeyCode::ArrowLeft) {
            move_x -= 1.0;
        }
        if keyboard.is_key_pressed(KeyCode::KeyD) || keyboard.is_key_pressed(KeyCode::ArrowRight) {
            move_x += 1.0;
        }
        if keyboard.is_key_pressed(KeyCode::KeyW) || keyboard.is_key_pressed(KeyCode::ArrowUp) {
            move_y += 1.0;
        }
        if keyboard.is_key_pressed(KeyCode::KeyS) || keyboard.is_key_pressed(KeyCode::ArrowDown) {
            move_y -= 1.0;
        }

        let magnitude = (move_x * move_x + move_y * move_y).sqrt();
        if magnitude > 0.0 {
            move_x /= magnitude;
            move_y /= magnitude;
        }

        let new_x = self.player_x + move_x * PLAYER_SPEED * delta_time;
        if !self.is_solid_at_world(new_x - PLAYER_RADIUS, self.player_y)
            && !self.is_solid_at_world(new_x + PLAYER_RADIUS, self.player_y)
        {
            self.player_x = new_x;
        }

        let new_y = self.player_y + move_y * PLAYER_SPEED * delta_time;
        if !self.is_solid_at_world(self.player_x, new_y - PLAYER_RADIUS)
            && !self.is_solid_at_world(self.player_x, new_y + PLAYER_RADIUS)
        {
            self.player_y = new_y;
        }
    }

    fn mouse_aim_system(&mut self, world: &mut World) {
        let mouse_pos = world.resources.input.mouse.position;

        let viewport_width = world
            .resources
            .window
            .handle
            .as_ref()
            .map(|handle| handle.inner_size().width as f32)
            .unwrap_or(1920.0);
        let viewport_height = world
            .resources
            .window
            .handle
            .as_ref()
            .map(|handle| handle.inner_size().height as f32)
            .unwrap_or(1080.0);

        if let Some(camera_entity) = self.camera_entity
            && let Some(camera) = world.core.get_camera(camera_entity)
            && let Projection::Orthographic(ortho) = &camera.projection
        {
            let world_mouse_x =
                self.player_x + (mouse_pos.x / viewport_width * 2.0 - 1.0) * ortho.x_mag;
            let world_mouse_y =
                self.player_y - (mouse_pos.y / viewport_height * 2.0 - 1.0) * ortho.y_mag;

            self.player_angle =
                (world_mouse_y - self.player_y).atan2(world_mouse_x - self.player_x);
        }
    }

    fn shooting_system(&mut self, world: &mut World, delta_time: f32) {
        if self.phase == GamePhase::GameOver {
            return;
        }

        self.fire_cooldown -= delta_time;

        let mouse_state = world.resources.input.mouse.state;
        if mouse_state.contains(MouseState::LEFT_CLICKED) && self.fire_cooldown <= 0.0 {
            self.fire_cooldown = FIRE_COOLDOWN;

            let bullet_x = self.player_x + self.player_angle.cos() * 25.0;
            let bullet_y = self.player_y + self.player_angle.sin() * 25.0;
            let velocity_x = self.player_angle.cos() * BULLET_SPEED;
            let velocity_y = self.player_angle.sin() * BULLET_SPEED;

            let entity = spawn_textured_sprite(
                world,
                Vec2::new(bullet_x, bullet_y),
                LAYER_BULLETS,
                Vec2::new(8.0, 8.0),
                SLOT_BULLET,
                &self.uv_max_table,
            );

            self.bullets.push(Bullet {
                entity,
                x: bullet_x,
                y: bullet_y,
                velocity_x,
                velocity_y,
                lifetime: BULLET_LIFETIME,
            });
        }
    }

    fn bullet_system(&mut self, world: &mut World, delta_time: f32) {
        let mut to_despawn = Vec::new();
        let solid_tiles = &self.solid_tiles;

        for bullet in &mut self.bullets {
            bullet.x += bullet.velocity_x * delta_time;
            bullet.y += bullet.velocity_y * delta_time;
            bullet.lifetime -= delta_time;

            let (col, row) = Self::world_to_grid(bullet.x, bullet.y);
            let is_solid = Self::is_solid_static(solid_tiles, col, row);
            if bullet.lifetime <= 0.0 || is_solid {
                to_despawn.push(bullet.entity);
            }
        }

        if !to_despawn.is_empty() {
            world.despawn_entities(&to_despawn);
            self.bullets
                .retain(|bullet| !to_despawn.contains(&bullet.entity));
        }
    }

    fn zombie_ai_system(&mut self, delta_time: f32) {
        let player_x = self.player_x;
        let player_y = self.player_y;

        for zombie in &mut self.zombies {
            let direction_x = player_x - zombie.x;
            let direction_y = player_y - zombie.y;
            let distance = (direction_x * direction_x + direction_y * direction_y).sqrt();

            if distance > 1.0 {
                let normalized_x = direction_x / distance;
                let normalized_y = direction_y / distance;

                let offset_angle = zombie.angle_offset;
                let cos_offset = offset_angle.cos();
                let sin_offset = offset_angle.sin();
                let adjusted_x = normalized_x * cos_offset - normalized_y * sin_offset;
                let adjusted_y = normalized_x * sin_offset + normalized_y * cos_offset;

                let speed = ZOMBIE_BASE_SPEED;
                let new_x = zombie.x + adjusted_x * speed * delta_time;
                let new_y = zombie.y + adjusted_y * speed * delta_time;

                let (col_x, row_x) = Self::world_to_grid(new_x, zombie.y);
                if !Self::is_solid_static(&self.solid_tiles, col_x, row_x) {
                    zombie.x = new_x;
                }

                let (col_y, row_y) = Self::world_to_grid(zombie.x, new_y);
                if !Self::is_solid_static(&self.solid_tiles, col_y, row_y) {
                    zombie.y = new_y;
                }
            }
        }
    }

    fn is_solid_static(solid_tiles: &[bool], col: i32, row: i32) -> bool {
        if col < 0 || row < 0 || col >= ARENA_WIDTH as i32 || row >= ARENA_HEIGHT as i32 {
            return true;
        }
        solid_tiles[row as usize * ARENA_WIDTH + col as usize]
    }

    fn bullet_vs_zombie_system(&mut self, world: &mut World) {
        let mut dead_zombies = Vec::new();
        let mut dead_bullets = Vec::new();

        for bullet in &self.bullets {
            for (zombie_index, zombie) in self.zombies.iter_mut().enumerate() {
                let distance_x = bullet.x - zombie.x;
                let distance_y = bullet.y - zombie.y;
                let distance_squared = distance_x * distance_x + distance_y * distance_y;
                let hit_radius = BULLET_RADIUS + ZOMBIE_RADIUS;

                if distance_squared < hit_radius * hit_radius {
                    zombie.health -= 1;
                    dead_bullets.push(bullet.entity);
                    if zombie.health <= 0 {
                        dead_zombies.push(zombie_index);
                    }
                    break;
                }
            }
        }

        dead_zombies.sort_unstable();
        dead_zombies.dedup();

        let mut entities_to_despawn: Vec<Entity> = dead_bullets.clone();
        for &zombie_index in dead_zombies.iter().rev() {
            if zombie_index < self.zombies.len() {
                entities_to_despawn.push(self.zombies[zombie_index].entity);
                self.zombies.remove(zombie_index);
                self.kills += 1;
                self.score += 100;
            }
        }

        if !entities_to_despawn.is_empty() {
            world.despawn_entities(&entities_to_despawn);
            self.bullets
                .retain(|bullet| !dead_bullets.contains(&bullet.entity));
        }
    }

    fn zombie_vs_player_system(&mut self) {
        if self.phase == GamePhase::GameOver {
            return;
        }

        for zombie in &self.zombies {
            let distance_x = self.player_x - zombie.x;
            let distance_y = self.player_y - zombie.y;
            let distance_squared = distance_x * distance_x + distance_y * distance_y;
            let hit_radius = PLAYER_RADIUS + ZOMBIE_RADIUS;

            if distance_squared < hit_radius * hit_radius {
                self.phase = GamePhase::GameOver;
                return;
            }
        }
    }

    fn wave_system(&mut self) {
        if self.phase == GamePhase::GameOver {
            return;
        }

        if self.zombies.is_empty() && self.zombies_to_spawn == 0 {
            self.wave += 1;
            self.zombies_to_spawn = 3 + self.wave * 2;
            self.spawn_interval = (1.0 - self.wave as f32 * 0.05).max(0.3);
            self.spawn_timer = 0.0;
        }
    }

    fn spawn_system(&mut self, world: &mut World, delta_time: f32) {
        if self.phase == GamePhase::GameOver || self.zombies_to_spawn == 0 {
            return;
        }

        self.spawn_timer += delta_time;
        if self.spawn_timer >= self.spawn_interval {
            self.spawn_timer -= self.spawn_interval;
            self.zombies_to_spawn -= 1;

            let mut rng = rand::rng();
            let arena_pixel_width = ARENA_WIDTH as f32 * TILE_SIZE;
            let arena_pixel_height = ARENA_HEIGHT as f32 * TILE_SIZE;

            let (spawn_x, spawn_y) = loop {
                let side = rng.random_range(0..4);
                let (candidate_x, candidate_y) = match side {
                    0 => (
                        TILE_SIZE + SPAWN_MARGIN,
                        rng.random_range(TILE_SIZE..arena_pixel_height - TILE_SIZE),
                    ),
                    1 => (
                        arena_pixel_width - TILE_SIZE - SPAWN_MARGIN,
                        rng.random_range(TILE_SIZE..arena_pixel_height - TILE_SIZE),
                    ),
                    2 => (
                        rng.random_range(TILE_SIZE..arena_pixel_width - TILE_SIZE),
                        arena_pixel_height - TILE_SIZE - SPAWN_MARGIN,
                    ),
                    _ => (
                        rng.random_range(TILE_SIZE..arena_pixel_width - TILE_SIZE),
                        TILE_SIZE + SPAWN_MARGIN,
                    ),
                };

                if !self.is_solid_at_world(candidate_x, candidate_y) {
                    let distance_to_player = ((candidate_x - self.player_x).powi(2)
                        + (candidate_y - self.player_y).powi(2))
                    .sqrt();
                    if distance_to_player > 200.0 {
                        break (candidate_x, candidate_y);
                    }
                }
            };

            let entity = spawn_textured_sprite(
                world,
                Vec2::new(spawn_x, spawn_y),
                LAYER_ENTITIES,
                Vec2::new(44.0, 51.0),
                SLOT_ZOMBIE,
                &self.uv_max_table,
            );

            let mut rng = rand::rng();
            self.zombies.push(Zombie {
                entity,
                x: spawn_x,
                y: spawn_y,
                health: 1 + (self.wave / 3) as i32,
                angle_offset: rng.random_range(-0.3..0.3),
            });
        }
    }

    fn camera_system(&mut self, world: &mut World) {
        if let Some(camera_entity) = self.camera_entity {
            if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
                transform.translation.x = self.player_x;
                transform.translation.y = self.player_y;
            }
            mark_local_transform_dirty(world, camera_entity);
        }
    }

    fn render_sync(&mut self, world: &mut World) {
        if let Some(player_entity) = self.player_entity
            && let Some(sprite) = world.core.get_sprite_mut(player_entity)
        {
            sprite.position = Vec2::new(self.player_x, self.player_y);
            sprite.rotation = self.player_angle;
        }

        for bullet in &self.bullets {
            if let Some(sprite) = world.core.get_sprite_mut(bullet.entity) {
                sprite.position = Vec2::new(bullet.x, bullet.y);
            }
        }

        for zombie in &self.zombies {
            let angle = (self.player_y - zombie.y).atan2(self.player_x - zombie.x);
            if let Some(sprite) = world.core.get_sprite_mut(zombie.entity) {
                sprite.position = Vec2::new(zombie.x, zombie.y);
                sprite.rotation = angle;
            }
        }
    }

    fn update_hud(&self, world: &mut World) {
        if let Some(score_entity) = self.score_hud {
            let text_index = world.core.get_hud_text(score_entity).map(|text| text.text_index);
            if let Some(text_index) = text_index {
                world.resources.text_cache.set_text(
                    text_index,
                    format!("Score: {}  Kills: {}", self.score, self.kills),
                );
                if let Some(hud_text) = world.core.get_hud_text_mut(score_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(wave_entity) = self.wave_hud {
            let text_index = world.core.get_hud_text(wave_entity).map(|text| text.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("Wave {}", self.wave));
                if let Some(hud_text) = world.core.get_hud_text_mut(wave_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(message_entity) = self.message_hud {
            let text_index = world
                .core.get_hud_text(message_entity)
                .map(|text| text.text_index);
            if let Some(text_index) = text_index {
                let message = match self.phase {
                    GamePhase::Playing => "WASD: Move  Mouse: Aim  Click: Shoot".to_string(),
                    GamePhase::GameOver => "GAME OVER - Press R to restart".to_string(),
                };
                world.resources.text_cache.set_text(text_index, message);
                if let Some(hud_text) = world.core.get_hud_text_mut(message_entity) {
                    hud_text.dirty = true;
                }
            }
        }
    }

    fn restart(&mut self, world: &mut World) {
        let mut to_despawn = Vec::new();
        for bullet in &self.bullets {
            to_despawn.push(bullet.entity);
        }
        for zombie in &self.zombies {
            to_despawn.push(zombie.entity);
        }
        if !to_despawn.is_empty() {
            world.despawn_entities(&to_despawn);
        }
        self.bullets.clear();
        self.zombies.clear();

        let (spawn_x, spawn_y) = Self::grid_to_world(ARENA_WIDTH / 2, ARENA_HEIGHT / 2);
        self.player_x = spawn_x;
        self.player_y = spawn_y;
        self.wave = 1;
        self.kills = 0;
        self.score = 0;
        self.zombies_to_spawn = 5;
        self.spawn_timer = 0.0;
        self.spawn_interval = 1.0;
        self.phase = GamePhase::Playing;
    }
}

impl State for TopdownShooter {
    fn title(&self) -> &str {
        "Top-Down Shooter"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.clear_color = [0.1, 0.1, 0.1, 1.0];

        let camera = spawn_ortho_camera(world, Vec2::new(0.0, 0.0));
        self.camera_entity = Some(camera);

        self.uv_max_table = load_textures(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        if !self.initialized {
            self.initialized = true;
            self.build_arena(world);

            self.score_hud = Some(spawn_hud_text_with_properties(
                world,
                "Score: 0  Kills: 0",
                HudAnchor::TopLeft,
                Vec2::new(10.0, 10.0),
                TextProperties {
                    font_size: 28.0,
                    color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                    ..Default::default()
                },
            ));

            self.wave_hud = Some(spawn_hud_text_with_properties(
                world,
                "Wave 1",
                HudAnchor::TopRight,
                Vec2::new(-10.0, 10.0),
                TextProperties {
                    font_size: 32.0,
                    color: Vec4::new(1.0, 0.3, 0.3, 1.0),
                    ..Default::default()
                },
            ));

            self.message_hud = Some(spawn_hud_text_with_properties(
                world,
                "WASD: Move  Mouse: Aim  Click: Shoot",
                HudAnchor::BottomLeft,
                Vec2::new(10.0, -10.0),
                TextProperties {
                    font_size: 22.0,
                    color: Vec4::new(0.8, 0.8, 0.8, 1.0),
                    ..Default::default()
                },
            ));
        }

        let delta_time = world.resources.window.timing.delta_time;

        escape_key_exit_system(world);

        let restart_pressed = world
            .resources
            .input
            .keyboard
            .frame_keys
            .iter()
            .any(|(key, pressed)| *key == KeyCode::KeyR && *pressed);
        if restart_pressed && self.phase == GamePhase::GameOver {
            self.restart(world);
        }

        self.input_system(world, delta_time);
        self.mouse_aim_system(world);
        self.shooting_system(world, delta_time);
        self.bullet_system(world, delta_time);
        self.zombie_ai_system(delta_time);
        self.bullet_vs_zombie_system(world);
        self.zombie_vs_player_system();
        self.wave_system();
        self.spawn_system(world, delta_time);
        self.camera_system(world);
        self.render_sync(world);
        self.update_hud(world);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(TopdownShooter::default())
}
