use nightshade::tui::prelude::*;
use rand::Rng;

const LEVEL_WIDTH: usize = 120;
const LEVEL_HEIGHT: usize = 20;
const VIEW_WIDTH: usize = 60;
const GRAVITY: f64 = 40.0;
const JUMP_VELOCITY: f64 = -14.0;
const MOVE_SPEED: f64 = 15.0;
const MAX_FALL_SPEED: f64 = 20.0;

const TILE_AIR: u8 = 0;
const TILE_GROUND: u8 = 1;
const TILE_PLATFORM: u8 = 2;
const TILE_SPIKE: u8 = 3;
const TILE_EXIT: u8 = 5;

struct LevelData {
    tiles: Vec<Vec<u8>>,
    player_start: (usize, usize),
    coin_positions: Vec<(usize, usize)>,
}

fn create_level(level_number: u32) -> LevelData {
    let mut tiles = vec![vec![TILE_AIR; LEVEL_WIDTH]; LEVEL_HEIGHT];

    for tile in tiles[LEVEL_HEIGHT - 1].iter_mut().take(LEVEL_WIDTH) {
        *tile = TILE_GROUND;
    }
    for tile in tiles[LEVEL_HEIGHT - 2].iter_mut().take(LEVEL_WIDTH) {
        *tile = TILE_GROUND;
    }

    let mut coin_positions = Vec::new();
    let mut rng = rand::rng();

    let safe_start = 12;
    let safe_end = LEVEL_WIDTH - 10;

    let gap_count = 3 + level_number as usize * 2;
    let usable_span = safe_end - safe_start;
    let spacing = usable_span / (gap_count + 1);
    let mut gap_ranges: Vec<(usize, usize)> = Vec::new();

    for gap_index in 0..gap_count {
        let center =
            safe_start + spacing * (gap_index + 1) + rng.random_range(0..spacing.max(1) / 2);
        let gap_width = rng.random_range(2..4_usize);
        let gap_start = center.saturating_sub(gap_width / 2).max(safe_start);
        let gap_end = (gap_start + gap_width).min(safe_end);

        let overlaps = gap_ranges.iter().any(|&(existing_start, existing_end)| {
            gap_start < existing_end + 4 && gap_end + 4 > existing_start
        });
        if overlaps {
            continue;
        }

        for tile in tiles[LEVEL_HEIGHT - 1]
            .iter_mut()
            .take(gap_end)
            .skip(gap_start)
        {
            *tile = TILE_AIR;
        }
        for tile in tiles[LEVEL_HEIGHT - 2]
            .iter_mut()
            .take(gap_end)
            .skip(gap_start)
        {
            *tile = TILE_AIR;
        }
        gap_ranges.push((gap_start, gap_end));

        let bridge_row = LEVEL_HEIGHT - 4;
        let bridge_start = if gap_start >= 2 {
            gap_start - 1
        } else {
            gap_start
        };
        let bridge_end = (gap_end + 1).min(LEVEL_WIDTH);
        for tile in tiles[bridge_row]
            .iter_mut()
            .take(bridge_end)
            .skip(bridge_start)
        {
            *tile = TILE_PLATFORM;
        }
    }

    let platform_count = 15 + level_number as usize * 5;
    for _ in 0..platform_count {
        let platform_column = rng.random_range(5..LEVEL_WIDTH - 10);
        let platform_row = rng.random_range(4..LEVEL_HEIGHT - 5);
        let platform_width = rng.random_range(3..8_usize);

        for offset in 0..platform_width {
            let column = platform_column + offset;
            if column < LEVEL_WIDTH {
                tiles[platform_row][column] = TILE_PLATFORM;
            }
        }

        if rng.random_range(0..3) == 0 {
            let coin_column = platform_column + platform_width / 2;
            if coin_column < LEVEL_WIDTH && platform_row >= 1 {
                coin_positions.push((coin_column, platform_row - 1));
            }
        }
    }

    let spike_count = 5 + level_number as usize * 3;
    for _ in 0..spike_count {
        let spike_column = rng.random_range(safe_start..safe_end);
        if tiles[LEVEL_HEIGHT - 3][spike_column] == TILE_AIR
            && tiles[LEVEL_HEIGHT - 2][spike_column] == TILE_GROUND
        {
            tiles[LEVEL_HEIGHT - 3][spike_column] = TILE_SPIKE;
        }
    }

    let ground_coin_count = 8 + level_number as usize * 2;
    for _ in 0..ground_coin_count {
        let coin_column = rng.random_range(5..LEVEL_WIDTH - 5);
        for row in 0..LEVEL_HEIGHT - 1 {
            if tiles[row][coin_column] == TILE_AIR
                && (tiles[row + 1][coin_column] == TILE_GROUND
                    || tiles[row + 1][coin_column] == TILE_PLATFORM)
            {
                coin_positions.push((coin_column, row));
                break;
            }
        }
    }

    let exit_column = LEVEL_WIDTH - 5;
    tiles[LEVEL_HEIGHT - 3][exit_column] = TILE_EXIT;

    LevelData {
        tiles,
        player_start: (3, LEVEL_HEIGHT - 3),
        coin_positions,
    }
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Platformer - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "PLATFORMER";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - title.len() as f64 / 2.0,
                row: center_row - 5.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: title.to_string(),
                foreground: TermColor::Cyan,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let art_lines = ["    @      ", "   /|\\    o", "   / \\  ===", "=========  "];
        for (line_index, line) in art_lines.iter().enumerate() {
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - line.len() as f64 / 2.0,
                    row: center_row - 2.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: line.to_string(),
                    foreground: TermColor::Yellow,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }

        let prompt = "Press ENTER to start";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - prompt.len() as f64 / 2.0,
                row: center_row + 4.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: prompt.to_string(),
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let controls = "Arrow keys: move | Z: jump | ESC: quit";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - controls.len() as f64 / 2.0,
                row: center_row + 6.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: controls.to_string(),
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Enter => self.start_game = true,
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.start_game {
            self.entities.despawn_all(world);
            return Some(Box::new(GameplayState::new(1)));
        }
        None
    }
}

