use nightshade::tui::prelude::*;

const LANE_COUNT: usize = 4;
const TARGET_ROW: f64 = 20.0;
const SCROLL_SPEED: f64 = 10.0;
const PERFECT_WINDOW: f64 = 0.10;
const GOOD_WINDOW: f64 = 0.25;
const MAX_HP: i32 = 100;
const HP_DRAIN_MISS: i32 = 15;
const HP_RECOVER_PERFECT: i32 = 3;
const HP_RECOVER_GOOD: i32 = 1;

const LANE_KEYS: [KeyCode; LANE_COUNT] = [
    KeyCode::Char('d'),
    KeyCode::Char('f'),
    KeyCode::Char('j'),
    KeyCode::Char('k'),
];

const LANE_LABELS: [char; LANE_COUNT] = ['D', 'F', 'J', 'K'];

fn lane_color(lane: usize) -> TermColor {
    match lane {
        0 => TermColor::Rgb {
            r: 255,
            g: 80,
            b: 80,
        },
        1 => TermColor::Rgb {
            r: 80,
            g: 200,
            b: 80,
        },
        2 => TermColor::Rgb {
            r: 80,
            g: 150,
            b: 255,
        },
        3 => TermColor::Rgb {
            r: 255,
            g: 200,
            b: 50,
        },
        _ => TermColor::White,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HitResult {
    Perfect,
    Good,
    Miss,
}

struct Note {
    lane: usize,
    time: f64,
    hit: bool,
}

struct Song {
    name: String,
    bpm: f64,
    notes: Vec<(f64, usize)>,
}

fn build_songs() -> Vec<Song> {
    vec![
        Song {
            name: "Easy Groove".to_string(),
            bpm: 100.0,
            notes: {
                let beat = 60.0 / 100.0;
                let mut notes = Vec::new();
                for measure in 0..8 {
                    let offset = measure as f64 * 4.0 * beat;
                    notes.push((offset, 0));
                    notes.push((offset + beat, 1));
                    notes.push((offset + 2.0 * beat, 2));
                    notes.push((offset + 3.0 * beat, 3));
                }
                notes
            },
        },
        Song {
            name: "Crossover".to_string(),
            bpm: 120.0,
            notes: {
                let beat = 60.0 / 120.0;
                let mut notes = Vec::new();
                for measure in 0..8 {
                    let offset = measure as f64 * 4.0 * beat;
                    notes.push((offset, 0));
                    notes.push((offset, 3));
                    notes.push((offset + beat, 1));
                    notes.push((offset + beat, 2));
                    notes.push((offset + 2.0 * beat, 0));
                    notes.push((offset + 2.5 * beat, 1));
                    notes.push((offset + 3.0 * beat, 2));
                    notes.push((offset + 3.5 * beat, 3));
                }
                notes
            },
        },
        Song {
            name: "Frenzy".to_string(),
            bpm: 150.0,
            notes: {
                let beat = 60.0 / 150.0;
                let mut notes = Vec::new();
                for measure in 0..10 {
                    let offset = measure as f64 * 4.0 * beat;
                    for sub in 0..8 {
                        let lane = match sub % 4 {
                            0 => 0,
                            1 => 1,
                            2 => 3,
                            3 => 2,
                            _ => 0,
                        };
                        notes.push((offset + sub as f64 * beat * 0.5, lane));
                    }
                }
                notes
            },
        },
    ]
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Rhythm Game - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 60;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "RHYTHM GAME";
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
                foreground: TermColor::Magenta,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let keys_display = "[ D ]  [ F ]  [ J ]  [ K ]";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - keys_display.len() as f64 / 2.0,
                row: center_row - 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: keys_display.to_string(),
                foreground: TermColor::Cyan,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let subtitle = "Hit the notes as they reach the target line";
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
                foreground: TermColor::White,
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
            return Some(Box::new(SongSelectState::new()));
        }
        None
    }
}

struct SongSelectState {
    songs: Vec<Song>,
    menu: Menu,
    entities: EntityGroup,
    selected: Option<usize>,
}

impl SongSelectState {
    fn new() -> Self {
        let songs = build_songs();
        let menu_items: Vec<String> = songs
            .iter()
            .enumerate()
            .map(|(index, song)| {
                format!(
                    "{}. {} ({} BPM, {} notes)",
                    index + 1,
                    song.name,
                    song.bpm as u32,
                    song.notes.len()
                )
            })
            .collect();
        Self {
            menu: Menu::new(menu_items, 0.0, 0.0, MenuColors::default(), 10),
            songs,
            entities: EntityGroup::new(),
            selected: None,
        }
    }
}

