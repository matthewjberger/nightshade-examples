use nightshade::tui::prelude::*;
use rand::Rng;

const BOARD_WIDTH: usize = 16;
const BOARD_HEIGHT: usize = 16;
const MINE_COUNT: usize = 40;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellState {
    Hidden,
    Revealed,
    Flagged,
}

fn number_color(count: u8) -> TermColor {
    match count {
        1 => TermColor::Rgb {
            r: 50,
            g: 100,
            b: 255,
        },
        2 => TermColor::Rgb {
            r: 50,
            g: 200,
            b: 50,
        },
        3 => TermColor::Rgb {
            r: 255,
            g: 50,
            b: 50,
        },
        4 => TermColor::Rgb {
            r: 100,
            g: 50,
            b: 200,
        },
        5 => TermColor::Rgb {
            r: 180,
            g: 50,
            b: 50,
        },
        6 => TermColor::Rgb {
            r: 50,
            g: 200,
            b: 200,
        },
        7 => TermColor::Rgb {
            r: 80,
            g: 80,
            b: 80,
        },
        8 => TermColor::Rgb {
            r: 150,
            g: 150,
            b: 150,
        },
        _ => TermColor::White,
    }
}

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Minesweeper - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r" __  __ _                                                ",
            r"|  \/  (_)_ __   ___  _____      _____  ___ _ __   ___ _ __",
            r"| |\/| | | '_ \ / _ \/ __\ \ /\ / / _ \/ _ \ '_ \ / _ \ '__|",
            r"| |  | | | | | |  __/\__ \\ V  V /  __/  __/ |_) |  __/ |  ",
            r"|_|  |_|_|_| |_|\___||___/ \_/\_/ \___|\___| .__/ \___|_|  ",
            r"                                           |_|              ",
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
                                r: 200,
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

        let grid_art = "# # * # F #";
        let grid_start = center_column - grid_art.len() as i32 / 2;
        for (char_index, character) in grid_art.chars().enumerate() {
            if character != ' ' {
                let color = match character {
                    '*' => TermColor::Red,
                    'F' => TermColor::Rgb {
                        r: 255,
                        g: 100,
                        b: 100,
                    },
                    _ => TermColor::Grey,
                };
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (grid_start + char_index as i32) as f64,
                        row: (title_start_row + 8) as f64,
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

        let info = format!(
            "{}x{} grid, {} mines",
            BOARD_WIDTH, BOARD_HEIGHT, MINE_COUNT
        );
        let info_start = center_column - info.len() as i32 / 2;
        for (char_index, character) in info.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (info_start + char_index as i32) as f64,
                    row: (title_start_row + 10) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: TermColor::Rgb {
                        r: 150,
                        g: 150,
                        b: 200,
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
                    row: (title_start_row + 12) as f64,
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
                    row: (title_start_row + 14) as f64,
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
    board_offset_x: i32,
    board_offset_y: i32,
    mines: [[bool; BOARD_WIDTH]; BOARD_HEIGHT],
    adjacent_counts: [[u8; BOARD_WIDTH]; BOARD_HEIGHT],
    cell_states: [[CellState; BOARD_WIDTH]; BOARD_HEIGHT],
    cell_entities: Vec<Entity>,
    cursor_entity: Entity,
    hud_entities: Vec<Entity>,
    side_panel_entities: Vec<Entity>,
    cursor_column: usize,
    cursor_row: usize,
    mines_generated: bool,
    flags_placed: u32,
    revealed_count: u32,
    game_over: bool,
    won: bool,
    elapsed: f64,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            board_offset_x: 0,
            board_offset_y: 0,
            mines: [[false; BOARD_WIDTH]; BOARD_HEIGHT],
            adjacent_counts: [[0; BOARD_WIDTH]; BOARD_HEIGHT],
            cell_states: [[CellState::Hidden; BOARD_WIDTH]; BOARD_HEIGHT],
            cell_entities: Vec::new(),
            cursor_entity: Entity::default(),
            hud_entities: Vec::new(),
            side_panel_entities: Vec::new(),
            cursor_column: BOARD_WIDTH / 2,
            cursor_row: BOARD_HEIGHT / 2,
            mines_generated: false,
            flags_placed: 0,
            revealed_count: 0,
            game_over: false,
            won: false,
            elapsed: 0.0,
        }
    }

    fn generate_mines(&mut self, safe_column: usize, safe_row: usize) {
        let mut rng = rand::rng();
        let mut placed = 0;

        while placed < MINE_COUNT {
            let column = rng.random_range(0..BOARD_WIDTH);
            let row = rng.random_range(0..BOARD_HEIGHT);

            let distance_column = (column as i32 - safe_column as i32).abs();
            let distance_row = (row as i32 - safe_row as i32).abs();
            if distance_column <= 1 && distance_row <= 1 {
                continue;
            }

            if self.mines[row][column] {
                continue;
            }

            self.mines[row][column] = true;
            placed += 1;
        }

        for row in 0..BOARD_HEIGHT {
            for column in 0..BOARD_WIDTH {
                if self.mines[row][column] {
                    continue;
                }
                let mut count = 0u8;
                for delta_row in -1i32..=1 {
                    for delta_column in -1i32..=1 {
                        if delta_row == 0 && delta_column == 0 {
                            continue;
                        }
                        let neighbor_row = row as i32 + delta_row;
                        let neighbor_column = column as i32 + delta_column;
                        if neighbor_row >= 0
                            && neighbor_row < BOARD_HEIGHT as i32
                            && neighbor_column >= 0
                            && neighbor_column < BOARD_WIDTH as i32
                            && self.mines[neighbor_row as usize][neighbor_column as usize]
                        {
                            count += 1;
                        }
                    }
                }
                self.adjacent_counts[row][column] = count;
            }
        }

        self.mines_generated = true;
    }

    fn reveal_cell(&mut self, column: usize, row: usize) {
        if column >= BOARD_WIDTH || row >= BOARD_HEIGHT {
            return;
        }
        if self.cell_states[row][column] != CellState::Hidden {
            return;
        }

        self.cell_states[row][column] = CellState::Revealed;
        self.revealed_count += 1;

        if self.mines[row][column] {
            self.game_over = true;
            self.won = false;
            self.reveal_all_mines();
            return;
        }

        if self.adjacent_counts[row][column] == 0 {
            for delta_row in -1i32..=1 {
                for delta_column in -1i32..=1 {
                    if delta_row == 0 && delta_column == 0 {
                        continue;
                    }
                    let neighbor_column = column as i32 + delta_column;
                    let neighbor_row = row as i32 + delta_row;
                    if neighbor_column >= 0
                        && neighbor_column < BOARD_WIDTH as i32
                        && neighbor_row >= 0
                        && neighbor_row < BOARD_HEIGHT as i32
                    {
                        self.reveal_cell(neighbor_column as usize, neighbor_row as usize);
                    }
                }
            }
        }

        let total_safe = (BOARD_WIDTH * BOARD_HEIGHT) - MINE_COUNT;
        if self.revealed_count as usize >= total_safe {
            self.game_over = true;
            self.won = true;
        }
    }

    fn reveal_all_mines(&mut self) {
        for row in 0..BOARD_HEIGHT {
            for column in 0..BOARD_WIDTH {
                if self.mines[row][column] {
                    self.cell_states[row][column] = CellState::Revealed;
                }
            }
        }
    }

    fn toggle_flag(&mut self) {
        match self.cell_states[self.cursor_row][self.cursor_column] {
            CellState::Hidden => {
                self.cell_states[self.cursor_row][self.cursor_column] = CellState::Flagged;
                self.flags_placed += 1;
            }
            CellState::Flagged => {
                self.cell_states[self.cursor_row][self.cursor_column] = CellState::Hidden;
                self.flags_placed = self.flags_placed.saturating_sub(1);
            }
            CellState::Revealed => {}
        }
    }

    fn spawn_board(&mut self, world: &mut World) {
        let entities =
            world.spawn_entities(POSITION | SPRITE | Z_INDEX, BOARD_WIDTH * BOARD_HEIGHT);
        for (index, &entity) in entities.iter().enumerate() {
            let column = index % BOARD_WIDTH;
            let row = index / BOARD_WIDTH;
            world.set_position(
                entity,
                Position {
                    column: (self.board_offset_x + column as i32) as f64,
                    row: (self.board_offset_y + row as i32) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: '#',
                    foreground: TermColor::Rgb {
                        r: 100,
                        g: 100,
                        b: 120,
                    },
                    background: TermColor::Rgb {
                        r: 30,
                        g: 30,
                        b: 40,
                    },
                },
            );
            world.set_z_index(entity, ZIndex(1));
        }
        self.cell_entities = entities;

        self.cursor_entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
        world.set_position(
            self.cursor_entity,
            Position {
                column: (self.board_offset_x + self.cursor_column as i32) as f64,
                row: (self.board_offset_y + self.cursor_row as i32) as f64,
            },
        );
        world.set_sprite(
            self.cursor_entity,
            Sprite {
                character: '#',
                foreground: TermColor::White,
                background: TermColor::Rgb {
                    r: 80,
                    g: 80,
                    b: 120,
                },
            },
        );
        world.set_z_index(self.cursor_entity, ZIndex(5));
    }

    fn render_board(&self, world: &mut World) {
        for row in 0..BOARD_HEIGHT {
            for column in 0..BOARD_WIDTH {
                let index = row * BOARD_WIDTH + column;
                let entity = self.cell_entities[index];

                let (character, foreground, background) = match self.cell_states[row][column] {
                    CellState::Hidden => (
                        '#',
                        TermColor::Rgb {
                            r: 100,
                            g: 100,
                            b: 120,
                        },
                        TermColor::Rgb {
                            r: 30,
                            g: 30,
                            b: 40,
                        },
                    ),
                    CellState::Flagged => (
                        'F',
                        TermColor::Rgb {
                            r: 255,
                            g: 80,
                            b: 80,
                        },
                        TermColor::Rgb {
                            r: 50,
                            g: 20,
                            b: 20,
                        },
                    ),
                    CellState::Revealed => {
                        if self.mines[row][column] {
                            (
                                '*',
                                TermColor::Red,
                                TermColor::Rgb {
                                    r: 60,
                                    g: 10,
                                    b: 10,
                                },
                            )
                        } else {
                            let count = self.adjacent_counts[row][column];
                            if count == 0 {
                                (
                                    ' ',
                                    TermColor::Black,
                                    TermColor::Rgb {
                                        r: 15,
                                        g: 15,
                                        b: 20,
                                    },
                                )
                            } else {
                                let digit = (b'0' + count) as char;
                                (
                                    digit,
                                    number_color(count),
                                    TermColor::Rgb {
                                        r: 15,
                                        g: 15,
                                        b: 20,
                                    },
                                )
                            }
                        }
                    }
                };

                if let Some(sprite) = world.get_sprite_mut(entity) {
                    sprite.character = character;
                    sprite.foreground = foreground;
                    sprite.background = background;
                }
            }
        }

        if let Some(position) = world.get_position_mut(self.cursor_entity) {
            position.column = (self.board_offset_x + self.cursor_column as i32) as f64;
            position.row = (self.board_offset_y + self.cursor_row as i32) as f64;
        }

        let index = self.cursor_row * BOARD_WIDTH + self.cursor_column;
        let base_entity = self.cell_entities[index];
        let base_char = world
            .get_sprite(base_entity)
            .map(|sprite| sprite.character)
            .unwrap_or('#');
        let base_fg = world
            .get_sprite(base_entity)
            .map(|sprite| sprite.foreground)
            .unwrap_or(TermColor::White);

        if let Some(sprite) = world.get_sprite_mut(self.cursor_entity) {
            sprite.character = base_char;
            sprite.foreground = base_fg;
            sprite.background = TermColor::Rgb {
                r: 60,
                g: 60,
                b: 100,
            };
        }
    }

    fn update_side_panel(&mut self, world: &mut World) {
        for &entity in &self.side_panel_entities {
            world.despawn_entities(&[entity]);
        }
        self.side_panel_entities.clear();

        let panel_x = self.board_offset_x + BOARD_WIDTH as i32 + 2;
        let panel_y = self.board_offset_y;

        let remaining_mines = MINE_COUNT as i32 - self.flags_placed as i32;
        let elapsed_seconds = self.elapsed as u32;

        let lines = [
            format!("Mines: {}", remaining_mines),
            format!("Time:  {}s", elapsed_seconds),
            String::new(),
            "Controls:".to_string(),
            "Arrows: Move".to_string(),
            "Space:  Reveal".to_string(),
            "F:      Flag".to_string(),
        ];

        for (line_index, text) in lines.iter().enumerate() {
            for (char_index, character) in text.chars().enumerate() {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (panel_x + char_index as i32) as f64,
                        row: (panel_y + line_index as i32) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: if line_index < 2 {
                            TermColor::White
                        } else {
                            TermColor::Grey
                        },
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(1));
                self.side_panel_entities.push(entity);
            }
        }
    }

    fn clear_all_entities(&mut self, world: &mut World) {
        for &entity in &self.cell_entities {
            world.despawn_entities(&[entity]);
        }
        self.cell_entities.clear();
        world.despawn_entities(&[self.cursor_entity]);
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();
        for &entity in &self.side_panel_entities {
            world.despawn_entities(&[entity]);
        }
        self.side_panel_entities.clear();
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Minesweeper - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        let total_width = BOARD_WIDTH as i32 + 2 + 14;
        self.board_offset_x = (terminal.columns as i32 - total_width) / 2;
        self.board_offset_y = (terminal.rows as i32 - BOARD_HEIGHT as i32) / 2;
        if self.board_offset_x < 0 {
            self.board_offset_x = 0;
        }
        if self.board_offset_y < 0 {
            self.board_offset_y = 0;
        }

        self.spawn_board(world);
        self.render_board(world);
        self.update_side_panel(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed || self.game_over {
            return;
        }

        match key {
            KeyCode::Up | KeyCode::Char('w') => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('s') => {
                if self.cursor_row < BOARD_HEIGHT - 1 {
                    self.cursor_row += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('a') => {
                if self.cursor_column > 0 {
                    self.cursor_column -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('d') => {
                if self.cursor_column < BOARD_WIDTH - 1 {
                    self.cursor_column += 1;
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                if self.cell_states[self.cursor_row][self.cursor_column] == CellState::Flagged {
                    return;
                }
                if !self.mines_generated {
                    self.generate_mines(self.cursor_column, self.cursor_row);
                }
                self.reveal_cell(self.cursor_column, self.cursor_row);
            }
            KeyCode::Char('f') => {
                self.toggle_flag();
            }
            KeyCode::Escape | KeyCode::Char('q') => {
                world.resources.should_exit = true;
            }
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if !self.game_over && self.mines_generated {
            self.elapsed += world.resources.timing.delta_seconds;
        }

        self.render_board(world);
        self.update_side_panel(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            let won = self.won;
            let elapsed = self.elapsed as u32;
            self.clear_all_entities(world);
            return Some(Box::new(GameOverState {
                won,
                elapsed_seconds: elapsed,
                restart: false,
            }));
        }
        None
    }
}

struct GameOverState {
    won: bool,
    elapsed_seconds: u32,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Minesweeper - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let (result_text, result_color) = if self.won {
            (
                "YOU WIN!",
                TermColor::Rgb {
                    r: 100,
                    g: 255,
                    b: 100,
                },
            )
        } else {
            ("BOOM!", TermColor::Red)
        };

        let lines: Vec<(String, TermColor)> = vec![
            (result_text.to_string(), result_color),
            (String::new(), TermColor::Black),
            (
                format!("Time: {}s", self.elapsed_seconds),
                TermColor::Rgb {
                    r: 255,
                    g: 255,
                    b: 100,
                },
            ),
            (String::new(), TermColor::Black),
            ("Press R to play again".to_string(), TermColor::White),
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