struct CoinData {
    entity: Entity,
    grid_column: usize,
    grid_row: usize,
    collected: bool,
}

struct GameplayState {
    level_number: u32,
    level: LevelData,
    tilemap_entity: Entity,
    player_entity: Entity,
    player_column: f64,
    player_row: f64,
    player_velocity_column: f64,
    player_velocity_row: f64,
    on_ground: bool,
    facing_right: bool,
    player_animation: SpriteAnimation,
    name_label_entity: Entity,
    coins: Vec<CoinData>,
    coins_collected: u32,
    total_coins: u32,
    particles: ParticleEmitter,
    camera_tween_column: Option<Tween>,
    camera_column: f64,
    hud_entities: EntityGroup,
    move_left: bool,
    move_right: bool,
    jump_pressed: bool,
    lives: u32,
    score: u32,
    level_complete: bool,
    player_dead: bool,
    respawn_timer: Timer,
    offset_row: i32,
}

impl GameplayState {
    fn new(level_number: u32) -> Self {
        let level = create_level(level_number);
        let total_coins = level.coin_positions.len() as u32;
        Self {
            level_number,
            level,
            tilemap_entity: Entity::default(),
            player_entity: Entity::default(),
            player_column: 0.0,
            player_row: 0.0,
            player_velocity_column: 0.0,
            player_velocity_row: 0.0,
            on_ground: false,
            facing_right: true,
            player_animation: SpriteAnimation {
                frames: vec!['@', '&'],
                frame_duration: 0.3,
                elapsed: 0.0,
                current_frame: 0,
                looping: true,
                finished: false,
            },
            name_label_entity: Entity::default(),
            coins: Vec::new(),
            coins_collected: 0,
            total_coins,
            particles: ParticleEmitter::new(),
            camera_tween_column: None,
            camera_column: 0.0,
            hud_entities: EntityGroup::new(),
            move_left: false,
            move_right: false,
            jump_pressed: false,
            lives: 3,
            score: 0,
            level_complete: false,
            player_dead: false,
            respawn_timer: Timer::once(1.0),
            offset_row: 0,
        }
    }

