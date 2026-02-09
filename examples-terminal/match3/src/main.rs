use nightshade::tui::prelude::*;
use rand::Rng;

const GRID_WIDTH: usize = 8;
const GRID_HEIGHT: usize = 8;
const CELL_RENDER_WIDTH: i32 = 3;
const GEM_TYPE_COUNT: usize = 6;
const GAME_DURATION: f64 = 90.0;
const FALL_SPEED: f64 = 12.0;
const SWAP_SPEED: f64 = 10.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum GemType {
    Diamond,
    Circle,
    Square,
    Triangle,
    Star,
    Heart,
}

impl GemType {
    fn character(self) -> char {
        match self {
            Self::Diamond => '◆',
            Self::Circle => '●',
            Self::Square => '■',
            Self::Triangle => '▲',
            Self::Star => '★',
            Self::Heart => '♥',
        }
    }

    fn foreground(self) -> TermColor {
        match self {
            Self::Diamond => TermColor::Rgb {
                r: 100,
                g: 200,
                b: 255,
            },
            Self::Circle => TermColor::Rgb {
                r: 255,
                g: 80,
                b: 80,
            },
            Self::Square => TermColor::Rgb {
                r: 80,
                g: 255,
                b: 80,
            },
            Self::Triangle => TermColor::Rgb {
                r: 255,
                g: 200,
                b: 50,
            },
            Self::Star => TermColor::Rgb {
                r: 255,
                g: 150,
                b: 255,
            },
            Self::Heart => TermColor::Rgb {
                r: 255,
                g: 100,
                b: 150,
            },
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Diamond,
            1 => Self::Circle,
            2 => Self::Square,
            3 => Self::Triangle,
            4 => Self::Star,
            _ => Self::Heart,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AnimationPhase {
    Idle,
    Swapping,
    Removing,
    Falling,
}

struct TitleScreenState {
    start_game: bool,
    entities: EntityGroup,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Match-3 - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r" __  __       _       _       _____ ",
            r"|  \/  | __ _| |_ ___| |__   |___ / ",
            r"| |\/| |/ _` | __/ __| '_ \    |_ \ ",
            r"| |  | | (_| | || (__| | | |  ___) |",
            r"|_|  |_|\__,_|\__\___|_| |_| |____/ ",
        ];

        let title_start_row = center_row - 8;

        for (line_index, line) in title_lines.iter().enumerate() {
            let start_column = center_column - line.len() as i32 / 2;
            for (char_index, character) in line.chars().enumerate() {
                if character != ' ' {
                    let entity = EntityBuilder::new()
                        .position(Position {
                            column: (start_column + char_index as i32) as f64,
                            row: (title_start_row + line_index as i32) as f64,
                        })
                        .sprite(Sprite {
                            character,
                            foreground: TermColor::Rgb {
                                r: 255,
                                g: 180,
                                b: 50,
                            },
                            background: TermColor::Black,
                        })
                        .z_index(ZIndex(10))
                        .spawn(world);
                    self.entities.add(entity);
                }
            }
        }

        let gem_preview_row = title_start_row + 7;
        let gem_types = [
            GemType::Diamond,
            GemType::Circle,
            GemType::Square,
            GemType::Triangle,
            GemType::Star,
            GemType::Heart,
        ];
        let preview_width = gem_types.len() as i32 * 3;
        let preview_start = center_column - preview_width / 2;

        for (gem_index, gem_type) in gem_types.iter().enumerate() {
            let entity = EntityBuilder::new()
                .position(Position {
                    column: (preview_start + gem_index as i32 * 3 + 1) as f64,
                    row: gem_preview_row as f64,
                })
                .sprite(Sprite {
                    character: gem_type.character(),
                    foreground: gem_type.foreground(),
                    background: TermColor::Black,
                })
                .z_index(ZIndex(10))
                .spawn(world);
            self.entities.add(entity);
        }

        let subtitle = "Match 3 or more gems to score!";
        let subtitle_entity = EntityBuilder::new()
            .position(Position {
                column: (center_column - subtitle.len() as i32 / 2) as f64,
                row: (gem_preview_row + 2) as f64,
            })
            .label(Label {
                text: subtitle.to_string(),
                foreground: TermColor::Rgb {
                    r: 180,
                    g: 180,
                    b: 200,
                },
                background: TermColor::Black,
            })
            .z_index(ZIndex(10))
            .spawn(world);
        self.entities.add(subtitle_entity);

        let prompt = "Press ENTER to start";
        let prompt_entity = EntityBuilder::new()
            .position(Position {
                column: (center_column - prompt.len() as i32 / 2) as f64,
                row: (gem_preview_row + 5) as f64,
            })
            .label(Label {
                text: prompt.to_string(),
                foreground: TermColor::White,
                background: TermColor::Black,
            })
            .z_index(ZIndex(10))
            .spawn(world);
        self.entities.add(prompt_entity);

        let controls = "Use mouse to select and swap gems";
        let controls_entity = EntityBuilder::new()
            .position(Position {
                column: (center_column - controls.len() as i32 / 2) as f64,
                row: (gem_preview_row + 7) as f64,
            })
            .label(Label {
                text: controls.to_string(),
                foreground: TermColor::Grey,
                background: TermColor::Black,
            })
            .z_index(ZIndex(10))
            .spawn(world);
        self.entities.add(controls_entity);

        let quit_hint = "Press ESC to quit";
        let quit_entity = EntityBuilder::new()
            .position(Position {
                column: (center_column - quit_hint.len() as i32 / 2) as f64,
                row: (gem_preview_row + 8) as f64,
            })
            .label(Label {
                text: quit_hint.to_string(),
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            })
            .z_index(ZIndex(10))
            .spawn(world);
        self.entities.add(quit_entity);
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
            self.entities.despawn_all(world);
            return Some(Box::new(GameplayState::new()));
        }
        None
    }
}

