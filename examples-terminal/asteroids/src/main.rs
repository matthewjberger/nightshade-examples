use nightshade::tui::prelude::*;
use rand::Rng;

const PLAY_WIDTH: i32 = 50;
const PLAY_HEIGHT: i32 = 25;
const SHIP_FIRE_COOLDOWN: f64 = 0.25;
const MOVE_TICK: f64 = 0.1;
const BULLET_LIFETIME: f64 = 2.0;
const INITIAL_ASTEROIDS: usize = 5;
const ROTATION_SPEED: f64 = 5.0;
const THRUST_POWER: f64 = 25.0;
const MAX_SPEED: f64 = 12.0;
const DRAG_PER_SECOND: f64 = 0.4;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    fn character(self) -> char {
        match self {
            Self::Large => 'O',
            Self::Medium => 'o',
            Self::Small => '.',
        }
    }

    fn color(self) -> TermColor {
        match self {
            Self::Large => TermColor::Rgb {
                r: 180,
                g: 150,
                b: 120,
            },
            Self::Medium => TermColor::Rgb {
                r: 160,
                g: 130,
                b: 100,
            },
            Self::Small => TermColor::Rgb {
                r: 140,
                g: 110,
                b: 80,
            },
        }
    }

    fn points(self) -> u32 {
        match self {
            Self::Large => 20,
            Self::Medium => 50,
            Self::Small => 100,
        }
    }

    fn smaller(self) -> Option<Self> {
        match self {
            Self::Large => Some(Self::Medium),
            Self::Medium => Some(Self::Small),
            Self::Small => None,
        }
    }
}

fn angle_to_character(angle: f64) -> char {
    let normalized = angle.rem_euclid(std::f64::consts::TAU);
    let sector =
        ((normalized + std::f64::consts::FRAC_PI_8) / std::f64::consts::FRAC_PI_4) as usize % 8;
    ['^', '/', '>', '\\', 'v', '/', '<', '\\'][sector]
}

fn angle_to_bullet_velocity(angle: f64) -> (f64, f64) {
    let normalized = angle.rem_euclid(std::f64::consts::TAU);
    let sector =
        ((normalized + std::f64::consts::FRAC_PI_8) / std::f64::consts::FRAC_PI_4) as usize % 8;
    [
        (0.0, -2.0),
        (1.0, -1.0),
        (2.0, 0.0),
        (1.0, 1.0),
        (0.0, 2.0),
        (-1.0, 1.0),
        (-2.0, 0.0),
        (-1.0, -1.0),
    ][sector]
}

