use nightshade::tui::prelude::*;
use rand::Rng;

const GRID_SIZE: usize = 4;
const CELL_WIDTH: i32 = 7;
const CELL_HEIGHT: i32 = 3;
const GRID_PIXEL_WIDTH: i32 = GRID_SIZE as i32 * CELL_WIDTH + 1;
const GRID_PIXEL_HEIGHT: i32 = GRID_SIZE as i32 * CELL_HEIGHT + 1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn tile_color(value: u32) -> TermColor {
    match value {
        0 => TermColor::DarkGrey,
        2 => TermColor::White,
        4 => TermColor::Rgb {
            r: 200,
            g: 200,
            b: 150,
        },
        8 => TermColor::Rgb {
            r: 255,
            g: 180,
            b: 100,
        },
        16 => TermColor::Rgb {
            r: 255,
            g: 140,
            b: 80,
        },
        32 => TermColor::Rgb {
            r: 255,
            g: 100,
            b: 60,
        },
        64 => TermColor::Rgb {
            r: 255,
            g: 60,
            b: 40,
        },
        128 => TermColor::Rgb {
            r: 255,
            g: 220,
            b: 100,
        },
        256 => TermColor::Rgb {
            r: 255,
            g: 210,
            b: 80,
        },
        512 => TermColor::Rgb {
            r: 255,
            g: 200,
            b: 60,
        },
        1024 => TermColor::Rgb {
            r: 255,
            g: 190,
            b: 40,
        },
        2048 => TermColor::Rgb {
            r: 255,
            g: 255,
            b: 50,
        },
        _ => TermColor::Rgb {
            r: 255,
            g: 50,
            b: 255,
        },
    }
}

fn tile_background(value: u32) -> TermColor {
    match value {
        0 => TermColor::Black,
        2 => TermColor::Rgb {
            r: 30,
            g: 30,
            b: 30,
        },
        4 => TermColor::Rgb {
            r: 35,
            g: 35,
            b: 25,
        },
        8 => TermColor::Rgb {
            r: 40,
            g: 30,
            b: 15,
        },
        16 => TermColor::Rgb {
            r: 45,
            g: 25,
            b: 10,
        },
        32 => TermColor::Rgb {
            r: 45,
            g: 20,
            b: 10,
        },
        64 => TermColor::Rgb {
            r: 50,
            g: 15,
            b: 10,
        },
        128 => TermColor::Rgb {
            r: 50,
            g: 40,
            b: 15,
        },
        256 => TermColor::Rgb {
            r: 50,
            g: 38,
            b: 12,
        },
        512 => TermColor::Rgb {
            r: 50,
            g: 35,
            b: 10,
        },
        1024 => TermColor::Rgb { r: 50, g: 33, b: 8 },
        2048 => TermColor::Rgb {
            r: 50,
            g: 50,
            b: 10,
        },
        _ => TermColor::Rgb {
            r: 50,
            g: 10,
            b: 50,
        },
    }
}

fn center_value_in_cell(value: u32) -> String {
    let text = format!("{}", value);
    let inner_width = (CELL_WIDTH - 1) as usize;
    let text_len = text.len();
    if text_len >= inner_width {
        return text[..inner_width].to_string();
    }
    let left_padding = (inner_width - text_len) / 2;
    let right_padding = inner_width - text_len - left_padding;
    format!(
        "{}{}{}",
        " ".repeat(left_padding),
        text,
        " ".repeat(right_padding)
    )
}

struct TextLineStyle {
    foreground: TermColor,
    background: TermColor,
    z_index: i32,
}

fn spawn_text_line(
    world: &mut World,
    text: &str,
    start_column: i32,
    row: i32,
    style: &TextLineStyle,
    entities: &mut Vec<Entity>,
) {
    for (char_index, character) in text.chars().enumerate() {
        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
        world.set_position(
            entity,
            Position {
                column: (start_column + char_index as i32) as f64,
                row: row as f64,
            },
        );
        world.set_sprite(
            entity,
            Sprite {
                character,
                foreground: style.foreground,
                background: style.background,
            },
        );
        world.set_z_index(entity, ZIndex(style.z_index));
        entities.push(entity);
    }
}