struct GameplayState {
    grid: [[Option<GemType>; GRID_WIDTH]; GRID_HEIGHT],
    board_offset_column: i32,
    board_offset_row: i32,
    selected: Option<(usize, usize)>,
    score: u32,
    combo_multiplier: u32,
    time_remaining: f64,
    phase: AnimationPhase,
    animation_timer: f64,
    swap_source: Option<(usize, usize)>,
    swap_target: Option<(usize, usize)>,
    swap_back: bool,
    matched_cells: Vec<(usize, usize)>,
    display_entities: EntityGroup,
    hud_entities: EntityGroup,
    particle_emitter: ParticleEmitter,
    time_bar: Option<ProgressBar>,
    game_over: bool,
    needs_redraw: bool,
}

impl GameplayState {
    fn new() -> Self {
        let mut state = Self {
            grid: [[None; GRID_WIDTH]; GRID_HEIGHT],
            board_offset_column: 0,
            board_offset_row: 0,
            selected: None,
            score: 0,
            combo_multiplier: 0,
            time_remaining: GAME_DURATION,
            phase: AnimationPhase::Idle,
            animation_timer: 0.0,
            swap_source: None,
            swap_target: None,
            swap_back: false,
            matched_cells: Vec::new(),
            display_entities: EntityGroup::new(),
            hud_entities: EntityGroup::new(),
            particle_emitter: ParticleEmitter::new(),
            time_bar: None,
            game_over: false,
            needs_redraw: true,
        };
        state.fill_grid_initial();
        state
    }

    fn fill_grid_initial(&mut self) {
        let mut rng = rand::rng();

        for row in 0..GRID_HEIGHT {
            for column in 0..GRID_WIDTH {
                loop {
                    let gem_type = GemType::from_index(rng.random_range(0..GEM_TYPE_COUNT));
                    self.grid[row][column] = Some(gem_type);

                    if self.would_match_at(column, row) {
                        continue;
                    }
                    break;
                }
            }
        }
    }

    fn would_match_at(&self, column: usize, row: usize) -> bool {
        let Some(gem) = self.grid[row][column] else {
            return false;
        };

        if column >= 2
            && self.grid[row][column - 1] == Some(gem)
            && self.grid[row][column - 2] == Some(gem)
        {
            return true;
        }

        if row >= 2
            && self.grid[row - 1][column] == Some(gem)
            && self.grid[row - 2][column] == Some(gem)
        {
            return true;
        }

        false
    }