fn angle_to_spawn_offset(angle: f64) -> (f64, f64) {
    let normalized = angle.rem_euclid(std::f64::consts::TAU);
    let sector =
        ((normalized + std::f64::consts::FRAC_PI_8) / std::f64::consts::FRAC_PI_4) as usize % 8;
    [
        (0.0, -1.0),
        (1.0, -1.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
        (-1.0, 1.0),
        (-1.0, 0.0),
        (-1.0, -1.0),
    ][sector]
}

struct AsteroidData {
    entity: Entity,
    size: AsteroidSize,
}

struct BulletData {
    entity: Entity,
    lifetime: f64,
}

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Asteroids - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r"    _        _                 _     _     ",
            r"   / \   ___| |_ ___ _ __ ___ (_) __| |___ ",
            r"  / _ \ / __| __/ _ \ '__/ _ \| |/ _` / __|",
            r" / ___ \\__ \ ||  __/ | | (_) | | (_| \__ \",
            r"/_/   \_\___/\__\___|_|  \___/|_|\__,_|___/",
        ];

        let title_start_row = center_row - 6;

        for (line_index, line) in title_lines.iter().enumerate() {
            let start_col = center_column - line.len() as i32 / 2;
            for (char_index, character) in line.chars().enumerate() {
                if character != ' ' {
                    let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                    world.set_position(
                        entity,
                        Position {
                            column: (start_col + char_index as i32) as f64,
                            row: (title_start_row + line_index as i32) as f64,
                        },
                    );
                    world.set_sprite(
                        entity,
                        Sprite {
                            character,
                            foreground: TermColor::Rgb {
                                r: 200,
                                g: 200,
                                b: 255,
                            },
                            background: TermColor::Black,
                        },
                    );
                    world.set_z_index(entity, ZIndex(10));
                }
            }
        }

        let art = "  O   o  .  o   O";
        let art_start = center_column - art.len() as i32 / 2;
        for (char_index, character) in art.chars().enumerate() {
            if character != ' ' {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (art_start + char_index as i32) as f64,
                        row: (title_start_row + 7) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: TermColor::Rgb {
                            r: 180,
                            g: 150,
                            b: 120,
                        },
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(10));
            }
        }

        let prompt = "Press ENTER to start";
        let prompt_start = center_column - prompt.len() as i32 / 2;
        for (char_index, character) in prompt.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (prompt_start + char_index as i32) as f64,
                    row: (title_start_row + 10) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: TermColor::White,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }

        let quit_hint = "Press ESC to quit";
        let quit_start = center_column - quit_hint.len() as i32 / 2;
        for (char_index, character) in quit_hint.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (quit_start + char_index as i32) as f64,
                    row: (title_start_row + 12) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: TermColor::Grey,
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
            KeyCode::Enter => self.start_game = true,
            KeyCode::Escape | KeyCode::Char('q') => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.start_game {
            let all_entities: Vec<Entity> = world.query_entities(POSITION | SPRITE).collect();
            world.despawn_entities(&all_entities);
            return Some(Box::new(GameplayState::new()));
        }
        None
    }
}

