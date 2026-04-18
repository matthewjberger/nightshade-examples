use nightshade::tui::prelude::*;
use rand::Rng;

const PLAY_WIDTH: i32 = 40;
const PLAY_HEIGHT: i32 = 22;
const BIRD_COLUMN: i32 = 10;
const PIPE_GAP: i32 = 6;
const GRAVITY: f64 = 0.4;
const FLAP_STRENGTH: f64 = -2.5;
const MAX_FALL_VELOCITY: f64 = 3.0;
const BIRD_TICK_INTERVAL: f64 = 0.1;
const PIPE_MOVE_INTERVAL: f64 = 0.12;
const PIPE_SPAWN_INTERVAL: f64 = 2.0;

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Flappy Bird - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r" _____ _                         ____  _         _ ",
            r"|  ___| | __ _ _ __  _ __  _   _| __ )(_)_ __ __| |",
            r"| |_  | |/ _` | '_ \| '_ \| | | |  _ \| | '__/ _` |",
            r"|  _| | | (_| | |_) | |_) | |_| | |_) | | | | (_| |",
            r"|_|   |_|\__,_| .__/| .__/ \__, |____/|_|_|  \__,_|",
            r"              |_|   |_|    |___/                    ",
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
                                r: 255,
                                g: 220,
                                b: 50,
                            },
                            background: TermColor::Black,
                        },
                    );
                    world.set_z_index(entity, ZIndex(10));
                }
            }
        }

        let bird_art = ">o>";
        let bird_start = center_column - bird_art.len() as i32 / 2;
        for (char_index, character) in bird_art.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (bird_start + char_index as i32) as f64,
                    row: (title_start_row + 8) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: TermColor::Rgb {
                        r: 255,
                        g: 200,
                        b: 50,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }

        let prompt = "Press SPACE to flap!";
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
            KeyCode::Enter | KeyCode::Char(' ') => {
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

struct PipeGroup {
    entities: Vec<Entity>,
    scored: bool,
}

struct GameplayState {
    play_offset_x: i32,
    play_offset_y: i32,
    bird_entity: Entity,
    bird_y: f64,
    bird_velocity_y: f64,
    pipe_groups: Vec<PipeGroup>,
    ground_entities: Vec<Entity>,
    hud_entities: Vec<Entity>,
    score: u32,
    game_over: bool,
    bird_timer: f64,
    pipe_move_timer: f64,
    pipe_spawn_timer: f64,
    started: bool,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            play_offset_x: 0,
            play_offset_y: 0,
            bird_entity: Entity::default(),
            bird_y: (PLAY_HEIGHT / 2) as f64,
            bird_velocity_y: 0.0,
            pipe_groups: Vec::new(),
            ground_entities: Vec::new(),
            hud_entities: Vec::new(),
            score: 0,
            game_over: false,
            bird_timer: 0.0,
            pipe_move_timer: 0.0,
            pipe_spawn_timer: 0.0,
            started: false,
        }
    }

    fn spawn_bird(&mut self, world: &mut World) {
        self.bird_y = (PLAY_HEIGHT / 2) as f64;
        self.bird_velocity_y = 0.0;
        self.bird_entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
        world.set_position(
            self.bird_entity,
            Position {
                column: (self.play_offset_x + BIRD_COLUMN) as f64,
                row: (self.play_offset_y + self.bird_y as i32) as f64,
            },
        );
        world.set_sprite(
            self.bird_entity,
            Sprite {
                character: '>',
                foreground: TermColor::Rgb {
                    r: 255,
                    g: 220,
                    b: 50,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(self.bird_entity, ZIndex(3));
        world.set_collider(
            self.bird_entity,
            Collider {
                width: 1,
                height: 1,
                ..Default::default()
            },
        );
    }

    fn spawn_ground(&mut self, world: &mut World) {
        for column in 0..PLAY_WIDTH {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (self.play_offset_x + column) as f64,
                    row: (self.play_offset_y + PLAY_HEIGHT - 1) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: '=',
                    foreground: TermColor::Rgb {
                        r: 139,
                        g: 90,
                        b: 43,
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
            self.ground_entities.push(entity);
        }
    }

    fn spawn_pipe(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        let min_gap_center = PIPE_GAP / 2 + 2;
        let max_gap_center = PLAY_HEIGHT - 2 - PIPE_GAP / 2;
        let gap_center = rng.random_range(min_gap_center..=max_gap_center);
        let gap_top = gap_center - PIPE_GAP / 2;
        let gap_bottom = gap_center + PIPE_GAP / 2;

        let spawn_column = self.play_offset_x + PLAY_WIDTH - 1;
        let mut entities = Vec::new();

        for row in 1..gap_top {
            let character = if row == gap_top - 1 { '[' } else { '|' };
            let entity =
                world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER | VELOCITY, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: spawn_column as f64,
                    row: (self.play_offset_y + row) as f64,
                },
            );
            world.set_velocity(
                entity,
                Velocity {
                    column: -1.0,
                    row: 0.0,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: TermColor::Rgb {
                        r: 50,
                        g: 180,
                        b: 50,
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
            entities.push(entity);
        }

        for row in (gap_bottom + 1)..(PLAY_HEIGHT - 1) {
            let character = if row == gap_bottom + 1 { '[' } else { '|' };
            let entity =
                world.spawn_entities(POSITION | SPRITE | Z_INDEX | COLLIDER | VELOCITY, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: spawn_column as f64,
                    row: (self.play_offset_y + row) as f64,
                },
            );
            world.set_velocity(
                entity,
                Velocity {
                    column: -1.0,
                    row: 0.0,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: TermColor::Rgb {
                        r: 50,
                        g: 180,
                        b: 50,
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
            entities.push(entity);
        }

        self.pipe_groups.push(PipeGroup {
            entities,
            scored: false,
        });
    }

    fn flap(&mut self) {
        self.bird_velocity_y = FLAP_STRENGTH;
        self.started = true;
    }

    fn update_bird(&mut self, world: &mut World) {
        if !self.started {
            return;
        }

        self.bird_velocity_y += GRAVITY;
        if self.bird_velocity_y > MAX_FALL_VELOCITY {
            self.bird_velocity_y = MAX_FALL_VELOCITY;
        }

        self.bird_y += self.bird_velocity_y;

        if self.bird_y < 1.0 {
            self.bird_y = 1.0;
            self.bird_velocity_y = 0.0;
        }

        let bird_row = self.bird_y as i32;
        if let Some(position) = world.get_position_mut(self.bird_entity) {
            position.row = (self.play_offset_y + bird_row) as f64;
        }

        let character = if self.bird_velocity_y < -1.0 {
            '^'
        } else if self.bird_velocity_y > 1.0 {
            'v'
        } else {
            '>'
        };
        if let Some(sprite) = world.get_sprite_mut(self.bird_entity) {
            sprite.character = character;
        }
    }

    fn check_collisions(&mut self, world: &World) {
        if self.bird_y as i32 >= PLAY_HEIGHT - 1 {
            self.game_over = true;
            return;
        }

        let contacts = collision_pairs(world);
        for contact in &contacts {
            if contact.entity_a == self.bird_entity || contact.entity_b == self.bird_entity {
                self.game_over = true;
                return;
            }
        }
    }

    fn check_scoring(&mut self, world: &World) {
        let bird_world_column = self.play_offset_x + BIRD_COLUMN;
        for group in &mut self.pipe_groups {
            if group.scored {
                continue;
            }
            if let Some(&first_entity) = group.entities.first()
                && let Some(position) = world.get_position(first_entity)
                && (position.column as i32) < bird_world_column
            {
                group.scored = true;
                self.score += 1;
            }
        }
    }

    fn cleanup_pipes(&mut self, world: &mut World) {
        let left_bound = self.play_offset_x - 2;
        let mut groups_to_remove: Vec<usize> = Vec::new();

        for (index, group) in self.pipe_groups.iter().enumerate() {
            let all_off_screen = group.entities.iter().all(|&entity| {
                world
                    .get_position(entity)
                    .is_some_and(|position| (position.column as i32) < left_bound)
            });
            if all_off_screen {
                groups_to_remove.push(index);
            }
        }

        for &index in groups_to_remove.iter().rev() {
            let group = self.pipe_groups.remove(index);
            for entity in group.entities {
                world.despawn_entities(&[entity]);
            }
        }
    }

    fn update_hud(&mut self, world: &mut World) {
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();

        let hud_text = if self.started {
            format!("Score: {}", self.score)
        } else {
            "Press SPACE to start".to_string()
        };

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
        world.despawn_entities(&[self.bird_entity]);
        for group in &self.pipe_groups {
            for &entity in &group.entities {
                world.despawn_entities(&[entity]);
            }
        }
        self.pipe_groups.clear();
        for &entity in &self.ground_entities {
            world.despawn_entities(&[entity]);
        }
        self.ground_entities.clear();
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Flappy Bird - Ember"
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

        self.spawn_ground(world);
        self.spawn_bird(world);
        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Char(' ') | KeyCode::Up if !self.game_over => {
                self.flap();
            }
            KeyCode::Escape | KeyCode::Char('q') => {
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

        self.bird_timer += delta;
        if self.bird_timer >= BIRD_TICK_INTERVAL {
            self.bird_timer -= BIRD_TICK_INTERVAL;
            self.update_bird(world);
        }

        self.pipe_move_timer += delta;
        if self.pipe_move_timer >= PIPE_MOVE_INTERVAL {
            self.pipe_move_timer -= PIPE_MOVE_INTERVAL;
            movement_system(world);
        }

        if self.started {
            self.pipe_spawn_timer += delta;
            if self.pipe_spawn_timer >= PIPE_SPAWN_INTERVAL {
                self.pipe_spawn_timer -= PIPE_SPAWN_INTERVAL;
                self.spawn_pipe(world);
            }
        }

        self.check_collisions(world);
        self.check_scoring(world);
        self.cleanup_pipes(world);
        self.update_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            let score = self.score;
            self.clear_all_entities(world);
            return Some(Box::new(GameOverState {
                score,
                restart: false,
            }));
        }
        None
    }
}

struct GameOverState {
    score: u32,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Flappy Bird - Game Over"
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
                    g: 220,
                    b: 50,
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
