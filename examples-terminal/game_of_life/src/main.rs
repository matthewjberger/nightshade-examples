use nightshade::tui::prelude::*;
use rand::Rng;
use std::collections::{HashMap, HashSet};

const HUD_HEIGHT: i32 = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Edit,
    Play,
}

struct GameOfLifeState {
    mode: Mode,
    live_cells: HashSet<(i32, i32)>,
    cell_entities: Vec<Entity>,
    hud_entities: Vec<Entity>,
    cursor_entity: Option<Entity>,
    cursor_column: i32,
    cursor_row: i32,
    grid_offset_column: i32,
    grid_offset_row: i32,
    generation: u64,
    simulation_timer: Timer,
    simulation_interval_ms: u64,
    paused: bool,
}

impl GameOfLifeState {
    fn new() -> Self {
        Self {
            mode: Mode::Edit,
            live_cells: HashSet::new(),
            cell_entities: Vec::new(),
            hud_entities: Vec::new(),
            cursor_entity: None,
            cursor_column: 0,
            cursor_row: 0,
            grid_offset_column: 0,
            grid_offset_row: 0,
            generation: 0,
            simulation_timer: Timer::repeating(0.1),
            simulation_interval_ms: 100,
            paused: false,
        }
    }

    fn step_simulation(&mut self) {
        let mut neighbor_counts: HashMap<(i32, i32), u8> = HashMap::new();

        for &(column, row) in &self.live_cells {
            for delta_row in -1i32..=1 {
                for delta_column in -1i32..=1 {
                    if delta_row == 0 && delta_column == 0 {
                        continue;
                    }
                    let neighbor = (column + delta_column, row + delta_row);
                    *neighbor_counts.entry(neighbor).or_insert(0) += 1;
                }
            }
        }

        let mut next_generation = HashSet::new();

        for (&cell, &count) in &neighbor_counts {
            let is_alive = self.live_cells.contains(&cell);
            let survives = is_alive && (count == 2 || count == 3);
            let is_born = !is_alive && count == 3;
            if survives || is_born {
                next_generation.insert(cell);
            }
        }

        self.live_cells = next_generation;
        self.generation += 1;
    }

    fn render_cells(&mut self, world: &mut World) {
        world.despawn_entities(&self.cell_entities);
        self.cell_entities.clear();

        let terminal_columns = world.resources.terminal_size.columns as i32;
        let terminal_rows = world.resources.terminal_size.rows as i32;

        let visible_cells: Vec<(i32, i32)> = self
            .live_cells
            .iter()
            .filter(|&&(column, row)| {
                let screen_column = column - self.grid_offset_column;
                let screen_row = row - self.grid_offset_row + HUD_HEIGHT;
                screen_column >= 0
                    && screen_column < terminal_columns
                    && screen_row >= HUD_HEIGHT
                    && screen_row < terminal_rows
            })
            .copied()
            .collect();

        if visible_cells.is_empty() {
            return;
        }

        let entities = world.spawn_entities(POSITION | SPRITE | Z_INDEX, visible_cells.len());

        for (index, &(column, row)) in visible_cells.iter().enumerate() {
            let entity = entities[index];
            let screen_column = column - self.grid_offset_column;
            let screen_row = row - self.grid_offset_row + HUD_HEIGHT;

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
                    character: 'O',
                    foreground: TermColor::Green,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(1));
        }