fn slide_row(row: &mut [u32; GRID_SIZE], score: &mut u32) -> bool {
    let mut changed = false;
    let values: Vec<u32> = row.iter().copied().filter(|&value| value != 0).collect();

    let mut merged = Vec::with_capacity(GRID_SIZE);
    let mut skip_next = false;
    for index in 0..values.len() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if index + 1 < values.len() && values[index] == values[index + 1] {
            let merged_value = values[index] * 2;
            merged.push(merged_value);
            *score += merged_value;
            skip_next = true;
        } else {
            merged.push(values[index]);
        }
    }

    while merged.len() < GRID_SIZE {
        merged.push(0);
    }

    for index in 0..GRID_SIZE {
        if row[index] != merged[index] {
            changed = true;
        }
        row[index] = merged[index];
    }

    changed
}

fn slide_board(
    board: &mut [[u32; GRID_SIZE]; GRID_SIZE],
    direction: Direction,
    score: &mut u32,
) -> bool {
    let mut changed = false;

    match direction {
        Direction::Left => {
            for row in board.iter_mut() {
                if slide_row(row, score) {
                    changed = true;
                }
            }
        }
        Direction::Right => {
            for row in board.iter_mut() {
                row.reverse();
                if slide_row(row, score) {
                    changed = true;
                }
                row.reverse();
            }
        }
        Direction::Up => {
            for column_index in 0..GRID_SIZE {
                let mut column_values = [0u32; GRID_SIZE];
                for (value, row) in column_values.iter_mut().zip(board.iter()) {
                    *value = row[column_index];
                }
                if slide_row(&mut column_values, score) {
                    changed = true;
                }
                for (row, &value) in board.iter_mut().zip(column_values.iter()) {
                    row[column_index] = value;
                }
            }
        }
        Direction::Down => {
            for column_index in 0..GRID_SIZE {
                let mut column_values = [0u32; GRID_SIZE];
                for (value, row) in column_values.iter_mut().zip(board.iter()) {
                    *value = row[column_index];
                }
                column_values.reverse();
                if slide_row(&mut column_values, score) {
                    changed = true;
                }
                column_values.reverse();
                for (row, &value) in board.iter_mut().zip(column_values.iter()) {
                    row[column_index] = value;
                }
            }
        }
    }

    changed
}

fn spawn_random_tile(board: &mut [[u32; GRID_SIZE]; GRID_SIZE]) {
    let mut empty_cells = Vec::new();
    for (row_index, row) in board.iter().enumerate() {
        for (column_index, &value) in row.iter().enumerate() {
            if value == 0 {
                empty_cells.push((row_index, column_index));
            }
        }
    }

    if empty_cells.is_empty() {
        return;
    }

    let mut rng = rand::rng();
    let chosen_index = rng.random_range(0..empty_cells.len());
    let (row_index, column_index) = empty_cells[chosen_index];
    let value = if rng.random_range(0..10) == 0 { 4 } else { 2 };
    board[row_index][column_index] = value;
}

fn has_valid_moves(board: &[[u32; GRID_SIZE]; GRID_SIZE]) -> bool {
    for row in board.iter() {
        for &value in row.iter() {
            if value == 0 {
                return true;
            }
        }
    }

    for row_index in 0..GRID_SIZE {
        for column_index in 0..GRID_SIZE {
            let value = board[row_index][column_index];
            if column_index + 1 < GRID_SIZE && board[row_index][column_index + 1] == value {
                return true;
            }
            if row_index + 1 < GRID_SIZE && board[row_index + 1][column_index] == value {
                return true;
            }
        }
    }

    false
}

