use nightshade::tui::prelude::*;

const PLAY_WIDTH: i32 = 50;
const PLAY_HEIGHT: i32 = 30;
const PADDLE_WIDTH: i32 = 7;
const PADDLE_ROW: i32 = PLAY_HEIGHT - 3;
const BRICK_ROWS: i32 = 8;
const BRICK_COLUMNS: i32 = 15;
const BRICK_START_ROW: i32 = 4;
const BRICK_START_COLUMN: i32 = 3;
const BRICK_SPACING_X: i32 = 3;

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Breakout - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r" ____                 _               _   ",
            r"| __ ) _ __ ___  __ _| | _____  _   _| |_ ",
            r"|  _ \| '__/ _ \/ _` | |/ / _ \| | | | __|",
            r"| |_) | | |  __/ (_| |   < (_) | |_| | |_ ",
            r"|____/|_|  \___|\__,_|_|\_\___/ \__,_|\__|",
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
                            foreground: TermColor::Cyan,
                            background: TermColor::Black,
                        },
                    );
                    world.set_z_index(entity, ZIndex(10));
                }
            }
        }

        let brick_display = "[=] [=] [=] [=] [=]";
        let brick_start = center_column - brick_display.len() as i32 / 2;
        let brick_colors = [
            TermColor::Rgb {
                r: 255,
                g: 50,
                b: 50,
            },
            TermColor::Rgb {
                r: 255,
                g: 165,
                b: 0,
            },
            TermColor::Rgb {
                r: 255,
                g: 255,
                b: 50,
            },
            TermColor::Rgb {
                r: 50,
                g: 255,
                b: 50,
            },
            TermColor::Rgb {
                r: 50,
                g: 150,
                b: 255,
            },
        ];
        for (char_index, character) in brick_display.chars().enumerate() {
            if character != ' ' {
                let color_index = char_index / 4;
                let color = brick_colors[color_index.min(brick_colors.len() - 1)];
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (brick_start + char_index as i32) as f64,
                        row: (title_start_row + 7) as f64,
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

fn brick_color_for_row(row: i32) -> TermColor {
    match row {
        0 => TermColor::Rgb {
            r: 255,
            g: 50,
            b: 50,
        },
        1 => TermColor::Rgb {
            r: 255,
            g: 100,
            b: 30,
        },
        2 => TermColor::Rgb {
            r: 255,
            g: 165,
            b: 0,
        },
        3 => TermColor::Rgb {
            r: 255,
            g: 220,
            b: 50,
        },
        4 => TermColor::Rgb {
            r: 100,
            g: 255,
            b: 50,
        },
        5 => TermColor::Rgb {
            r: 50,
            g: 200,
            b: 255,
        },
        6 => TermColor::Rgb {
            r: 100,
            g: 100,
            b: 255,
        },
        _ => TermColor::Rgb {
            r: 180,
            g: 80,
            b: 255,
        },
    }
}

fn brick_points_for_row(row: i32) -> u32 {
    match row {
        0 => 80,
        1 => 70,
        2 => 60,
        3 => 50,
        4 => 40,
        5 => 30,
        6 => 20,
        _ => 10,
    }
}

struct BrickData {
    entity: Entity,
    alive: bool,
    points: u32,
}

struct GameplayState {
    play_offset_x: i32,
    play_offset_y: i32,
    paddle_entities: Vec<Entity>,
    paddle_center: i32,
    ball_entity: Entity,
    ball_velocity_column: i32,
    ball_velocity_row: i32,
    ball_attached: bool,
    bricks: Vec<BrickData>,
    bricks_alive_count: usize,
    wall_entities: Vec<Entity>,
    hud_entities: Vec<Entity>,
    score: u32,
    lives: u32,
    level: u32,
    game_over: bool,
    move_left: bool,
    move_right: bool,
    move_timer: f64,
    move_interval: f64,
    ball_timer: f64,
    ball_interval: f64,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            play_offset_x: 0,
            play_offset_y: 0,
            paddle_entities: Vec::new(),
            paddle_center: PLAY_WIDTH / 2,
            ball_entity: Entity::default(),
            ball_velocity_column: 1,
            ball_velocity_row: -1,
            ball_attached: true,
            bricks: Vec::new(),
            bricks_alive_count: 0,
            wall_entities: Vec::new(),
            hud_entities: Vec::new(),
            score: 0,
            lives: 3,
            level: 1,
            game_over: false,
            move_left: false,
            move_right: false,
            move_timer: 0.0,
            move_interval: 0.03,
            ball_timer: 0.0,
            ball_interval: 0.08,
        }
    }

    fn spawn_walls(&mut self, world: &mut World) {
        for row in 0..PLAY_HEIGHT {
            let left_entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
            world.set_position(
                left_entity,
                Position {
                    column: (self.play_offset_x - 1) as f64,
                    row: (self.play_offset_y + row) as f64,
                },
            );
            world.set_sprite(
                left_entity,
                Sprite {
                    character: '|',
                    foreground: TermColor::Rgb {
                        r: 80,
                        g: 80,
                        b: 120,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(left_entity, ZIndex(1));
            world.set_collider(
                left_entity,
                Collider {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
            );
            self.wall_entities.push(left_entity);

            let right_entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
            world.set_position(
                right_entity,
                Position {
                    column: (self.play_offset_x + PLAY_WIDTH) as f64,
                    row: (self.play_offset_y + row) as f64,
                },
            );
            world.set_sprite(
                right_entity,
                Sprite {
                    character: '|',
                    foreground: TermColor::Rgb {
                        r: 80,
                        g: 80,
                        b: 120,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(right_entity, ZIndex(1));
            world.set_collider(
                right_entity,
                Collider {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
            );
            self.wall_entities.push(right_entity);
        }

        for column in -1..=PLAY_WIDTH {
            let top_entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
            world.set_position(
                top_entity,
                Position {
                    column: (self.play_offset_x + column) as f64,
                    row: (self.play_offset_y - 1) as f64,
                },
            );
            world.set_sprite(
                top_entity,
                Sprite {
                    character: '-',
                    foreground: TermColor::Rgb {
                        r: 80,
                        g: 80,
                        b: 120,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(top_entity, ZIndex(1));
            world.set_collider(
                top_entity,
                Collider {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
            );
            self.wall_entities.push(top_entity);
        }
    }

    fn spawn_paddle(&mut self, world: &mut World) {
        self.paddle_center = PLAY_WIDTH / 2;
        let half = PADDLE_WIDTH / 2;

        for offset in -half..=half {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (self.play_offset_x + self.paddle_center + offset) as f64,
                    row: (self.play_offset_y + PADDLE_ROW) as f64,
                },
            );
            let character = if offset == -half || offset == half {
                '['
            } else {
                '='
            };
            let character = if offset == half { ']' } else { character };
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
            world.set_z_index(entity, ZIndex(3));
            world.set_collider(
                entity,
                Collider {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
            );
            self.paddle_entities.push(entity);
        }
    }

    fn spawn_ball(&mut self, world: &mut World) {
        self.ball_attached = true;
        self.ball_velocity_column = 1;
        self.ball_velocity_row = -1;

        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
        world.set_position(
            entity,
            Position {
                column: (self.play_offset_x + self.paddle_center) as f64,
                row: (self.play_offset_y + PADDLE_ROW - 1) as f64,
            },
        );
        world.set_sprite(
            entity,
            Sprite {
                character: 'O',
                foreground: TermColor::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
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
        self.ball_entity = entity;
    }

    fn spawn_bricks(&mut self, world: &mut World) {
        self.bricks.clear();
        self.bricks_alive_count = 0;

        let actual_columns =
            ((PLAY_WIDTH - BRICK_START_COLUMN * 2) / BRICK_SPACING_X).min(BRICK_COLUMNS);

        for grid_row in 0..BRICK_ROWS {
            let color = brick_color_for_row(grid_row);
            let points = brick_points_for_row(grid_row);
            for grid_column in 0..actual_columns {
                let column =
                    self.play_offset_x + BRICK_START_COLUMN + grid_column * BRICK_SPACING_X;
                let row = self.play_offset_y + BRICK_START_ROW + grid_row;

                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
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
                        character: '=',
                        foreground: color,
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

                self.bricks.push(BrickData {
                    entity,
                    alive: true,
                    points,
                });
                self.bricks_alive_count += 1;
            }
        }
    }

    fn update_paddle_positions(&mut self, world: &mut World) {
        let half = PADDLE_WIDTH / 2;
        for (index, &entity) in self.paddle_entities.iter().enumerate() {
            let offset = index as i32 - half;
            if let Some(position) = world.get_position_mut(entity) {
                position.column = (self.play_offset_x + self.paddle_center + offset) as f64;
            }
        }
    }

    fn update_attached_ball(&mut self, world: &mut World) {
        if self.ball_attached
            && let Some(position) = world.get_position_mut(self.ball_entity)
        {
            position.column = (self.play_offset_x + self.paddle_center) as f64;
            position.row = (self.play_offset_y + PADDLE_ROW - 1) as f64;
        }
    }

    fn step_ball(&mut self, world: &mut World) {
        if self.ball_attached {
            return;
        }

        let ball_position = match world.get_position(self.ball_entity) {
            Some(position) => *position,
            None => return,
        };

        let new_column = ball_position.column as i32 + self.ball_velocity_column;
        let new_row = ball_position.row as i32 + self.ball_velocity_row;

        if new_column <= self.play_offset_x || new_column >= self.play_offset_x + PLAY_WIDTH - 1 {
            self.ball_velocity_column = -self.ball_velocity_column;
        }

        if new_row <= self.play_offset_y {
            self.ball_velocity_row = -self.ball_velocity_row;
        }

        if new_row >= self.play_offset_y + PLAY_HEIGHT {
            self.lose_ball(world);
            return;
        }

        let final_column = ball_position.column as i32 + self.ball_velocity_column;
        let final_row = ball_position.row as i32 + self.ball_velocity_row;

        if let Some(position) = world.get_position_mut(self.ball_entity) {
            position.column = final_column as f64;
            position.row = final_row as f64;
        }

        self.check_ball_collisions(world);
    }

    fn check_ball_collisions(&mut self, world: &mut World) {
        let contacts = collision_pairs(world);
        let mut bricks_to_destroy: Vec<usize> = Vec::new();
        let mut hit_paddle = false;
        let mut paddle_hit_offset: i32 = 0;

        for contact in &contacts {
            let a_is_ball = contact.entity_a == self.ball_entity;
            let b_is_ball = contact.entity_b == self.ball_entity;

            if !a_is_ball && !b_is_ball {
                continue;
            }

            let other = if a_is_ball {
                contact.entity_b
            } else {
                contact.entity_a
            };

            if let Some(brick_index) = self
                .bricks
                .iter()
                .position(|brick| brick.alive && brick.entity == other)
            {
                if !bricks_to_destroy.contains(&brick_index) {
                    bricks_to_destroy.push(brick_index);
                }
                continue;
            }

            if self.paddle_entities.contains(&other) {
                hit_paddle = true;
                if let Some(other_pos) = world.get_position(other) {
                    let paddle_world_center = self.play_offset_x + self.paddle_center;
                    paddle_hit_offset = other_pos.column as i32 - paddle_world_center;
                }
                continue;
            }

            if self.wall_entities.contains(&other) {
                let ball_pos = world.get_position(self.ball_entity).copied();
                let wall_pos = world.get_position(other).copied();
                if let (Some(ball_pos), Some(wall_pos)) = (ball_pos, wall_pos) {
                    if wall_pos.row as i32 == self.play_offset_y - 1 {
                        self.ball_velocity_row = self.ball_velocity_row.abs();
                    } else if wall_pos.column as i32 <= self.play_offset_x {
                        self.ball_velocity_column = self.ball_velocity_column.abs();
                    } else if wall_pos.column as i32 >= self.play_offset_x + PLAY_WIDTH {
                        self.ball_velocity_column = -(self.ball_velocity_column.abs());
                    }

                    if let Some(position) = world.get_position_mut(self.ball_entity) {
                        position.column =
                            (ball_pos.column as i32 + self.ball_velocity_column) as f64;
                        position.row = (ball_pos.row as i32 + self.ball_velocity_row) as f64;
                    }
                }
                continue;
            }
        }

        if !bricks_to_destroy.is_empty() {
            self.ball_velocity_row = -self.ball_velocity_row;

            for &brick_index in &bricks_to_destroy {
                self.score += self.bricks[brick_index].points;
                self.bricks[brick_index].alive = false;
                self.bricks_alive_count -= 1;
                world.despawn_entities(&[self.bricks[brick_index].entity]);
            }

            if self.bricks_alive_count == 0 {
                self.next_level(world);
                return;
            }
        }

        if hit_paddle {
            self.ball_velocity_row = -(self.ball_velocity_row.abs());

            let half = PADDLE_WIDTH / 2;
            if paddle_hit_offset <= -half + 1 {
                self.ball_velocity_column = -2;
            } else if paddle_hit_offset >= half - 1 {
                self.ball_velocity_column = 2;
            } else if paddle_hit_offset < 0 {
                self.ball_velocity_column = -1;
            } else if paddle_hit_offset > 0 {
                self.ball_velocity_column = 1;
            }

            if let Some(position) = world.get_position_mut(self.ball_entity) {
                position.row = (self.play_offset_y + PADDLE_ROW - 1) as f64;
            }
        }
    }

    fn lose_ball(&mut self, world: &mut World) {
        self.lives = self.lives.saturating_sub(1);
        if self.lives == 0 {
            self.game_over = true;
        } else {
            self.ball_attached = true;
            self.ball_velocity_column = 1;
            self.ball_velocity_row = -1;
            self.update_attached_ball(world);
        }
    }

    fn next_level(&mut self, world: &mut World) {
        self.level += 1;
        self.ball_interval = (self.ball_interval - 0.005).max(0.03);

        world.despawn_entities(&[self.ball_entity]);

        for brick in &self.bricks {
            if brick.alive {
                world.despawn_entities(&[brick.entity]);
            }
        }
        self.bricks.clear();

        self.spawn_bricks(world);
        self.spawn_ball(world);
    }

    fn update_hud(&mut self, world: &mut World) {
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();

        let hud_text = format!(
            "Score: {:06}   Lives: {}   Level: {}",
            self.score, self.lives, self.level
        );

        let hud_row = self.play_offset_y + PLAY_HEIGHT;

        for (char_index, character) in hud_text.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (self.play_offset_x + char_index as i32) as f64,
                    row: hud_row as f64,
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
                    row: hud_row as f64,
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
        for &entity in &self.paddle_entities {
            world.despawn_entities(&[entity]);
        }
        self.paddle_entities.clear();

        world.despawn_entities(&[self.ball_entity]);

        for brick in &self.bricks {
            if brick.alive {
                world.despawn_entities(&[brick.entity]);
            }
        }
        self.bricks.clear();

        for &entity in &self.wall_entities {
            world.despawn_entities(&[entity]);
        }
        self.wall_entities.clear();

        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Breakout - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 60;
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

        self.spawn_walls(world);
        self.spawn_paddle(world);
        self.spawn_ball(world);
        self.spawn_bricks(world);
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
            KeyCode::Char(' ') if pressed && self.ball_attached && !self.game_over => {
                self.ball_attached = false;
            }
            KeyCode::Escape | KeyCode::Char('q') if pressed => {
                world.resources.should_exit = true;
            }
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.game_over {
            return;
        }

        let delta = world.resources.timing.delta_seconds;

        self.move_timer += delta;
        if self.move_timer >= self.move_interval {
            self.move_timer = 0.0;

            let half = PADDLE_WIDTH / 2;
            if self.move_left && self.paddle_center - half > 0 {
                self.paddle_center -= 1;
            }
            if self.move_right && self.paddle_center + half < PLAY_WIDTH - 1 {
                self.paddle_center += 1;
            }
            self.update_paddle_positions(world);
            self.update_attached_ball(world);
        }

        self.ball_timer += delta;
        if self.ball_timer >= self.ball_interval {
            self.ball_timer = 0.0;
            self.step_ball(world);
        }

        self.update_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            self.clear_all_entities(world);
            return Some(Box::new(GameOverState {
                score: self.score,
                level: self.level,
                restart: false,
            }));
        }
        None
    }
}

struct GameOverState {
    score: u32,
    level: u32,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Breakout - Game Over"
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
                format!("Level Reached: {}", self.level),
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