    fn build_tilemap(&mut self, world: &mut World) {
        let terminal = world.resources.terminal_size;
        self.offset_row = ((terminal.rows as i32 - LEVEL_HEIGHT as i32 - 2) / 2).max(0);

        let mut tilemap = Tilemap::new(LEVEL_WIDTH, LEVEL_HEIGHT);
        for row in 0..LEVEL_HEIGHT {
            for column in 0..LEVEL_WIDTH {
                let cell = match self.level.tiles[row][column] {
                    TILE_GROUND => TilemapCell {
                        character: '#',
                        foreground: TermColor::Rgb {
                            r: 100,
                            g: 80,
                            b: 50,
                        },
                        background: TermColor::Rgb {
                            r: 40,
                            g: 30,
                            b: 15,
                        },
                    },
                    TILE_PLATFORM => TilemapCell {
                        character: '=',
                        foreground: TermColor::Rgb {
                            r: 150,
                            g: 120,
                            b: 80,
                        },
                        background: TermColor::Black,
                    },
                    TILE_SPIKE => TilemapCell {
                        character: '^',
                        foreground: TermColor::Red,
                        background: TermColor::Black,
                    },
                    TILE_EXIT => TilemapCell {
                        character: 'D',
                        foreground: TermColor::Green,
                        background: TermColor::Rgb { r: 0, g: 40, b: 0 },
                    },
                    _ => TilemapCell {
                        character: ' ',
                        foreground: TermColor::Black,
                        background: TermColor::Black,
                    },
                };
                tilemap.set(column, row, cell);
            }
        }

        self.tilemap_entity = EntityBuilder::new()
            .position(Position {
                column: 0.0,
                row: self.offset_row as f64,
            })
            .tilemap(tilemap)
            .z_index(ZIndex(0))
            .spawn(world);
    }

    fn spawn_player(&mut self, world: &mut World) {
        self.player_column = self.level.player_start.0 as f64;
        self.player_row = self.level.player_start.1 as f64;
        self.player_velocity_column = 0.0;
        self.player_velocity_row = 0.0;
        self.on_ground = false;
        self.player_dead = false;

        self.player_entity = EntityBuilder::new()
            .position(Position {
                column: self.player_column,
                row: self.player_row + self.offset_row as f64,
            })
            .sprite(Sprite {
                character: '@',
                foreground: TermColor::Cyan,
                background: TermColor::Black,
            })
            .sprite_animation(self.player_animation.clone())
            .z_index(ZIndex(5))
            .visibility(Visibility { visible: true })
            .spawn(world);

        self.name_label_entity = EntityBuilder::new()
            .position(Position {
                column: self.player_column - 2.0,
                row: self.player_row + self.offset_row as f64 - 1.0,
            })
            .label(Label {
                text: "Player".to_string(),
                foreground: TermColor::Cyan,
                background: TermColor::Black,
            })
            .z_index(ZIndex(6))
            .parent(Parent(self.player_entity))
            .local_offset(LocalOffset {
                column: -2.0,
                row: -1.0,
            })
            .spawn(world);
    }

    fn spawn_coins(&mut self, world: &mut World) {
        for &(column, row) in &self.level.coin_positions {
            let entity = EntityBuilder::new()
                .position(Position {
                    column: column as f64,
                    row: row as f64 + self.offset_row as f64,
                })
                .sprite(Sprite {
                    character: 'o',
                    foreground: TermColor::Yellow,
                    background: TermColor::Black,
                })
                .sprite_animation(SpriteAnimation {
                    frames: vec!['o', 'O', '0', 'O'],
                    frame_duration: 0.25,
                    elapsed: 0.0,
                    current_frame: 0,
                    looping: true,
                    finished: false,
                })
                .z_index(ZIndex(2))
                .spawn(world);

            self.coins.push(CoinData {
                entity,
                grid_column: column,
                grid_row: row,
                collected: false,
            });
        }
    }

