use image::GenericImageView;
use nightshade::ecs::transform::commands::mark_local_transform_dirty;
use nightshade::prelude::*;

const TILE_SIZE: f32 = 36.0;
const PLAYER_SPEED: f32 = 200.0;
const JUMP_VELOCITY: f32 = 450.0;
const GRAVITY: f32 = 1200.0;
const MAX_FALL_SPEED: f32 = 800.0;
const CAMERA_LERP_SPEED: f32 = 5.0;
const ENEMY_SPEED: f32 = 60.0;
const COIN_RADIUS: f32 = 14.0;
const PLAYER_WIDTH: f32 = 28.0;
const PLAYER_HEIGHT: f32 = 32.0;
const ANIM_FRAME_DURATION: f32 = 0.15;

const SLOT_GRASS_TL: u32 = 0;
const SLOT_GRASS_TM: u32 = 1;
const SLOT_GRASS_TR: u32 = 2;
const SLOT_DIRT: u32 = 3;
const SLOT_CRATE: u32 = 4;
const SLOT_COIN: u32 = 5;
const SLOT_FLAG: u32 = 6;
const SLOT_WATER_TOP: u32 = 7;
const SLOT_WATER: u32 = 8;
const SLOT_CHARACTERS: u32 = 9;
const SLOT_DIRT_LEFT: u32 = 10;
const SLOT_DIRT_MID: u32 = 11;
const SLOT_DIRT_RIGHT: u32 = 12;

const LAYER_TILES: f32 = 1.0;
const LAYER_COINS: f32 = 2.0;
const LAYER_ENEMIES: f32 = 3.0;
const LAYER_PLAYER: f32 = 4.0;
const LAYER_FLAG: f32 = 2.5;

const CHAR_CELL_W: u32 = 24;
const CHAR_CELL_H: u32 = 24;

const LEVEL_WIDTH: usize = 60;
const LEVEL_HEIGHT: usize = 17;

const LEVEL_DATA: &str = "\
............................................................
............................................................
............................................................
............................................................
..............................C.C.C........................
............................GGGGGGG.........................
..........C.C.................C.C...................C.......
.........GGGGG.................GGGGG..............GGGGG.....
.P..............C......E............E..................F....
GGGGGG..GGGGGGGG...GGGGGGGGG..GGGGGGGGGGGG..GGGGGGGGGGGGGGGG
DDDDDD..DDDDDDDD...DDDDDDDDD..DDDDDDDDDDDD..DDDDDDDDDDDDDDDD
DDDDDD..DDDDDDDD...DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD
DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD
DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD
DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD
DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD
DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD";

struct TextureEntry {
    slot: u32,
    bytes: &'static [u8],
}

