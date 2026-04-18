use nightshade::tui::prelude::*;
use rand::Rng;

const PLAY_WIDTH: i32 = 60;
const PLAY_HEIGHT: i32 = 22;
const PADDLE_HEIGHT: i32 = 5;
const PADDLE_LEFT_COLUMN: i32 = 2;
const PADDLE_RIGHT_COLUMN: i32 = PLAY_WIDTH - 3;
const BALL_TICK_INTERVAL: f64 = 0.06;
const AI_TICK_INTERVAL: f64 = 0.05;
const WIN_SCORE: u32 = 10;

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Pong - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 60;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r" ____                   ",
            r"|  _ \ ___  _ __   __ _ ",
            r"| |_) / _ \| '_ \ / _` |",
            r"|  __/ (_) | | | | (_| |",
            r"|_|   \___/|_| |_|\__, |",
            r"                  |___/ ",
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
                            foreground: TermColor::White,
                            background: TermColor::Black,
                        },
                    );
                    world.set_z_index(entity, ZIndex(10));
                }
            }
        }

        let subtitle = "Player vs AI";
        let subtitle_start = center_column - subtitle.len() as i32 / 2;
        for (char_index, character) in subtitle.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (subtitle_start + char_index as i32) as f64,
                    row: (title_start_row + 8) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: TermColor::Rgb {
                        r: 100,
                        g: 200,
                        b: 255,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
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
    left_paddle_entities: Vec<Entity>,
    right_paddle_entities: Vec<Entity>,
    left_paddle_center: i32,
    right_paddle_center: i32,
    ball_entity: Entity,
    ball_velocity_column: i32,
    ball_velocity_row: i32,
    wall_entities: Vec<Entity>,
    center_line_entities: Vec<Entity>,
    hud_entities: Vec<Entity>,
    player_score: u32,
    ai_score: u32,
    game_over: bool,
    winner_is_player: bool,
    move_up: bool,
    move_down: bool,
    ball_timer: f64,
    ai_timer: f64,
    serve_timer: f64,
    serving: bool,
    ball_speed_factor: f64,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            play_offset_x: 0,
            play_offset_y: 0,
            left_paddle_entities: Vec::new(),
            right_paddle_entities: Vec::new(),
            left_paddle_center: PLAY_HEIGHT / 2,
            right_paddle_center: PLAY_HEIGHT / 2,
            ball_entity: Entity::default(),
            ball_velocity_column: 1,
            ball_velocity_row: 1,
            wall_entities: Vec::new(),
            center_line_entities: Vec::new(),
            hud_entities: Vec::new(),
            player_score: 0,
            ai_score: 0,
            game_over: false,
            winner_is_player: false,
            move_up: false,
            move_down: false,
            ball_timer: 0.0,
            ai_timer: 0.0,
            serve_timer: 1.0,
            serving: true,
            ball_speed_factor: 1.0,
        }
    }

    fn spawn_walls(&mut self, world: &mut World) {
        for column in 0..PLAY_WIDTH {
            let top = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                top,
                Position {
                    column: (self.play_offset_x + column) as f64,
                    row: self.play_offset_y as f64,
                },
            );
            world.set_sprite(
                top,
                Sprite {
                    character: '=',
                    foreground: TermColor::Rgb {
                        r: 80,
                        g: 80,
                        b: 100,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(top, ZIndex(1));
            self.wall_entities.push(top);

            let bottom = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                bottom,
                Position {
                    column: (self.play_offset_x + column) as f64,
                    row: (self.play_offset_y + PLAY_HEIGHT - 1) as f64,
                },
            );
            world.set_sprite(
                bottom,
                Sprite {
                    character: '=',
                    foreground: TermColor::Rgb {
                        r: 80,
                        g: 80,
                        b: 100,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(bottom, ZIndex(1));
            self.wall_entities.push(bottom);
        }

        for row in 1..(PLAY_HEIGHT - 1) {
            if row % 2 == 0 {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (self.play_offset_x + PLAY_WIDTH / 2) as f64,
                        row: (self.play_offset_y + row) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character: ':',
                        foreground: TermColor::Rgb {
                            r: 50,
                            g: 50,
                            b: 60,
                        },
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(0));
                self.center_line_entities.push(entity);
            }
        }
    }

    fn spawn_paddle(
        world: &mut World,
        play_offset_x: i32,
        play_offset_y: i32,
        column: i32,
        center: i32,
        color: TermColor,
    ) -> Vec<Entity> {
        let half = PADDLE_HEIGHT / 2;
        let mut entities = Vec::new();
        for offset in -half..=half {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (play_offset_x + column) as f64,
                    row: (play_offset_y + center + offset) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: '|',
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
            entities.push(entity);
        }
        entities
    }

    fn update_paddle_positions(
        entities: &[Entity],
        world: &mut World,
        play_offset_x: i32,
        play_offset_y: i32,
        column: i32,
        center: i32,
    ) {
        let half = PADDLE_HEIGHT / 2;
        for (index, &entity) in entities.iter().enumerate() {
            let offset = index as i32 - half;
            if let Some(position) = world.get_position_mut(entity) {
                position.column = (play_offset_x + column) as f64;
                position.row = (play_offset_y + center + offset) as f64;
            }
        }
    }

    fn spawn_ball(&mut self, world: &mut World) {
        self.ball_entity =
            world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER | VELOCITY, 1)[0];
        world.set_position(
            self.ball_entity,
            Position {
                column: (self.play_offset_x + PLAY_WIDTH / 2) as f64,
                row: (self.play_offset_y + PLAY_HEIGHT / 2) as f64,
            },
        );
        world.set_sprite(
            self.ball_entity,
            Sprite {
                character: 'O',
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(self.ball_entity, ZIndex(3));
        world.set_collider(
            self.ball_entity,
            Collider {
                width: 1,
                height: 1,
                ..Default::default()
            },
        );
        world.set_velocity(
            self.ball_entity,
            Velocity {
                column: 0.0,
                row: 0.0,
            },
        );
    }

    fn serve_ball(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        self.ball_velocity_column = if rng.random_bool(0.5) { 1 } else { -1 };
        self.ball_velocity_row = if rng.random_bool(0.5) { 1 } else { -1 };

        if let Some(position) = world.get_position_mut(self.ball_entity) {
            position.column = (self.play_offset_x + PLAY_WIDTH / 2) as f64;
            position.row = (self.play_offset_y + PLAY_HEIGHT / 2) as f64;
        }
        world.set_velocity(
            self.ball_entity,
            Velocity {
                column: self.ball_velocity_column as f64,
                row: self.ball_velocity_row as f64,
            },
        );
        self.serving = false;
    }

    fn step_ball(&mut self, world: &mut World) {
        if self.serving {
            return;
        }

        movement_system(world);

        let ball_position = match world.get_position(self.ball_entity) {
            Some(position) => *position,
            None => return,
        };

        let ball_row_local = ball_position.row as i32 - self.play_offset_y;
        let ball_col_local = ball_position.column as i32 - self.play_offset_x;

        if ball_row_local <= 1 || ball_row_local >= PLAY_HEIGHT - 2 {
            self.ball_velocity_row = -self.ball_velocity_row;
            world.set_velocity(
                self.ball_entity,
                Velocity {
                    column: self.ball_velocity_column as f64,
                    row: self.ball_velocity_row as f64,
                },
            );
            if let Some(position) = world.get_position_mut(self.ball_entity) {
                position.row =
                    (self.play_offset_y + ball_row_local.clamp(1, PLAY_HEIGHT - 2)) as f64;
            }
        }

        let contacts = collision_pairs(world);
        for contact in &contacts {
            let ball_involved =
                contact.entity_a == self.ball_entity || contact.entity_b == self.ball_entity;
            if !ball_involved {
                continue;
            }
            let other = if contact.entity_a == self.ball_entity {
                contact.entity_b
            } else {
                contact.entity_a
            };

            if self.left_paddle_entities.contains(&other) {
                self.ball_velocity_column = self.ball_velocity_column.abs();
                let paddle_index = self
                    .left_paddle_entities
                    .iter()
                    .position(|&entity| entity == other)
                    .unwrap_or(PADDLE_HEIGHT as usize / 2);
                self.adjust_ball_angle(paddle_index);
                world.set_velocity(
                    self.ball_entity,
                    Velocity {
                        column: self.ball_velocity_column as f64,
                        row: self.ball_velocity_row as f64,
                    },
                );
                if let Some(position) = world.get_position_mut(self.ball_entity) {
                    position.column = (self.play_offset_x + PADDLE_LEFT_COLUMN + 1) as f64;
                }
                return;
            }

            if self.right_paddle_entities.contains(&other) {
                self.ball_velocity_column = -(self.ball_velocity_column.abs());
                let paddle_index = self
                    .right_paddle_entities
                    .iter()
                    .position(|&entity| entity == other)
                    .unwrap_or(PADDLE_HEIGHT as usize / 2);
                self.adjust_ball_angle(paddle_index);
                world.set_velocity(
                    self.ball_entity,
                    Velocity {
                        column: self.ball_velocity_column as f64,
                        row: self.ball_velocity_row as f64,
                    },
                );
                if let Some(position) = world.get_position_mut(self.ball_entity) {
                    position.column = (self.play_offset_x + PADDLE_RIGHT_COLUMN - 1) as f64;
                }
                return;
            }
        }

        if ball_col_local <= 0 {
            self.ai_score += 1;
            self.start_serve();
            self.reset_ball_position(world);
        } else if ball_col_local >= PLAY_WIDTH - 1 {
            self.player_score += 1;
            self.start_serve();
            self.reset_ball_position(world);
        }

        if self.player_score >= WIN_SCORE {
            self.game_over = true;
            self.winner_is_player = true;
        } else if self.ai_score >= WIN_SCORE {
            self.game_over = true;
            self.winner_is_player = false;
        }
    }

    fn adjust_ball_angle(&mut self, paddle_hit_index: usize) {
        let center = PADDLE_HEIGHT as usize / 2;
        if paddle_hit_index == 0 {
            self.ball_velocity_row = -1;
        } else if paddle_hit_index == PADDLE_HEIGHT as usize - 1 {
            self.ball_velocity_row = 1;
        } else if paddle_hit_index < center {
            self.ball_velocity_row = -1;
        } else if paddle_hit_index > center {
            self.ball_velocity_row = 1;
        }
    }

    fn start_serve(&mut self) {
        self.serving = true;
        self.serve_timer = 1.0;
        self.ball_speed_factor = (self.ball_speed_factor - 0.002).max(0.7);
    }

    fn reset_ball_position(&mut self, world: &mut World) {
        if let Some(position) = world.get_position_mut(self.ball_entity) {
            position.column = (self.play_offset_x + PLAY_WIDTH / 2) as f64;
            position.row = (self.play_offset_y + PLAY_HEIGHT / 2) as f64;
        }
        world.set_velocity(
            self.ball_entity,
            Velocity {
                column: 0.0,
                row: 0.0,
            },
        );
    }

    fn update_ai(&mut self, world: &World) {
        let ball_position = match world.get_position(self.ball_entity) {
            Some(position) => *position,
            None => return,
        };

        let ball_row_local = ball_position.row as i32 - self.play_offset_y;
        let half = PADDLE_HEIGHT / 2;
        let min_center = 1 + half;
        let max_center = PLAY_HEIGHT - 2 - half;

        if ball_row_local < self.right_paddle_center - 1 && self.right_paddle_center > min_center {
            self.right_paddle_center -= 1;
        } else if ball_row_local > self.right_paddle_center + 1
            && self.right_paddle_center < max_center
        {
            self.right_paddle_center += 1;
        }
    }

    fn update_hud(&mut self, world: &mut World) {
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();

        let score_text = format!("Player: {}    AI: {}", self.player_score, self.ai_score);

        let hud_row = self.play_offset_y - 1;
        let start_col = self.play_offset_x + PLAY_WIDTH / 2 - score_text.len() as i32 / 2;

        for (char_index, character) in score_text.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (start_col + char_index as i32) as f64,
                    row: hud_row as f64,
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
            self.hud_entities.push(entity);
        }
    }

    fn clear_all_entities(&mut self, world: &mut World) {
        for &entity in &self.left_paddle_entities {
            world.despawn_entities(&[entity]);
        }
        self.left_paddle_entities.clear();
        for &entity in &self.right_paddle_entities {
            world.despawn_entities(&[entity]);
        }
        self.right_paddle_entities.clear();
        world.despawn_entities(&[self.ball_entity]);
        for &entity in &self.wall_entities {
            world.despawn_entities(&[entity]);
        }
        self.wall_entities.clear();
        for &entity in &self.center_line_entities {
            world.despawn_entities(&[entity]);
        }
        self.center_line_entities.clear();
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Pong - Ember"
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
        if self.play_offset_y < 1 {
            self.play_offset_y = 1;
        }

        self.spawn_walls(world);
        self.left_paddle_entities = Self::spawn_paddle(
            world,
            self.play_offset_x,
            self.play_offset_y,
            PADDLE_LEFT_COLUMN,
            self.left_paddle_center,
            TermColor::Rgb {
                r: 100,
                g: 200,
                b: 255,
            },
        );
        self.right_paddle_entities = Self::spawn_paddle(
            world,
            self.play_offset_x,
            self.play_offset_y,
            PADDLE_RIGHT_COLUMN,
            self.right_paddle_center,
            TermColor::Rgb {
                r: 255,
                g: 100,
                b: 100,
            },
        );
        self.spawn_ball(world);
        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::Up | KeyCode::Char('w') => self.move_up = pressed,
            KeyCode::Down | KeyCode::Char('s') => self.move_down = pressed,
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

        if self.serving {
            self.serve_timer -= delta;
            if self.serve_timer <= 0.0 {
                self.serve_ball(world);
            }
        }

        let half = PADDLE_HEIGHT / 2;
        let min_center = 1 + half;
        let max_center = PLAY_HEIGHT - 2 - half;

        if self.move_up && self.left_paddle_center > min_center {
            self.left_paddle_center -= 1;
        }
        if self.move_down && self.left_paddle_center < max_center {
            self.left_paddle_center += 1;
        }
        Self::update_paddle_positions(
            &self.left_paddle_entities,
            world,
            self.play_offset_x,
            self.play_offset_y,
            PADDLE_LEFT_COLUMN,
            self.left_paddle_center,
        );

        self.ai_timer += delta;
        if self.ai_timer >= AI_TICK_INTERVAL {
            self.ai_timer -= AI_TICK_INTERVAL;
            self.update_ai(world);
            Self::update_paddle_positions(
                &self.right_paddle_entities,
                world,
                self.play_offset_x,
                self.play_offset_y,
                PADDLE_RIGHT_COLUMN,
                self.right_paddle_center,
            );
        }

        self.ball_timer += delta;
        let effective_tick = BALL_TICK_INTERVAL * self.ball_speed_factor;
        if self.ball_timer >= effective_tick {
            self.ball_timer -= effective_tick;
            self.step_ball(world);
        }

        self.update_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            let player_score = self.player_score;
            let ai_score = self.ai_score;
            let winner_is_player = self.winner_is_player;
            self.clear_all_entities(world);
            return Some(Box::new(GameOverState {
                player_score,
                ai_score,
                winner_is_player,
                restart: false,
            }));
        }
        None
    }
}

struct GameOverState {
    player_score: u32,
    ai_score: u32,
    winner_is_player: bool,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Pong - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let winner_text = if self.winner_is_player {
            "YOU WIN!"
        } else {
            "AI WINS!"
        };
        let winner_color = if self.winner_is_player {
            TermColor::Rgb {
                r: 100,
                g: 255,
                b: 100,
            }
        } else {
            TermColor::Rgb {
                r: 255,
                g: 100,
                b: 100,
            }
        };

        let lines: Vec<(String, TermColor)> = vec![
            (winner_text.to_string(), winner_color),
            (String::new(), TermColor::Black),
            (
                format!("Player: {}  -  AI: {}", self.player_score, self.ai_score),
                TermColor::White,
            ),
            (String::new(), TermColor::Black),
            ("Press R to rematch".to_string(), TermColor::White),
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
                        row: (center_row - 3 + line_index as i32) as f64,
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
