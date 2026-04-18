use nightshade::tui::prelude::*;
use rand::Rng;

const WORD_COLORS: [TermColor; 5] = [
    TermColor::Cyan,
    TermColor::Green,
    TermColor::Yellow,
    TermColor::Magenta,
    TermColor::Rgb {
        r: 255,
        g: 150,
        b: 50,
    },
];

const SHORT_WORDS: [&str; 40] = [
    "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was", "one",
    "our", "out", "day", "get", "has", "him", "his", "how", "its", "may", "new", "now", "old",
    "see", "way", "who", "did", "let", "say", "she", "too", "use", "run", "cat", "dog", "red",
    "big",
];

const MEDIUM_WORDS: [&str; 37] = [
    "about", "after", "again", "below", "could", "every", "first", "found", "great", "house",
    "large", "learn", "never", "other", "place", "plant", "point", "right", "small", "sound",
    "spell", "still", "study", "their", "there", "these", "thing", "think", "three", "water",
    "where", "which", "world", "write", "flame", "storm", "brave",
];

const LONG_WORDS: [&str; 19] = [
    "because", "between", "country", "develop", "example", "general", "history", "picture",
    "problem", "program", "thought", "through", "machine", "science", "kingdom", "chamber",
    "warrior", "balance", "mystery",
];

struct FallingWord {
    text: String,
    entities: Vec<Entity>,
    column: i32,
    row: f64,
    color: TermColor,
}

