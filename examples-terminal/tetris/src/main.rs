use nightshade::tui::prelude::*;
use rand::Rng;

const BOARD_WIDTH: i32 = 10;
const BOARD_HEIGHT: i32 = 20;
const SIDE_PANEL_WIDTH: i32 = 14;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PieceKind {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl PieceKind {
    fn cells(self) -> [(i32, i32); 4] {
        match self {
            Self::I => [(-1, 0), (0, 0), (1, 0), (2, 0)],
            Self::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
            Self::T => [(-1, 0), (0, 0), (1, 0), (0, -1)],
            Self::S => [(0, 0), (1, 0), (-1, 1), (0, 1)],
            Self::Z => [(-1, 0), (0, 0), (0, 1), (1, 1)],
            Self::J => [(-1, -1), (-1, 0), (0, 0), (1, 0)],
            Self::L => [(-1, 0), (0, 0), (1, 0), (1, -1)],
        }
    }

    fn color(self) -> TermColor {
        match self {
            Self::I => TermColor::Rgb {
                r: 0,
                g: 255,
                b: 255,
            },
            Self::O => TermColor::Rgb {
                r: 255,
                g: 255,
                b: 0,
            },
            Self::T => TermColor::Rgb {
                r: 180,
                g: 0,
                b: 255,
            },
            Self::S => TermColor::Rgb { r: 0, g: 255, b: 0 },
            Self::Z => TermColor::Rgb { r: 255, g: 0, b: 0 },
            Self::J => TermColor::Rgb { r: 0, g: 0, b: 255 },
            Self::L => TermColor::Rgb {
                r: 255,
                g: 165,
                b: 0,
            },
        }
    }

    fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..7) {
            0 => Self::I,
            1 => Self::O,
            2 => Self::T,
            3 => Self::S,
            4 => Self::Z,
            5 => Self::J,
            _ => Self::L,
        }
    }
}