fn load_textures(world: &mut World) -> Vec<Vec2> {
    let entries = [
        TextureEntry {
            slot: SLOT_GRASS_TL,
            bytes: include_bytes!("../assets/grass_top_left.png"),
        },
        TextureEntry {
            slot: SLOT_GRASS_TM,
            bytes: include_bytes!("../assets/grass_top_mid.png"),
        },
        TextureEntry {
            slot: SLOT_GRASS_TR,
            bytes: include_bytes!("../assets/grass_top_right.png"),
        },
        TextureEntry {
            slot: SLOT_DIRT,
            bytes: include_bytes!("../assets/dirt.png"),
        },
        TextureEntry {
            slot: SLOT_CRATE,
            bytes: include_bytes!("../assets/crate.png"),
        },
        TextureEntry {
            slot: SLOT_FLAG,
            bytes: include_bytes!("../assets/flag_top.png"),
        },
        TextureEntry {
            slot: SLOT_WATER_TOP,
            bytes: include_bytes!("../assets/water_top.png"),
        },
        TextureEntry {
            slot: SLOT_WATER,
            bytes: include_bytes!("../assets/water.png"),
        },
        TextureEntry {
            slot: SLOT_CHARACTERS,
            bytes: include_bytes!("../assets/characters.png"),
        },
        TextureEntry {
            slot: SLOT_DIRT_LEFT,
            bytes: include_bytes!("../assets/dirt_left.png"),
        },
        TextureEntry {
            slot: SLOT_DIRT_MID,
            bytes: include_bytes!("../assets/dirt_mid.png"),
        },
        TextureEntry {
            slot: SLOT_DIRT_RIGHT,
            bytes: include_bytes!("../assets/dirt_right.png"),
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

    let coin_size = 32u32;
    let mut coin_data = vec![0u8; (coin_size * coin_size * 4) as usize];
    for pixel_y in 0..coin_size {
        for pixel_x in 0..coin_size {
            let center = coin_size as f32 / 2.0 - 0.5;
            let distance_x = pixel_x as f32 - center;
            let distance_y = pixel_y as f32 - center;
            let distance = (distance_x * distance_x + distance_y * distance_y).sqrt();
            let outer_radius = coin_size as f32 / 2.0 - 1.0;
            let inner_radius = outer_radius - 2.0;
            let index = ((pixel_y * coin_size + pixel_x) * 4) as usize;
            if distance < outer_radius {
                let brightness = if distance < inner_radius {
                    0.9 + 0.1 * (1.0 - distance / inner_radius)
                } else {
                    0.7
                };
                let highlight = if distance_x + distance_y < 0.0 {
                    0.1
                } else {
                    0.0
                };
                coin_data[index] = ((brightness + highlight).min(1.0) * 255.0) as u8;
                coin_data[index + 1] = ((brightness * 0.85 + highlight).min(1.0) * 215.0) as u8;
                coin_data[index + 2] = 0;
                coin_data[index + 3] = 255;
            }
        }
    }
    world
        .resources
        .command_queue
        .push(WorldCommand::UploadSpriteTexture {
            slot: SLOT_COIN,
            rgba_data: coin_data,
            width: coin_size,
            height: coin_size,
        });
    let half_texel_x = 0.5 / atlas_slot_size.0 as f32;
    let half_texel_y = 0.5 / atlas_slot_size.1 as f32;
    uv_max_table[SLOT_COIN as usize] = Vec2::new(
        coin_size as f32 / atlas_slot_size.0 as f32 - half_texel_x,
        coin_size as f32 / atlas_slot_size.1 as f32 - half_texel_y,
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

fn char_uv(uv_max_table: &[Vec2], col: u32, row: u32) -> (Vec2, Vec2) {
    let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
    let _ = uv_max_table;
    let slot_w = atlas_slot_size.0 as f32;
    let slot_h = atlas_slot_size.1 as f32;

    let pixel_x = col * CHAR_CELL_W;
    let pixel_y = row * CHAR_CELL_H;

    let uv_min = Vec2::new(
        (pixel_x as f32 + 0.5) / slot_w,
        (pixel_y as f32 + 0.5) / slot_h,
    );
    let uv_max = Vec2::new(
        ((pixel_x + CHAR_CELL_W) as f32 - 0.5) / slot_w,
        ((pixel_y + CHAR_CELL_H) as f32 - 0.5) / slot_h,
    );

    (uv_min, uv_max)
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

fn spawn_char_sprite(
    world: &mut World,
    position: Vec3,
    size: Vec2,
    col: u32,
    row: u32,
    uv_max_table: &[Vec2],
) -> Entity {
    let entity = spawn_sprite(world, position, size);
    let (uv_min, uv_max) = char_uv(uv_max_table, col, row);
    if let Some(sprite) = world.get_sprite_mut(entity) {
        sprite.texture_index = SLOT_CHARACTERS;
        sprite.texture_index2 = SLOT_CHARACTERS;
        sprite.uv_min = uv_min;
        sprite.uv_max = uv_max;
    }
    entity
}

fn set_char_uv(world: &mut World, entity: Entity, col: u32, row: u32, uv_max_table: &[Vec2]) {
    let (uv_min, uv_max) = char_uv(uv_max_table, col, row);
    if let Some(sprite) = world.get_sprite_mut(entity) {
        sprite.uv_min = uv_min;
        sprite.uv_max = uv_max;
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
enum PlayerState {
    #[default]
    Idle,
    Walking,
    Jumping,
    Falling,
}

struct Coin {
    entity: Entity,
    x: f32,
    y: f32,
    collected: bool,
}

struct EnemyData {
    entity: Entity,
    x: f32,
    y: f32,
    velocity_x: f32,
    left_bound: f32,
    right_bound: f32,
}

struct PixelPlatformer {
    camera_entity: Option<Entity>,
    uv_max_table: Vec<Vec2>,
    initialized: bool,

    player_entity: Option<Entity>,
    player_x: f32,
    player_y: f32,
    player_velocity_x: f32,
    player_velocity_y: f32,
    player_on_ground: bool,
    player_facing_right: bool,
    player_state: PlayerState,
    player_anim_timer: f32,
    player_anim_frame: u32,

    tile_entities: Vec<Entity>,
    solid_tiles: Vec<bool>,

    coins: Vec<Coin>,
    enemies: Vec<EnemyData>,

    flag_entity: Option<Entity>,
    flag_x: f32,
    flag_y: f32,

    score: u32,
    level_complete: bool,
    respawn_x: f32,
    respawn_y: f32,

    camera_x: f32,
    camera_y: f32,

    score_hud: Option<Entity>,
    message_hud: Option<Entity>,
}

impl Default for PixelPlatformer {
    fn default() -> Self {
        Self {
            camera_entity: None,
            uv_max_table: Vec::new(),
            initialized: false,
            player_entity: None,
            player_x: 0.0,
            player_y: 0.0,
            player_velocity_x: 0.0,
            player_velocity_y: 0.0,
            player_on_ground: false,
            player_facing_right: true,
            player_state: PlayerState::default(),
            player_anim_timer: 0.0,
            player_anim_frame: 0,
            tile_entities: Vec::new(),
            solid_tiles: Vec::new(),
            coins: Vec::new(),
            enemies: Vec::new(),
            flag_entity: None,
            flag_x: 0.0,
            flag_y: 0.0,
            score: 0,
            level_complete: false,
            respawn_x: 0.0,
            respawn_y: 0.0,
            camera_x: 0.0,
            camera_y: 0.0,
            score_hud: None,
            message_hud: None,
        }
    }
}

impl PixelPlatformer {
    fn grid_to_world(col: usize, row: usize) -> (f32, f32) {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let y = (LEVEL_HEIGHT - 1 - row) as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        (x, y)
    }

    fn world_to_grid(x: f32, y: f32) -> (i32, i32) {
        let col = (x / TILE_SIZE).floor() as i32;
        let row = LEVEL_HEIGHT as i32 - 1 - (y / TILE_SIZE).floor() as i32;
        (col, row)
    }

    fn is_solid(&self, col: i32, row: i32) -> bool {
        if col < 0 || row < 0 || col >= LEVEL_WIDTH as i32 || row >= LEVEL_HEIGHT as i32 {
            return true;
        }
        self.solid_tiles[row as usize * LEVEL_WIDTH + col as usize]
    }

    fn build_level(&mut self, world: &mut World) {
        self.solid_tiles = vec![false; LEVEL_WIDTH * LEVEL_HEIGHT];
        let grid: Vec<Vec<char>> = LEVEL_DATA
            .lines()
            .map(|line| line.chars().collect())
            .collect();

        for (row, line) in grid.iter().enumerate() {
            for (col, &character) in line.iter().enumerate() {
                let index = row * LEVEL_WIDTH + col;
                let (world_x, world_y) = Self::grid_to_world(col, row);

                match character {
                    'G' => {
                        self.solid_tiles[index] = true;
                        let entity = spawn_textured_sprite(
                            world,
                            Vec3::new(world_x, world_y, LAYER_TILES),
                            Vec2::new(TILE_SIZE, TILE_SIZE),
                            SLOT_GRASS_TM,
                            &self.uv_max_table,
                        );
                        self.tile_entities.push(entity);
                    }
                    'D' => {
                        self.solid_tiles[index] = true;
                        let entity = spawn_textured_sprite(
                            world,
                            Vec3::new(world_x, world_y, LAYER_TILES),
                            Vec2::new(TILE_SIZE, TILE_SIZE),
                            SLOT_DIRT,
                            &self.uv_max_table,
                        );
                        self.tile_entities.push(entity);
                    }
                    'C' => {
                        let entity = spawn_textured_sprite(
                            world,
                            Vec3::new(world_x, world_y, LAYER_COINS),
                            Vec2::new(TILE_SIZE, TILE_SIZE),
                            SLOT_COIN,
                            &self.uv_max_table,
                        );
                        self.coins.push(Coin {
                            entity,
                            x: world_x,
                            y: world_y,
                            collected: false,
                        });
                    }
                    'P' => {
                        self.player_x = world_x;
                        self.player_y = world_y;
                        self.respawn_x = world_x;
                        self.respawn_y = world_y;
                    }
                    'F' => {
                        let entity = spawn_textured_sprite(
                            world,
                            Vec3::new(world_x, world_y, LAYER_FLAG),
                            Vec2::new(TILE_SIZE, TILE_SIZE),
                            SLOT_FLAG,
                            &self.uv_max_table,
                        );
                        self.flag_entity = Some(entity);
                        self.flag_x = world_x;
                        self.flag_y = world_y;
                    }
                    'E' => {
                        let entity = spawn_char_sprite(
                            world,
                            Vec3::new(world_x, world_y, LAYER_ENEMIES),
                            Vec2::new(TILE_SIZE, TILE_SIZE),
                            3,
                            0,
                            &self.uv_max_table,
                        );

                        let mut left = col;
                        while left > 0 {
                            let check_col = left - 1;
                            let below_row = row + 1;
                            if below_row < grid.len()
                                && check_col < grid[below_row].len()
                                && (grid[below_row][check_col] == 'G'
                                    || grid[below_row][check_col] == 'D')
                                && check_col < grid[row].len()
                                && grid[row][check_col] != 'G'
                                && grid[row][check_col] != 'D'
                            {
                                left -= 1;
                            } else {
                                break;
                            }
                        }
                        let mut right = col;
                        while right < LEVEL_WIDTH - 1 {
                            let check_col = right + 1;
                            let below_row = row + 1;
                            if below_row < grid.len()
                                && check_col < grid[below_row].len()
                                && (grid[below_row][check_col] == 'G'
                                    || grid[below_row][check_col] == 'D')
                                && check_col < grid[row].len()
                                && grid[row][check_col] != 'G'
                                && grid[row][check_col] != 'D'
                            {
                                right += 1;
                            } else {
                                break;
                            }
                        }

                        let (left_x, _) = Self::grid_to_world(left, row);
                        let (right_x, _) = Self::grid_to_world(right, row);

                        self.enemies.push(EnemyData {
                            entity,
                            x: world_x,
                            y: world_y,
                            velocity_x: ENEMY_SPEED,
                            left_bound: left_x,
                            right_bound: right_x,
                        });
                    }
                    _ => {}
                }
            }
        }

        let player_entity = spawn_char_sprite(
            world,
            Vec3::new(self.player_x, self.player_y, LAYER_PLAYER),
            Vec2::new(TILE_SIZE, TILE_SIZE),
            0,
            0,
            &self.uv_max_table,
        );
        self.player_entity = Some(player_entity);
    }

    fn input_system(&mut self, world: &mut World) {
        if self.level_complete {
            return;
        }

        let keyboard = &world.resources.input.keyboard;
        let left =
            keyboard.is_key_pressed(KeyCode::KeyA) || keyboard.is_key_pressed(KeyCode::ArrowLeft);
        let right =
            keyboard.is_key_pressed(KeyCode::KeyD) || keyboard.is_key_pressed(KeyCode::ArrowRight);
        let jump = keyboard.is_key_pressed(KeyCode::Space)
            || keyboard.is_key_pressed(KeyCode::ArrowUp)
            || keyboard.is_key_pressed(KeyCode::KeyW);

        if left {
            self.player_velocity_x = -PLAYER_SPEED;
            self.player_facing_right = false;
        } else if right {
            self.player_velocity_x = PLAYER_SPEED;
            self.player_facing_right = true;
        } else {
            self.player_velocity_x = 0.0;
        }

        if jump && self.player_on_ground {
            self.player_velocity_y = JUMP_VELOCITY;
            self.player_on_ground = false;
        }
    }

    fn physics_system(&mut self, delta_time: f32) {
        if self.level_complete {
            return;
        }

        self.player_velocity_y -= GRAVITY * delta_time;
        if self.player_velocity_y < -MAX_FALL_SPEED {
            self.player_velocity_y = -MAX_FALL_SPEED;
        }

        let new_x = self.player_x + self.player_velocity_x * delta_time;
        let new_y = self.player_y + self.player_velocity_y * delta_time;

        let half_w = PLAYER_WIDTH / 2.0;
        let half_h = PLAYER_HEIGHT / 2.0;

        self.player_x = new_x;
        self.resolve_horizontal_collisions(half_w, half_h);

        self.player_y = new_y;
        self.player_on_ground = false;
        self.resolve_vertical_collisions(half_w, half_h);

        if self.player_y < -TILE_SIZE * 2.0 {
            self.respawn_player();
        }
    }

    fn resolve_horizontal_collisions(&mut self, half_w: f32, half_h: f32) {
        let left = self.player_x - half_w;
        let right = self.player_x + half_w;
        let bottom = self.player_y - half_h;
        let top = self.player_y + half_h;

        let (col_left, row_top) = Self::world_to_grid(left, top - 1.0);
        let (col_right, row_bottom) = Self::world_to_grid(right, bottom + 1.0);

        for row in row_top..=row_bottom {
            for col in col_left..=col_right {
                if self.is_solid(col, row) {
                    let tile_left = col as f32 * TILE_SIZE;
                    let tile_right = tile_left + TILE_SIZE;

                    if right > tile_left && left < tile_right {
                        if self.player_velocity_x > 0.0 {
                            self.player_x = tile_left - half_w;
                            self.player_velocity_x = 0.0;
                        } else if self.player_velocity_x < 0.0 {
                            self.player_x = tile_right + half_w;
                            self.player_velocity_x = 0.0;
                        }
                    }
                }
            }
        }
    }

    fn resolve_vertical_collisions(&mut self, half_w: f32, half_h: f32) {
        let left = self.player_x - half_w;
        let right = self.player_x + half_w;
        let bottom = self.player_y - half_h;
        let top = self.player_y + half_h;

        let (col_left, row_top) = Self::world_to_grid(left + 1.0, top);
        let (col_right, row_bottom) = Self::world_to_grid(right - 1.0, bottom);

        for row in row_top..=row_bottom {
            for col in col_left..=col_right {
                if self.is_solid(col, row) {
                    let tile_bottom = (LEVEL_HEIGHT as i32 - 1 - row) as f32 * TILE_SIZE;
                    let tile_top = tile_bottom + TILE_SIZE;

                    if self.player_velocity_y < 0.0
                        && bottom < tile_top
                        && self.player_y > tile_bottom
                    {
                        self.player_y = tile_top + half_h;
                        self.player_velocity_y = 0.0;
                        self.player_on_ground = true;
                    } else if self.player_velocity_y > 0.0
                        && top > tile_bottom
                        && self.player_y < tile_top
                    {
                        self.player_y = tile_bottom - half_h;
                        self.player_velocity_y = 0.0;
                    }
                }
            }
        }
    }

    fn enemy_system(&mut self, delta_time: f32) {
        for enemy in &mut self.enemies {
            enemy.x += enemy.velocity_x * delta_time;

            if enemy.x <= enemy.left_bound {
                enemy.x = enemy.left_bound;
                enemy.velocity_x = enemy.velocity_x.abs();
            } else if enemy.x >= enemy.right_bound {
                enemy.x = enemy.right_bound;
                enemy.velocity_x = -enemy.velocity_x.abs();
            }
        }
    }

    fn coin_system(&mut self) {
        for coin in &mut self.coins {
            if coin.collected {
                continue;
            }
            let distance_x = (self.player_x - coin.x).abs();
            let distance_y = (self.player_y - coin.y).abs();
            if distance_x < COIN_RADIUS + PLAYER_WIDTH / 2.0
                && distance_y < COIN_RADIUS + PLAYER_HEIGHT / 2.0
            {
                coin.collected = true;
                self.score += 100;
            }
        }
    }

    fn hazard_system(&mut self) {
        let half_w = PLAYER_WIDTH / 2.0;
        let half_h = PLAYER_HEIGHT / 2.0;

        for enemy in &self.enemies {
            let distance_x = (self.player_x - enemy.x).abs();
            let distance_y = (self.player_y - enemy.y).abs();
            if distance_x < half_w + 12.0 && distance_y < half_h + 12.0 {
                self.respawn_player();
                return;
            }
        }
    }

    fn respawn_player(&mut self) {
        self.player_x = self.respawn_x;
        self.player_y = self.respawn_y;
        self.player_velocity_x = 0.0;
        self.player_velocity_y = 0.0;
    }

    fn goal_system(&mut self) {
        if self.flag_entity.is_some() {
            let distance_x = (self.player_x - self.flag_x).abs();
            let distance_y = (self.player_y - self.flag_y).abs();
            if distance_x < TILE_SIZE && distance_y < TILE_SIZE {
                self.level_complete = true;
            }
        }
    }

    fn camera_system(&mut self, world: &mut World, delta_time: f32) {
        let target_x = self.player_x;
        let target_y = self.player_y;

        let level_pixel_width = LEVEL_WIDTH as f32 * TILE_SIZE;
        let level_pixel_height = LEVEL_HEIGHT as f32 * TILE_SIZE;

        let lerp_factor = 1.0 - (-CAMERA_LERP_SPEED * delta_time).exp();
        self.camera_x += (target_x - self.camera_x) * lerp_factor;
        self.camera_y += (target_y - self.camera_y) * lerp_factor;

        let half_view_x = 480.0;
        let half_view_y = 270.0;

        self.camera_x = self.camera_x.clamp(
            half_view_x,
            (level_pixel_width - half_view_x).max(half_view_x),
        );
        self.camera_y = self.camera_y.clamp(
            half_view_y,
            (level_pixel_height - half_view_y).max(half_view_y),
        );

        if let Some(camera_entity) = self.camera_entity {
            if let Some(transform) = world.get_local_transform_mut(camera_entity) {
                transform.translation.x = self.camera_x;
                transform.translation.y = self.camera_y;
            }
            mark_local_transform_dirty(world, camera_entity);
        }
    }

    fn animation_system(&mut self, delta_time: f32) {
        if self.player_velocity_y > 0.1 {
            self.player_state = PlayerState::Jumping;
        } else if !self.player_on_ground {
            self.player_state = PlayerState::Falling;
        } else if self.player_velocity_x.abs() > 1.0 {
            self.player_state = PlayerState::Walking;
        } else {
            self.player_state = PlayerState::Idle;
        }

        self.player_anim_timer += delta_time;
        if self.player_anim_timer >= ANIM_FRAME_DURATION {
            self.player_anim_timer -= ANIM_FRAME_DURATION;
            self.player_anim_frame = (self.player_anim_frame + 1) % 2;
        }
    }

    fn render_sync(&mut self, world: &mut World) {
        let uv_max_table = self.uv_max_table.clone();

        if let Some(player_entity) = self.player_entity {
            if let Some(transform) = world.get_local_transform_mut(player_entity) {
                transform.translation.x = self.player_x;
                transform.translation.y = self.player_y;

                let scale_x = if self.player_facing_right { 1.0 } else { -1.0 };
                transform.scale = Vec3::new(scale_x, 1.0, 1.0);
            }
            mark_local_transform_dirty(world, player_entity);

            let char_col = match self.player_state {
                PlayerState::Idle => 0,
                PlayerState::Walking => self.player_anim_frame,
                PlayerState::Jumping => 1,
                PlayerState::Falling => 0,
            };
            set_char_uv(world, player_entity, char_col, 0, &uv_max_table);
        }

        for enemy in &self.enemies {
            if let Some(transform) = world.get_local_transform_mut(enemy.entity) {
                transform.translation.x = enemy.x;
                transform.translation.y = enemy.y;
                let scale_x = if enemy.velocity_x > 0.0 { 1.0 } else { -1.0 };
                transform.scale = Vec3::new(scale_x, 1.0, 1.0);
            }
            mark_local_transform_dirty(world, enemy.entity);
        }

        for coin in &self.coins {
            if let Some(visibility) = world.get_visibility_mut(coin.entity) {
                visibility.visible = !coin.collected;
            }
        }
    }

    fn update_hud(&self, world: &mut World) {
        if let Some(score_entity) = self.score_hud {
            let text_index = world.get_hud_text(score_entity).map(|text| text.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("Score: {}", self.score));
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
                let message = if self.level_complete {
                    "Level Complete!".to_string()
                } else {
                    "WASD/Arrows: Move  Space: Jump".to_string()
                };
                world.resources.text_cache.set_text(text_index, message);
                if let Some(hud_text) = world.get_hud_text_mut(message_entity) {
                    hud_text.dirty = true;
                }
            }
        }
    }
}

impl State for PixelPlatformer {
    fn title(&self) -> &str {
        "Pixel Platformer"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.clear_color = [0.4, 0.6, 0.9, 1.0];

        let camera = spawn_ortho_camera(world, Vec2::new(0.0, 0.0));
        self.camera_entity = Some(camera);

        if let Some(camera_data) = world.get_camera_mut(camera)
            && let Projection::Orthographic(ref mut ortho) = camera_data.projection
        {
            ortho.x_mag = 480.0;
            ortho.y_mag = 270.0;
        }

        self.uv_max_table = load_textures(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        if !self.initialized {
            self.initialized = true;
            self.build_level(world);
            self.camera_x = self.player_x;
            self.camera_y = self.player_y;

            self.score_hud = Some(spawn_hud_text_with_properties(
                world,
                "Score: 0",
                HudAnchor::TopLeft,
                Vec2::new(10.0, 10.0),
                TextProperties {
                    font_size: 32.0,
                    color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                    ..Default::default()
                },
            ));

            self.message_hud = Some(spawn_hud_text_with_properties(
                world,
                "WASD/Arrows: Move  Space: Jump",
                HudAnchor::BottomLeft,
                Vec2::new(10.0, -10.0),
                TextProperties {
                    font_size: 22.0,
                    color: Vec4::new(1.0, 1.0, 1.0, 0.8),
                    ..Default::default()
                },
            ));
        }

        let delta_time = world.resources.window.timing.delta_time;

        escape_key_exit_system(world);
        self.input_system(world);
        self.physics_system(delta_time);
        self.enemy_system(delta_time);
        self.coin_system();
        self.hazard_system();
        self.goal_system();
        self.animation_system(delta_time);
        self.camera_system(world, delta_time);
        self.render_sync(world);
        self.update_hud(world);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(PixelPlatformer::default())
}