struct GameplayState {
    play_offset_x: i32,
    play_offset_y: i32,
    ship_entity: Entity,
    ship_x: f64,
    ship_y: f64,
    ship_angle: f64,
    ship_velocity_x: f64,
    ship_velocity_y: f64,
    rotating_left: bool,
    rotating_right: bool,
    thrusting: bool,
    asteroids: Vec<AsteroidData>,
    bullets: Vec<BulletData>,
    hud_entities: Vec<Entity>,
    score: u32,
    lives: u32,
    wave: u32,
    game_over: bool,
    fire_cooldown: f64,
    move_tick_timer: f64,
    invulnerable_timer: f64,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            play_offset_x: 0,
            play_offset_y: 0,
            ship_entity: Entity::default(),
            ship_x: PLAY_WIDTH as f64 / 2.0,
            ship_y: PLAY_HEIGHT as f64 / 2.0,
            ship_angle: 0.0,
            ship_velocity_x: 0.0,
            ship_velocity_y: 0.0,
            rotating_left: false,
            rotating_right: false,
            thrusting: false,
            asteroids: Vec::new(),
            bullets: Vec::new(),
            hud_entities: Vec::new(),
            score: 0,
            lives: 3,
            wave: 1,
            game_over: false,
            fire_cooldown: 0.0,
            move_tick_timer: 0.0,
            invulnerable_timer: 2.0,
        }
    }

    fn spawn_ship(&mut self, world: &mut World) {
        self.ship_x = PLAY_WIDTH as f64 / 2.0;
        self.ship_y = PLAY_HEIGHT as f64 / 2.0;
        self.ship_angle = 0.0;
        self.ship_velocity_x = 0.0;
        self.ship_velocity_y = 0.0;
        self.ship_entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
        world.set_position(
            self.ship_entity,
            Position {
                column: self.play_offset_x as f64 + self.ship_x,
                row: self.play_offset_y as f64 + self.ship_y,
            },
        );
        world.set_sprite(
            self.ship_entity,
            Sprite {
                character: '^',
                foreground: TermColor::Rgb {
                    r: 100,
                    g: 255,
                    b: 100,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(self.ship_entity, ZIndex(3));
        world.set_collider(
            self.ship_entity,
            Collider {
                width: 1,
                height: 1,
                ..Default::default()
            },
        );
    }

    fn spawn_asteroid(
        &mut self,
        world: &mut World,
        column: f64,
        row: f64,
        velocity_column: f64,
        velocity_row: f64,
        size: AsteroidSize,
    ) {
        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | VELOCITY | COLLIDER, 1)[0];
        world.set_position(
            entity,
            Position {
                column: self.play_offset_x as f64 + column,
                row: self.play_offset_y as f64 + row,
            },
        );
        world.set_velocity(
            entity,
            Velocity {
                column: velocity_column,
                row: velocity_row,
            },
        );
        world.set_sprite(
            entity,
            Sprite {
                character: size.character(),
                foreground: size.color(),
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(1));
        world.set_collider(
            entity,
            Collider {
                width: 1,
                height: 1,
                ..Default::default()
            },
        );
        self.asteroids.push(AsteroidData { entity, size });
    }

    fn spawn_wave(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        let asteroid_count = INITIAL_ASTEROIDS + (self.wave as usize - 1) * 2;

        for _ in 0..asteroid_count {
            let edge = rng.random_range(0..4);
            let (column, row): (f64, f64) = match edge {
                0 => (rng.random_range(0..PLAY_WIDTH) as f64, 0.0),
                1 => (
                    rng.random_range(0..PLAY_WIDTH) as f64,
                    (PLAY_HEIGHT - 1) as f64,
                ),
                2 => (0.0, rng.random_range(0..PLAY_HEIGHT) as f64),
                _ => (
                    (PLAY_WIDTH - 1) as f64,
                    rng.random_range(0..PLAY_HEIGHT) as f64,
                ),
            };

            let velocity_column: i32 = rng.random_range(-1..=1);
            let velocity_row: i32 = rng.random_range(-1..=1);
            let velocity_column = if velocity_column == 0 && velocity_row == 0 {
                1
            } else {
                velocity_column
            };

            self.spawn_asteroid(
                world,
                column,
                row,
                velocity_column as f64,
                velocity_row as f64,
                AsteroidSize::Large,
            );
        }
    }

    fn fire_bullet(&mut self, world: &mut World) {
        let (offset_column, offset_row) = angle_to_spawn_offset(self.ship_angle);
        let (velocity_column, velocity_row) = angle_to_bullet_velocity(self.ship_angle);

        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | VELOCITY | COLLIDER, 1)[0];
        world.set_position(
            entity,
            Position {
                column: self.play_offset_x as f64 + self.ship_x + offset_column,
                row: self.play_offset_y as f64 + self.ship_y + offset_row,
            },
        );
        world.set_velocity(
            entity,
            Velocity {
                column: velocity_column,
                row: velocity_row,
            },
        );
        world.set_sprite(
            entity,
            Sprite {
                character: '*',
                foreground: TermColor::Rgb {
                    r: 255,
                    g: 255,
                    b: 100,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(2));
        world.set_collider(
            entity,
            Collider {
                width: 1,
                height: 1,
                ..Default::default()
            },
        );
        self.bullets.push(BulletData {
            entity,
            lifetime: BULLET_LIFETIME,
        });
    }

    fn wrap_position(value: f64, max: i32) -> f64 {
        let max_f = max as f64;
        if value < 0.0 {
            max_f - 1.0
        } else if value >= max_f {
            0.0
        } else {
            value
        }
    }

    fn wrap_entities(&self, world: &mut World) {
        for asteroid in &self.asteroids {
            if let Some(position) = world.get_position_mut(asteroid.entity) {
                let local_col = position.column - self.play_offset_x as f64;
                let local_row = position.row - self.play_offset_y as f64;
                position.column =
                    self.play_offset_x as f64 + Self::wrap_position(local_col, PLAY_WIDTH);
                position.row =
                    self.play_offset_y as f64 + Self::wrap_position(local_row, PLAY_HEIGHT);
            }
        }
    }

    fn handle_collisions(&mut self, world: &mut World) {
        let contacts = collision_pairs(world);
        let mut bullets_to_remove: Vec<Entity> = Vec::new();
        let mut asteroids_to_split: Vec<(usize, f64, f64)> = Vec::new();
        let mut ship_hit = false;

        for contact in &contacts {
            let a_is_bullet = self
                .bullets
                .iter()
                .any(|bullet| bullet.entity == contact.entity_a);
            let b_is_bullet = self
                .bullets
                .iter()
                .any(|bullet| bullet.entity == contact.entity_b);
            let a_asteroid_idx = self
                .asteroids
                .iter()
                .position(|asteroid| asteroid.entity == contact.entity_a);
            let b_asteroid_idx = self
                .asteroids
                .iter()
                .position(|asteroid| asteroid.entity == contact.entity_b);
            let a_is_ship = contact.entity_a == self.ship_entity;
            let b_is_ship = contact.entity_b == self.ship_entity;

            if let (true, Some(asteroid_idx)) = (a_is_bullet, b_asteroid_idx) {
                if !asteroids_to_split
                    .iter()
                    .any(|(idx, _, _)| *idx == asteroid_idx)
                {
                    let position = world.get_position(contact.entity_b).copied();
                    if let Some(position) = position {
                        asteroids_to_split.push((
                            asteroid_idx,
                            position.column - self.play_offset_x as f64,
                            position.row - self.play_offset_y as f64,
                        ));
                    }
                }
                bullets_to_remove.push(contact.entity_a);
            } else if let (true, Some(asteroid_idx)) = (b_is_bullet, a_asteroid_idx) {
                if !asteroids_to_split
                    .iter()
                    .any(|(idx, _, _)| *idx == asteroid_idx)
                {
                    let position = world.get_position(contact.entity_a).copied();
                    if let Some(position) = position {
                        asteroids_to_split.push((
                            asteroid_idx,
                            position.column - self.play_offset_x as f64,
                            position.row - self.play_offset_y as f64,
                        ));
                    }
                }
                bullets_to_remove.push(contact.entity_b);
            }

            if ((a_is_ship && b_asteroid_idx.is_some()) || (b_is_ship && a_asteroid_idx.is_some()))
                && self.invulnerable_timer <= 0.0
            {
                ship_hit = true;
            }
        }

        asteroids_to_split.sort_by(|a, b| b.0.cmp(&a.0));
        for (asteroid_idx, column, row) in &asteroids_to_split {
            let size = self.asteroids[*asteroid_idx].size;
            self.score += size.points();
            world.despawn_entities(&[self.asteroids[*asteroid_idx].entity]);
            self.asteroids.remove(*asteroid_idx);

            if let Some(smaller_size) = size.smaller() {
                let mut rng = rand::rng();
                for _ in 0..2 {
                    let velocity_column: i32 = rng.random_range(-1..=1);
                    let velocity_row: i32 = rng.random_range(-1..=1);
                    let velocity_column = if velocity_column == 0 && velocity_row == 0 {
                        1
                    } else {
                        velocity_column
                    };
                    self.spawn_asteroid(
                        world,
                        *column,
                        *row,
                        velocity_column as f64,
                        velocity_row as f64,
                        smaller_size,
                    );
                }
            }
        }

        for entity in &bullets_to_remove {
            world.despawn_entities(&[*entity]);
        }
        self.bullets
            .retain(|bullet| !bullets_to_remove.contains(&bullet.entity));

        if ship_hit {
            self.lives = self.lives.saturating_sub(1);
            if self.lives == 0 {
                self.game_over = true;
            } else {
                self.invulnerable_timer = 2.0;
                self.ship_x = PLAY_WIDTH as f64 / 2.0;
                self.ship_y = PLAY_HEIGHT as f64 / 2.0;
                self.ship_velocity_x = 0.0;
                self.ship_velocity_y = 0.0;
                if let Some(position) = world.get_position_mut(self.ship_entity) {
                    position.column = self.play_offset_x as f64 + self.ship_x;
                    position.row = self.play_offset_y as f64 + self.ship_y;
                }
            }
        }

        if self.asteroids.is_empty() && !self.game_over {
            self.wave += 1;
            self.spawn_wave(world);
        }
    }

    fn update_bullet_lifetimes(&mut self, world: &mut World, delta: f64) {
        let mut expired = Vec::new();
        for bullet in &mut self.bullets {
            bullet.lifetime -= delta;
            if bullet.lifetime <= 0.0 {
                expired.push(bullet.entity);
            }
        }
        for entity in &expired {
            world.despawn_entities(&[*entity]);
        }
        self.bullets
            .retain(|bullet| !expired.contains(&bullet.entity));
    }

    fn update_ship_blink(&self, world: &mut World) {
        if self.invulnerable_timer > 0.0 {
            let blink = (self.invulnerable_timer * 10.0) as i32 % 2 == 0;
            if let Some(sprite) = world.get_sprite_mut(self.ship_entity) {
                sprite.foreground = if blink {
                    TermColor::Rgb {
                        r: 100,
                        g: 255,
                        b: 100,
                    }
                } else {
                    TermColor::Rgb {
                        r: 40,
                        g: 80,
                        b: 40,
                    }
                };
            }
        } else if let Some(sprite) = world.get_sprite_mut(self.ship_entity) {
            sprite.foreground = TermColor::Rgb {
                r: 100,
                g: 255,
                b: 100,
            };
        }
    }

    fn update_hud(&mut self, world: &mut World) {
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();

        let hud_text = format!(
            "Score: {:06}  Lives: {}  Wave: {}",
            self.score, self.lives, self.wave
        );

        for (char_index, character) in hud_text.chars().enumerate() {
            if char_index >= PLAY_WIDTH as usize {
                break;
            }
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: self.play_offset_x as f64 + char_index as f64,
                    row: self.play_offset_y as f64 + PLAY_HEIGHT as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: TermColor::White,
                    background: TermColor::Rgb {
                        r: 10,
                        g: 10,
                        b: 30,
                    },
                },
            );
            world.set_z_index(entity, ZIndex(10));
            self.hud_entities.push(entity);
        }

        for fill_index in hud_text.len()..PLAY_WIDTH as usize {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: self.play_offset_x as f64 + fill_index as f64,
                    row: self.play_offset_y as f64 + PLAY_HEIGHT as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: ' ',
                    foreground: TermColor::White,
                    background: TermColor::Rgb {
                        r: 10,
                        g: 10,
                        b: 30,
                    },
                },
            );
            world.set_z_index(entity, ZIndex(10));
            self.hud_entities.push(entity);
        }
    }

    fn clear_all_entities(&mut self, world: &mut World) {
        world.despawn_entities(&[self.ship_entity]);
        for asteroid in &self.asteroids {
            world.despawn_entities(&[asteroid.entity]);
        }
        self.asteroids.clear();
        for bullet in &self.bullets {
            world.despawn_entities(&[bullet.entity]);
        }
        self.bullets.clear();
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Asteroids - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        self.play_offset_x = (terminal.columns as i32 - PLAY_WIDTH) / 2;
        self.play_offset_y = (terminal.rows as i32 - PLAY_HEIGHT - 1) / 2;
        if self.play_offset_x < 0 {
            self.play_offset_x = 0;
        }
        if self.play_offset_y < 0 {
            self.play_offset_y = 0;
        }

        self.spawn_ship(world);
        self.spawn_wave(world);
        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::Left | KeyCode::Char('a') => self.rotating_left = pressed,
            KeyCode::Right | KeyCode::Char('d') => self.rotating_right = pressed,
            KeyCode::Up | KeyCode::Char('w') => self.thrusting = pressed,
            KeyCode::Char(' ') => {
                if pressed && self.fire_cooldown <= 0.0 && !self.game_over {
                    self.fire_bullet(world);
                    self.fire_cooldown = SHIP_FIRE_COOLDOWN;
                }
            }
            KeyCode::Escape | KeyCode::Char('q') => {
                if pressed {
                    world.resources.should_exit = true;
                }
            }
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.game_over {
            return;
        }

        let delta = world.resources.timing.delta_seconds;

        if self.fire_cooldown > 0.0 {
            self.fire_cooldown -= delta;
        }
        if self.invulnerable_timer > 0.0 {
            self.invulnerable_timer -= delta;
        }

        if self.rotating_left {
            self.ship_angle -= ROTATION_SPEED * delta;
        }
        if self.rotating_right {
            self.ship_angle += ROTATION_SPEED * delta;
        }
        self.ship_angle = self.ship_angle.rem_euclid(std::f64::consts::TAU);

        if self.thrusting {
            self.ship_velocity_x += self.ship_angle.sin() * THRUST_POWER * delta;
            self.ship_velocity_y += -(self.ship_angle.cos()) * THRUST_POWER * delta;
            let speed = (self.ship_velocity_x * self.ship_velocity_x
                + self.ship_velocity_y * self.ship_velocity_y)
                .sqrt();
            if speed > MAX_SPEED {
                self.ship_velocity_x = self.ship_velocity_x / speed * MAX_SPEED;
                self.ship_velocity_y = self.ship_velocity_y / speed * MAX_SPEED;
            }
        }

        let drag = DRAG_PER_SECOND.powf(delta);
        self.ship_velocity_x *= drag;
        self.ship_velocity_y *= drag;

        self.ship_x += self.ship_velocity_x * delta;
        self.ship_y += self.ship_velocity_y * delta;
        self.ship_x = self.ship_x.rem_euclid(PLAY_WIDTH as f64);
        self.ship_y = self.ship_y.rem_euclid(PLAY_HEIGHT as f64);

        if let Some(position) = world.get_position_mut(self.ship_entity) {
            position.column = self.play_offset_x as f64 + self.ship_x;
            position.row = self.play_offset_y as f64 + self.ship_y;
        }
        if let Some(sprite) = world.get_sprite_mut(self.ship_entity) {
            sprite.character = angle_to_character(self.ship_angle);
        }

        self.move_tick_timer += delta;
        if self.move_tick_timer >= MOVE_TICK {
            self.move_tick_timer -= MOVE_TICK;
            movement_system(world);
            self.wrap_entities(world);
        }

        self.handle_collisions(world);
        self.update_bullet_lifetimes(world, delta);
        self.update_ship_blink(world);
        self.update_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            let score = self.score;
            let wave = self.wave;
            self.clear_all_entities(world);
            return Some(Box::new(GameOverState {
                score,
                wave,
                restart: false,
            }));
        }
        None
    }
}

