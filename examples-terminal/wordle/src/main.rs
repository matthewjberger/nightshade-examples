use nightshade::tui::prelude::*;
use rand::Rng;

const WORD_LENGTH: usize = 5;
const MAX_GUESSES: usize = 6;

const WORD_LIST: &[&str] = &[
    "about", "above", "abuse", "actor", "acute", "admit", "adopt", "adult", "after", "again",
    "agent", "agree", "ahead", "alarm", "album", "alert", "alien", "align", "alive", "alley",
    "allow", "alone", "alter", "angel", "anger", "angle", "angry", "anime", "apple", "apply",
    "arena", "argue", "arise", "armor", "array", "asset", "avoid", "award", "aware", "badge",
    "basic", "basin", "basis", "beach", "begun", "being", "bench", "bible", "blade", "blame",
    "blank", "blast", "blaze", "bleed", "blend", "bless", "blind", "block", "blood", "blown",
    "board", "bonus", "booth", "bound", "brain", "brand", "brave", "bread", "break", "breed",
    "brick", "brief", "bring", "broad", "brown", "brush", "buddy", "build", "bunch", "burst",
    "buyer", "cabin", "candy", "carry", "catch", "cause", "chain", "chair", "charm", "chart",
    "chase", "cheap", "check", "chess", "chest", "chief", "child", "china", "chunk", "civil",
    "claim", "clash", "class", "clean", "clear", "climb", "cling", "clock", "clone", "close",
    "cloud", "coach", "coast", "could", "count", "court", "cover", "crack", "craft", "crane",
    "crash", "crazy", "cream", "crime", "cross", "crowd", "crown", "cruel", "crush", "curve",
    "cycle", "daily", "dance", "death", "delay", "depth", "devil", "dirty", "donor", "doubt",
    "dozen", "draft", "drain", "drama", "drawn", "dream", "dress", "dried", "drift", "drink",
    "drive", "droit", "drops", "drove", "dying", "eager", "early", "earth", "eight", "elect",
    "elite", "empty", "enemy", "enjoy", "enter", "entry", "equal", "error", "essay", "event",
    "every", "exact", "exile", "exist", "extra", "faith", "false", "fault", "feast", "fence",
    "fewer", "fiber", "field", "fifth", "fifty", "fight", "final", "first", "fixed", "flame",
    "flash", "fleet", "flesh", "float", "flood", "floor", "flour", "fluid", "focus", "force",
    "forge", "forth", "found", "frame", "frank", "fraud", "fresh", "front", "frost", "fruit",
    "fully", "funny", "gamma", "ghost", "giant", "given", "glass", "globe", "going", "grace",
    "grade", "grain", "grand", "grant", "graph", "grasp", "grass", "grave", "great", "green",
    "gross", "group", "grove", "grown", "guard", "guess", "guide", "guild", "happy", "harsh",
    "heart", "heavy", "hence", "hobby", "honor", "horse", "hotel", "house", "human", "humor",
    "ideal", "image", "imply", "index", "indie", "inner", "input", "irony", "ivory", "joint",
    "judge", "juice", "knife", "known", "label", "labor", "large", "laser", "later", "laugh",
    "layer", "learn", "lease", "legal", "level", "light", "limit", "linen", "liver", "local",
    "logic", "loose", "lover", "lower", "loyal", "lucky", "lunch", "lying", "magic", "major",
    "maker", "manor", "maple", "march", "marry", "match", "mayor", "media", "mercy", "merit",
    "metal", "might", "minor", "minus", "mixed", "model", "money", "month", "moral", "motif",
    "motor", "mount", "mouse", "mouth", "movie", "music", "naive", "nerve", "never", "night",
    "noble", "noise", "north", "noted", "novel", "nurse", "ocean", "offer", "often", "opera",
    "orbit", "order", "other", "outer", "ought", "overt", "owner", "oxide", "paint", "panel",
    "panic", "party", "pasta", "patch", "pause", "peace", "penny", "phase", "phone", "photo",
    "piano", "piece", "pilot", "pitch", "pixel", "pizza", "place", "plain", "plane", "plant",
    "plate", "plaza", "plead", "point", "pound", "power", "press", "price", "pride", "prime",
    "prince", "print", "prior", "prize", "probe", "proof", "proud", "prove", "psalm", "pulse",
    "punch", "pupil", "queen", "quest", "quiet", "quota", "quote", "radar", "radio", "raise",
    "range", "rapid", "ratio", "reach", "realm", "rebel", "reign", "relax", "renew", "reply",
    "rider", "ridge", "rifle", "right", "risky", "rival", "river", "robin", "robot", "rocky",
    "roman", "rough", "round", "route", "royal", "rugby", "ruler", "rural", "saint", "salad",
    "scale", "scene", "scope", "score", "sense", "serve", "seven", "shade", "shall", "shame",
    "shape", "share", "sharp", "sheep", "sheer", "sheet", "shelf", "shell", "shift", "shine",
    "shirt", "shock", "shoot", "shore", "short", "shout", "sight", "sigma", "since", "sixth",
    "sixty", "skill", "skirt", "skull", "slave", "sleep", "slice", "slide", "slope", "smart",
    "smell", "smile", "smoke", "snake", "solar", "solid", "solve", "sorry", "sound", "south",
    "space", "spare", "speak", "speed", "spend", "spice", "spine", "spite", "split", "sport",
    "spray", "squad", "stack", "staff", "stage", "stake", "stall", "stamp", "stand", "stare",
    "start", "state", "stays", "steam", "steel", "steep", "steer", "stern", "stick", "still",
    "stock", "stone", "stood", "store", "storm", "story", "stove", "strip", "stuck", "study",
    "stuff", "style", "sugar", "suite", "super", "surge", "swamp", "swear", "sweep", "sweet",
    "swift", "swing", "sword", "syrup", "table", "taste", "teeth", "thank", "theme", "there",
    "thick", "thing", "think", "third", "those", "three", "throw", "thumb", "tiger", "tight",
    "timer", "tired", "title", "today", "token", "total", "touch", "tough", "tower", "toxic",
    "trace", "track", "trade", "trail", "train", "trait", "trash", "treat", "trend", "trial",
    "tribe", "trick", "tried", "troop", "truck", "truly", "trunk", "trust", "truth", "tumor",
    "twice", "twist", "ultra", "uncle", "under", "union", "unity", "until", "upper", "upset",
    "urban", "usage", "usual", "valid", "value", "valve", "verse", "video", "vigor", "vinyl",
    "viral", "virus", "visit", "vital", "vivid", "vocal", "voice", "voter", "wagon", "waste",
    "watch", "water", "weave", "weird", "whale", "wheat", "wheel", "where", "which", "while",
    "white", "whole", "whose", "widow", "width", "woman", "world", "worry", "worse", "worst",
    "worth", "would", "wound", "wrath", "write", "wrong", "wrote", "yacht", "yield", "young",
    "youth",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum LetterResult {
    Correct,
    WrongPosition,
    NotInWord,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyboardLetterState {
    Unknown,
    Correct,
    WrongPosition,
    NotInWord,
}

fn evaluate_guess(guess: &str, answer: &str) -> [LetterResult; WORD_LENGTH] {
    let mut results = [LetterResult::NotInWord; WORD_LENGTH];
    let guess_bytes: Vec<u8> = guess.bytes().collect();
    let answer_bytes: Vec<u8> = answer.bytes().collect();
    let mut answer_used = [false; WORD_LENGTH];

    for index in 0..WORD_LENGTH {
        if guess_bytes[index] == answer_bytes[index] {
            results[index] = LetterResult::Correct;
            answer_used[index] = true;
        }
    }

    for guess_index in 0..WORD_LENGTH {
        if results[guess_index] == LetterResult::Correct {
            continue;
        }
        for answer_index in 0..WORD_LENGTH {
            if !answer_used[answer_index] && guess_bytes[guess_index] == answer_bytes[answer_index]
            {
                results[guess_index] = LetterResult::WrongPosition;
                answer_used[answer_index] = true;
                break;
            }
        }
    }

    results
}

fn result_foreground(result: LetterResult) -> TermColor {
    match result {
        LetterResult::Correct => TermColor::White,
        LetterResult::WrongPosition => TermColor::White,
        LetterResult::NotInWord => TermColor::Rgb {
            r: 180,
            g: 180,
            b: 190,
        },
    }
}

fn result_background(result: LetterResult) -> TermColor {
    match result {
        LetterResult::Correct => TermColor::Rgb {
            r: 80,
            g: 180,
            b: 80,
        },
        LetterResult::WrongPosition => TermColor::Rgb {
            r: 200,
            g: 180,
            b: 50,
        },
        LetterResult::NotInWord => TermColor::Rgb {
            r: 60,
            g: 60,
            b: 68,
        },
    }
}

fn keyboard_state_foreground(state: KeyboardLetterState) -> TermColor {
    match state {
        KeyboardLetterState::Unknown => TermColor::White,
        KeyboardLetterState::Correct => TermColor::White,
        KeyboardLetterState::WrongPosition => TermColor::White,
        KeyboardLetterState::NotInWord => TermColor::Rgb {
            r: 100,
            g: 100,
            b: 108,
        },
    }
}

fn keyboard_state_background(state: KeyboardLetterState) -> TermColor {
    match state {
        KeyboardLetterState::Unknown => TermColor::Rgb {
            r: 120,
            g: 120,
            b: 130,
        },
        KeyboardLetterState::Correct => TermColor::Rgb {
            r: 80,
            g: 180,
            b: 80,
        },
        KeyboardLetterState::WrongPosition => TermColor::Rgb {
            r: 200,
            g: 180,
            b: 50,
        },
        KeyboardLetterState::NotInWord => TermColor::Rgb {
            r: 40,
            g: 40,
            b: 44,
        },
    }
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Wordle - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "W O R D L E";
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
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let sample_row = center_row - 3.0;
        let sample_word = "EMBER";
        let sample_results = [
            LetterResult::Correct,
            LetterResult::WrongPosition,
            LetterResult::NotInWord,
            LetterResult::Correct,
            LetterResult::WrongPosition,
        ];
        for (char_index, character) in sample_word.chars().enumerate() {
            let col = center_column - 12.0 + char_index as f64 * 5.0;
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: col,
                    row: sample_row,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: format!("  {}  ", character),
                    foreground: result_foreground(sample_results[char_index]),
                    background: result_background(sample_results[char_index]),
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }

        let subtitle = "Guess the 5-letter word in 6 tries";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - subtitle.len() as f64 / 2.0,
                row: center_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: subtitle.to_string(),
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let prompt = "Press ENTER to start";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - prompt.len() as f64 / 2.0,
                row: center_row + 2.0,
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

        let quit = "ESC to quit";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - quit.len() as f64 / 2.0,
                row: center_row + 4.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: quit.to_string(),
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
            return Some(Box::new(GameplayState::new()));
        }
        None
    }
}