    fn is_solid(&self, column: i32, row: i32) -> bool {
        if column < 0 || column >= LEVEL_WIDTH as i32 || row < 0 || row >= LEVEL_HEIGHT as i32 {
            return row >= LEVEL_HEIGHT as i32;
        }
        let tile = self.level.tiles[row as usize][column as usize];
        tile == TILE_GROUND || tile == TILE_PLATFORM
    }

    fn is_spike(&self, column: i32, row: i32) -> bool {
        if column < 0 || column >= LEVEL_WIDTH as i32 || row < 0 || row >= LEVEL_HEIGHT as i32 {
            return false;
        }
        self.level.tiles[row as usize][column as usize] == TILE_SPIKE
    }

    fn is_exit(&self, column: i32, row: i32) -> bool {
        if column < 0 || column >= LEVEL_WIDTH as i32 || row < 0 || row >= LEVEL_HEIGHT as i32 {
            return false;
        }
        self.level.tiles[row as usize][column as usize] == TILE_EXIT
    }

    fn update_player(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;

        self.player_velocity_column = 0.0;
        if self.move_left {
            self.player_velocity_column = -MOVE_SPEED;
            self.facing_right = false;
        }
        if self.move_right {
            self.player_velocity_column = MOVE_SPEED;
            self.facing_right = true;
        }

        if self.jump_pressed && self.on_ground {
            self.player_velocity_row = JUMP_VELOCITY;
            self.on_ground = false;
        }

        self.player_velocity_row += GRAVITY * delta;
        if self.player_velocity_row > MAX_FALL_SPEED {
            self.player_velocity_row = MAX_FALL_SPEED;
        }

        let new_column = self.player_column + self.player_velocity_column * delta;
        let new_row = self.player_row + self.player_velocity_row * delta;

        let player_grid_column = new_column.round() as i32;
        let player_grid_row_at_new_column = self.player_row.round() as i32;

        if !self.is_solid(player_grid_column, player_grid_row_at_new_column) {
            self.player_column = new_column;
        }

        let check_column = self.player_column.round() as i32;
        let new_grid_row = new_row.round() as i32;

        if self.player_velocity_row > 0.0 && self.is_solid(check_column, new_grid_row) {
            self.player_row = (new_grid_row - 1) as f64;
            self.player_velocity_row = 0.0;
            self.on_ground = true;
        } else if self.player_velocity_row < 0.0 && self.is_solid(check_column, new_grid_row) {
            self.player_row = (new_grid_row + 1) as f64;
            self.player_velocity_row = 0.0;
        } else {
            self.player_row = new_row;
            self.on_ground = false;
        }

        self.player_column = self.player_column.max(0.0).min((LEVEL_WIDTH - 1) as f64);

        if self.player_row > LEVEL_HEIGHT as f64 + 5.0 {
            self.kill_player(world);
            return;
        }

        let grid_column = self.player_column.round() as i32;
        let grid_row = self.player_row.round() as i32;

        if self.is_spike(grid_column, grid_row) {
            self.kill_player(world);
            return;
        }

        if self.is_exit(grid_column, grid_row) {
            self.level_complete = true;
            return;
        }

        if let Some(position) = world.get_position_mut(self.player_entity) {
            position.column = self.player_column;
            position.row = self.player_row + self.offset_row as f64;
        }

        if self.player_velocity_column.abs() > 0.1 && self.on_ground {
            if let Some(animation) = world.get_sprite_animation_mut(self.player_entity) {
                animation.frame_duration = 0.15;
            }
        } else if let Some(animation) = world.get_sprite_animation_mut(self.player_entity) {
            animation.frame_duration = 0.5;
        }
    }