struct GameOverState {
    score: u32,
    wave: u32,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Asteroids - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let lines: Vec<(String, TermColor)> = vec![
            ("GAME OVER".to_string(), TermColor::Red),
            (String::new(), TermColor::Black),
            (
                format!("Score: {:06}", self.score),
                TermColor::Rgb {
                    r: 255,
                    g: 255,
                    b: 100,
                },
            ),
            (
                format!("Wave: {}", self.wave),
                TermColor::Rgb {
                    r: 100,
                    g: 200,
                    b: 255,
                },
            ),
            (String::new(), TermColor::Black),
            ("Press R to restart".to_string(), TermColor::White),
            ("Press ESC to quit".to_string(), TermColor::Grey),
        ];

        for (line_index, (text, color)) in lines.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let start_col = center_column - text.len() as i32 / 2;
            for (char_index, character) in text.chars().enumerate() {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (start_col + char_index as i32) as f64,
                        row: (center_row - 4 + line_index as i32) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: *color,
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(10));
            }
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Char('r') => self.restart = true,
            KeyCode::Escape | KeyCode::Char('q') => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.restart {
            let all_entities: Vec<Entity> = world.query_entities(POSITION | SPRITE).collect();
            world.despawn_entities(&all_entities);
            return Some(Box::new(GameplayState::new()));
        }
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(TitleScreenState { start_game: false }))
}