struct GameplayState {
    answer: String,
    guesses: Vec<(String, [LetterResult; WORD_LENGTH])>,
    current_input: String,
    keyboard_states: [KeyboardLetterState; 26],
    entities: EntityGroup,
    message: String,
    message_timer: f64,
    won: bool,
    lost: bool,
}

impl GameplayState {
    fn new() -> Self {
        let mut rng = rand::rng();
        let word_index = rng.random_range(0..WORD_LIST.len());
        Self {
            answer: WORD_LIST[word_index].to_string(),
            guesses: Vec::new(),
            current_input: String::new(),
            keyboard_states: [KeyboardLetterState::Unknown; 26],
            entities: EntityGroup::new(),
            message: String::new(),
            message_timer: 0.0,
            won: false,
            lost: false,
        }
    }

    fn submit_guess(&mut self) {
        if self.current_input.len() != WORD_LENGTH {
            self.message = "Not enough letters".to_string();
            self.message_timer = 2.0;
            return;
        }

        let guess = self.current_input.to_lowercase();

        let results = evaluate_guess(&guess, &self.answer);
        for (char_index, character) in guess.chars().enumerate() {
            let letter_index = (character as u8 - b'a') as usize;
            let new_state = match results[char_index] {
                LetterResult::Correct => KeyboardLetterState::Correct,
                LetterResult::WrongPosition => KeyboardLetterState::WrongPosition,
                LetterResult::NotInWord => KeyboardLetterState::NotInWord,
            };
            match self.keyboard_states[letter_index] {
                KeyboardLetterState::Correct => {}
                KeyboardLetterState::WrongPosition => {
                    if new_state == KeyboardLetterState::Correct {
                        self.keyboard_states[letter_index] = new_state;
                    }
                }
                _ => {
                    self.keyboard_states[letter_index] = new_state;
                }
            }
        }

        if results
            .iter()
            .all(|result| *result == LetterResult::Correct)
        {
            self.won = true;
        }

        self.guesses.push((guess, results));
        self.current_input.clear();

        if !self.won && self.guesses.len() >= MAX_GUESSES {
            self.lost = true;
        }
    }