    fn is_adjacent(column_a: usize, row_a: usize, column_b: usize, row_b: usize) -> bool {
        let distance_column = (column_a as i32 - column_b as i32).abs();
        let distance_row = (row_a as i32 - row_b as i32).abs();
        (distance_column == 1 && distance_row == 0) || (distance_column == 0 && distance_row == 1)
    }

    fn swap_gems(&mut self, column_a: usize, row_a: usize, column_b: usize, row_b: usize) {
        let temp = self.grid[row_a][column_a];
        self.grid[row_a][column_a] = self.grid[row_b][column_b];
        self.grid[row_b][column_b] = temp;
    }

    fn find_matches(&self) -> Vec<(usize, usize)> {
        let mut matched = [false; GRID_WIDTH * GRID_HEIGHT];

        for row in 0..GRID_HEIGHT {
            let mut run_start = 0;
            for column in 1..=GRID_WIDTH {
                let current = if column < GRID_WIDTH {
                    self.grid[row][column]
                } else {
                    None
                };
                let run_gem = self.grid[row][run_start];

                if current.is_some() && current == run_gem {
                    continue;
                }

                let run_length = column - run_start;
                if run_length >= 3 && run_gem.is_some() {
                    for matched_column in run_start..column {
                        matched[row * GRID_WIDTH + matched_column] = true;
                    }
                }
                run_start = column;
            }
        }

        for column in 0..GRID_WIDTH {
            let mut run_start = 0;
            for row in 1..=GRID_HEIGHT {
                let current = if row < GRID_HEIGHT {
                    self.grid[row][column]
                } else {
                    None
                };
                let run_gem = self.grid[run_start][column];

                if current.is_some() && current == run_gem {
                    continue;
                }

                let run_length = row - run_start;
                if run_length >= 3 && run_gem.is_some() {
                    for matched_row in run_start..row {
                        matched[matched_row * GRID_WIDTH + column] = true;
                    }
                }
                run_start = row;
            }
        }

        let mut result = Vec::new();
        for row in 0..GRID_HEIGHT {
            for column in 0..GRID_WIDTH {
                if matched[row * GRID_WIDTH + column] {
                    result.push((column, row));
                }
            }
        }
        result
    }

    fn remove_matched(&mut self) {
        for &(column, row) in &self.matched_cells {
            self.grid[row][column] = None;
        }
    }

    fn apply_gravity(&mut self) -> bool {
        let mut fell = false;

        for column in 0..GRID_WIDTH {
            let mut write_row = GRID_HEIGHT;
            for read_row in (0..GRID_HEIGHT).rev() {
                if self.grid[read_row][column].is_some() {
                    write_row -= 1;
                    if write_row != read_row {
                        self.grid[write_row][column] = self.grid[read_row][column];
                        self.grid[read_row][column] = None;
                        fell = true;
                    }
                }
            }
        }

        fell
    }

    fn fill_empty_from_top(&mut self) {
        let mut rng = rand::rng();

        for column in 0..GRID_WIDTH {
            for row in 0..GRID_HEIGHT {
                if self.grid[row][column].is_none() {
                    self.grid[row][column] =
                        Some(GemType::from_index(rng.random_range(0..GEM_TYPE_COUNT)));
                }
            }
        }
    }

    fn screen_column_for_gem(&self, grid_column: usize) -> f64 {
        (self.board_offset_column + grid_column as i32 * CELL_RENDER_WIDTH + 1) as f64
    }

    fn screen_row_for_gem(&self, grid_row: usize) -> f64 {
        (self.board_offset_row + grid_row as i32) as f64
    }

    fn grid_from_screen(&self, screen_column: u16, screen_row: u16) -> Option<(usize, usize)> {
        let relative_column = screen_column as i32 - self.board_offset_column;
        let relative_row = screen_row as i32 - self.board_offset_row;

        if relative_column < 0
            || relative_row < 0
            || relative_column >= GRID_WIDTH as i32 * CELL_RENDER_WIDTH
            || relative_row >= GRID_HEIGHT as i32
        {
            return None;
        }

        let grid_column = relative_column / CELL_RENDER_WIDTH;
        let grid_row = relative_row;

        if grid_column >= 0
            && grid_column < GRID_WIDTH as i32
            && grid_row >= 0
            && grid_row < GRID_HEIGHT as i32
        {
            Some((grid_column as usize, grid_row as usize))
        } else {
            None
        }
    }