    fn kill_player(&mut self, world: &mut World) {
        self.player_dead = true;
        self.lives = self.lives.saturating_sub(1);

        self.particles.emit(
            world,
            self.player_column,
            self.player_row + self.offset_row as f64,
            10,
            &ParticleConfig {
                characters: vec!['*', '+', 'x'],
                colors: vec![TermColor::Red, TermColor::Yellow],
                lifetime: 0.8,
                speed_min: 3.0,
                speed_max: 8.0,
                spread: std::f64::consts::PI * 2.0,
                direction: 0.0,
                z_index: 8,
            },
        );

        if let Some(visibility) = world.get_visibility_mut(self.player_entity) {
            visibility.visible = false;
        }

        self.respawn_timer.reset();
    }

    fn respawn_player(&mut self, world: &mut World) {
        self.player_column = self.level.player_start.0 as f64;
        self.player_row = self.level.player_start.1 as f64;
        self.player_velocity_column = 0.0;
        self.player_velocity_row = 0.0;
        self.on_ground = false;
        self.player_dead = false;

        if let Some(position) = world.get_position_mut(self.player_entity) {
            position.column = self.player_column;
            position.row = self.player_row + self.offset_row as f64;
        }
        if let Some(visibility) = world.get_visibility_mut(self.player_entity) {
            visibility.visible = true;
        }

        let target_camera = (self.player_column - VIEW_WIDTH as f64 / 2.0)
            .max(0.0)
            .min((LEVEL_WIDTH - VIEW_WIDTH) as f64);
        self.camera_tween_column = Some(Tween::new(
            self.camera_column,
            target_camera,
            0.3,
            Easing::EaseOut,
        ));
    }

    fn check_coin_collection(&mut self, world: &mut World) {
        let grid_column = self.player_column.round() as usize;
        let grid_row = self.player_row.round() as usize;

        for coin in &mut self.coins {
            if coin.collected {
                continue;
            }
            if coin.grid_column == grid_column && coin.grid_row == grid_row {
                coin.collected = true;
                self.coins_collected += 1;
                self.score += 100;
                world.despawn_entities(&[coin.entity]);

                self.particles.emit(
                    world,
                    coin.grid_column as f64,
                    coin.grid_row as f64 + self.offset_row as f64,
                    6,
                    &ParticleConfig {
                        characters: vec!['*', '.', '+'],
                        colors: vec![
                            TermColor::Yellow,
                            TermColor::Rgb {
                                r: 255,
                                g: 200,
                                b: 50,
                            },
                        ],
                        lifetime: 0.4,
                        speed_min: 2.0,
                        speed_max: 5.0,
                        spread: std::f64::consts::PI * 2.0,
                        direction: -std::f64::consts::FRAC_PI_2,
                        z_index: 7,
                    },
                );
            }
        }
    }

    fn update_camera(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;
        let target_column = (self.player_column - VIEW_WIDTH as f64 / 2.0)
            .max(0.0)
            .min((LEVEL_WIDTH - VIEW_WIDTH) as f64);

        if let Some(tween) = &mut self.camera_tween_column {
            self.camera_column = tween.tick(delta);
            if tween.finished() {
                self.camera_tween_column = None;
            }
        } else {
            let distance = (target_column - self.camera_column).abs();
            if distance > 10.0 {
                self.camera_tween_column = Some(Tween::new(
                    self.camera_column,
                    target_column,
                    0.5,
                    Easing::EaseOut,
                ));
            } else {
                self.camera_column += (target_column - self.camera_column) * 5.0 * delta;
            }
        }

        world.resources.camera.offset_column = self.camera_column;
        world.resources.camera.offset_row = 0.0;
    }

    fn update_hud(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);