impl State for SongSelectState {
    fn title(&self) -> &str {
        "Rhythm Game - Song Select"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "SELECT A SONG";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - title.len() as f64 / 2.0,
                row: center_row - 4.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: title.to_string(),
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        self.menu = Menu::new(
            self.songs
                .iter()
                .enumerate()
                .map(|(index, song)| {
                    format!(
                        "{}. {} ({} BPM, {} notes)",
                        index + 1,
                        song.name,
                        song.bpm as u32,
                        song.notes.len()
                    )
                })
                .collect(),
            center_column - 20.0,
            center_row - 1.0,
            MenuColors::default(),
            10,
        );
        self.menu.render(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Up => {
                self.menu.up();
                self.menu.render(world);
            }
            KeyCode::Down => {
                self.menu.down();
                self.menu.render(world);
            }
            KeyCode::Enter => {
                self.selected = Some(self.menu.selected_index());
            }
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if let Some(index) = self.selected {
            self.entities.despawn_all(world);
            self.menu.despawn(world);
            let song = &self.songs[index];
            return Some(Box::new(GameplayState::new(
                song.name.clone(),
                song.notes.clone(),
            )));
        }
        None
    }
}

struct GameplayState {
    song_name: String,
    notes: Vec<Note>,
    elapsed: f64,
    hp: i32,
    score: u32,
    combo: u32,
    max_combo: u32,
    perfect_count: u32,
    good_count: u32,
    miss_count: u32,
    entities: EntityGroup,
    particles: ParticleEmitter,
    lane_flash: [f64; LANE_COUNT],
    last_hit_result: Option<(HitResult, f64)>,
    finished: bool,
    failed: bool,
    lane_start_column: f64,
}

impl GameplayState {
    fn new(song_name: String, note_data: Vec<(f64, usize)>) -> Self {
        let notes: Vec<Note> = note_data
            .into_iter()
            .map(|(time, lane)| Note {
                lane,
                time: time + 3.0,
                hit: false,
            })
            .collect();
        Self {
            song_name,
            notes,
            elapsed: 0.0,
            hp: MAX_HP,
            score: 0,
            combo: 0,
            max_combo: 0,
            perfect_count: 0,
            good_count: 0,
            miss_count: 0,
            entities: EntityGroup::new(),
            particles: ParticleEmitter::new(),
            lane_flash: [0.0; LANE_COUNT],
            last_hit_result: None,
            finished: false,
            failed: false,
            lane_start_column: 0.0,
        }
    }

    fn try_hit_lane(&mut self, lane: usize, world: &mut World) {
        let mut best_match: Option<(usize, f64)> = None;

        for (note_index, note) in self.notes.iter().enumerate() {
            if note.hit || note.lane != lane {
                continue;
            }
            let difference = (note.time - self.elapsed).abs();
            if difference <= GOOD_WINDOW
                && (best_match.is_none() || difference < best_match.unwrap().1)
            {
                best_match = Some((note_index, difference));
            }
        }

        if let Some((note_index, difference)) = best_match {
            self.notes[note_index].hit = true;
            let result = if difference <= PERFECT_WINDOW {
                HitResult::Perfect
            } else {
                HitResult::Good
            };

            match result {
                HitResult::Perfect => {
                    self.perfect_count += 1;
                    self.combo += 1;
                    let multiplier = (self.combo / 10).min(4) + 1;
                    self.score += 300 * multiplier;
                    self.hp = (self.hp + HP_RECOVER_PERFECT).min(MAX_HP);
                    self.particles.emit(
                        world,
                        self.lane_start_column + lane as f64 * 6.0 + 2.0,
                        TARGET_ROW,
                        8,
                        &ParticleConfig {
                            characters: vec!['*', '+', '.'],
                            colors: vec![lane_color(lane), TermColor::White, TermColor::Yellow],
                            lifetime: 0.4,
                            speed_min: 3.0,
                            speed_max: 8.0,
                            spread: std::f64::consts::PI,
                            direction: -std::f64::consts::FRAC_PI_2,
                            z_index: 15,
                        },
                    );
                }
                HitResult::Good => {
                    self.good_count += 1;
                    self.combo += 1;
                    let multiplier = (self.combo / 10).min(4) + 1;
                    self.score += 100 * multiplier;
                    self.hp = (self.hp + HP_RECOVER_GOOD).min(MAX_HP);
                }
                HitResult::Miss => {}
            }

            if self.combo > self.max_combo {
                self.max_combo = self.combo;
            }
            self.lane_flash[lane] = 0.15;
            self.last_hit_result = Some((result, 0.5));
        }
    }