fn board_has_2048(board: &[[u32; GRID_SIZE]; GRID_SIZE]) -> bool {
    for row in board.iter() {
        for &value in row.iter() {
            if value >= 2048 {
                return true;
            }
        }
    }
    false
}

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "2048 - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r" ____   ___  _  _    ___  ",
            r"|___ \ / _ \| || |  ( _ ) ",
            r"  __) | | | | || |_ / _ \ ",
            r" / __/| |_| |__   _| (_) |",
            r"|_____|\___/   |_|  \___/ ",
        ];

        let title_start_row = center_row - 7;

        let mut entities = Vec::new();

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
                                g: 200,
                                b: 50,
                            },
                            background: TermColor::Black,
                        },
                    );
                    world.set_z_index(entity, ZIndex(10));
                }
            }
        }

        let subtitle = "The sliding number puzzle";
        spawn_text_line(
            world,
            subtitle,
            center_column - subtitle.len() as i32 / 2,
            title_start_row + 7,
            &TextLineStyle {
                foreground: TermColor::Rgb {
                    r: 180,
                    g: 180,
                    b: 180,
                },
                background: TermColor::Black,
                z_index: 10,
            },
            &mut entities,
        );

        let sample = "  2   4   8  16  32  64 ";
        let sample_start = center_column - sample.len() as i32 / 2;
        let sample_values: Vec<u32> = vec![2, 4, 8, 16, 32, 64];
        for (value_index, &value) in sample_values.iter().enumerate() {
            let text = format!("{:^4}", value);
            let text_start = sample_start + value_index as i32 * 4;
            for (char_index, character) in text.chars().enumerate() {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (text_start + char_index as i32) as f64,
                        row: (title_start_row + 9) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: tile_color(value),
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(10));
            }
        }

        let prompt = "Press ENTER to start";
        spawn_text_line(
            world,
            prompt,
            center_column - prompt.len() as i32 / 2,
            title_start_row + 12,
            &TextLineStyle {
                foreground: TermColor::White,
                background: TermColor::Black,
                z_index: 10,
            },
            &mut entities,
        );

        let controls = "Arrow keys or WASD to slide tiles";
        spawn_text_line(
            world,
            controls,
            center_column - controls.len() as i32 / 2,
            title_start_row + 14,
            &TextLineStyle {
                foreground: TermColor::Grey,
                background: TermColor::Black,
                z_index: 10,
            },
            &mut entities,
        );

        let quit_hint = "Press ESC to quit";
        spawn_text_line(
            world,
            quit_hint,
            center_column - quit_hint.len() as i32 / 2,
            title_start_row + 15,
            &TextLineStyle {
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
                z_index: 10,
            },
            &mut entities,
        );
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
    board: [[u32; GRID_SIZE]; GRID_SIZE],
    score: u32,
    best_score: u32,
    game_over: bool,
    won: bool,
    win_shown: bool,
    show_win_overlay: bool,
    display_entities: Vec<Entity>,
    grid_offset_column: i32,
    grid_offset_row: i32,
    needs_redraw: bool,
}

impl GameplayState {
    fn new() -> Self {
        let mut board = [[0u32; GRID_SIZE]; GRID_SIZE];
        spawn_random_tile(&mut board);
        spawn_random_tile(&mut board);

        Self {
            board,
            score: 0,
            best_score: 0,
            game_over: false,
            won: false,
            win_shown: false,
            show_win_overlay: false,
            display_entities: Vec::new(),
            grid_offset_column: 0,
            grid_offset_row: 0,
            needs_redraw: true,
        }
    }

    fn perform_move(&mut self, direction: Direction) {
        if self.game_over || self.show_win_overlay {
            return;
        }

        let changed = slide_board(&mut self.board, direction, &mut self.score);

        if changed {
            if self.score > self.best_score {
                self.best_score = self.score;
            }

            spawn_random_tile(&mut self.board);

            if !self.won && board_has_2048(&self.board) {
                self.won = true;
                self.show_win_overlay = true;
            }

            if !has_valid_moves(&self.board) {
                self.game_over = true;
            }

            self.needs_redraw = true;
        }
    }