struct FlashEffect {
    entities: Vec<Entity>,
    remaining: f64,
}

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Type Storm - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r" _____                  ____  _                      ",
            r"|_   _|   _ _ __   ___ / ___|| |_ ___  _ __ _ __ ___ ",
            r"  | || | | | '_ \ / _ \\___ \| __/ _ \| '__| '_ ` _ \",
            r"  | || |_| | |_) |  __/ ___) | || (_) | |  | | | | | |",
            r"  |_| \__, | .__/ \___|____/ \__\___/|_|  |_| |_| |_|",
            r"      |___/|_|                                        ",
        ];

        let title_colors = [
            TermColor::Rgb {
                r: 0,
                g: 200,
                b: 255,
            },
            TermColor::Rgb {
                r: 0,
                g: 220,
                b: 240,
            },
            TermColor::Rgb {
                r: 0,
                g: 240,
                b: 220,
            },
            TermColor::Rgb {
                r: 0,
                g: 255,
                b: 200,
            },
            TermColor::Rgb {
                r: 50,
                g: 255,
                b: 180,
            },
            TermColor::Rgb {
                r: 100,
                g: 255,
                b: 160,
            },
        ];

        let title_start_row = center_row - 8;

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
                            foreground: title_colors[line_index],
                            background: TermColor::Black,
                        },
                    );
                    world.set_z_index(entity, ZIndex(10));
                }
            }
        }

        let instructions = [
            (
                "Words fall from the sky. Type them to destroy them!",
                TermColor::White,
            ),
            ("", TermColor::Black),
            (
                "Type the word and it auto-destroys on match",
                TermColor::Grey,
            ),
            ("Backspace to correct mistakes", TermColor::Grey),
            ("Don't let words reach the bottom!", TermColor::Grey),
            ("", TermColor::Black),
            ("Build combos for bonus points!", TermColor::Yellow),
            ("", TermColor::Black),
            (
                "Press ENTER to start",
                TermColor::Rgb {
                    r: 100,
                    g: 255,
                    b: 100,
                },
            ),
            ("Press ESC to quit", TermColor::DarkGrey),
        ];

        let instructions_start_row = title_start_row + title_lines.len() as i32 + 2;

        for (line_index, (text, color)) in instructions.iter().enumerate() {
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
                        row: (instructions_start_row + line_index as i32) as f64,
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
    falling_words: Vec<FallingWord>,
    input_buffer: String,
    input_entities: Vec<Entity>,
    hud_entities: Vec<Entity>,
    flash_effects: Vec<FlashEffect>,
    score: u32,
    lives: u32,
    level: u32,
    combo: u32,
    words_destroyed: u32,
    spawn_timer: Timer,
    fall_speed: f64,
    game_over: bool,
    cursor_blink_timer: f64,
    cursor_visible: bool,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            falling_words: Vec::new(),
            input_buffer: String::new(),
            input_entities: Vec::new(),
            hud_entities: Vec::new(),
            flash_effects: Vec::new(),
            score: 0,
            lives: 5,
            level: 1,
            combo: 0,
            words_destroyed: 0,
            spawn_timer: Timer::repeating(2.0),
            fall_speed: 1.5,
            game_over: false,
            cursor_blink_timer: 0.0,
            cursor_visible: true,
        }
    }

    fn pick_random_word(&self) -> &'static str {
        let mut rng = rand::rng();
        match self.level {
            1 => SHORT_WORDS[rng.random_range(0..SHORT_WORDS.len())],
            2 => {
                if rng.random_range(0..100) < 80 {
                    SHORT_WORDS[rng.random_range(0..SHORT_WORDS.len())]
                } else {
                    MEDIUM_WORDS[rng.random_range(0..MEDIUM_WORDS.len())]
                }
            }
            3 => {
                if rng.random_range(0..100) < 50 {
                    SHORT_WORDS[rng.random_range(0..SHORT_WORDS.len())]
                } else {
                    MEDIUM_WORDS[rng.random_range(0..MEDIUM_WORDS.len())]
                }
            }
            4 => {
                let roll = rng.random_range(0..100);
                if roll < 30 {
                    SHORT_WORDS[rng.random_range(0..SHORT_WORDS.len())]
                } else if roll < 80 {
                    MEDIUM_WORDS[rng.random_range(0..MEDIUM_WORDS.len())]
                } else {
                    LONG_WORDS[rng.random_range(0..LONG_WORDS.len())]
                }
            }
            _ => {
                let roll = rng.random_range(0..100);
                if roll < 20 {
                    SHORT_WORDS[rng.random_range(0..SHORT_WORDS.len())]
                } else if roll < 60 {
                    MEDIUM_WORDS[rng.random_range(0..MEDIUM_WORDS.len())]
                } else {
                    LONG_WORDS[rng.random_range(0..LONG_WORDS.len())]
                }
            }
        }
    }

    fn spawn_word(&mut self, world: &mut World) {
        let word_text = self.pick_random_word();

        let already_active = self
            .falling_words
            .iter()
            .any(|falling_word| falling_word.text == word_text);
        if already_active {
            return;
        }

        let terminal_columns = world.resources.terminal_size.columns as i32;
        let max_column = terminal_columns - word_text.len() as i32 - 1;
        let column = if max_column > 1 {
            rand::rng().random_range(1..max_column)
        } else {
            1
        };

        let color = WORD_COLORS[rand::rng().random_range(0..WORD_COLORS.len())];

        let mut entities = Vec::with_capacity(word_text.len());
        for (char_index, character) in word_text.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (column + char_index as i32) as f64,
                    row: 2.0,
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
            world.set_z_index(entity, ZIndex(5));
            entities.push(entity);
        }

        self.falling_words.push(FallingWord {
            text: word_text.to_string(),
            entities,
            column,
            row: 2.0,
            color,
        });
    }

    fn update_falling_words(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;
        let bottom_row = world.resources.terminal_size.rows as f64 - 3.0;

        let mut missed_indices = Vec::new();

        for (word_index, falling_word) in self.falling_words.iter_mut().enumerate() {
            falling_word.row += self.fall_speed * delta;

            for (char_index, entity) in falling_word.entities.iter().enumerate() {
                if let Some(position) = world.get_position_mut(*entity) {
                    position.row = falling_word.row;
                    position.column = (falling_word.column + char_index as i32) as f64;
                }
            }

            if falling_word.row >= bottom_row {
                missed_indices.push(word_index);
            }
        }

        missed_indices.sort_unstable_by(|first, second| second.cmp(first));

        for word_index in missed_indices {
            let falling_word = self.falling_words.remove(word_index);
            world.despawn_entities(&falling_word.entities);
            self.lives = self.lives.saturating_sub(1);
            self.combo = 0;
            if self.lives == 0 {
                self.game_over = true;
            }
        }
    }

    fn highlight_matching_words(&self, world: &mut World) {
        for falling_word in &self.falling_words {
            let is_prefix =
                !self.input_buffer.is_empty() && falling_word.text.starts_with(&self.input_buffer);

            for (char_index, entity) in falling_word.entities.iter().enumerate() {
                if let Some(sprite) = world.get_sprite_mut(*entity) {
                    if is_prefix && char_index < self.input_buffer.len() {
                        sprite.foreground = TermColor::White;
                        sprite.background = TermColor::Rgb {
                            r: 40,
                            g: 40,
                            b: 80,
                        };
                    } else {
                        sprite.foreground = falling_word.color;
                        sprite.background = TermColor::Black;
                    }
                }
            }
        }
    }

    fn check_word_match(&mut self, world: &mut World) {
        let mut best_match_index: Option<usize> = None;
        let mut best_row: f64 = f64::MIN;

        for (word_index, falling_word) in self.falling_words.iter().enumerate() {
            if falling_word.text == self.input_buffer && falling_word.row > best_row {
                best_match_index = Some(word_index);
                best_row = falling_word.row;
            }
        }

        if let Some(match_index) = best_match_index {
            let matched_word = self.falling_words.remove(match_index);

            let mut flash_entities = Vec::new();
            for (char_index, character) in matched_word.text.chars().enumerate() {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (matched_word.column + char_index as i32) as f64,
                        row: matched_word.row,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: TermColor::White,
                        background: TermColor::Rgb {
                            r: 255,
                            g: 255,
                            b: 200,
                        },
                    },
                );
                world.set_z_index(entity, ZIndex(20));
                flash_entities.push(entity);
            }

            world.despawn_entities(&matched_word.entities);

            self.flash_effects.push(FlashEffect {
                entities: flash_entities,
                remaining: 0.15,
            });

            self.combo += 1;
            let word_length = matched_word.text.len() as u32;
            self.score += word_length * 10 * self.combo;
            self.words_destroyed += 1;
            self.input_buffer.clear();

            if self.words_destroyed > 0 && self.words_destroyed.is_multiple_of(10) {
                self.level += 1;
                self.fall_speed += 0.3;
                let new_spawn_duration = (2.0 - (self.level as f64 - 1.0) * 0.2).max(0.5);
                self.spawn_timer.set_duration(new_spawn_duration);
            }
        }
    }

    fn update_flash_effects(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;
        let mut finished_indices = Vec::new();

        for (effect_index, flash) in self.flash_effects.iter_mut().enumerate() {
            flash.remaining -= delta;
            if flash.remaining <= 0.0 {
                finished_indices.push(effect_index);
            } else {
                let brightness = (flash.remaining / 0.15 * 255.0) as u8;
                for entity in &flash.entities {
                    if let Some(sprite) = world.get_sprite_mut(*entity) {
                        sprite.foreground = TermColor::Rgb {
                            r: brightness,
                            g: brightness,
                            b: brightness,
                        };
                        sprite.background = TermColor::Black;
                    }
                }
            }
        }

        finished_indices.sort_unstable_by(|first, second| second.cmp(first));

        for effect_index in finished_indices {
            let flash = self.flash_effects.remove(effect_index);
            world.despawn_entities(&flash.entities);
        }
    }

    fn render_input_prompt(&mut self, world: &mut World) {
        for entity in &self.input_entities {
            world.despawn_entities(&[*entity]);
        }
        self.input_entities.clear();

        let terminal = world.resources.terminal_size;
        let prompt_row = terminal.rows as i32 - 1;
        let prompt_text = format!(
            "> {}{}",
            self.input_buffer,
            if self.cursor_visible { "_" } else { " " }
        );

        for (char_index, character) in prompt_text.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (1 + char_index as i32) as f64,
                    row: prompt_row as f64,
                },
            );

            let foreground = if char_index < 2 {
                TermColor::DarkCyan
            } else if char_index == prompt_text.len() - 1 {
                TermColor::Rgb {
                    r: 100,
                    g: 200,
                    b: 255,
                }
            } else {
                TermColor::White
            };

            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
            self.input_entities.push(entity);
        }
    }

    fn render_hud(&mut self, world: &mut World) {
        for entity in &self.hud_entities {
            world.despawn_entities(&[*entity]);
        }
        self.hud_entities.clear();

        let terminal = world.resources.terminal_size;

        let hud_text = format!(
            "Score: {} | Lives: {} | Level: {} | Combo: {}x",
            self.score, self.lives, self.level, self.combo
        );

        let center_column = terminal.columns as i32 / 2;
        let start_column = center_column - hud_text.len() as i32 / 2;

        for (char_index, character) in hud_text.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (start_column + char_index as i32) as f64,
                    row: 0.0,
                },
            );

            let foreground = if char_index < 6 + self.score.to_string().len() {
                TermColor::Yellow
            } else if hud_text[..char_index + 1].contains("Lives") && {
                let lives_start = hud_text.find("Lives").unwrap_or(0);
                let lives_end = lives_start + 7 + self.lives.to_string().len();
                char_index >= lives_start && char_index < lives_end
            } {
                if self.lives <= 1 {
                    TermColor::Red
                } else if self.lives <= 2 {
                    TermColor::Rgb {
                        r: 255,
                        g: 165,
                        b: 0,
                    }
                } else {
                    TermColor::Green
                }
            } else if hud_text[..char_index + 1].contains("Combo") && {
                let combo_start = hud_text.find("Combo").unwrap_or(0);
                let combo_end = combo_start + 7 + self.combo.to_string().len() + 1;
                char_index >= combo_start && char_index < combo_end
            } {
                if self.combo >= 5 {
                    TermColor::Rgb {
                        r: 255,
                        g: 100,
                        b: 255,
                    }
                } else if self.combo >= 3 {
                    TermColor::Rgb {
                        r: 255,
                        g: 200,
                        b: 50,
                    }
                } else {
                    TermColor::Cyan
                }
            } else {
                TermColor::White
            };

            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
            self.hud_entities.push(entity);
        }

        let separator_row = 1;
        let terminal_width = terminal.columns as i32;
        for column_index in 0..terminal_width {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: column_index as f64,
                    row: separator_row as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: '-',
                    foreground: TermColor::DarkGrey,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(1));
            self.hud_entities.push(entity);
        }

        let bottom_separator_row = terminal.rows as i32 - 2;
        for column_index in 0..terminal_width {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: column_index as f64,
                    row: bottom_separator_row as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: '-',
                    foreground: TermColor::DarkGrey,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(1));
            self.hud_entities.push(entity);
        }
    }

    fn cleanup_all(&mut self, world: &mut World) {
        for falling_word in &self.falling_words {
            world.despawn_entities(&falling_word.entities);
        }
        self.falling_words.clear();

        for flash in &self.flash_effects {
            world.despawn_entities(&flash.entities);
        }
        self.flash_effects.clear();

        for entity in &self.input_entities {
            world.despawn_entities(&[*entity]);
        }
        self.input_entities.clear();

        for entity in &self.hud_entities {
            world.despawn_entities(&[*entity]);
        }
        self.hud_entities.clear();
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Type Storm - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }

        if self.game_over {
            return;
        }

        match key {
            KeyCode::Escape => {
                world.resources.should_exit = true;
            }
            KeyCode::Char(character) if character.is_ascii_alphabetic() => {
                self.input_buffer.push(character.to_ascii_lowercase());
                self.check_word_match(world);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.game_over {
            return;
        }

        let delta = world.resources.timing.delta_seconds;

        self.cursor_blink_timer += delta;
        if self.cursor_blink_timer >= 0.5 {
            self.cursor_blink_timer = 0.0;
            self.cursor_visible = !self.cursor_visible;
        }

        if self.spawn_timer.tick(delta) {
            self.spawn_word(world);
        }

        self.update_falling_words(world);
        self.update_flash_effects(world);
        self.highlight_matching_words(world);
        self.render_input_prompt(world);
        self.render_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            let score = self.score;
            let level = self.level;
            let words_destroyed = self.words_destroyed;
            self.cleanup_all(world);
            return Some(Box::new(GameOverState {
                score,
                level,
                words_destroyed,
                restart: false,
            }));
        }
        None
    }
}

struct GameOverState {
    score: u32,
    level: u32,
    words_destroyed: u32,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Type Storm - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

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
                format!("Level Reached: {}", self.level),
                TermColor::Rgb {
                    r: 100,
                    g: 255,
                    b: 200,
                },
            ),
            (
                format!("Words Typed: {}", self.words_destroyed),
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
            KeyCode::Char('r') => {
                self.restart = true;
            }
            KeyCode::Escape => {
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