    fn render_board(&mut self, world: &mut World) {
        self.display_entities.despawn_all(world);

        let border_color = TermColor::Rgb {
            r: 60,
            g: 60,
            b: 80,
        };

        for row in 0..GRID_HEIGHT {
            for column in 0..GRID_WIDTH {
                let cell_screen_column =
                    self.board_offset_column + column as i32 * CELL_RENDER_WIDTH;
                let cell_screen_row = self.board_offset_row + row as i32;

                let is_selected = self.selected == Some((column, row));

                let background = if is_selected {
                    TermColor::Rgb {
                        r: 60,
                        g: 60,
                        b: 120,
                    }
                } else if (row + column) % 2 == 0 {
                    TermColor::Rgb {
                        r: 20,
                        g: 20,
                        b: 30,
                    }
                } else {
                    TermColor::Rgb {
                        r: 28,
                        g: 28,
                        b: 40,
                    }
                };

                for offset in 0..CELL_RENDER_WIDTH {
                    let entity = self
                        .display_entities
                        .spawn_one(world, POSITION | SPRITE | Z_INDEX);
                    world.set_position(
                        entity,
                        Position {
                            column: (cell_screen_column + offset) as f64,
                            row: cell_screen_row as f64,
                        },
                    );

                    let (character, foreground) = if offset == 1 {
                        if let Some(gem) = self.grid[row][column] {
                            (gem.character(), gem.foreground())
                        } else {
                            (' ', TermColor::Black)
                        }
                    } else {
                        (' ', TermColor::Black)
                    };

                    world.set_sprite(
                        entity,
                        Sprite {
                            character,
                            foreground,
                            background,
                        },
                    );
                    world.set_z_index(entity, ZIndex(1));
                }
            }
        }

        let top_row = self.board_offset_row - 1;
        let bottom_row = self.board_offset_row + GRID_HEIGHT as i32;
        let left_column = self.board_offset_column - 1;
        let right_column = self.board_offset_column + GRID_WIDTH as i32 * CELL_RENDER_WIDTH;

        for column_offset in 0..=(GRID_WIDTH as i32 * CELL_RENDER_WIDTH + 1) {
            let screen_column = left_column + column_offset;

            let top_entity = self
                .display_entities
                .spawn_one(world, POSITION | SPRITE | Z_INDEX);
            world.set_position(
                top_entity,
                Position {
                    column: screen_column as f64,
                    row: top_row as f64,
                },
            );
            let border_char = if column_offset == 0 {
                '┌'
            } else if column_offset == GRID_WIDTH as i32 * CELL_RENDER_WIDTH + 1 {
                '┐'
            } else {
                '─'
            };
            world.set_sprite(
                top_entity,
                Sprite {
                    character: border_char,
                    foreground: border_color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(top_entity, ZIndex(2));

            let bottom_entity = self
                .display_entities
                .spawn_one(world, POSITION | SPRITE | Z_INDEX);
            world.set_position(
                bottom_entity,
                Position {
                    column: screen_column as f64,
                    row: bottom_row as f64,
                },
            );
            let border_char = if column_offset == 0 {
                '└'
            } else if column_offset == GRID_WIDTH as i32 * CELL_RENDER_WIDTH + 1 {
                '┘'
            } else {
                '─'
            };
            world.set_sprite(
                bottom_entity,
                Sprite {
                    character: border_char,
                    foreground: border_color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(bottom_entity, ZIndex(2));
        }

        for row_offset in 0..GRID_HEIGHT as i32 {
            let screen_row = self.board_offset_row + row_offset;

            let left_entity = self
                .display_entities
                .spawn_one(world, POSITION | SPRITE | Z_INDEX);
            world.set_position(
                left_entity,
                Position {
                    column: left_column as f64,
                    row: screen_row as f64,
                },
            );
            world.set_sprite(
                left_entity,
                Sprite {
                    character: '│',
                    foreground: border_color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(left_entity, ZIndex(2));

            let right_entity = self
                .display_entities
                .spawn_one(world, POSITION | SPRITE | Z_INDEX);
            world.set_position(
                right_entity,
                Position {
                    column: right_column as f64,
                    row: screen_row as f64,
                },
            );
            world.set_sprite(
                right_entity,
                Sprite {
                    character: '│',
                    foreground: border_color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(right_entity, ZIndex(2));
        }
    }

    fn render_hud(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);

        let hud_column = self.board_offset_column;
        let hud_row = self.board_offset_row - 3;

        let score_text = format!("Score: {}", self.score);
        let score_entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            score_entity,
            Position {
                column: hud_column as f64,
                row: hud_row as f64,
            },
        );
        world.set_label(
            score_entity,
            Label {
                text: score_text,
                foreground: TermColor::Rgb {
                    r: 255,
                    g: 255,
                    b: 100,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(score_entity, ZIndex(5));

        let time_text = format!("Time: {:.0}s", self.time_remaining.max(0.0));
        let time_color = if self.time_remaining < 10.0 {
            TermColor::Red
        } else if self.time_remaining < 30.0 {
            TermColor::Rgb {
                r: 255,
                g: 200,
                b: 50,
            }
        } else {
            TermColor::White
        };
        let time_column = self.board_offset_column + GRID_WIDTH as i32 * CELL_RENDER_WIDTH
            - time_text.len() as i32;
        let time_entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            time_entity,
            Position {
                column: time_column as f64,
                row: hud_row as f64,
            },
        );
        world.set_label(
            time_entity,
            Label {
                text: time_text,
                foreground: time_color,
                background: TermColor::Black,
            },
        );
        world.set_z_index(time_entity, ZIndex(5));

        if let Some(time_bar) = &mut self.time_bar {
            let fraction = self.time_remaining / GAME_DURATION;
            time_bar.render(world, fraction);
        }

        if self.combo_multiplier > 1 {
            let combo_text = format!("{}x COMBO!", self.combo_multiplier);
            let combo_column = self.board_offset_column
                + (GRID_WIDTH as i32 * CELL_RENDER_WIDTH) / 2
                - combo_text.len() as i32 / 2;
            let combo_entity = self
                .hud_entities
                .spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                combo_entity,
                Position {
                    column: combo_column as f64,
                    row: (self.board_offset_row + GRID_HEIGHT as i32 + 2) as f64,
                },
            );
            world.set_label(
                combo_entity,
                Label {
                    text: combo_text,
                    foreground: TermColor::Rgb {
                        r: 255,
                        g: 100,
                        b: 255,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(combo_entity, ZIndex(5));
        }

        let controls = "Click to select | ESC to quit";
        let controls_column = self.board_offset_column
            + (GRID_WIDTH as i32 * CELL_RENDER_WIDTH) / 2
            - controls.len() as i32 / 2;
        let controls_entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            controls_entity,
            Position {
                column: controls_column as f64,
                row: (self.board_offset_row + GRID_HEIGHT as i32 + 4) as f64,
            },
        );
        world.set_label(
            controls_entity,
            Label {
                text: controls.to_string(),
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(controls_entity, ZIndex(5));
    }

    fn spawn_match_particles(&mut self, world: &mut World) {
        let config = ParticleConfig {
            characters: vec!['·', '∗', '✦', '✧'],
            lifetime: 0.6,
            speed_min: 2.0,
            speed_max: 6.0,
            spread: std::f64::consts::PI * 2.0,
            direction: 0.0,
            z_index: 15,
            ..Default::default()
        };

        for &(column, row) in &self.matched_cells {
            if let Some(gem) = self.grid[row][column] {
                let particle_config = ParticleConfig {
                    colors: vec![gem.foreground(), TermColor::White],
                    ..config.clone()
                };
                self.particle_emitter.emit(
                    world,
                    self.screen_column_for_gem(column),
                    self.screen_row_for_gem(row),
                    4,
                    &particle_config,
                );
            }
        }
    }

    fn try_start_swap(
        &mut self,
        source_column: usize,
        source_row: usize,
        target_column: usize,
        target_row: usize,
    ) {
        if self.phase != AnimationPhase::Idle {
            return;
        }

        self.swap_source = Some((source_column, source_row));
        self.swap_target = Some((target_column, target_row));
        self.swap_back = false;
        self.phase = AnimationPhase::Swapping;
        self.animation_timer = 0.0;

        self.swap_gems(source_column, source_row, target_column, target_row);

        let matches = self.find_matches();
        if matches.is_empty() {
            self.swap_back = true;
        }
    }

    fn process_chain(&mut self, world: &mut World) {
        let matches = self.find_matches();
        if matches.is_empty() {
            self.combo_multiplier = 0;
            self.phase = AnimationPhase::Idle;
            self.needs_redraw = true;
            return;
        }

        self.combo_multiplier += 1;
        self.matched_cells = matches;

        let match_count = self.matched_cells.len() as u32;
        let base_score = match_count * 10;
        let combo_bonus = base_score * self.combo_multiplier;
        self.score += combo_bonus;

        self.spawn_match_particles(world);
        self.remove_matched();
        self.phase = AnimationPhase::Removing;
        self.animation_timer = 0.0;
        self.needs_redraw = true;
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Match-3 - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let board_pixel_width = GRID_WIDTH as i32 * CELL_RENDER_WIDTH;
        let board_pixel_height = GRID_HEIGHT as i32;

        self.board_offset_column = (terminal.columns as i32 - board_pixel_width) / 2;
        self.board_offset_row = (terminal.rows as i32 - board_pixel_height) / 2;
        if self.board_offset_column < 2 {
            self.board_offset_column = 2;
        }
        if self.board_offset_row < 5 {
            self.board_offset_row = 5;
        }

        let bar_row = self.board_offset_row - 2;
        let bar_width = board_pixel_width as usize;
        self.time_bar = Some(ProgressBar::new(
            bar_width,
            self.board_offset_column as f64,
            bar_row as f64,
            ProgressBarColors {
                filled_foreground: TermColor::Rgb {
                    r: 50,
                    g: 200,
                    b: 50,
                },
                filled_background: TermColor::Rgb {
                    r: 10,
                    g: 40,
                    b: 10,
                },
                empty_foreground: TermColor::Rgb {
                    r: 40,
                    g: 40,
                    b: 40,
                },
                empty_background: TermColor::Rgb {
                    r: 10,
                    g: 10,
                    b: 10,
                },
            },
            5,
        ));

        self.needs_redraw = true;
    }

    fn on_mouse_input(
        &mut self,
        _world: &mut World,
        button: MouseButton,
        column: u16,
        row: u16,
        pressed: bool,
    ) {
        if !pressed || button != MouseButton::Left || self.game_over {
            return;
        }

        if self.phase != AnimationPhase::Idle {
            return;
        }

        if let Some((grid_column, grid_row)) = self.grid_from_screen(column, row) {
            if self.grid[grid_row][grid_column].is_none() {
                return;
            }

            if let Some((selected_column, selected_row)) = self.selected {
                if selected_column == grid_column && selected_row == grid_row {
                    self.selected = None;
                    self.needs_redraw = true;
                } else if Self::is_adjacent(selected_column, selected_row, grid_column, grid_row) {
                    self.selected = None;
                    self.try_start_swap(selected_column, selected_row, grid_column, grid_row);
                } else {
                    self.selected = Some((grid_column, grid_row));
                    self.needs_redraw = true;
                }
            } else {
                self.selected = Some((grid_column, grid_row));
                self.needs_redraw = true;
            }
        } else {
            self.selected = None;
            self.needs_redraw = true;
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Escape | KeyCode::Char('q') => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;

        if !self.game_over {
            self.time_remaining -= delta;
            if self.time_remaining <= 0.0 {
                self.time_remaining = 0.0;
                self.game_over = true;
                self.needs_redraw = true;
            }
        }

        self.particle_emitter.update(world, delta);

        match self.phase {
            AnimationPhase::Swapping => {
                self.animation_timer += delta * SWAP_SPEED;
                if self.animation_timer >= 1.0 {
                    if self.swap_back {
                        if let (
                            Some((source_column, source_row)),
                            Some((target_column, target_row)),
                        ) = (self.swap_source, self.swap_target)
                        {
                            self.swap_gems(source_column, source_row, target_column, target_row);
                        }
                        self.phase = AnimationPhase::Idle;
                        self.swap_source = None;
                        self.swap_target = None;
                    } else {
                        self.swap_source = None;
                        self.swap_target = None;
                        self.process_chain(world);
                    }
                    self.needs_redraw = true;
                }
            }
            AnimationPhase::Removing => {
                self.animation_timer += delta * SWAP_SPEED;
                if self.animation_timer >= 1.0 {
                    self.matched_cells.clear();
                    self.apply_gravity();
                    self.phase = AnimationPhase::Falling;
                    self.animation_timer = 0.0;
                    self.needs_redraw = true;
                }
            }
            AnimationPhase::Falling => {
                self.animation_timer += delta * FALL_SPEED;
                if self.animation_timer >= 1.0 {
                    self.fill_empty_from_top();
                    self.needs_redraw = true;

                    let matches = self.find_matches();
                    if matches.is_empty() {
                        self.combo_multiplier = 0;
                        self.phase = AnimationPhase::Idle;
                    } else {
                        self.process_chain(world);
                    }
                }
            }
            AnimationPhase::Idle => {}
        }

        if self.needs_redraw {
            self.render_board(world);
            self.needs_redraw = false;
        }

        self.render_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over && self.phase == AnimationPhase::Idle {
            self.display_entities.despawn_all(world);
            self.hud_entities.despawn_all(world);
            self.particle_emitter.despawn_all(world);
            if let Some(bar) = &mut self.time_bar {
                bar.despawn(world);
            }
            return Some(Box::new(GameOverState {
                final_score: self.score,
                restart: false,
                entities: EntityGroup::new(),
            }));
        }
        None
    }
}

struct GameOverState {
    final_score: u32,
    restart: bool,
    entities: EntityGroup,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Match-3 - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let game_over_text = "TIME'S UP!";
        let game_over_entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            game_over_entity,
            Position {
                column: (center_column - game_over_text.len() as i32 / 2) as f64,
                row: (center_row - 4) as f64,
            },
        );
        world.set_label(
            game_over_entity,
            Label {
                text: game_over_text.to_string(),
                foreground: TermColor::Rgb {
                    r: 255,
                    g: 100,
                    b: 100,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(game_over_entity, ZIndex(10));

        let score_text = format!("Final Score: {}", self.final_score);
        let score_entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            score_entity,
            Position {
                column: (center_column - score_text.len() as i32 / 2) as f64,
                row: (center_row - 1) as f64,
            },
        );
        world.set_label(
            score_entity,
            Label {
                text: score_text,
                foreground: TermColor::Rgb {
                    r: 255,
                    g: 255,
                    b: 100,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(score_entity, ZIndex(10));

        let rating = if self.final_score >= 5000 {
            "LEGENDARY!"
        } else if self.final_score >= 3000 {
            "Amazing!"
        } else if self.final_score >= 1500 {
            "Great job!"
        } else if self.final_score >= 500 {
            "Not bad!"
        } else {
            "Keep trying!"
        };

        let rating_entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            rating_entity,
            Position {
                column: (center_column - rating.len() as i32 / 2) as f64,
                row: (center_row + 1) as f64,
            },
        );
        world.set_label(
            rating_entity,
            Label {
                text: rating.to_string(),
                foreground: TermColor::Rgb {
                    r: 200,
                    g: 200,
                    b: 255,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(rating_entity, ZIndex(10));

        let restart_text = "Press R to play again";
        let restart_entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            restart_entity,
            Position {
                column: (center_column - restart_text.len() as i32 / 2) as f64,
                row: (center_row + 4) as f64,
            },
        );
        world.set_label(
            restart_entity,
            Label {
                text: restart_text.to_string(),
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(restart_entity, ZIndex(10));

        let quit_text = "Press ESC to quit";
        let quit_entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            quit_entity,
            Position {
                column: (center_column - quit_text.len() as i32 / 2) as f64,
                row: (center_row + 6) as f64,
            },
        );
        world.set_label(
            quit_entity,
            Label {
                text: quit_text.to_string(),
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(quit_entity, ZIndex(10));
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
            self.entities.despawn_all(world);
            return Some(Box::new(GameplayState::new()));
        }
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(TitleScreenState {
        start_game: false,
        entities: EntityGroup::new(),
    }))
}
