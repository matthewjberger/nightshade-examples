use nightshade::tui::prelude::*;
use rand::Rng;

const PLAY_WIDTH: i32 = 40;
const GOAL_ROW: i32 = 0;
const WATER_ROW_START: i32 = 1;
const WATER_ROW_END: i32 = 4;
const MEDIAN_ROW: i32 = 5;
const ROAD_ROW_START: i32 = 6;
const ROAD_ROW_END: i32 = 9;
const START_ROW: i32 = 10;
const PLAY_HEIGHT: i32 = 12;
const TICK_INTERVAL: f64 = 0.15;
const HOME_PAD_COUNT: i32 = 5;

struct LaneConfig {
    row: i32,
    velocity: i32,
    entity_count: i32,
    is_water: bool,
    character: char,
    color: TermColor,
}

fn lane_configs() -> Vec<LaneConfig> {
    vec![
        LaneConfig {
            row: ROAD_ROW_END,
            velocity: 1,
            entity_count: 6,
            is_water: false,
            character: '>',
            color: TermColor::Rgb {
                r: 255,
                g: 80,
                b: 80,
            },
        },
        LaneConfig {
            row: ROAD_ROW_END - 1,
            velocity: -2,
            entity_count: 4,
            is_water: false,
            character: '<',
            color: TermColor::Rgb {
                r: 255,
                g: 200,
                b: 50,
            },
        },
        LaneConfig {
            row: ROAD_ROW_START + 1,
            velocity: 1,
            entity_count: 5,
            is_water: false,
            character: '>',
            color: TermColor::Rgb {
                r: 255,
                g: 100,
                b: 100,
            },
        },
        LaneConfig {
            row: ROAD_ROW_START,
            velocity: -1,
            entity_count: 5,
            is_water: false,
            character: '<',
            color: TermColor::Rgb {
                r: 200,
                g: 200,
                b: 50,
            },
        },
        LaneConfig {
            row: WATER_ROW_END,
            velocity: 1,
            entity_count: 8,
            is_water: true,
            character: '=',
            color: TermColor::Rgb {
                r: 139,
                g: 90,
                b: 43,
            },
        },
        LaneConfig {
            row: WATER_ROW_END - 1,
            velocity: -1,
            entity_count: 7,
            is_water: true,
            character: '=',
            color: TermColor::Rgb {
                r: 160,
                g: 100,
                b: 50,
            },
        },
        LaneConfig {
            row: WATER_ROW_START + 1,
            velocity: 2,
            entity_count: 6,
            is_water: true,
            character: '=',
            color: TermColor::Rgb {
                r: 139,
                g: 90,
                b: 43,
            },
        },
        LaneConfig {
            row: WATER_ROW_START,
            velocity: -1,
            entity_count: 8,
            is_water: true,
            character: '=',
            color: TermColor::Rgb {
                r: 160,
                g: 100,
                b: 50,
            },
        },
    ]
}

fn is_water_row(row: i32) -> bool {
    (WATER_ROW_START..=WATER_ROW_END).contains(&row)
}