        self.cell_entities = entities;
    }

    fn render_cursor(&mut self, world: &mut World) {
        if let Some(entity) = self.cursor_entity {
            world.despawn_entities(&[entity]);
            self.cursor_entity = None;
        }

        if self.mode != Mode::Edit {
            return;
        }

        let screen_column = self.cursor_column - self.grid_offset_column;
        let screen_row = self.cursor_row - self.grid_offset_row + HUD_HEIGHT;
        let terminal_columns = world.resources.terminal_size.columns as i32;
        let terminal_rows = world.resources.terminal_size.rows as i32;

        if screen_column < 0
            || screen_column >= terminal_columns
            || screen_row < HUD_HEIGHT
            || screen_row >= terminal_rows
        {
            return;
        }

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
                character: '+',
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));
        self.cursor_entity = Some(entity);
    }

    fn render_hud(&mut self, world: &mut World) {
        world.despawn_entities(&self.hud_entities);
        self.hud_entities.clear();

        let terminal_columns = world.resources.terminal_size.columns as usize;

        let mode_label = match self.mode {
            Mode::Edit => "EDIT",
            Mode::Play => {
                if self.paused {
                    "PAUSED"
                } else {
                    "PLAY"
                }
            }
        };

        let row_0 = format!(
            "Game of Life | {} | Gen: {} | Cells: {} | Speed: {}ms",
            mode_label,
            self.generation,
            self.live_cells.len(),
            self.simulation_interval_ms,
        );

        let row_1 = match self.mode {
            Mode::Edit => {
                "Arrows=move Space=toggle Enter=play C=clear R=random 1-5=patterns +/-=speed Q=quit"
                    .to_string()
            }
            Mode::Play => {
                "Arrows=pan Space=pause Enter=edit N=step C=clear R=random 1-5=patterns +/-=speed Q=quit"
                    .to_string()
            }
        };

        self.spawn_hud_line(world, &row_0, 0, terminal_columns);
        self.spawn_hud_line(world, &row_1, 1, terminal_columns);
    }

    fn spawn_hud_line(&mut self, world: &mut World, text: &str, row: i32, terminal_columns: usize) {
        let truncated: String = text.chars().take(terminal_columns).collect();
        let padded = format!("{:<width$}", truncated, width = terminal_columns);

        let entities = world.spawn_entities(POSITION | SPRITE | Z_INDEX, padded.len());

        for (char_index, character) in padded.chars().enumerate() {
            let entity = entities[char_index];
            world.set_position(
                entity,
                Position {
                    column: char_index as f64,
                    row: row as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: if row == 0 {
                        TermColor::Cyan
                    } else {
                        TermColor::DarkGrey
                    },
                    background: TermColor::Rgb {
                        r: 20,
                        g: 20,
                        b: 30,
                    },
                },
            );
            world.set_z_index(entity, ZIndex(20));
        }

        self.hud_entities.extend_from_slice(&entities);
    }

    fn toggle_cell_at_cursor(&mut self) {
        let position = (self.cursor_column, self.cursor_row);
        if self.live_cells.contains(&position) {
            self.live_cells.remove(&position);
        } else {
            self.live_cells.insert(position);
        }
    }

    fn clear_all_cells(&mut self) {
        self.live_cells.clear();
        self.generation = 0;
    }

    fn fill_random(&mut self, world: &World) {
        self.live_cells.clear();
        self.generation = 0;
        let mut rng = rand::rng();

        let terminal_columns = world.resources.terminal_size.columns as i32;
        let terminal_rows = world.resources.terminal_size.rows as i32 - HUD_HEIGHT;

        for row in 0..terminal_rows {
            for column in 0..terminal_columns {
                if rng.random_range(0..4) == 0 {
                    self.live_cells
                        .insert((column + self.grid_offset_column, row + self.grid_offset_row));
                }
            }
        }
    }

    fn load_pattern(&mut self, pattern_number: u8) {
        let center_column = self.cursor_column;
        let center_row = self.cursor_row;

        let offsets: Vec<(i32, i32)> = match pattern_number {
            1 => gosper_glider_gun(),
            2 => pulsar(),
            3 => lightweight_spaceship(),
            4 => r_pentomino(),
            5 => acorn(),
            _ => return,
        };

        for (delta_column, delta_row) in offsets {
            self.live_cells
                .insert((center_column + delta_column, center_row + delta_row));
        }
    }

    fn increase_speed(&mut self) {
        if self.simulation_interval_ms > 10 {
            self.simulation_interval_ms -= 10;
        }
        self.simulation_timer = Timer::repeating(self.simulation_interval_ms as f64 / 1000.0);
    }

    fn decrease_speed(&mut self) {
        if self.simulation_interval_ms < 1000 {
            self.simulation_interval_ms += 10;
        }
        self.simulation_timer = Timer::repeating(self.simulation_interval_ms as f64 / 1000.0);
    }
}

impl State for GameOfLifeState {
    fn title(&self) -> &str {
        "Game of Life - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 60;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal_columns = world.resources.terminal_size.columns as i32;
        let terminal_rows = world.resources.terminal_size.rows as i32 - HUD_HEIGHT;

        self.cursor_column = self.grid_offset_column + terminal_columns / 2;
        self.cursor_row = self.grid_offset_row + terminal_rows / 2;

        self.render_cells(world);
        self.render_cursor(world);
        self.render_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }

        match key {
            KeyCode::Escape | KeyCode::Char('q') | KeyCode::Char('Q') => {
                world.resources.should_exit = true;
            }
            KeyCode::Enter => match self.mode {
                Mode::Edit => {
                    self.mode = Mode::Play;
                    self.paused = false;
                }
                Mode::Play => {
                    self.mode = Mode::Edit;
                }
            },
            KeyCode::Char(' ') => match self.mode {
                Mode::Edit => {
                    self.toggle_cell_at_cursor();
                }
                Mode::Play => {
                    self.paused = !self.paused;
                }
            },
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => match self.mode {
                Mode::Edit => {
                    self.cursor_row -= 1;
                }
                Mode::Play => {
                    self.grid_offset_row -= 1;
                }
            },
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => match self.mode {
                Mode::Edit => {
                    self.cursor_row += 1;
                }
                Mode::Play => {
                    self.grid_offset_row += 1;
                }
            },
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => match self.mode {
                Mode::Edit => {
                    self.cursor_column -= 1;
                }
                Mode::Play => {
                    self.grid_offset_column -= 1;
                }
            },
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => match self.mode {
                Mode::Edit => {
                    self.cursor_column += 1;
                }
                Mode::Play => {
                    self.grid_offset_column += 1;
                }
            },
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.step_simulation();
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.clear_all_cells();
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.fill_random(world);
            }
            KeyCode::Char('1') => {
                self.load_pattern(1);
            }
            KeyCode::Char('2') => {
                self.load_pattern(2);
            }
            KeyCode::Char('3') => {
                self.load_pattern(3);
            }
            KeyCode::Char('4') => {
                self.load_pattern(4);
            }
            KeyCode::Char('5') => {
                self.load_pattern(5);
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.increase_speed();
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                self.decrease_speed();
            }
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.mode == Mode::Play && !self.paused {
            let delta = world.resources.timing.delta_seconds;
            if self.simulation_timer.tick(delta) {
                self.step_simulation();
            }
        }

        self.render_cells(world);
        self.render_cursor(world);
        self.render_hud(world);
    }
}

fn gosper_glider_gun() -> Vec<(i32, i32)> {
    vec![
        (0, 4),
        (0, 5),
        (1, 4),
        (1, 5),
        (10, 4),
        (10, 5),
        (10, 6),
        (11, 3),
        (11, 7),
        (12, 2),
        (12, 8),
        (13, 2),
        (13, 8),
        (14, 5),
        (15, 3),
        (15, 7),
        (16, 4),
        (16, 5),
        (16, 6),
        (17, 5),
        (20, 2),
        (20, 3),
        (20, 4),
        (21, 2),
        (21, 3),
        (21, 4),
        (22, 1),
        (22, 5),
        (24, 0),
        (24, 1),
        (24, 5),
        (24, 6),
        (34, 2),
        (34, 3),
        (35, 2),
        (35, 3),
    ]
}

fn pulsar() -> Vec<(i32, i32)> {
    let mut cells = Vec::new();
    let template = [
        (2, 0),
        (3, 0),
        (4, 0),
        (0, 2),
        (0, 3),
        (0, 4),
        (5, 2),
        (5, 3),
        (5, 4),
        (2, 5),
        (3, 5),
        (4, 5),
    ];
    for &(column, row) in &template {
        cells.push((column, row));
        cells.push((-column, row));
        cells.push((column, -row));
        cells.push((-column, -row));
    }
    cells.sort();
    cells.dedup();
    cells
}

fn lightweight_spaceship() -> Vec<(i32, i32)> {
    vec![
        (0, 0),
        (3, 0),
        (4, 1),
        (0, 2),
        (4, 2),
        (1, 3),
        (2, 3),
        (3, 3),
        (4, 3),
    ]
}

fn r_pentomino() -> Vec<(i32, i32)> {
    vec![(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)]
}

fn acorn() -> Vec<(i32, i32)> {
    vec![(0, 0), (1, 2), (3, 1), (4, 2), (5, 2), (6, 2), (7, 2)]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(GameOfLifeState::new()))
}