fn rotate_cells_clockwise(cells: &[(i32, i32); 4]) -> [(i32, i32); 4] {
    [
        (-cells[0].1, cells[0].0),
        (-cells[1].1, cells[1].0),
        (-cells[2].1, cells[2].0),
        (-cells[3].1, cells[3].0),
    ]
}

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Tetris - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r" _____    _        _     ",
            r"|_   _|__| |_ _ __(_)___ ",
            r"  | |/ _ \ __| '__| / __|",
            r"  | |  __/ |_| |  | \__ \",
            r"  |_|\___|\__|_|  |_|___/",
        ];

        let title_start_row = center_row - 6;

        let colors = [
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
            TermColor::Rgb { r: 0, g: 255, b: 0 },
            TermColor::Rgb {
                r: 0,
                g: 200,
                b: 255,
            },
        ];

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
                            foreground: colors[line_index],
                            background: TermColor::Black,
                        },
                    );
                    world.set_z_index(entity, ZIndex(10));
                }
            }
        }

        let blocks = "[ ][ ][ ][ ]";
        let block_colors = [
            TermColor::Rgb { r: 255, g: 0, b: 0 },
            TermColor::Rgb { r: 0, g: 255, b: 0 },
            TermColor::Rgb {
                r: 0,
                g: 200,
                b: 255,
            },
            TermColor::Rgb {
                r: 255,
                g: 255,
                b: 0,
            },
        ];
        let block_start = center_column - blocks.len() as i32 / 2;
        for (char_index, character) in blocks.chars().enumerate() {
            let color_index = char_index / 3;
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (block_start + char_index as i32) as f64,
                    row: (title_start_row + 7) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: block_colors[color_index.min(block_colors.len() - 1)],
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

struct GameplayState {
    board_offset_x: i32,
    board_offset_y: i32,
    board: [[Option<TermColor>; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
    board_entities: Vec<Entity>,
    wall_entities: Vec<Entity>,
    piece_entities: Vec<Entity>,
    ghost_entities: Vec<Entity>,
    next_piece_entities: Vec<Entity>,
    hud_entities: Vec<Entity>,
    current_piece: PieceKind,
    current_cells: [(i32, i32); 4],
    current_x: i32,
    current_y: i32,
    next_piece: PieceKind,
    score: u32,
    lines_cleared: u32,
    level: u32,
    game_over: bool,
    fall_timer: f64,
    move_cooldown: f64,
    soft_dropping: bool,
}

impl GameplayState {
    fn new() -> Self {
        let first_piece = PieceKind::random();
        let next_piece = PieceKind::random();
        Self {
            board_offset_x: 0,
            board_offset_y: 0,
            board: [[None; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
            board_entities: Vec::new(),
            wall_entities: Vec::new(),
            piece_entities: Vec::new(),
            ghost_entities: Vec::new(),
            next_piece_entities: Vec::new(),
            hud_entities: Vec::new(),
            current_piece: first_piece,
            current_cells: first_piece.cells(),
            current_x: BOARD_WIDTH / 2,
            current_y: 1,
            next_piece,
            score: 0,
            lines_cleared: 0,
            level: 1,
            game_over: false,
            fall_timer: 0.0,
            move_cooldown: 0.0,
            soft_dropping: false,
        }
    }

    fn fall_interval(&self) -> f64 {
        (0.8 - (self.level as f64 - 1.0) * 0.07).max(0.05)
    }

    fn piece_world_cells(&self) -> [(i32, i32); 4] {
        self.current_cells
            .map(|(col, row)| (self.current_x + col, self.current_y + row))
    }

    fn ghost_y(&self) -> i32 {
        let mut test_y = self.current_y;
        loop {
            test_y += 1;
            let blocked = self.current_cells.iter().any(|&(col, row)| {
                let world_col = self.current_x + col;
                let world_row = test_y + row;
                world_row >= BOARD_HEIGHT
                    || !(0..BOARD_WIDTH).contains(&world_col)
                    || (world_row >= 0
                        && self.board[world_row as usize][world_col as usize].is_some())
            });
            if blocked {
                return test_y - 1;
            }
        }
    }

    fn can_place(&self, cells: &[(i32, i32); 4], offset_x: i32, offset_y: i32) -> bool {
        cells.iter().all(|&(col, row)| {
            let world_col = offset_x + col;
            let world_row = offset_y + row;
            (0..BOARD_WIDTH).contains(&world_col)
                && world_row < BOARD_HEIGHT
                && (world_row < 0 || self.board[world_row as usize][world_col as usize].is_none())
        })
    }

    fn try_move(&mut self, delta_x: i32, delta_y: i32) -> bool {
        let new_x = self.current_x + delta_x;
        let new_y = self.current_y + delta_y;
        if self.can_place(&self.current_cells, new_x, new_y) {
            self.current_x = new_x;
            self.current_y = new_y;
            return true;
        }
        false
    }

    fn try_rotate(&mut self) {
        let rotated = rotate_cells_clockwise(&self.current_cells);
        if self.can_place(&rotated, self.current_x, self.current_y) {
            self.current_cells = rotated;
            return;
        }
        if self.can_place(&rotated, self.current_x - 1, self.current_y) {
            self.current_cells = rotated;
            self.current_x -= 1;
            return;
        }
        if self.can_place(&rotated, self.current_x + 1, self.current_y) {
            self.current_cells = rotated;
            self.current_x += 1;
            return;
        }
        if self.can_place(&rotated, self.current_x, self.current_y - 1) {
            self.current_cells = rotated;
            self.current_y -= 1;
        }
    }

    fn hard_drop(&mut self) {
        let ghost = self.ghost_y();
        let rows_dropped = ghost - self.current_y;
        self.score += rows_dropped as u32 * 2;
        self.current_y = ghost;
        self.lock_piece();
    }

    fn lock_piece(&mut self) {
        let color = self.current_piece.color();
        for (col, row) in self.piece_world_cells() {
            if (0..BOARD_HEIGHT).contains(&row) && (0..BOARD_WIDTH).contains(&col) {
                self.board[row as usize][col as usize] = Some(color);
            }
        }

        self.clear_lines();
        self.spawn_next_piece();
    }

    fn clear_lines(&mut self) {
        let mut lines_this_clear = 0;

        let mut write_row = BOARD_HEIGHT as usize - 1;
        for read_row in (0..BOARD_HEIGHT as usize).rev() {
            let full = self.board[read_row].iter().all(|cell| cell.is_some());
            if full {
                lines_this_clear += 1;
            } else {
                if write_row != read_row {
                    self.board[write_row] = self.board[read_row];
                }
                write_row = write_row.wrapping_sub(1);
            }
        }

        for row in 0..=write_row {
            if write_row == usize::MAX {
                break;
            }
            self.board[row] = [None; BOARD_WIDTH as usize];
        }

        if lines_this_clear > 0 {
            self.lines_cleared += lines_this_clear;
            let points = match lines_this_clear {
                1 => 100,
                2 => 300,
                3 => 500,
                4 => 800,
                _ => 800,
            };
            self.score += points * self.level;
            self.level = (self.lines_cleared / 10) + 1;
        }
    }

    fn spawn_next_piece(&mut self) {
        self.current_piece = self.next_piece;
        self.current_cells = self.current_piece.cells();
        self.current_x = BOARD_WIDTH / 2;
        self.current_y = 0;
        self.next_piece = PieceKind::random();
        self.fall_timer = 0.0;

        if !self.can_place(&self.current_cells, self.current_x, self.current_y) {
            self.game_over = true;
        }
    }

    fn spawn_board_entities(&mut self, world: &mut World) {
        let entities = world.spawn_entities(
            POSITION | SPRITE | Z_INDEX,
            (BOARD_WIDTH * BOARD_HEIGHT) as usize,
        );
        for (index, &entity) in entities.iter().enumerate() {
            let col = (index as i32) % BOARD_WIDTH;
            let row = (index as i32) / BOARD_WIDTH;
            world.set_position(
                entity,
                Position {
                    column: (self.board_offset_x + col) as f64,
                    row: (self.board_offset_y + row) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: '.',
                    foreground: TermColor::Rgb {
                        r: 30,
                        g: 30,
                        b: 30,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(1));
        }
        self.board_entities = entities;
    }

    fn spawn_walls(&mut self, world: &mut World) {
        for row in 0..BOARD_HEIGHT {
            let left = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                left,
                Position {
                    column: (self.board_offset_x - 1) as f64,
                    row: (self.board_offset_y + row) as f64,
                },
            );
            world.set_sprite(
                left,
                Sprite {
                    character: '|',
                    foreground: TermColor::Rgb {
                        r: 100,
                        g: 100,
                        b: 120,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(left, ZIndex(1));
            self.wall_entities.push(left);

            let right = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                right,
                Position {
                    column: (self.board_offset_x + BOARD_WIDTH) as f64,
                    row: (self.board_offset_y + row) as f64,
                },
            );
            world.set_sprite(
                right,
                Sprite {
                    character: '|',
                    foreground: TermColor::Rgb {
                        r: 100,
                        g: 100,
                        b: 120,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(right, ZIndex(1));
            self.wall_entities.push(right);
        }

        for col in -1..=BOARD_WIDTH {
            let bottom = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                bottom,
                Position {
                    column: (self.board_offset_x + col) as f64,
                    row: (self.board_offset_y + BOARD_HEIGHT) as f64,
                },
            );
            world.set_sprite(
                bottom,
                Sprite {
                    character: '=',
                    foreground: TermColor::Rgb {
                        r: 100,
                        g: 100,
                        b: 120,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(bottom, ZIndex(1));
            self.wall_entities.push(bottom);
        }
    }

    fn render_board(&self, world: &mut World) {
        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                let index = (row * BOARD_WIDTH + col) as usize;
                let entity = self.board_entities[index];

                let (character, foreground) =
                    if let Some(color) = self.board[row as usize][col as usize] {
                        ('#', color)
                    } else {
                        (
                            '.',
                            TermColor::Rgb {
                                r: 30,
                                g: 30,
                                b: 30,
                            },
                        )
                    };

                if let Some(sprite) = world.get_sprite_mut(entity) {
                    sprite.character = character;
                    sprite.foreground = foreground;
                }
            }
        }

        let color = self.current_piece.color();
        let ghost = self.ghost_y();
        for &(col, row) in &self.current_cells {
            let ghost_row = ghost + row;
            let ghost_col = self.current_x + col;
            if (0..BOARD_HEIGHT).contains(&ghost_row) && (0..BOARD_WIDTH).contains(&ghost_col) {
                let index = (ghost_row * BOARD_WIDTH + ghost_col) as usize;
                if let Some(sprite) = world.get_sprite_mut(self.board_entities[index])
                    && self.board[ghost_row as usize][ghost_col as usize].is_none()
                {
                    sprite.character = ':';
                    sprite.foreground = TermColor::Rgb {
                        r: match color {
                            TermColor::Rgb { r, .. } => r / 3,
                            _ => 40,
                        },
                        g: match color {
                            TermColor::Rgb { g, .. } => g / 3,
                            _ => 40,
                        },
                        b: match color {
                            TermColor::Rgb { b, .. } => b / 3,
                            _ => 40,
                        },
                    };
                }
            }
        }

        for &(col, row) in &self.current_cells {
            let world_row = self.current_y + row;
            let world_col = self.current_x + col;
            if (0..BOARD_HEIGHT).contains(&world_row) && (0..BOARD_WIDTH).contains(&world_col) {
                let index = (world_row * BOARD_WIDTH + world_col) as usize;
                if let Some(sprite) = world.get_sprite_mut(self.board_entities[index]) {
                    sprite.character = '#';
                    sprite.foreground = color;
                }
            }
        }
    }

    fn update_side_panel(&mut self, world: &mut World) {
        for &entity in &self.next_piece_entities {
            world.despawn_entities(&[entity]);
        }
        self.next_piece_entities.clear();
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();

        let panel_x = self.board_offset_x + BOARD_WIDTH + 2;
        let panel_y = self.board_offset_y;

        let info_lines = [
            format!("Score: {}", self.score),
            format!("Lines: {}", self.lines_cleared),
            format!("Level: {}", self.level),
            String::new(),
            "Next:".to_string(),
        ];

        for (line_index, text) in info_lines.iter().enumerate() {
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
                        foreground: TermColor::White,
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(1));
                self.hud_entities.push(entity);
            }
        }

        let next_cells = self.next_piece.cells();
        let next_color = self.next_piece.color();
        let preview_y = panel_y + 6;
        let preview_x = panel_x + 2;

        for &(col, row) in &next_cells {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (preview_x + col) as f64,
                    row: (preview_y + row) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: '#',
                    foreground: next_color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(1));
            self.next_piece_entities.push(entity);
        }

        let controls_y = panel_y + 10;
        let controls = [
            "Controls:",
            "</>: Move",
            " ^:  Rotate",
            " v:  Soft drop",
            "Spc: Hard drop",
        ];

        for (line_index, text) in controls.iter().enumerate() {
            for (char_index, character) in text.chars().enumerate() {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (panel_x + char_index as i32) as f64,
                        row: (controls_y + line_index as i32) as f64,
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
                world.set_z_index(entity, ZIndex(1));
                self.hud_entities.push(entity);
            }
        }
    }

    fn clear_all_entities(&mut self, world: &mut World) {
        for &entity in &self.board_entities {
            world.despawn_entities(&[entity]);
        }
        self.board_entities.clear();
        for &entity in &self.wall_entities {
            world.despawn_entities(&[entity]);
        }
        self.wall_entities.clear();
        for &entity in &self.piece_entities {
            world.despawn_entities(&[entity]);
        }
        self.piece_entities.clear();
        for &entity in &self.ghost_entities {
            world.despawn_entities(&[entity]);
        }
        self.ghost_entities.clear();
        for &entity in &self.next_piece_entities {
            world.despawn_entities(&[entity]);
        }
        self.next_piece_entities.clear();
        for &entity in &self.hud_entities {
            world.despawn_entities(&[entity]);
        }
        self.hud_entities.clear();
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Tetris - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        let total_width = BOARD_WIDTH + 2 + SIDE_PANEL_WIDTH;
        self.board_offset_x = (terminal.columns as i32 - total_width) / 2 + 1;
        self.board_offset_y = (terminal.rows as i32 - BOARD_HEIGHT) / 2;
        if self.board_offset_x < 1 {
            self.board_offset_x = 1;
        }
        if self.board_offset_y < 0 {
            self.board_offset_y = 0;
        }

        self.spawn_board_entities(world);
        self.spawn_walls(world);
        self.render_board(world);
        self.update_side_panel(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if self.game_over {
            return;
        }

        match key {
            KeyCode::Left | KeyCode::Char('a') if pressed && self.move_cooldown <= 0.0 => {
                self.try_move(-1, 0);
                self.move_cooldown = 0.08;
            }
            KeyCode::Right | KeyCode::Char('d') if pressed && self.move_cooldown <= 0.0 => {
                self.try_move(1, 0);
                self.move_cooldown = 0.08;
            }
            KeyCode::Up | KeyCode::Char('w') if pressed => {
                self.try_rotate();
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.soft_dropping = pressed;
            }
            KeyCode::Char(' ') if pressed => {
                self.hard_drop();
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

        if self.move_cooldown > 0.0 {
            self.move_cooldown -= delta;
        }

        let interval = if self.soft_dropping {
            self.fall_interval() / 10.0
        } else {
            self.fall_interval()
        };

        self.fall_timer += delta;
        if self.fall_timer >= interval {
            self.fall_timer = 0.0;
            if !self.try_move(0, 1) {
                self.lock_piece();
            }
            if self.soft_dropping {
                self.score += 1;
            }
        }

        self.render_board(world);
        self.update_side_panel(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            let score = self.score;
            let lines = self.lines_cleared;
            let level = self.level;
            self.clear_all_entities(world);
            return Some(Box::new(GameOverState {
                score,
                lines_cleared: lines,
                level,
                restart: false,
            }));
        }
        None
    }
}

struct GameOverState {
    score: u32,
    lines_cleared: u32,
    level: u32,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Tetris - Game Over"
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
                format!("Lines: {}", self.lines_cleared),
                TermColor::Rgb {
                    r: 100,
                    g: 255,
                    b: 200,
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