fn lane_velocity_for_row(configs: &[LaneConfig], row: i32) -> i32 {
    configs
        .iter()
        .find(|config| config.row == row)
        .map(|config| config.velocity)
        .unwrap_or(0)
}

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Frogger"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r" _____                              ",
            r"|  ___| __ ___   __ _  __ _  ___ _ __",
            r"| |_ | '__/ _ \ / _` |/ _` |/ _ \ '__|",
            r"|  _|| | | (_) | (_| | (_| |  __/ |  ",
            r"|_|  |_|  \___/ \__, |\__, |\___|_|  ",
            r"                |___/ |___/           ",
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

        let frog_art = " @('.')@ ";
        let frog_start = center_column - frog_art.len() as i32 / 2;
        for (char_index, character) in frog_art.chars().enumerate() {
            if character != ' ' {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (frog_start + char_index as i32) as f64,
                        row: (title_start_row + 8) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: TermColor::Rgb {
                            r: 50,
                            g: 255,
                            b: 50,
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

struct GameplayState {
    play_offset_x: i32,
    play_offset_y: i32,
    player_entity: Entity,
    player_column: i32,
    player_row: i32,
    car_entities: Vec<Entity>,
    log_entities: Vec<Entity>,
    background_entities: Vec<Entity>,
    home_pad_entities: Vec<Entity>,
    home_filled: Vec<bool>,
    hud_entities: Vec<Entity>,
    lane_configs: Vec<LaneConfig>,
    score: u32,
    lives: u32,
    level: u32,
    game_over: bool,
    tick_timer: f64,
    move_cooldown: f64,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            play_offset_x: 0,
            play_offset_y: 0,
            player_entity: Entity::default(),
            player_column: PLAY_WIDTH / 2,
            player_row: START_ROW,
            car_entities: Vec::new(),
            log_entities: Vec::new(),
            background_entities: Vec::new(),
            home_pad_entities: Vec::new(),
            home_filled: vec![false; HOME_PAD_COUNT as usize],
            hud_entities: Vec::new(),
            lane_configs: lane_configs(),
            score: 0,
            lives: 5,
            level: 1,
            game_over: false,
            tick_timer: 0.0,
            move_cooldown: 0.0,
        }
    }

    fn spawn_background(&mut self, world: &mut World) {
        for row in 0..PLAY_HEIGHT {
            let (character, foreground, background) = if row == GOAL_ROW {
                (
                    ' ',
                    TermColor::Black,
                    TermColor::Rgb {
                        r: 20,
                        g: 60,
                        b: 20,
                    },
                )
            } else if is_water_row(row) {
                (
                    '~',
                    TermColor::Rgb {
                        r: 30,
                        g: 80,
                        b: 180,
                    },
                    TermColor::Rgb {
                        r: 10,
                        g: 30,
                        b: 80,
                    },
                )
            } else if row == MEDIAN_ROW || row == START_ROW {
                (
                    ',',
                    TermColor::Rgb {
                        r: 40,
                        g: 100,
                        b: 40,
                    },
                    TermColor::Rgb {
                        r: 15,
                        g: 40,
                        b: 15,
                    },
                )
            } else {
                (
                    '.',
                    TermColor::Rgb {
                        r: 50,
                        g: 50,
                        b: 50,
                    },
                    TermColor::Rgb {
                        r: 30,
                        g: 30,
                        b: 30,
                    },
                )
            };

            for column in 0..PLAY_WIDTH {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (self.play_offset_x + column) as f64,
                        row: (self.play_offset_y + row) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground,
                        background,
                    },
                );
                world.set_z_index(entity, ZIndex(0));
                self.background_entities.push(entity);
            }
        }
    }

    fn spawn_home_pads(&mut self, world: &mut World) {
        let spacing = PLAY_WIDTH / (HOME_PAD_COUNT + 1);
        for pad_index in 0..HOME_PAD_COUNT {
            let column = spacing * (pad_index + 1);
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (self.play_offset_x + column) as f64,
                    row: (self.play_offset_y + GOAL_ROW) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: 'V',
                    foreground: TermColor::Rgb {
                        r: 50,
                        g: 200,
                        b: 50,
                    },
                    background: TermColor::Rgb {
                        r: 20,
                        g: 60,
                        b: 20,
                    },
                },
            );
            world.set_z_index(entity, ZIndex(1));
            self.home_pad_entities.push(entity);
        }
    }

    fn spawn_player(&mut self, world: &mut World) {
        self.player_column = PLAY_WIDTH / 2;
        self.player_row = START_ROW;
        self.player_entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
        world.set_position(
            self.player_entity,
            Position {
                column: (self.play_offset_x + self.player_column) as f64,
                row: (self.play_offset_y + self.player_row) as f64,
            },
        );
        world.set_sprite(
            self.player_entity,
            Sprite {
                character: '@',
                foreground: TermColor::Rgb {
                    r: 50,
                    g: 255,
                    b: 50,
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

    fn spawn_lane_entities(&mut self, world: &mut World) {
        let mut rng = rand::rng();

        for config in &self.lane_configs {
            for _ in 0..config.entity_count {
                let column = rng.random_range(0..PLAY_WIDTH);

                if config.is_water {
                    let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | VELOCITY, 1)[0];
                    world.set_position(
                        entity,
                        Position {
                            column: (self.play_offset_x + column) as f64,
                            row: (self.play_offset_y + config.row) as f64,
                        },
                    );
                    world.set_velocity(
                        entity,
                        Velocity {
                            column: config.velocity as f64,
                            row: 0.0,
                        },
                    );
                    world.set_sprite(
                        entity,
                        Sprite {
                            character: config.character,
                            foreground: config.color,
                            background: TermColor::Rgb {
                                r: 10,
                                g: 30,
                                b: 80,
                            },
                        },
                    );
                    world.set_z_index(entity, ZIndex(2));
                    self.log_entities.push(entity);
                } else {
                    let entity = world
                        .spawn_entities(POSITION | SPRITE | Z_INDEX | VELOCITY | COLLIDER, 1)[0];
                    world.set_position(
                        entity,
                        Position {
                            column: (self.play_offset_x + column) as f64,
                            row: (self.play_offset_y + config.row) as f64,
                        },
                    );
                    world.set_velocity(
                        entity,
                        Velocity {
                            column: config.velocity as f64,
                            row: 0.0,
                        },
                    );
                    world.set_sprite(
                        entity,
                        Sprite {
                            character: config.character,
                            foreground: config.color,
                            background: TermColor::Rgb {
                                r: 30,
                                g: 30,
                                b: 30,
                            },
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
                    self.car_entities.push(entity);
                }
            }
        }
    }

    fn move_player(&mut self, delta_column: i32, delta_row: i32, world: &mut World) {
        let new_column = self.player_column + delta_column;
        let new_row = self.player_row + delta_row;

        if !(0..PLAY_WIDTH).contains(&new_column) || !(GOAL_ROW..=START_ROW).contains(&new_row) {
            return;
        }

        self.player_column = new_column;
        self.player_row = new_row;

        if let Some(position) = world.get_position_mut(self.player_entity) {
            position.column = (self.play_offset_x + self.player_column) as f64;
            position.row = (self.play_offset_y + self.player_row) as f64;
        }

        if self.player_row == GOAL_ROW {
            self.check_home_arrival(world);
            return;
        }

        if is_water_row(self.player_row) && !self.is_player_on_log(world) {
            self.drown(world);
        }
    }

    fn check_home_arrival(&mut self, world: &mut World) {
        let spacing = PLAY_WIDTH / (HOME_PAD_COUNT + 1);
        let mut landed_on_pad = false;

        for pad_index in 0..HOME_PAD_COUNT as usize {
            let pad_column = spacing * (pad_index as i32 + 1);
            if (self.player_column - pad_column).abs() <= 1 && !self.home_filled[pad_index] {
                self.home_filled[pad_index] = true;
                self.score += 100;
                landed_on_pad = true;

                if let Some(sprite) = world.get_sprite_mut(self.home_pad_entities[pad_index]) {
                    sprite.character = '@';
                    sprite.foreground = TermColor::Rgb {
                        r: 255,
                        g: 255,
                        b: 50,
                    };
                }
                break;
            }
        }

        if !landed_on_pad {
            self.lose_life(world);
            return;
        }

        if self.home_filled.iter().all(|&filled| filled) {
            self.next_level(world);
            return;
        }

        self.reset_player_position(world);
    }

    fn is_player_on_log(&self, world: &World) -> bool {
        let player_world_column = self.play_offset_x + self.player_column;
        let player_world_row = self.play_offset_y + self.player_row;

        self.log_entities.iter().any(|&entity| {
            world.get_position(entity).is_some_and(|position| {
                position.column as i32 == player_world_column
                    && position.row as i32 == player_world_row
            })
        })
    }

    fn drown(&mut self, world: &mut World) {
        self.lose_life(world);
    }

    fn lose_life(&mut self, world: &mut World) {
        self.lives = self.lives.saturating_sub(1);
        if self.lives == 0 {
            self.game_over = true;
        } else {
            self.reset_player_position(world);
        }
    }

    fn reset_player_position(&mut self, world: &mut World) {
        self.player_column = PLAY_WIDTH / 2;
        self.player_row = START_ROW;
        if let Some(position) = world.get_position_mut(self.player_entity) {
            position.column = (self.play_offset_x + self.player_column) as f64;
            position.row = (self.play_offset_y + self.player_row) as f64;
        }
    }

    fn wrap_entities(&self, world: &mut World) {
        let left_bound = self.play_offset_x;
        let right_bound = self.play_offset_x + PLAY_WIDTH;

        for &entity in self.car_entities.iter().chain(self.log_entities.iter()) {
            if let Some(position) = world.get_position_mut(entity) {
                if position.column as i32 >= right_bound {
                    position.column = left_bound as f64;
                } else if (position.column as i32) < left_bound {
                    position.column = (right_bound - 1) as f64;
                }
            }
        }
    }

    fn handle_log_riding(&mut self, world: &mut World) {
        if !is_water_row(self.player_row) {
            return;
        }

        let velocity = lane_velocity_for_row(&self.lane_configs, self.player_row);
        self.player_column += velocity;

        if self.player_column < 0 || self.player_column >= PLAY_WIDTH {
            self.drown(world);
            return;
        }

        if let Some(position) = world.get_position_mut(self.player_entity) {
            position.column = (self.play_offset_x + self.player_column) as f64;
        }

        if !self.is_player_on_log(world) {
            self.drown(world);
        }
    }

    fn check_car_collisions(&mut self, world: &mut World) {
        let contacts = collision_pairs(world);
        for contact in &contacts {
            let player_involved =
                contact.entity_a == self.player_entity || contact.entity_b == self.player_entity;
            if !player_involved {
                continue;
            }
            let other = if contact.entity_a == self.player_entity {
                contact.entity_b
            } else {
                contact.entity_a
            };
            if self.car_entities.contains(&other) {
                self.lose_life(world);
                return;
            }
        }
    }

    fn next_level(&mut self, world: &mut World) {
        self.level += 1;
        self.score += 500;
        self.home_filled = vec![false; HOME_PAD_COUNT as usize];

        for entity in &self.home_pad_entities {
            if let Some(sprite) = world.get_sprite_mut(*entity) {
                sprite.character = 'V';
                sprite.foreground = TermColor::Rgb {
                    r: 50,
                    g: 200,
                    b: 50,
                };
            }
        }

        self.reset_player_position(world);
    }

    fn update_hud(&mut self, world: &mut World) {
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();

        let hud_text = format!(
            "Score: {}  Lives: {}  Level: {}",
            self.score, self.lives, self.level
        );

        let hud_row = self.play_offset_y + PLAY_HEIGHT;

        for (char_index, character) in hud_text.chars().enumerate() {
            if char_index >= PLAY_WIDTH as usize {
                break;
            }
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
        world.despawn_entities(&[self.player_entity]);
        for &entity in &self.car_entities {
            world.despawn_entities(&[entity]);
        }
        self.car_entities.clear();
        for &entity in &self.log_entities {
            world.despawn_entities(&[entity]);
        }
        self.log_entities.clear();
        for &entity in &self.background_entities {
            world.despawn_entities(&[entity]);
        }
        self.background_entities.clear();
        for &entity in &self.home_pad_entities {
            world.despawn_entities(&[entity]);
        }
        self.home_pad_entities.clear();
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Frogger"
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

        self.spawn_background(world);
        self.spawn_home_pads(world);
        self.spawn_lane_entities(world);
        self.spawn_player(world);
        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed || self.game_over || self.move_cooldown > 0.0 {
            return;
        }

        let (delta_column, delta_row) = match key {
            KeyCode::Up | KeyCode::Char('w') => (0, -1),
            KeyCode::Down | KeyCode::Char('s') => (0, 1),
            KeyCode::Left | KeyCode::Char('a') => (-1, 0),
            KeyCode::Right | KeyCode::Char('d') => (1, 0),
            KeyCode::Escape | KeyCode::Char('q') => {
                world.resources.should_exit = true;
                return;
            }
            _ => return,
        };

        self.move_player(delta_column, delta_row, world);
        self.move_cooldown = 0.1;
        self.score += 10;
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.game_over {
            return;
        }

        let delta = world.resources.timing.delta_seconds;

        if self.move_cooldown > 0.0 {
            self.move_cooldown -= delta;
        }

        self.tick_timer += delta;
        if self.tick_timer >= TICK_INTERVAL {
            self.tick_timer -= TICK_INTERVAL;

            let was_on_log = is_water_row(self.player_row) && self.is_player_on_log(world);

            movement_system(world);
            self.wrap_entities(world);

            if was_on_log {
                self.handle_log_riding(world);
            } else if is_water_row(self.player_row) {
                self.drown(world);
            }
        }

        self.check_car_collisions(world);
        self.update_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            let score = self.score;
            let level = self.level;
            self.clear_all_entities(world);
            return Some(Box::new(GameOverState {
                score,
                level,
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
        "Frogger - Game Over"
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
                format!("Score: {}", self.score),
                TermColor::Rgb {
                    r: 255,
                    g: 255,
                    b: 100,
                },
            ),
            (
                format!("Level: {}", self.level),
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
