use nightshade::tui::prelude::*;
use rand::Rng;

const PLAY_WIDTH: i32 = 60;
const PLAY_HEIGHT: i32 = 30;
const ALIEN_COLUMNS: i32 = 11;
const ALIEN_ROWS: i32 = 5;
const ALIEN_SPACING_X: i32 = 4;
const ALIEN_SPACING_Y: i32 = 2;
const PLAYER_ROW: i32 = PLAY_HEIGHT - 2;
const BARRIER_ROW: i32 = PLAY_HEIGHT - 6;
const BARRIER_COUNT: i32 = 4;
const BARRIER_WIDTH: i32 = 5;
const BARRIER_HEIGHT: i32 = 3;
const MAX_PLAYER_BULLETS: usize = 3;
const STAR_COUNT: usize = 40;
const INITIAL_ALIEN_TICK: f64 = 0.5;
const MIN_ALIEN_TICK: f64 = 0.05;
const ENEMY_FIRE_CHANCE: f64 = 0.01;

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Space Invaders - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r" ____                        ___                     _               ",
            r"/ ___| _ __   __ _  ___ ___  |_ _|_ ____   ____ _  __| | ___ _ __ ___ ",
            r"\___ \| '_ \ / _` |/ __/ _ \  | || '_ \ \ / / _` |/ _` |/ _ \ '__/ __|",
            r" ___) | |_) | (_| | (_|  __/  | || | | \ V / (_| | (_| |  __/ |  \__ \",
            r"|____/| .__/ \__,_|\___\___| |___|_| |_|\_/ \__,_|\__,_|\___|_|  |___/",
            r"      |_|                                                              ",
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
                            foreground: TermColor::Green,
                            background: TermColor::Black,
                        },
                    );
                    world.set_z_index(entity, ZIndex(10));
                }
            }
        }

        let alien_display = "  /\\    <*>    {O}    <#>    /V\\";
        let alien_start = center_column - alien_display.len() as i32 / 2;
        for (char_index, character) in alien_display.chars().enumerate() {
            if character != ' ' {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (alien_start + char_index as i32) as f64,
                        row: (title_start_row + 8) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: TermColor::Rgb {
                            r: 255,
                            g: 100,
                            b: 100,
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
                    row: (title_start_row + 11) as f64,
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
                    row: (title_start_row + 13) as f64,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum AlienRow {
    Top,
    UpperMiddle,
    Middle,
    LowerMiddle,
    Bottom,
}

impl AlienRow {
    fn character(self) -> char {
        match self {
            Self::Top => 'W',
            Self::UpperMiddle => 'M',
            Self::Middle => 'X',
            Self::LowerMiddle => 'H',
            Self::Bottom => 'A',
        }
    }

    fn color(self) -> TermColor {
        match self {
            Self::Top => TermColor::Rgb {
                r: 255,
                g: 50,
                b: 50,
            },
            Self::UpperMiddle => TermColor::Rgb {
                r: 255,
                g: 150,
                b: 50,
            },
            Self::Middle => TermColor::Rgb {
                r: 255,
                g: 255,
                b: 50,
            },
            Self::LowerMiddle => TermColor::Rgb {
                r: 50,
                g: 255,
                b: 100,
            },
            Self::Bottom => TermColor::Rgb {
                r: 50,
                g: 150,
                b: 255,
            },
        }
    }

    fn points(self) -> u32 {
        match self {
            Self::Top => 50,
            Self::UpperMiddle => 40,
            Self::Middle => 30,
            Self::LowerMiddle => 20,
            Self::Bottom => 10,
        }
    }

    fn from_row_index(index: i32) -> Self {
        match index {
            0 => Self::Top,
            1 => Self::UpperMiddle,
            2 => Self::Middle,
            3 => Self::LowerMiddle,
            _ => Self::Bottom,
        }
    }
}

struct AlienData {
    entity: Entity,
    grid_column: i32,
    grid_row: i32,
    row_kind: AlienRow,
    alive: bool,
}

struct GameplayState {
    play_offset_x: i32,
    play_offset_y: i32,
    player_entity: Entity,
    player_column: i32,
    aliens: Vec<AlienData>,
    alien_base_x: i32,
    alien_base_y: i32,
    alien_direction: i32,
    alien_tick_timer: f64,
    alien_tick_interval: f64,
    alien_total: usize,
    alien_alive_count: usize,
    player_bullet_entities: Vec<Entity>,
    enemy_bullet_entities: Vec<Entity>,
    barrier_entities: Vec<Entity>,
    star_entities: Vec<Entity>,
    hud_entities: Vec<Entity>,
    explosion_entities: Vec<(Entity, f64)>,
    score: u32,
    lives: u32,
    wave: u32,
    game_over: bool,
    move_left: bool,
    move_right: bool,
    fire_cooldown: f64,
    enemy_fire_timer: f64,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            play_offset_x: 0,
            play_offset_y: 0,
            player_entity: Entity::default(),
            player_column: PLAY_WIDTH / 2,
            aliens: Vec::new(),
            alien_base_x: 2,
            alien_base_y: 3,
            alien_direction: 1,
            alien_tick_timer: 0.0,
            alien_tick_interval: INITIAL_ALIEN_TICK,
            alien_total: 0,
            alien_alive_count: 0,
            player_bullet_entities: Vec::new(),
            enemy_bullet_entities: Vec::new(),
            barrier_entities: Vec::new(),
            star_entities: Vec::new(),
            hud_entities: Vec::new(),
            explosion_entities: Vec::new(),
            score: 0,
            lives: 3,
            wave: 1,
            game_over: false,
            move_left: false,
            move_right: false,
            fire_cooldown: 0.0,
            enemy_fire_timer: 0.0,
        }
    }

    fn spawn_starfield(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        for _ in 0..STAR_COUNT {
            let column = rng.random_range(self.play_offset_x..self.play_offset_x + PLAY_WIDTH);
            let row = rng.random_range(self.play_offset_y..self.play_offset_y + PLAY_HEIGHT);
            let brightness = rng.random_range(30u8..100u8);
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: column as f64,
                    row: row as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: '.',
                    foreground: TermColor::Rgb {
                        r: brightness,
                        g: brightness,
                        b: brightness,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(0));
            self.star_entities.push(entity);
        }
    }

    fn spawn_player(&mut self, world: &mut World) {
        self.player_column = PLAY_WIDTH / 2;
        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
        world.set_position(
            entity,
            Position {
                column: (self.play_offset_x + self.player_column) as f64,
                row: (self.play_offset_y + PLAYER_ROW) as f64,
            },
        );
        world.set_sprite(
            entity,
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
        world.set_z_index(entity, ZIndex(3));
        world.set_collider(
            entity,
            Collider {
                width: 1,
                height: 1,
                ..Default::default()
            },
        );
        self.player_entity = entity;
    }

    fn spawn_aliens(&mut self, world: &mut World) {
        self.aliens.clear();
        self.alien_base_x = 2;
        self.alien_base_y = 3;
        self.alien_direction = 1;
        self.alien_tick_interval =
            (INITIAL_ALIEN_TICK - (self.wave as f64 - 1.0) * 0.05).max(MIN_ALIEN_TICK + 0.1);
        self.alien_total = (ALIEN_COLUMNS * ALIEN_ROWS) as usize;
        self.alien_alive_count = self.alien_total;

        for grid_row in 0..ALIEN_ROWS {
            let row_kind = AlienRow::from_row_index(grid_row);
            for grid_column in 0..ALIEN_COLUMNS {
                let world_column =
                    self.play_offset_x + self.alien_base_x + grid_column * ALIEN_SPACING_X;
                let world_row = self.play_offset_y + self.alien_base_y + grid_row * ALIEN_SPACING_Y;

                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: world_column as f64,
                        row: world_row as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character: row_kind.character(),
                        foreground: row_kind.color(),
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

                self.aliens.push(AlienData {
                    entity,
                    grid_column,
                    grid_row,
                    row_kind,
                    alive: true,
                });
            }
        }
    }

    fn spawn_barriers(&mut self, world: &mut World) {
        let total_barrier_span = BARRIER_COUNT * BARRIER_WIDTH + (BARRIER_COUNT - 1) * 3;
        let start_x = (PLAY_WIDTH - total_barrier_span) / 2;

        for barrier_index in 0..BARRIER_COUNT {
            let base_x = start_x + barrier_index * (BARRIER_WIDTH + 3);
            for row_offset in 0..BARRIER_HEIGHT {
                for col_offset in 0..BARRIER_WIDTH {
                    if row_offset == BARRIER_HEIGHT - 1
                        && (col_offset == 0 || col_offset == BARRIER_WIDTH - 1)
                    {
                        continue;
                    }
                    let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
                    world.set_position(
                        entity,
                        Position {
                            column: (self.play_offset_x + base_x + col_offset) as f64,
                            row: (self.play_offset_y + BARRIER_ROW + row_offset) as f64,
                        },
                    );
                    world.set_sprite(
                        entity,
                        Sprite {
                            character: '#',
                            foreground: TermColor::Rgb {
                                r: 80,
                                g: 200,
                                b: 80,
                            },
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
                    self.barrier_entities.push(entity);
                }
            }
        }
    }

    fn fire_player_bullet(&mut self, world: &mut World) {
        if self.player_bullet_entities.len() >= MAX_PLAYER_BULLETS {
            return;
        }

        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | VELOCITY | COLLIDER, 1)[0];
        world.set_position(
            entity,
            Position {
                column: (self.play_offset_x + self.player_column) as f64,
                row: (self.play_offset_y + PLAYER_ROW - 1) as f64,
            },
        );
        world.set_velocity(
            entity,
            Velocity {
                column: 0.0,
                row: -1.0,
            },
        );
        world.set_sprite(
            entity,
            Sprite {
                character: '|',
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
        self.player_bullet_entities.push(entity);
    }

    fn fire_enemy_bullet(&mut self, world: &mut World) {
        let bottom_aliens = self.get_bottom_row_aliens();
        if bottom_aliens.is_empty() {
            return;
        }

        let mut rng = rand::rng();
        let chosen_index = rng.random_range(0..bottom_aliens.len());
        let alien_index = bottom_aliens[chosen_index];
        let alien = &self.aliens[alien_index];

        let position = world.get_position(alien.entity);
        let Some(position) = position else { return };
        let column = position.column;
        let row = position.row;

        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | VELOCITY | COLLIDER, 1)[0];
        world.set_position(
            entity,
            Position {
                column,
                row: row + 1.0,
            },
        );
        world.set_velocity(
            entity,
            Velocity {
                column: 0.0,
                row: 1.0,
            },
        );
        world.set_sprite(
            entity,
            Sprite {
                character: ':',
                foreground: TermColor::Rgb {
                    r: 255,
                    g: 80,
                    b: 80,
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
        self.enemy_bullet_entities.push(entity);
    }

    fn get_bottom_row_aliens(&self) -> Vec<usize> {
        let mut bottom_per_column: std::collections::HashMap<i32, (usize, i32)> =
            std::collections::HashMap::new();
        for (index, alien) in self.aliens.iter().enumerate() {
            if !alien.alive {
                continue;
            }
            let entry = bottom_per_column
                .entry(alien.grid_column)
                .or_insert((index, alien.grid_row));
            if alien.grid_row > entry.1 {
                *entry = (index, alien.grid_row);
            }
        }
        bottom_per_column
            .values()
            .map(|(index, _)| *index)
            .collect()
    }

    fn spawn_explosion(&mut self, world: &mut World, column: i32, row: i32) {
        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
        world.set_position(
            entity,
            Position {
                column: column as f64,
                row: row as f64,
            },
        );
        world.set_sprite(
            entity,
            Sprite {
                character: '*',
                foreground: TermColor::Rgb {
                    r: 255,
                    g: 200,
                    b: 50,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));
        self.explosion_entities.push((entity, 0.15));
    }

    fn update_explosions(&mut self, world: &mut World, delta: f64) {
        let mut despawn_list = Vec::new();
        for (entity, timer) in &mut self.explosion_entities {
            *timer -= delta;
            if *timer <= 0.0 {
                despawn_list.push(*entity);
            }
        }
        for entity in &despawn_list {
            world.despawn_entities(&[*entity]);
        }
        self.explosion_entities
            .retain(|(entity, _)| !despawn_list.contains(entity));
    }

    fn update_alien_positions(&mut self, world: &mut World) {
        for alien in &self.aliens {
            if !alien.alive {
                continue;
            }
            let world_column =
                self.play_offset_x + self.alien_base_x + alien.grid_column * ALIEN_SPACING_X;
            let world_row =
                self.play_offset_y + self.alien_base_y + alien.grid_row * ALIEN_SPACING_Y;
            if let Some(position) = world.get_position_mut(alien.entity) {
                position.column = world_column as f64;
                position.row = world_row as f64;
            }
        }
    }

    fn tick_aliens(&mut self, world: &mut World) {
        let mut leftmost = PLAY_WIDTH;
        let mut rightmost = 0;
        for alien in &self.aliens {
            if !alien.alive {
                continue;
            }
            let x = self.alien_base_x + alien.grid_column * ALIEN_SPACING_X;
            if x < leftmost {
                leftmost = x;
            }
            if x > rightmost {
                rightmost = x;
            }
        }

        let next_x = self.alien_base_x + self.alien_direction;
        let next_leftmost = leftmost + self.alien_direction;
        let next_rightmost = rightmost + self.alien_direction;

        if next_leftmost < 1 || next_rightmost >= PLAY_WIDTH - 1 {
            self.alien_direction = -self.alien_direction;
            self.alien_base_y += 1;

            let max_alien_y = self
                .aliens
                .iter()
                .filter(|alien| alien.alive)
                .map(|alien| self.alien_base_y + alien.grid_row * ALIEN_SPACING_Y)
                .max()
                .unwrap_or(0);

            if max_alien_y >= BARRIER_ROW {
                self.game_over = true;
            }
        } else {
            self.alien_base_x = next_x;
        }

        self.update_alien_positions(world);
    }

    fn handle_collisions(&mut self, world: &mut World) {
        let contacts = collision_pairs(world);
        let mut command_queue = CommandQueue::default();
        let mut player_bullets_to_remove: Vec<Entity> = Vec::new();
        let mut enemy_bullets_to_remove: Vec<Entity> = Vec::new();
        let mut barriers_to_remove: Vec<Entity> = Vec::new();
        let mut aliens_killed: Vec<(usize, i32, i32)> = Vec::new();
        let mut player_hit = false;

        for contact in &contacts {
            let entity_a = &contact.entity_a;
            let entity_b = &contact.entity_b;
            let a_is_player_bullet = self.player_bullet_entities.contains(entity_a);
            let b_is_player_bullet = self.player_bullet_entities.contains(entity_b);
            let a_is_enemy_bullet = self.enemy_bullet_entities.contains(entity_a);
            let b_is_enemy_bullet = self.enemy_bullet_entities.contains(entity_b);
            let a_is_barrier = self.barrier_entities.contains(entity_a);
            let b_is_barrier = self.barrier_entities.contains(entity_b);
            let a_is_player = *entity_a == self.player_entity;
            let b_is_player = *entity_b == self.player_entity;

            let a_alien_index = self
                .aliens
                .iter()
                .position(|alien| alien.alive && alien.entity == *entity_a);
            let b_alien_index = self
                .aliens
                .iter()
                .position(|alien| alien.alive && alien.entity == *entity_b);

            if let (true, Some(alien_idx)) = (a_is_player_bullet, b_alien_index) {
                let position = world.get_position(self.aliens[alien_idx].entity).copied();
                if let Some(position) = position {
                    aliens_killed.push((alien_idx, position.column as i32, position.row as i32));
                }
                player_bullets_to_remove.push(*entity_a);
                command_queue.queue_despawn(*entity_a);
                command_queue.queue_despawn(*entity_b);
            } else if let (true, Some(alien_idx)) = (b_is_player_bullet, a_alien_index) {
                let position = world.get_position(self.aliens[alien_idx].entity).copied();
                if let Some(position) = position {
                    aliens_killed.push((alien_idx, position.column as i32, position.row as i32));
                }
                player_bullets_to_remove.push(*entity_b);
                command_queue.queue_despawn(*entity_a);
                command_queue.queue_despawn(*entity_b);
            }

            if a_is_player_bullet && b_is_barrier {
                player_bullets_to_remove.push(*entity_a);
                barriers_to_remove.push(*entity_b);
                command_queue.queue_despawn(*entity_a);
                command_queue.queue_despawn(*entity_b);
            } else if b_is_player_bullet && a_is_barrier {
                player_bullets_to_remove.push(*entity_b);
                barriers_to_remove.push(*entity_a);
                command_queue.queue_despawn(*entity_a);
                command_queue.queue_despawn(*entity_b);
            }

            if a_is_enemy_bullet && b_is_barrier {
                enemy_bullets_to_remove.push(*entity_a);
                barriers_to_remove.push(*entity_b);
                command_queue.queue_despawn(*entity_a);
                command_queue.queue_despawn(*entity_b);
            } else if b_is_enemy_bullet && a_is_barrier {
                enemy_bullets_to_remove.push(*entity_b);
                barriers_to_remove.push(*entity_a);
                command_queue.queue_despawn(*entity_a);
                command_queue.queue_despawn(*entity_b);
            }

            if (a_is_enemy_bullet && b_is_player) || (b_is_enemy_bullet && a_is_player) {
                player_hit = true;
                let bullet = if a_is_enemy_bullet {
                    *entity_a
                } else {
                    *entity_b
                };
                enemy_bullets_to_remove.push(bullet);
                command_queue.queue_despawn(bullet);
            }

            if a_is_player_bullet && b_is_enemy_bullet {
                player_bullets_to_remove.push(*entity_a);
                enemy_bullets_to_remove.push(*entity_b);
                command_queue.queue_despawn(*entity_a);
                command_queue.queue_despawn(*entity_b);
            } else if b_is_player_bullet && a_is_enemy_bullet {
                player_bullets_to_remove.push(*entity_b);
                enemy_bullets_to_remove.push(*entity_a);
                command_queue.queue_despawn(*entity_a);
                command_queue.queue_despawn(*entity_b);
            }
        }

        for (alien_idx, column, row) in &aliens_killed {
            let points = self.aliens[*alien_idx].row_kind.points();
            self.score += points;
            self.aliens[*alien_idx].alive = false;
            self.alien_alive_count -= 1;
            self.spawn_explosion(world, *column, *row);
        }

        self.player_bullet_entities
            .retain(|entity| !player_bullets_to_remove.contains(entity));
        self.enemy_bullet_entities
            .retain(|entity| !enemy_bullets_to_remove.contains(entity));
        self.barrier_entities
            .retain(|entity| !barriers_to_remove.contains(entity));

        if player_hit {
            self.lives = self.lives.saturating_sub(1);
            if self.lives == 0 {
                self.game_over = true;
            } else {
                let position = world.get_position(self.player_entity).copied();
                if let Some(position) = position {
                    self.spawn_explosion(world, position.column as i32, position.row as i32);
                }
            }
        }

        for command in command_queue.drain() {
            if let WorldCommand::DespawnEntity { entity } = command {
                world.despawn_entities(&[entity]);
            }
        }

        if self.alien_alive_count == 0 && !self.game_over {
            self.next_wave(world);
        }

        self.recalculate_alien_tick_interval();
    }

    fn recalculate_alien_tick_interval(&mut self) {
        if self.alien_total == 0 {
            return;
        }
        let ratio = self.alien_alive_count as f64 / self.alien_total as f64;
        let base = (INITIAL_ALIEN_TICK - (self.wave as f64 - 1.0) * 0.05).max(MIN_ALIEN_TICK + 0.1);
        self.alien_tick_interval = (base * ratio).max(MIN_ALIEN_TICK);
    }

    fn cleanup_offscreen_bullets(&mut self, world: &mut World) {
        let top = self.play_offset_y;
        let bottom = self.play_offset_y + PLAY_HEIGHT;

        let mut to_remove = Vec::new();
        for &entity in &self.player_bullet_entities {
            if let Some(position) = world.get_position(entity)
                && (position.row as i32) < top
            {
                to_remove.push(entity);
            }
        }
        for entity in &to_remove {
            world.despawn_entities(&[*entity]);
        }
        self.player_bullet_entities
            .retain(|entity| !to_remove.contains(entity));

        let mut to_remove = Vec::new();
        for &entity in &self.enemy_bullet_entities {
            if let Some(position) = world.get_position(entity)
                && (position.row as i32) >= bottom
            {
                to_remove.push(entity);
            }
        }
        for entity in &to_remove {
            world.despawn_entities(&[*entity]);
        }
        self.enemy_bullet_entities
            .retain(|entity| !to_remove.contains(entity));
    }

    fn next_wave(&mut self, world: &mut World) {
        self.wave += 1;

        for &entity in &self.player_bullet_entities {
            world.despawn_entities(&[entity]);
        }
        self.player_bullet_entities.clear();

        for &entity in &self.enemy_bullet_entities {
            world.despawn_entities(&[entity]);
        }
        self.enemy_bullet_entities.clear();

        for &entity in &self.barrier_entities {
            world.despawn_entities(&[entity]);
        }
        self.barrier_entities.clear();

        for (entity, _) in &self.explosion_entities {
            world.despawn_entities(&[*entity]);
        }
        self.explosion_entities.clear();

        for alien in &self.aliens {
            if alien.alive {
                world.despawn_entities(&[alien.entity]);
            }
        }
        self.aliens.clear();

        self.spawn_aliens(world);
        self.spawn_barriers(world);
    }

    fn update_hud(&mut self, world: &mut World) {
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();

        let hud_text = format!(
            "Score: {:06}   Lives: {}   Wave: {}",
            self.score, self.lives, self.wave
        );

        for (char_index, character) in hud_text.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (self.play_offset_x + char_index as i32) as f64,
                    row: self.play_offset_y as f64,
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
                    column: (self.play_offset_x + fill_index as i32) as f64,
                    row: self.play_offset_y as f64,
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
        world.despawn_entities(&[self.player_entity]);
        for alien in &self.aliens {
            if alien.alive {
                world.despawn_entities(&[alien.entity]);
            }
        }
        self.aliens.clear();
        for &entity in &self.player_bullet_entities {
            world.despawn_entities(&[entity]);
        }
        self.player_bullet_entities.clear();
        for &entity in &self.enemy_bullet_entities {
            world.despawn_entities(&[entity]);
        }
        self.enemy_bullet_entities.clear();
        for &entity in &self.barrier_entities {
            world.despawn_entities(&[entity]);
        }
        self.barrier_entities.clear();
        for &entity in &self.star_entities {
            world.despawn_entities(&[entity]);
        }
        self.star_entities.clear();
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();
        for (entity, _) in &self.explosion_entities {
            world.despawn_entities(&[*entity]);
        }
        self.explosion_entities.clear();
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Space Invaders - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        self.play_offset_x = (terminal.columns as i32 - PLAY_WIDTH) / 2;
        self.play_offset_y = (terminal.rows as i32 - PLAY_HEIGHT) / 2;
        if self.play_offset_x < 0 {
            self.play_offset_x = 0;
        }
        if self.play_offset_y < 0 {
            self.play_offset_y = 0;
        }

        self.spawn_starfield(world);
        self.spawn_player(world);
        self.spawn_aliens(world);
        self.spawn_barriers(world);
        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::Left | KeyCode::Char('a') => {
                self.move_left = pressed;
            }
            KeyCode::Right | KeyCode::Char('d') => {
                self.move_right = pressed;
            }
            KeyCode::Char(' ') => {
                if pressed && self.fire_cooldown <= 0.0 && !self.game_over {
                    self.fire_player_bullet(world);
                    self.fire_cooldown = 0.2;
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

        if self.move_left && self.player_column > 1 {
            self.player_column -= 1;
        }
        if self.move_right && self.player_column < PLAY_WIDTH - 2 {
            self.player_column += 1;
        }
        if let Some(position) = world.get_position_mut(self.player_entity) {
            position.column = (self.play_offset_x + self.player_column) as f64;
        }

        movement_system(world);

        self.alien_tick_timer += delta;
        if self.alien_tick_timer >= self.alien_tick_interval {
            self.alien_tick_timer = 0.0;
            self.tick_aliens(world);
        }

        self.enemy_fire_timer += delta;
        if self.enemy_fire_timer >= 0.5 {
            self.enemy_fire_timer = 0.0;
            let mut rng = rand::rng();
            let fire_chance = ENEMY_FIRE_CHANCE * (1.0 + self.wave as f64 * 0.5);
            if rng.random::<f64>() < fire_chance * 15.0 {
                self.fire_enemy_bullet(world);
            }
        }

        self.cleanup_offscreen_bullets(world);
        self.handle_collisions(world);
        self.update_explosions(world, delta);
        self.update_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            self.clear_all_entities(world);
            return Some(Box::new(GameOverState {
                score: self.score,
                wave: self.wave,
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
        "Space Invaders - Game Over"
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
                format!("Final Score: {:06}", self.score),
                TermColor::Rgb {
                    r: 255,
                    g: 255,
                    b: 100,
                },
            ),
            (
                format!("Waves Survived: {}", self.wave),
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
            KeyCode::Char('r') => {
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
