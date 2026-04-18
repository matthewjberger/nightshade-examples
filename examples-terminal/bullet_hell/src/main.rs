use nightshade::tui::prelude::*;
use rand::Rng;

const PLAY_WIDTH: i32 = 60;
const PLAY_HEIGHT: i32 = 25;
const PLAYER_SPEED: f64 = 30.0;
const PLAYER_FOCUSED_SPEED: f64 = 14.0;
const PLAYER_FIRE_RATE: f64 = 0.08;
const PLAYER_BULLET_SPEED: f64 = 40.0;
const ENEMY_BULLET_SPEED: f64 = 12.0;
const BOSS_BULLET_SPEED: f64 = 10.0;
const COMBO_DECAY_TIME: f64 = 2.0;
const BOMB_DURATION: f64 = 0.5;
const INVULNERABILITY_DURATION: f64 = 2.0;
const BOSS_WAVE_INTERVAL: u32 = 5;
const STAR_COUNT: usize = 50;
const INITIAL_LIVES: u32 = 3;
const INITIAL_BOMBS: u32 = 2;
const WAVE_SPAWN_INTERVAL: f64 = 3.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BulletPatternKind {
    Spiral,
    Aimed,
    Spread,
    Ring,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnemyKind {
    Normal,
    Boss,
}

struct Bullet {
    entity: Entity,
    velocity_column: f64,
    velocity_row: f64,
}

struct EnemyData {
    entity: Entity,
    kind: EnemyKind,
    health: i32,
    max_health: i32,
    column: f64,
    row: f64,
    pattern: BulletPatternKind,
    fire_timer: f64,
    fire_interval: f64,
    spiral_angle: f64,
    movement_timer: f64,
    target_column: f64,
    points: u32,
    entered: bool,
}

struct PlayerBullet {
    entity: Entity,
    velocity_row: f64,
}

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Bullet Hell - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 60;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r" ____        _ _      _     _   _      _ _ ",
            r"| __ ) _   _| | | ___| |_  | | | | ___| | |",
            r"|  _ \| | | | | |/ _ \ __| | |_| |/ _ \ | |",
            r"| |_) | |_| | | |  __/ |_  |  _  |  __/ | |",
            r"|____/ \__,_|_|_|\___|\__| |_| |_|\___|_|_|",
        ];

        let title_start_row = center_row - 7;

        for (line_index, line) in title_lines.iter().enumerate() {
            let start_column = center_column - line.len() as i32 / 2;
            for (char_index, character) in line.chars().enumerate() {
                if character != ' ' {
                    let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                    world.set_position(
                        entity,
                        Position {
                            column: (start_column + char_index as i32) as f64,
                            row: (title_start_row + line_index as i32) as f64,
                        },
                    );
                    world.set_sprite(
                        entity,
                        Sprite {
                            character,
                            foreground: TermColor::Rgb {
                                r: 255,
                                g: 80,
                                b: 80,
                            },
                            background: TermColor::Black,
                        },
                    );
                    world.set_z_index(entity, ZIndex(10));
                }
            }
        }

        let controls_lines = [
            "Arrow Keys - Move",
            "Z - Shoot",
            "X - Bomb (clears bullets)",
            "C - Focus (slow movement)",
            "",
            "Press ENTER to start",
            "Press ESC to quit",
        ];

        for (line_index, line) in controls_lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            let start_column = center_column - line.len() as i32 / 2;
            let color = if line_index >= 5 {
                if line_index == 5 {
                    TermColor::White
                } else {
                    TermColor::Grey
                }
            } else {
                TermColor::Rgb {
                    r: 200,
                    g: 200,
                    b: 255,
                }
            };
            for (char_index, character) in line.chars().enumerate() {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (start_column + char_index as i32) as f64,
                        row: (title_start_row + 7 + line_index as i32) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: color,
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
            KeyCode::Enter => {
                self.start_game = true;
            }
            KeyCode::Escape | KeyCode::Char('q') => {
                world.resources.should_exit = true;
            }
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
    play_offset_column: i32,
    play_offset_row: i32,
    player_entity: Entity,
    player_column: f64,
    player_row: f64,
    player_bullets: Vec<PlayerBullet>,
    enemy_bullets: Vec<Bullet>,
    enemies: Vec<EnemyData>,
    star_entities: Vec<Entity>,
    hud_entities: Vec<Entity>,
    particle_emitter: ParticleEmitter,
    boss_health_bar: Option<ProgressBar>,
    score: u64,
    combo: u32,
    combo_timer: f64,
    lives: u32,
    bombs: u32,
    wave: u32,
    wave_timer: f64,
    fire_cooldown: f64,
    bomb_active_timer: f64,
    invulnerable_timer: f64,
    game_over: bool,
    move_up: bool,
    move_down: bool,
    move_left: bool,
    move_right: bool,
    shooting: bool,
    focused: bool,
    enemies_spawned_this_wave: u32,
    enemies_per_wave: u32,
    spawn_timer: f64,
    wave_complete: bool,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            play_offset_column: 0,
            play_offset_row: 0,
            player_entity: Entity::default(),
            player_column: PLAY_WIDTH as f64 / 2.0,
            player_row: PLAY_HEIGHT as f64 - 3.0,
            player_bullets: Vec::new(),
            enemy_bullets: Vec::new(),
            enemies: Vec::new(),
            star_entities: Vec::new(),
            hud_entities: Vec::new(),
            particle_emitter: ParticleEmitter::new(),
            boss_health_bar: None,
            score: 0,
            combo: 0,
            combo_timer: 0.0,
            lives: INITIAL_LIVES,
            bombs: INITIAL_BOMBS,
            wave: 0,
            wave_timer: 0.0,
            fire_cooldown: 0.0,
            bomb_active_timer: 0.0,
            invulnerable_timer: INVULNERABILITY_DURATION,
            game_over: false,
            move_up: false,
            move_down: false,
            move_left: false,
            move_right: false,
            shooting: false,
            focused: false,
            enemies_spawned_this_wave: 0,
            enemies_per_wave: 3,
            spawn_timer: 0.0,
            wave_complete: false,
        }
    }

    fn spawn_starfield(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        for _ in 0..STAR_COUNT {
            let column = rng.random_range(0..PLAY_WIDTH);
            let row = rng.random_range(0..PLAY_HEIGHT);
            let brightness = rng.random_range(20u8..80u8);
            let characters = ['.', '+', '*'];
            let character = characters[rng.random_range(0..characters.len())];
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (self.play_offset_column + column) as f64,
                    row: (self.play_offset_row + row) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: TermColor::Rgb {
                        r: brightness,
                        g: brightness,
                        b: brightness + 20,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(0));
            self.star_entities.push(entity);
        }
    }

    fn spawn_player(&mut self, world: &mut World) {
        self.player_column = PLAY_WIDTH as f64 / 2.0;
        self.player_row = PLAY_HEIGHT as f64 - 3.0;
        self.player_entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
        world.set_position(
            self.player_entity,
            Position {
                column: self.play_offset_column as f64 + self.player_column,
                row: self.play_offset_row as f64 + self.player_row,
            },
        );
        world.set_sprite(
            self.player_entity,
            Sprite {
                character: 'A',
                foreground: TermColor::Rgb {
                    r: 100,
                    g: 255,
                    b: 100,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(self.player_entity, ZIndex(5));
        world.set_collider(
            self.player_entity,
            Collider {
                width: 1,
                height: 1,
                ..Default::default()
            },
        );
    }

    fn start_next_wave(&mut self) {
        self.wave += 1;
        self.enemies_spawned_this_wave = 0;
        self.wave_complete = false;
        self.spawn_timer = 0.0;

        if self.wave.is_multiple_of(BOSS_WAVE_INTERVAL) {
            self.enemies_per_wave = 1;
        } else {
            self.enemies_per_wave = 3 + self.wave / 2;
            if self.enemies_per_wave > 8 {
                self.enemies_per_wave = 8;
            }
        }
    }

    fn spawn_enemy(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        let is_boss = self.wave.is_multiple_of(BOSS_WAVE_INTERVAL);

        if is_boss {
            let column = PLAY_WIDTH as f64 / 2.0;
            let health = 30 + (self.wave as i32 / BOSS_WAVE_INTERVAL as i32) * 15;
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: self.play_offset_column as f64 + column,
                    row: self.play_offset_row as f64 - 1.0,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: 'W',
                    foreground: TermColor::Rgb {
                        r: 255,
                        g: 50,
                        b: 200,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(3));
            world.set_collider(
                entity,
                Collider {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
            );

            let boss_bar = ProgressBar::new(
                20,
                self.play_offset_column as f64 + (PLAY_WIDTH as f64 / 2.0) - 10.0,
                self.play_offset_row as f64 + 1.0,
                ProgressBarColors {
                    filled_foreground: TermColor::Red,
                    filled_background: TermColor::Black,
                    empty_foreground: TermColor::DarkGrey,
                    empty_background: TermColor::Black,
                },
                15,
            );
            self.boss_health_bar = Some(boss_bar);

            self.enemies.push(EnemyData {
                entity,
                kind: EnemyKind::Boss,
                health,
                max_health: health,
                column,
                row: -1.0,
                pattern: BulletPatternKind::Spiral,
                fire_timer: 0.0,
                fire_interval: 0.12,
                spiral_angle: 0.0,
                movement_timer: 0.0,
                target_column: column,
                points: 1000 + self.wave * 200,
                entered: false,
            });
        } else {
            let column = rng.random_range(4.0..(PLAY_WIDTH - 4) as f64);
            let patterns = [
                BulletPatternKind::Aimed,
                BulletPatternKind::Spread,
                BulletPatternKind::Ring,
                BulletPatternKind::Spiral,
            ];
            let pattern = patterns[rng.random_range(0..patterns.len())];
            let health = 2 + (self.wave as i32 / 3);
            let fire_interval = match pattern {
                BulletPatternKind::Spiral => 0.15,
                BulletPatternKind::Aimed => 0.8,
                BulletPatternKind::Spread => 1.0,
                BulletPatternKind::Ring => 1.2,
            };

            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: self.play_offset_column as f64 + column,
                    row: self.play_offset_row as f64 - 1.0,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: 'V',
                    foreground: TermColor::Rgb {
                        r: 255,
                        g: 100,
                        b: 100,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(3));
            world.set_collider(
                entity,
                Collider {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
            );

            self.enemies.push(EnemyData {
                entity,
                kind: EnemyKind::Normal,
                health,
                max_health: health,
                column,
                row: -1.0,
                pattern,
                fire_timer: 0.0,
                fire_interval,
                spiral_angle: rng.random_range(0.0..std::f64::consts::TAU),
                movement_timer: 0.0,
                target_column: rng.random_range(4.0..(PLAY_WIDTH - 4) as f64),
                points: 100 + self.wave * 10,
                entered: false,
            });
        }
    }

    fn fire_player_bullet(&mut self, world: &mut World) {
        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
        world.set_position(
            entity,
            Position {
                column: self.play_offset_column as f64 + self.player_column,
                row: self.play_offset_row as f64 + self.player_row - 1.0,
            },
        );
        world.set_sprite(
            entity,
            Sprite {
                character: '|',
                foreground: TermColor::Rgb {
                    r: 100,
                    g: 255,
                    b: 255,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(4));
        world.set_collider(
            entity,
            Collider {
                width: 1,
                height: 1,
                ..Default::default()
            },
        );
        self.player_bullets.push(PlayerBullet {
            entity,
            velocity_row: -PLAYER_BULLET_SPEED,
        });
    }

    fn spawn_enemy_bullet(
        &mut self,
        world: &mut World,
        column: f64,
        row: f64,
        velocity_column: f64,
        velocity_row: f64,
        color: TermColor,
    ) {
        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
        world.set_position(
            entity,
            Position {
                column: self.play_offset_column as f64 + column,
                row: self.play_offset_row as f64 + row,
            },
        );
        world.set_sprite(
            entity,
            Sprite {
                character: '.',
                foreground: color,
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
        self.enemy_bullets.push(Bullet {
            entity,
            velocity_column,
            velocity_row,
        });
    }

    fn fire_enemy_patterns(&mut self, world: &mut World) {
        let mut bullets_to_spawn: Vec<(f64, f64, f64, f64, TermColor)> = Vec::new();

        for enemy in &mut self.enemies {
            if !enemy.entered {
                continue;
            }

            enemy.fire_timer -= world.resources.timing.delta_seconds;
            if enemy.fire_timer > 0.0 {
                continue;
            }
            enemy.fire_timer = enemy.fire_interval;

            let bullet_speed = if enemy.kind == EnemyKind::Boss {
                BOSS_BULLET_SPEED
            } else {
                ENEMY_BULLET_SPEED
            };

            match enemy.pattern {
                BulletPatternKind::Spiral => {
                    let bullet_count = if enemy.kind == EnemyKind::Boss { 3 } else { 2 };
                    let angle_step = std::f64::consts::TAU / bullet_count as f64;
                    for bullet_index in 0..bullet_count {
                        let angle = enemy.spiral_angle + angle_step * bullet_index as f64;
                        let velocity_column = angle.cos() * bullet_speed;
                        let velocity_row = angle.sin() * bullet_speed;
                        let color = if enemy.kind == EnemyKind::Boss {
                            TermColor::Rgb {
                                r: 255,
                                g: 50,
                                b: 200,
                            }
                        } else {
                            TermColor::Rgb {
                                r: 255,
                                g: 200,
                                b: 50,
                            }
                        };
                        bullets_to_spawn.push((
                            enemy.column,
                            enemy.row,
                            velocity_column,
                            velocity_row,
                            color,
                        ));
                    }
                    enemy.spiral_angle += 0.3;
                }
                BulletPatternKind::Aimed => {
                    let delta_column = self.player_column - enemy.column;
                    let delta_row = self.player_row - enemy.row;
                    let distance = (delta_column * delta_column + delta_row * delta_row).sqrt();
                    if distance > 0.01 {
                        let velocity_column = (delta_column / distance) * bullet_speed;
                        let velocity_row = (delta_row / distance) * bullet_speed;
                        bullets_to_spawn.push((
                            enemy.column,
                            enemy.row,
                            velocity_column,
                            velocity_row,
                            TermColor::Rgb {
                                r: 255,
                                g: 80,
                                b: 80,
                            },
                        ));
                    }
                }
                BulletPatternKind::Spread => {
                    let delta_column = self.player_column - enemy.column;
                    let delta_row = self.player_row - enemy.row;
                    let base_angle = delta_row.atan2(delta_column);
                    let spread_count = if enemy.kind == EnemyKind::Boss { 7 } else { 5 };
                    let total_spread = std::f64::consts::PI * 0.5;
                    let angle_step = total_spread / (spread_count - 1) as f64;
                    let start_angle = base_angle - total_spread / 2.0;
                    for bullet_index in 0..spread_count {
                        let angle = start_angle + angle_step * bullet_index as f64;
                        let velocity_column = angle.cos() * bullet_speed;
                        let velocity_row = angle.sin() * bullet_speed;
                        bullets_to_spawn.push((
                            enemy.column,
                            enemy.row,
                            velocity_column,
                            velocity_row,
                            TermColor::Rgb {
                                r: 80,
                                g: 80,
                                b: 255,
                            },
                        ));
                    }
                }
                BulletPatternKind::Ring => {
                    let ring_count = if enemy.kind == EnemyKind::Boss { 16 } else { 8 };
                    let angle_step = std::f64::consts::TAU / ring_count as f64;
                    for bullet_index in 0..ring_count {
                        let angle = angle_step * bullet_index as f64;
                        let velocity_column = angle.cos() * bullet_speed;
                        let velocity_row = angle.sin() * bullet_speed;
                        bullets_to_spawn.push((
                            enemy.column,
                            enemy.row,
                            velocity_column,
                            velocity_row,
                            TermColor::Rgb {
                                r: 50,
                                g: 255,
                                b: 50,
                            },
                        ));
                    }
                }
            }
        }

        for (column, row, velocity_column, velocity_row, color) in bullets_to_spawn {
            self.spawn_enemy_bullet(world, column, row, velocity_column, velocity_row, color);
        }
    }

    fn update_boss_pattern(&mut self) {
        for enemy in &mut self.enemies {
            if enemy.kind != EnemyKind::Boss {
                continue;
            }
            let health_fraction = enemy.health as f64 / enemy.max_health as f64;
            if health_fraction < 0.25 {
                enemy.pattern = BulletPatternKind::Ring;
                enemy.fire_interval = 0.3;
            } else if health_fraction < 0.5 {
                enemy.pattern = BulletPatternKind::Spread;
                enemy.fire_interval = 0.4;
            } else if health_fraction < 0.75 {
                enemy.pattern = BulletPatternKind::Aimed;
                enemy.fire_interval = 0.2;
            }
        }
    }

    fn update_enemy_movement(&mut self, world: &mut World, delta: f64) {
        let mut rng = rand::rng();

        for enemy in &mut self.enemies {
            if !enemy.entered {
                let target_row = match enemy.kind {
                    EnemyKind::Boss => 3.0,
                    EnemyKind::Normal => rng.random_range(2.0..8.0),
                };
                enemy.row += 8.0 * delta;
                if enemy.row >= target_row {
                    enemy.row = target_row;
                    enemy.entered = true;
                }
            } else {
                enemy.movement_timer += delta;
                match enemy.kind {
                    EnemyKind::Boss => {
                        if enemy.movement_timer > 2.0 {
                            enemy.movement_timer = 0.0;
                            enemy.target_column = rng.random_range(8.0..(PLAY_WIDTH - 8) as f64);
                        }
                        let direction = (enemy.target_column - enemy.column).signum();
                        let speed = 10.0;
                        enemy.column += direction * speed * delta;
                        if (enemy.column - enemy.target_column).abs() < speed * delta {
                            enemy.column = enemy.target_column;
                        }
                    }
                    EnemyKind::Normal => {
                        if enemy.movement_timer > 3.0 {
                            enemy.movement_timer = 0.0;
                            enemy.target_column = rng.random_range(4.0..(PLAY_WIDTH - 4) as f64);
                        }
                        let direction = (enemy.target_column - enemy.column).signum();
                        let speed = 6.0;
                        enemy.column += direction * speed * delta;
                        if (enemy.column - enemy.target_column).abs() < speed * delta {
                            enemy.column = enemy.target_column;
                        }
                    }
                }
            }

            if let Some(position) = world.get_position_mut(enemy.entity) {
                position.column = self.play_offset_column as f64 + enemy.column;
                position.row = self.play_offset_row as f64 + enemy.row;
            }
        }
    }

    fn update_bullets(&mut self, world: &mut World, delta: f64) {
        for bullet in &mut self.player_bullets {
            if let Some(position) = world.get_position_mut(bullet.entity) {
                position.row += bullet.velocity_row * delta;
            }
        }

        for bullet in &mut self.enemy_bullets {
            if let Some(position) = world.get_position_mut(bullet.entity) {
                position.column += bullet.velocity_column * delta;
                position.row += bullet.velocity_row * delta;
            }
        }
    }

    fn cleanup_offscreen(&mut self, world: &mut World) {
        let top = self.play_offset_row as f64 - 2.0;
        let bottom = (self.play_offset_row + PLAY_HEIGHT) as f64 + 2.0;
        let left = self.play_offset_column as f64 - 2.0;
        let right = (self.play_offset_column + PLAY_WIDTH) as f64 + 2.0;

        let mut player_bullets_to_remove = Vec::new();
        for (index, bullet) in self.player_bullets.iter().enumerate() {
            if let Some(position) = world.get_position(bullet.entity)
                && position.row < top
            {
                player_bullets_to_remove.push(index);
            }
        }
        for &index in player_bullets_to_remove.iter().rev() {
            let bullet = self.player_bullets.swap_remove(index);
            world.despawn_entities(&[bullet.entity]);
        }

        let mut enemy_bullets_to_remove = Vec::new();
        for (index, bullet) in self.enemy_bullets.iter().enumerate() {
            if let Some(position) = world.get_position(bullet.entity)
                && (position.row < top
                    || position.row > bottom
                    || position.column < left
                    || position.column > right)
            {
                enemy_bullets_to_remove.push(index);
            }
        }
        for &index in enemy_bullets_to_remove.iter().rev() {
            let bullet = self.enemy_bullets.swap_remove(index);
            world.despawn_entities(&[bullet.entity]);
        }
    }

    fn handle_collisions(&mut self, world: &mut World) {
        let mut player_bullets_hit: Vec<usize> = Vec::new();
        let mut enemy_bullets_hit: Vec<usize> = Vec::new();
        let mut enemies_hit: Vec<(usize, bool)> = Vec::new();
        let mut player_hit = false;

        for (bullet_index, bullet) in self.player_bullets.iter().enumerate() {
            let bullet_position = match world.get_position(bullet.entity) {
                Some(position) => *position,
                None => continue,
            };

            for (enemy_index, enemy) in self.enemies.iter().enumerate() {
                let enemy_position = match world.get_position(enemy.entity) {
                    Some(position) => *position,
                    None => continue,
                };

                let distance_column = (bullet_position.column - enemy_position.column).abs();
                let distance_row = (bullet_position.row - enemy_position.row).abs();

                let hit_radius = if enemy.kind == EnemyKind::Boss {
                    1.5
                } else {
                    0.8
                };

                if distance_column < hit_radius && distance_row < hit_radius {
                    if !player_bullets_hit.contains(&bullet_index) {
                        player_bullets_hit.push(bullet_index);
                    }
                    let already_tracked = enemies_hit
                        .iter()
                        .any(|(tracked_index, _)| *tracked_index == enemy_index);
                    if !already_tracked {
                        enemies_hit.push((enemy_index, false));
                    }
                    break;
                }
            }
        }

        if self.invulnerable_timer <= 0.0 && self.bomb_active_timer <= 0.0 {
            let player_position = match world.get_position(self.player_entity) {
                Some(position) => *position,
                None => Position {
                    column: 0.0,
                    row: 0.0,
                },
            };

            for (bullet_index, bullet) in self.enemy_bullets.iter().enumerate() {
                let bullet_position = match world.get_position(bullet.entity) {
                    Some(position) => *position,
                    None => continue,
                };

                let distance_column = (bullet_position.column - player_position.column).abs();
                let distance_row = (bullet_position.row - player_position.row).abs();

                if distance_column < 0.8 && distance_row < 0.8 {
                    if !enemy_bullets_hit.contains(&bullet_index) {
                        enemy_bullets_hit.push(bullet_index);
                    }
                    player_hit = true;
                }
            }
        }

        for &(enemy_index, _) in &enemies_hit {
            let enemy = &mut self.enemies[enemy_index];
            enemy.health -= 1;
            if enemy.health <= 0 {
                self.combo += 1;
                self.combo_timer = COMBO_DECAY_TIME;
                let combo_multiplier = self.combo.min(10) as u64;
                self.score += enemy.points as u64 * combo_multiplier;

                let column = enemy.column;
                let row = enemy.row;
                let is_boss = enemy.kind == EnemyKind::Boss;

                let particle_config = ParticleConfig {
                    characters: vec!['*', '+', '.', '#'],
                    colors: if is_boss {
                        vec![
                            TermColor::Rgb {
                                r: 255,
                                g: 50,
                                b: 200,
                            },
                            TermColor::Rgb {
                                r: 255,
                                g: 100,
                                b: 255,
                            },
                            TermColor::Rgb {
                                r: 255,
                                g: 200,
                                b: 50,
                            },
                            TermColor::White,
                        ]
                    } else {
                        vec![
                            TermColor::Rgb {
                                r: 255,
                                g: 200,
                                b: 50,
                            },
                            TermColor::Rgb {
                                r: 255,
                                g: 100,
                                b: 50,
                            },
                            TermColor::Red,
                        ]
                    },
                    lifetime: if is_boss { 1.5 } else { 0.6 },
                    speed_min: 3.0,
                    speed_max: if is_boss { 15.0 } else { 8.0 },
                    spread: std::f64::consts::TAU,
                    direction: 0.0,
                    z_index: 8,
                };

                let particle_count = if is_boss { 30 } else { 10 };
                self.particle_emitter.emit(
                    world,
                    self.play_offset_column as f64 + column,
                    self.play_offset_row as f64 + row,
                    particle_count,
                    &particle_config,
                );

                world.despawn_entities(&[enemy.entity]);
            }
        }

        self.enemies.retain(|enemy| enemy.health > 0);

        if self
            .enemies
            .iter()
            .all(|enemy| enemy.kind != EnemyKind::Boss)
        {
            if let Some(ref mut bar) = self.boss_health_bar {
                bar.despawn(world);
            }
            self.boss_health_bar = None;
        }

        player_bullets_hit.sort_unstable();
        player_bullets_hit.dedup();
        for &index in player_bullets_hit.iter().rev() {
            let bullet = self.player_bullets.swap_remove(index);
            world.despawn_entities(&[bullet.entity]);
        }

        enemy_bullets_hit.sort_unstable();
        enemy_bullets_hit.dedup();
        for &index in enemy_bullets_hit.iter().rev() {
            let bullet = self.enemy_bullets.swap_remove(index);
            world.despawn_entities(&[bullet.entity]);
        }

        if player_hit {
            self.lives = self.lives.saturating_sub(1);
            self.combo = 0;
            self.combo_timer = 0.0;

            if self.lives == 0 {
                self.game_over = true;
            } else {
                self.invulnerable_timer = INVULNERABILITY_DURATION;

                let particle_config = ParticleConfig {
                    characters: vec!['*', '+', 'x'],
                    colors: vec![
                        TermColor::Rgb {
                            r: 100,
                            g: 255,
                            b: 100,
                        },
                        TermColor::White,
                        TermColor::Rgb {
                            r: 255,
                            g: 255,
                            b: 100,
                        },
                    ],
                    lifetime: 0.8,
                    speed_min: 5.0,
                    speed_max: 12.0,
                    spread: std::f64::consts::TAU,
                    direction: 0.0,
                    z_index: 8,
                };

                self.particle_emitter.emit(
                    world,
                    self.play_offset_column as f64 + self.player_column,
                    self.play_offset_row as f64 + self.player_row,
                    15,
                    &particle_config,
                );
            }
        }
    }

    fn activate_bomb(&mut self, world: &mut World) {
        if self.bombs == 0 || self.bomb_active_timer > 0.0 {
            return;
        }
        self.bombs -= 1;
        self.bomb_active_timer = BOMB_DURATION;

        let entities_to_despawn: Vec<Entity> = self
            .enemy_bullets
            .iter()
            .map(|bullet| bullet.entity)
            .collect();
        if !entities_to_despawn.is_empty() {
            world.despawn_entities(&entities_to_despawn);
        }

        let bullet_count = self.enemy_bullets.len() as u64;
        self.score += bullet_count * 10;

        self.enemy_bullets.clear();

        let particle_config = ParticleConfig {
            characters: vec!['*', '+', '#', '@'],
            colors: vec![
                TermColor::White,
                TermColor::Rgb {
                    r: 255,
                    g: 255,
                    b: 100,
                },
                TermColor::Rgb {
                    r: 100,
                    g: 200,
                    b: 255,
                },
            ],
            lifetime: 1.0,
            speed_min: 5.0,
            speed_max: 20.0,
            spread: std::f64::consts::TAU,
            direction: 0.0,
            z_index: 9,
        };

        self.particle_emitter.emit(
            world,
            self.play_offset_column as f64 + self.player_column,
            self.play_offset_row as f64 + self.player_row,
            40,
            &particle_config,
        );
    }

    fn update_player_blink(&self, world: &mut World) {
        if self.invulnerable_timer > 0.0 {
            let blink = (self.invulnerable_timer * 10.0) as i32 % 2 == 0;
            if let Some(sprite) = world.get_sprite_mut(self.player_entity) {
                sprite.foreground = if blink {
                    TermColor::Rgb {
                        r: 100,
                        g: 255,
                        b: 100,
                    }
                } else {
                    TermColor::Rgb {
                        r: 30,
                        g: 80,
                        b: 30,
                    }
                };
            }
        } else if self.bomb_active_timer > 0.0 {
            if let Some(sprite) = world.get_sprite_mut(self.player_entity) {
                sprite.foreground = TermColor::White;
            }
        } else if let Some(sprite) = world.get_sprite_mut(self.player_entity) {
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

        let combo_display = if self.combo > 1 {
            format!(" x{}", self.combo)
        } else {
            String::new()
        };

        let hud_top = format!(
            " Score: {:08}{}  Wave: {}  Lives: {}  Bombs: {} ",
            self.score, combo_display, self.wave, self.lives, self.bombs
        );

        let hud_row = self.play_offset_row as f64;
        let hud_start = self.play_offset_column as f64;

        for char_index in 0..PLAY_WIDTH as usize {
            let character = if char_index < hud_top.len() {
                hud_top.chars().nth(char_index).unwrap_or(' ')
            } else {
                ' '
            };
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: hud_start + char_index as f64,
                    row: hud_row,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: TermColor::White,
                    background: TermColor::Rgb {
                        r: 15,
                        g: 10,
                        b: 30,
                    },
                },
            );
            world.set_z_index(entity, ZIndex(10));
            self.hud_entities.push(entity);
        }

        for enemy in &self.enemies {
            if enemy.kind == EnemyKind::Boss {
                if let Some(ref mut bar) = self.boss_health_bar {
                    let fraction = enemy.health as f64 / enemy.max_health as f64;
                    bar.render(world, fraction);
                }
                break;
            }
        }
    }

    fn update_star_scroll(&mut self, world: &mut World, delta: f64) {
        for &entity in &self.star_entities {
            if let Some(position) = world.get_position_mut(entity) {
                position.row += 2.0 * delta;
                let local_row = position.row - self.play_offset_row as f64;
                if local_row >= PLAY_HEIGHT as f64 {
                    position.row -= PLAY_HEIGHT as f64;
                    let mut rng = rand::rng();
                    position.column =
                        self.play_offset_column as f64 + rng.random_range(0..PLAY_WIDTH) as f64;
                }
            }
        }
    }

    fn draw_border(&mut self, world: &mut World) {
        let left = self.play_offset_column as f64 - 1.0;
        let right = (self.play_offset_column + PLAY_WIDTH) as f64;

        for row in 0..PLAY_HEIGHT {
            let world_row = (self.play_offset_row + row) as f64;

            let entity_left = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity_left,
                Position {
                    column: left,
                    row: world_row,
                },
            );
            world.set_sprite(
                entity_left,
                Sprite {
                    character: '|',
                    foreground: TermColor::DarkGrey,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity_left, ZIndex(1));
            self.hud_entities.push(entity_left);

            let entity_right = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity_right,
                Position {
                    column: right,
                    row: world_row,
                },
            );
            world.set_sprite(
                entity_right,
                Sprite {
                    character: '|',
                    foreground: TermColor::DarkGrey,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity_right, ZIndex(1));
            self.hud_entities.push(entity_right);
        }
    }

    fn clear_all_entities(&mut self, world: &mut World) {
        world.despawn_entities(&[self.player_entity]);

        let player_bullet_entities: Vec<Entity> = self
            .player_bullets
            .iter()
            .map(|bullet| bullet.entity)
            .collect();
        if !player_bullet_entities.is_empty() {
            world.despawn_entities(&player_bullet_entities);
        }
        self.player_bullets.clear();

        let enemy_bullet_entities: Vec<Entity> = self
            .enemy_bullets
            .iter()
            .map(|bullet| bullet.entity)
            .collect();
        if !enemy_bullet_entities.is_empty() {
            world.despawn_entities(&enemy_bullet_entities);
        }
        self.enemy_bullets.clear();

        let enemy_entities: Vec<Entity> = self.enemies.iter().map(|enemy| enemy.entity).collect();
        if !enemy_entities.is_empty() {
            world.despawn_entities(&enemy_entities);
        }
        self.enemies.clear();

        if !self.star_entities.is_empty() {
            world.despawn_entities(&self.star_entities);
        }
        self.star_entities.clear();

        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();

        if let Some(ref mut bar) = self.boss_health_bar {
            bar.despawn(world);
        }
        self.boss_health_bar = None;

        self.particle_emitter.despawn_all(world);
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Bullet Hell - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 60;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        self.play_offset_column = (terminal.columns as i32 - PLAY_WIDTH) / 2;
        self.play_offset_row = (terminal.rows as i32 - PLAY_HEIGHT) / 2;
        if self.play_offset_column < 1 {
            self.play_offset_column = 1;
        }
        if self.play_offset_row < 0 {
            self.play_offset_row = 0;
        }

        self.spawn_starfield(world);
        self.spawn_player(world);
        self.start_next_wave();
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::Up => self.move_up = pressed,
            KeyCode::Down => self.move_down = pressed,
            KeyCode::Left => self.move_left = pressed,
            KeyCode::Right => self.move_right = pressed,
            KeyCode::Char('z') | KeyCode::Char('Z') => self.shooting = pressed,
            KeyCode::Char('x') | KeyCode::Char('X') if pressed && !self.game_over => {
                self.activate_bomb(world);
            }
            KeyCode::Char('c') | KeyCode::Char('C') => self.focused = pressed,
            _ => {}
        }
        if let KeyCode::Escape | KeyCode::Char('q') = key
            && pressed
        {
            world.resources.should_exit = true;
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.game_over {
            return;
        }

        let delta = world.resources.timing.delta_seconds;

        if self.invulnerable_timer > 0.0 {
            self.invulnerable_timer -= delta;
        }
        if self.bomb_active_timer > 0.0 {
            self.bomb_active_timer -= delta;
        }

        if self.combo_timer > 0.0 {
            self.combo_timer -= delta;
            if self.combo_timer <= 0.0 {
                self.combo = 0;
            }
        }

        let speed = if self.focused {
            PLAYER_FOCUSED_SPEED
        } else {
            PLAYER_SPEED
        };

        if self.move_up {
            self.player_row -= speed * delta;
        }
        if self.move_down {
            self.player_row += speed * delta;
        }
        if self.move_left {
            self.player_column -= speed * delta;
        }
        if self.move_right {
            self.player_column += speed * delta;
        }

        self.player_column = self.player_column.clamp(1.0, (PLAY_WIDTH - 2) as f64);
        self.player_row = self.player_row.clamp(2.0, (PLAY_HEIGHT - 2) as f64);

        if let Some(position) = world.get_position_mut(self.player_entity) {
            position.column = self.play_offset_column as f64 + self.player_column;
            position.row = self.play_offset_row as f64 + self.player_row;
        }

        if self.shooting {
            self.fire_cooldown -= delta;
            if self.fire_cooldown <= 0.0 {
                self.fire_player_bullet(world);
                self.fire_cooldown = PLAYER_FIRE_RATE;
            }
        } else {
            self.fire_cooldown = 0.0;
        }

        if !self.wave_complete {
            if self.enemies_spawned_this_wave < self.enemies_per_wave {
                self.spawn_timer += delta;
                let spawn_interval = if self.wave.is_multiple_of(BOSS_WAVE_INTERVAL) {
                    0.5
                } else {
                    WAVE_SPAWN_INTERVAL / self.enemies_per_wave as f64
                };
                if self.spawn_timer >= spawn_interval {
                    self.spawn_timer -= spawn_interval;
                    self.spawn_enemy(world);
                    self.enemies_spawned_this_wave += 1;
                }
            }

            if self.enemies_spawned_this_wave >= self.enemies_per_wave && self.enemies.is_empty() {
                self.wave_complete = true;
                self.wave_timer = 0.0;
            }
        } else {
            self.wave_timer += delta;
            if self.wave_timer >= 2.0 {
                self.start_next_wave();
            }
        }

        self.update_enemy_movement(world, delta);
        self.fire_enemy_patterns(world);
        self.update_boss_pattern();
        self.update_bullets(world, delta);
        self.cleanup_offscreen(world);
        self.handle_collisions(world);
        self.update_player_blink(world);
        self.update_star_scroll(world, delta);
        self.particle_emitter.update(world, delta);
        self.update_hud(world);
        self.draw_border(world);
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
    score: u64,
    wave: u32,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Bullet Hell - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 60;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let lines: Vec<(String, TermColor)> = vec![
            ("GAME OVER".to_string(), TermColor::Red),
            (String::new(), TermColor::Black),
            (
                format!("Final Score: {:08}", self.score),
                TermColor::Rgb {
                    r: 255,
                    g: 255,
                    b: 100,
                },
            ),
            (
                format!("Wave Reached: {}", self.wave),
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
            let start_column = center_column - text.len() as i32 / 2;
            for (char_index, character) in text.chars().enumerate() {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (start_column + char_index as i32) as f64,
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
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.restart = true;
            }
            KeyCode::Escape | KeyCode::Char('q') => {
                world.resources.should_exit = true;
            }
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