    fn render_all(&mut self, world: &mut World) {
        self.entities.despawn_all(world);

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let grid_start_row = 2.0;
        let cell_width = 5.0;
        let grid_width = WORD_LENGTH as f64 * cell_width;
        let grid_start_column = center_column - grid_width / 2.0;

        for guess_index in 0..MAX_GUESSES {
            let row = grid_start_row + guess_index as f64 * 2.0;

            if guess_index < self.guesses.len() {
                let (ref word, ref results) = self.guesses[guess_index];
                for (char_index, character) in word.chars().enumerate() {
                    let col = grid_start_column + char_index as f64 * cell_width;
                    let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
                    world.set_position(entity, Position { column: col, row });
                    world.set_label(
                        entity,
                        Label {
                            text: format!(
                                "  {}  ",
                                character.to_uppercase().next().unwrap_or(character)
                            ),
                            foreground: result_foreground(results[char_index]),
                            background: result_background(results[char_index]),
                        },
                    );
                    world.set_z_index(entity, ZIndex(5));
                }
            } else if guess_index == self.guesses.len() {
                for char_index in 0..WORD_LENGTH {
                    let col = grid_start_column + char_index as f64 * cell_width;
                    let character = self.current_input.chars().nth(char_index);
                    let text = match character {
                        Some(letter) => {
                            format!("  {}  ", letter.to_uppercase().next().unwrap_or(letter))
                        }
                        None => "  _  ".to_string(),
                    };
                    let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
                    world.set_position(entity, Position { column: col, row });
                    world.set_label(
                        entity,
                        Label {
                            text,
                            foreground: TermColor::White,
                            background: TermColor::Rgb {
                                r: 50,
                                g: 50,
                                b: 55,
                            },
                        },
                    );
                    world.set_z_index(entity, ZIndex(5));
                }
            } else {
                for char_index in 0..WORD_LENGTH {
                    let col = grid_start_column + char_index as f64 * cell_width;
                    let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
                    world.set_position(entity, Position { column: col, row });
                    world.set_label(
                        entity,
                        Label {
                            text: "  .  ".to_string(),
                            foreground: TermColor::Rgb {
                                r: 60,
                                g: 60,
                                b: 68,
                            },
                            background: TermColor::Rgb {
                                r: 30,
                                g: 30,
                                b: 34,
                            },
                        },
                    );
                    world.set_z_index(entity, ZIndex(5));
                }
            }
        }

        let keyboard_row = grid_start_row + MAX_GUESSES as f64 * 2.0 + 1.0;
        let keyboard_rows = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];