    fn render_all(&mut self, world: &mut World) {
        self.entities.despawn_all(world);

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let total_lane_width = LANE_COUNT as f64 * 6.0;
        self.lane_start_column = center_column - total_lane_width / 2.0;

        for (lane_index, lane_label) in LANE_LABELS.iter().enumerate() {
            let lane_column = self.lane_start_column + lane_index as f64 * 6.0;

            for row in 0..=(TARGET_ROW as usize + 2) {
                let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
                world.set_position(
                    entity,
                    Position {
                        column: lane_column,
                        row: row as f64,
                    },
                );
                let border_char = if row == TARGET_ROW as usize {
                    "====="
                } else {
                    "  |  "
                };
                let border_color = if row == TARGET_ROW as usize {
                    if self.lane_flash[lane_index] > 0.0 {
                        TermColor::White
                    } else {
                        lane_color(lane_index)
                    }
                } else {
                    TermColor::Rgb {
                        r: 40,
                        g: 40,
                        b: 50,
                    }
                };
                world.set_label(
                    entity,
                    Label {
                        text: border_char.to_string(),
                        foreground: border_color,
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(1));
            }

            let key_row = TARGET_ROW + 2.0;
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: lane_column,
                    row: key_row,
                },
            );
            let key_text = format!("[ {} ]", lane_label);
            let key_color = if self.lane_flash[lane_index] > 0.0 {
                TermColor::White
            } else {
                lane_color(lane_index)
            };
            world.set_label(
                entity,
                Label {
                    text: key_text,
                    foreground: key_color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }

        for note in &self.notes {
            if note.hit {
                continue;
            }
            let time_until = note.time - self.elapsed;
            let note_row = TARGET_ROW - time_until * SCROLL_SPEED;
            if !(0.0..=TARGET_ROW + 1.0).contains(&note_row) {
                continue;
            }

            let lane_column = self.lane_start_column + note.lane as f64 * 6.0;
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: lane_column,
                    row: note_row,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: "[===]".to_string(),
                    foreground: TermColor::Black,
                    background: lane_color(note.lane),
                },
            );
            world.set_z_index(entity, ZIndex(5));
        }