        let hud_text = format!(
            "Level: {}  Coins: {}/{}  Lives: {}  Score: {}",
            self.level_number, self.coins_collected, self.total_coins, self.lives, self.score
        );

        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: self.camera_column + 1.0,
                row: self.offset_row as f64 + LEVEL_HEIGHT as f64 + 0.5,
            },
        );
        world.set_label(
            entity,
            Label {
                text: hud_text,
                foreground: TermColor::White,
                background: TermColor::Rgb {
                    r: 20,
                    g: 20,
                    b: 40,
                },
            },
        );
        world.set_z_index(entity, ZIndex(15));
    }

    fn clear_all(&mut self, world: &mut World) {
        world.despawn_entities(&[
            self.tilemap_entity,
            self.player_entity,
            self.name_label_entity,
        ]);
        for coin in &self.coins {
            if !coin.collected {
                world.despawn_entities(&[coin.entity]);
            }
        }
        self.coins.clear();
        self.particles.despawn_all(world);
        self.hud_entities.despawn_all(world);
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Platformer - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 60;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        self.build_tilemap(world);
        self.spawn_player(world);
        self.spawn_coins(world);

        self.camera_column = (self.player_column - VIEW_WIDTH as f64 / 2.0).max(0.0);
        world.resources.camera.offset_column = self.camera_column;

        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, _world: &mut World, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::Left | KeyCode::Char('a') => self.move_left = pressed,
            KeyCode::Right | KeyCode::Char('d') => self.move_right = pressed,
            KeyCode::Char('z') | KeyCode::Up | KeyCode::Char('w') => self.jump_pressed = pressed,
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;

        if self.player_dead {
            if self.respawn_timer.tick(delta) && self.lives > 0 {
                self.respawn_player(world);
            }
            self.particles.update(world, delta);
            self.update_hud(world);
            return;
        }

        self.update_player(world);
        self.check_coin_collection(world);
        animation_system(world);
        parent_transform_system(world);
        self.update_camera(world);
        self.particles.update(world, delta);
        self.update_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.level_complete {
            let next_level = self.level_number + 1;
            let score = self.score;
            let lives = self.lives;
            self.clear_all(world);

            if next_level > 3 {
                return Some(Box::new(WinState {
                    score,
                    entities: EntityGroup::new(),
                    restart: false,
                }));
            }

            let mut next = GameplayState::new(next_level);
            next.score = score;
            next.lives = lives;
            return Some(Box::new(next));
        }

        if self.player_dead && self.lives == 0 && self.respawn_timer.finished() {
            let score = self.score;
            self.clear_all(world);
            return Some(Box::new(GameOverState {
                score,
                level: self.level_number,
                entities: EntityGroup::new(),
                restart: false,
            }));
        }

        None
    }
}

struct GameOverState {
    score: u32,
    level: u32,
    entities: EntityGroup,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Platformer - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let lines: Vec<(String, TermColor)> = vec![
            ("GAME OVER".to_string(), TermColor::Red),
            (String::new(), TermColor::Black),
            (format!("Score: {}", self.score), TermColor::Yellow),
            (format!("Reached Level: {}", self.level), TermColor::Cyan),
            (String::new(), TermColor::Black),
            ("Press R to restart".to_string(), TermColor::White),
            ("Press ESC to quit".to_string(), TermColor::Grey),
        ];

        for (line_index, (text, color)) in lines.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - text.len() as f64 / 2.0,
                    row: center_row - 4.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: text.clone(),
                    foreground: *color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Char('r') => self.restart = true,
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.restart {
            self.entities.despawn_all(world);
            return Some(Box::new(GameplayState::new(1)));
        }
        None
    }
}

struct WinState {
    score: u32,
    entities: EntityGroup,
    restart: bool,
}

impl State for WinState {
    fn title(&self) -> &str {
        "Platformer - Victory!"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let lines: Vec<(String, TermColor)> = vec![
            ("YOU WIN!".to_string(), TermColor::Green),
            (String::new(), TermColor::Black),
            (format!("Final Score: {}", self.score), TermColor::Yellow),
            (String::new(), TermColor::Black),
            ("Press R to play again".to_string(), TermColor::White),
            ("Press ESC to quit".to_string(), TermColor::Grey),
        ];

        for (line_index, (text, color)) in lines.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - text.len() as f64 / 2.0,
                    row: center_row - 3.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: text.clone(),
                    foreground: *color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Char('r') => self.restart = true,
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.restart {
            self.entities.despawn_all(world);
            return Some(Box::new(GameplayState::new(1)));
        }
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(TitleScreenState {
        entities: EntityGroup::new(),
        start_game: false,
    }))
}