        for (row_index, keys) in keyboard_rows.iter().enumerate() {
            let key_width = 4.0;
            let row_width = keys.len() as f64 * key_width;
            let row_start = center_column - row_width / 2.0;
            for (key_index, key_char) in keys.chars().enumerate() {
                let letter_index = (key_char.to_lowercase().next().unwrap() as u8 - b'a') as usize;
                let state = self.keyboard_states[letter_index];
                let col = row_start + key_index as f64 * key_width;
                let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
                world.set_position(
                    entity,
                    Position {
                        column: col,
                        row: keyboard_row + row_index as f64 * 2.0,
                    },
                );
                world.set_label(
                    entity,
                    Label {
                        text: format!(" {}  ", key_char),
                        foreground: keyboard_state_foreground(state),
                        background: keyboard_state_background(state),
                    },
                );
                world.set_z_index(entity, ZIndex(5));
            }
        }

        if !self.message.is_empty() {
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - self.message.len() as f64 / 2.0,
                    row: grid_start_row + MAX_GUESSES as f64 * 2.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: self.message.clone(),
                    foreground: TermColor::Yellow,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Wordle - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        self.render_all(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        if self.won || self.lost {
            return;
        }

        match key {
            KeyCode::Char(character) if character.is_ascii_alphabetic() => {
                if self.current_input.len() < WORD_LENGTH {
                    self.current_input
                        .push(character.to_lowercase().next().unwrap_or(character));
                    self.render_all(world);
                }
            }
            KeyCode::Backspace => {
                if !self.current_input.is_empty() {
                    self.current_input.pop();
                    self.render_all(world);
                }
            }
            KeyCode::Enter => {
                self.submit_guess();
                self.render_all(world);
            }
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.message_timer > 0.0 {
            self.message_timer -= world.resources.timing.delta_seconds;
            if self.message_timer <= 0.0 {
                self.message.clear();
                self.render_all(world);
            }
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.won || self.lost {
            self.entities.despawn_all(world);
            return Some(Box::new(ResultState {
                won: self.won,
                answer: self.answer.clone(),
                guess_count: self.guesses.len(),
                entities: EntityGroup::new(),
                restart: false,
            }));
        }
        None
    }
}

struct ResultState {
    won: bool,
    answer: String,
    guess_count: usize,
    entities: EntityGroup,
    restart: bool,
}

impl State for ResultState {
    fn title(&self) -> &str {
        "Wordle - Result"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let (result_text, result_color) = if self.won {
            (
                format!(
                    "Solved in {} guess{}!",
                    self.guess_count,
                    if self.guess_count == 1 { "" } else { "es" }
                ),
                TermColor::Rgb {
                    r: 80,
                    g: 200,
                    b: 80,
                },
            )
        } else {
            ("Better luck next time!".to_string(), TermColor::Red)
        };

        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - result_text.len() as f64 / 2.0,
                row: center_row - 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: result_text,
                foreground: result_color,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let answer_text = format!("The word was: {}", self.answer.to_uppercase());
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - answer_text.len() as f64 / 2.0,
                row: center_row - 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: answer_text,
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let prompt = "Press R to play again";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - prompt.len() as f64 / 2.0,
                row: center_row + 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: prompt.to_string(),
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let quit = "ESC to quit";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - quit.len() as f64 / 2.0,
                row: center_row + 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: quit.to_string(),
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
            KeyCode::Char('r') => self.restart = true,
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.restart {
            self.entities.despawn_all(world);
            return Some(Box::new(GameplayState::new()));
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