        let hud_column = self.lane_start_column + total_lane_width + 3.0;
        let score_text = format!("Score: {}", self.score);
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: hud_column,
                row: 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: score_text,
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let combo_text = format!("Combo: {}", self.combo);
        let combo_color = if self.combo >= 50 {
            TermColor::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }
        } else if self.combo >= 20 {
            TermColor::Yellow
        } else if self.combo >= 10 {
            TermColor::Green
        } else {
            TermColor::White
        };
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: hud_column,
                row: 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: combo_text,
                foreground: combo_color,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let hp_bar_width = 15;
        let hp_fraction = self.hp as f64 / MAX_HP as f64;
        let filled = (hp_fraction * hp_bar_width as f64).round() as usize;
        let hp_bar: String = format!(
            "{}{}",
            "#".repeat(filled),
            "-".repeat(hp_bar_width - filled)
        );
        let hp_color = if hp_fraction > 0.6 {
            TermColor::Green
        } else if hp_fraction > 0.3 {
            TermColor::Yellow
        } else {
            TermColor::Red
        };
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: hud_column,
                row: 5.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: format!("HP: [{}]", hp_bar),
                foreground: hp_color,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        if let Some((result, timer)) = &self.last_hit_result
            && *timer > 0.0
        {
            let (text, color) = match result {
                HitResult::Perfect => (
                    "PERFECT!",
                    TermColor::Rgb {
                        r: 255,
                        g: 215,
                        b: 0,
                    },
                ),
                HitResult::Good => ("GOOD", TermColor::Green),
                HitResult::Miss => ("MISS", TermColor::Red),
            };
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - text.len() as f64 / 2.0,
                    row: TARGET_ROW - 3.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: text.to_string(),
                    foreground: color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(20));
        }

        let song_label = format!("Playing: {}", self.song_name);
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 1.0,
                row: 0.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: song_label,
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Rhythm Game - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 60;
        self.render_all(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        if self.finished || self.failed {
            return;
        }

        for (lane_index, lane_key) in LANE_KEYS.iter().enumerate() {
            if key == *lane_key {
                self.try_hit_lane(lane_index, world);
                return;
            }
        }

        if key == KeyCode::Escape {
            world.resources.should_exit = true;
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;
        self.elapsed += delta;
        self.particles.update(world, delta);

        for lane_index in 0..LANE_COUNT {
            if self.lane_flash[lane_index] > 0.0 {
                self.lane_flash[lane_index] -= delta;
            }
        }

        if let Some((_, ref mut timer)) = self.last_hit_result {
            *timer -= delta;
        }

        for note in &mut self.notes {
            if note.hit {
                continue;
            }
            if self.elapsed - note.time > GOOD_WINDOW {
                note.hit = true;
                self.miss_count += 1;
                self.combo = 0;
                self.hp -= HP_DRAIN_MISS;
                self.last_hit_result = Some((HitResult::Miss, 0.5));
            }
        }

        if self.hp <= 0 {
            self.failed = true;
        }

        let all_done = self.notes.iter().all(|note| note.hit);
        if all_done && !self.failed {
            self.finished = true;
        }

        self.render_all(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.finished || self.failed {
            self.entities.despawn_all(world);
            self.particles.despawn_all(world);
            return Some(Box::new(ResultState {
                song_name: self.song_name.clone(),
                score: self.score,
                max_combo: self.max_combo,
                perfect_count: self.perfect_count,
                good_count: self.good_count,
                miss_count: self.miss_count,
                cleared: !self.failed,
                entities: EntityGroup::new(),
                restart: false,
            }));
        }
        None
    }
}

struct ResultState {
    song_name: String,
    score: u32,
    max_combo: u32,
    perfect_count: u32,
    good_count: u32,
    miss_count: u32,
    cleared: bool,
    entities: EntityGroup,
    restart: bool,
}

impl State for ResultState {
    fn title(&self) -> &str {
        "Rhythm Game - Results"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let (result_text, result_color) = if self.cleared {
            (
                "SONG CLEARED!",
                TermColor::Rgb {
                    r: 80,
                    g: 255,
                    b: 80,
                },
            )
        } else {
            ("STAGE FAILED", TermColor::Red)
        };

        let total_notes = self.perfect_count + self.good_count + self.miss_count;
        let accuracy = if total_notes > 0 {
            ((self.perfect_count as f64 * 100.0 + self.good_count as f64 * 50.0)
                / (total_notes as f64 * 100.0))
                * 100.0
        } else {
            0.0
        };

        let grade = if accuracy >= 95.0 {
            "S"
        } else if accuracy >= 90.0 {
            "A"
        } else if accuracy >= 80.0 {
            "B"
        } else if accuracy >= 70.0 {
            "C"
        } else {
            "D"
        };

        let grade_color = match grade {
            "S" => TermColor::Rgb {
                r: 255,
                g: 215,
                b: 0,
            },
            "A" => TermColor::Green,
            "B" => TermColor::Cyan,
            "C" => TermColor::Yellow,
            _ => TermColor::Red,
        };

        let lines: Vec<(String, TermColor)> = vec![
            (result_text.to_string(), result_color),
            (format!("Song: {}", self.song_name), TermColor::White),
            (String::new(), TermColor::Black),
            (format!("Score:    {}", self.score), TermColor::White),
            (format!("Max Combo: {}", self.max_combo), TermColor::Yellow),
            (format!("Accuracy: {:.1}%", accuracy), TermColor::Cyan),
            (format!("Grade:    {}", grade), grade_color),
            (String::new(), TermColor::Black),
            (
                format!("Perfect: {}", self.perfect_count),
                TermColor::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                },
            ),
            (format!("Good:    {}", self.good_count), TermColor::Green),
            (format!("Miss:    {}", self.miss_count), TermColor::Red),
            (String::new(), TermColor::Black),
            (
                "Press R to retry | ESC to quit".to_string(),
                TermColor::Grey,
            ),
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
                    row: center_row - 7.0 + line_index as f64,
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
            return Some(Box::new(SongSelectState::new()));
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