    fn render_display(&mut self, world: &mut World) {
        if !self.needs_redraw {
            return;
        }
        self.needs_redraw = false;

        world.despawn_entities(&self.display_entities);
        self.display_entities.clear();

        let border_color = TermColor::DarkGrey;
        let border_background = TermColor::Black;

        for row_index in 0..=GRID_SIZE {
            let screen_row = self.grid_offset_row + row_index as i32 * CELL_HEIGHT;
            for column_pixel in 0..GRID_PIXEL_WIDTH {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                let character = if column_pixel % CELL_WIDTH == 0 {
                    '+'
                } else {
                    '-'
                };
                world.set_position(
                    entity,
                    Position {
                        column: (self.grid_offset_column + column_pixel) as f64,
                        row: screen_row as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: border_color,
                        background: border_background,
                    },
                );
                world.set_z_index(entity, ZIndex(1));
                self.display_entities.push(entity);
            }
        }

        for row_index in 0..GRID_SIZE {
            for cell_row_offset in 1..CELL_HEIGHT {
                let screen_row =
                    self.grid_offset_row + row_index as i32 * CELL_HEIGHT + cell_row_offset;

                for column_index in 0..=GRID_SIZE {
                    let screen_column = self.grid_offset_column + column_index as i32 * CELL_WIDTH;
                    let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                    world.set_position(
                        entity,
                        Position {
                            column: screen_column as f64,
                            row: screen_row as f64,
                        },
                    );
                    world.set_sprite(
                        entity,
                        Sprite {
                            character: '|',
                            foreground: border_color,
                            background: border_background,
                        },
                    );
                    world.set_z_index(entity, ZIndex(1));
                    self.display_entities.push(entity);
                }
            }
        }

        for row_index in 0..GRID_SIZE {
            let value_screen_row = self.grid_offset_row + row_index as i32 * CELL_HEIGHT + 1;

            for column_index in 0..GRID_SIZE {
                let value = self.board[row_index][column_index];
                let cell_start_column =
                    self.grid_offset_column + column_index as i32 * CELL_WIDTH + 1;

                let foreground = tile_color(value);
                let background = tile_background(value);

                let inner_width = (CELL_WIDTH - 1) as usize;
                let empty_screen_row = self.grid_offset_row + row_index as i32 * CELL_HEIGHT + 2;

                if value == 0 {
                    for pixel_offset in 0..inner_width {
                        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                        world.set_position(
                            entity,
                            Position {
                                column: (cell_start_column + pixel_offset as i32) as f64,
                                row: value_screen_row as f64,
                            },
                        );
                        world.set_sprite(
                            entity,
                            Sprite {
                                character: ' ',
                                foreground: TermColor::Black,
                                background: TermColor::Black,
                            },
                        );
                        world.set_z_index(entity, ZIndex(1));
                        self.display_entities.push(entity);
                    }
                } else {
                    let centered_text = center_value_in_cell(value);
                    for (char_index, character) in centered_text.chars().enumerate() {
                        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                        world.set_position(
                            entity,
                            Position {
                                column: (cell_start_column + char_index as i32) as f64,
                                row: value_screen_row as f64,
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
                        world.set_z_index(entity, ZIndex(1));
                        self.display_entities.push(entity);
                    }
                }

                if CELL_HEIGHT > 2 {
                    for pixel_offset in 0..inner_width {
                        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                        world.set_position(
                            entity,
                            Position {
                                column: (cell_start_column + pixel_offset as i32) as f64,
                                row: empty_screen_row as f64,
                            },
                        );
                        world.set_sprite(
                            entity,
                            Sprite {
                                character: ' ',
                                foreground: TermColor::Black,
                                background: if value == 0 {
                                    TermColor::Black
                                } else {
                                    background
                                },
                            },
                        );
                        world.set_z_index(entity, ZIndex(1));
                        self.display_entities.push(entity);
                    }
                }
            }
        }

        let hud_row = self.grid_offset_row - 2;
        let score_text = format!("Score: {}  Best: {}", self.score, self.best_score);
        spawn_text_line(
            world,
            &score_text,
            self.grid_offset_column,
            hud_row,
            &TextLineStyle {
                foreground: TermColor::White,
                background: TermColor::Black,
                z_index: 2,
            },
            &mut self.display_entities,
        );

        let footer_row = self.grid_offset_row + GRID_PIXEL_HEIGHT + 1;
        let footer_text = "Arrows/WASD: Slide | R: Restart | ESC: Quit";
        spawn_text_line(
            world,
            footer_text,
            self.grid_offset_column,
            footer_row,
            &TextLineStyle {
                foreground: TermColor::Grey,
                background: TermColor::Black,
                z_index: 2,
            },
            &mut self.display_entities,
        );

        if self.show_win_overlay {
            let overlay_row = self.grid_offset_row + GRID_PIXEL_HEIGHT / 2 - 1;
            let win_text = " YOU WIN! Press ENTER to continue ";
            let overlay_start =
                self.grid_offset_column + GRID_PIXEL_WIDTH / 2 - win_text.len() as i32 / 2;
            spawn_text_line(
                world,
                win_text,
                overlay_start,
                overlay_row,
                &TextLineStyle {
                    foreground: TermColor::Rgb {
                        r: 255,
                        g: 255,
                        b: 50,
                    },
                    background: TermColor::Rgb { r: 40, g: 40, b: 0 },
                    z_index: 5,
                },
                &mut self.display_entities,
            );
        }

        if self.game_over {
            let overlay_row = self.grid_offset_row + GRID_PIXEL_HEIGHT / 2 - 1;
            let game_over_text = " GAME OVER! Press R to restart ";
            let overlay_start =
                self.grid_offset_column + GRID_PIXEL_WIDTH / 2 - game_over_text.len() as i32 / 2;
            spawn_text_line(
                world,
                game_over_text,
                overlay_start,
                overlay_row,
                &TextLineStyle {
                    foreground: TermColor::Red,
                    background: TermColor::Rgb {
                        r: 50,
                        g: 10,
                        b: 10,
                    },
                    z_index: 5,
                },
                &mut self.display_entities,
            );
        }
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "2048 - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        self.grid_offset_column = (terminal.columns as i32 - GRID_PIXEL_WIDTH) / 2;
        self.grid_offset_row = (terminal.rows as i32 - GRID_PIXEL_HEIGHT) / 2;
        if self.grid_offset_column < 0 {
            self.grid_offset_column = 0;
        }
        if self.grid_offset_row < 3 {
            self.grid_offset_row = 3;
        }

        self.needs_redraw = true;
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }

        if self.show_win_overlay {
            if key == KeyCode::Enter {
                self.show_win_overlay = false;
                self.win_shown = true;
                self.needs_redraw = true;
            }
            return;
        }

        match key {
            KeyCode::Up | KeyCode::Char('w') => self.perform_move(Direction::Up),
            KeyCode::Down | KeyCode::Char('s') => self.perform_move(Direction::Down),
            KeyCode::Left | KeyCode::Char('a') => self.perform_move(Direction::Left),
            KeyCode::Right | KeyCode::Char('d') => self.perform_move(Direction::Right),
            KeyCode::Char('r') => {
                self.board = [[0u32; GRID_SIZE]; GRID_SIZE];
                self.score = 0;
                self.game_over = false;
                self.won = false;
                self.win_shown = false;
                self.show_win_overlay = false;
                spawn_random_tile(&mut self.board);
                spawn_random_tile(&mut self.board);
                self.needs_redraw = true;
            }
            KeyCode::Escape | KeyCode::Char('q') => {
                world.resources.should_exit = true;
            }
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        self.render_display(world);
    }

    fn next_state(&mut self, _world: &mut World) -> Option<Box<dyn State>> {
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(TitleScreenState { start_game: false }))
}
