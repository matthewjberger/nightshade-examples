//! Nightshade adapter. Everything in here is about drawing the engine's
//! state into the terminal grid and turning typed commands into `Engine::pick`.
//!
//! Structure of a frame:
//!
//! ```text
//! +- top: scrolling transcript (wrapped) ----------------+
//! | ... narration, player echoes, dialogue, system ...   |
//! +- separator ------------------------------------------+
//! | > take key_                                          |
//! +------------------------------------------------------+
//! ```
//!
//! The player types free-form commands. `help` (or `?`) prints the list of
//! actions currently available into the transcript.

pub mod input;

use crate::data::TranscriptEntry;
use crate::engine::Engine;
use crate::game;
use nightshade::tui::prelude::*;

pub struct LighthouseState {
    engine: Engine,
    state: crate::data::RuntimeState,
    entities: Vec<Entity>,
    needs_redraw: bool,
    input_buffer: String,
    cursor_visible: bool,
    cursor_timer: f64,
    started: bool,
    undo_stack: std::collections::VecDeque<crate::data::RuntimeState>,
    quit_requested: bool,
}

impl LighthouseState {
    pub fn new() -> Self {
        let world = game::build_world();
        let engine = Engine::new(world).unwrap_or_else(|errors| {
            eprintln!("World validation failed:");
            for error in &errors {
                eprintln!("  - {error}");
            }
            panic!("world validation failed");
        });
        let state = engine.start_state();
        Self {
            engine,
            state,
            entities: Vec::new(),
            needs_redraw: true,
            input_buffer: String::new(),
            cursor_visible: true,
            cursor_timer: 0.0,
            started: false,
            undo_stack: std::collections::VecDeque::new(),
            quit_requested: false,
        }
    }

    fn ensure_started(&mut self) {
        if !self.started {
            self.engine.start(&mut self.state);
            self.started = true;
        }
    }

    fn snapshot(&mut self) {
        const UNDO_CAPACITY: usize = 20;
        self.undo_stack.push_back(self.state.clone());
        while self.undo_stack.len() > UNDO_CAPACITY {
            self.undo_stack.pop_front();
        }
    }

    fn undo(&mut self) {
        if let Some(previous) = self.undo_stack.pop_back() {
            self.state = previous;
            self.needs_redraw = true;
        }
    }

    fn pick_index(&mut self, index: usize) {
        if self.state.game_over.is_some() {
            return;
        }
        self.snapshot();
        self.engine.pick(&mut self.state, index);
        self.needs_redraw = true;
    }

    fn submit_input(&mut self) {
        let raw = self.input_buffer.trim().to_string();
        self.input_buffer.clear();
        self.needs_redraw = true;

        let choices = self.engine.available_choices(&self.state);
        match input::parse(&self.engine, &self.state, &choices, &raw) {
            input::Parsed::Empty => {}
            input::Parsed::Quit => {
                self.quit_requested = true;
            }
            input::Parsed::Undo => self.undo(),
            input::Parsed::Help => self.show_help(&choices),
            input::Parsed::Choose(index) => self.pick_index(index),
            input::Parsed::NoMatch => {
                self.state.push_transcript(TranscriptEntry::System(
                    "You can't do that here. Type 'help' to see what you can do.".to_string(),
                ));
            }
            input::Parsed::Ambiguous => {
                self.state.push_transcript(TranscriptEntry::System(
                    "Which one? Be more specific.".to_string(),
                ));
            }
        }
    }

    fn show_help(&mut self, choices: &[crate::data::Choice]) {
        self.state
            .push_transcript(TranscriptEntry::System("You could:".to_string()));
        if choices.is_empty() {
            self.state
                .push_transcript(TranscriptEntry::System("  (nothing)".to_string()));
        } else {
            for choice in choices {
                let label = self.engine.resolve_text(&self.state, &choice.label);
                self.state
                    .push_transcript(TranscriptEntry::System(format!("  - {label}")));
            }
        }
        self.state.push_transcript(TranscriptEntry::System(
            "Also: 'undo' (u), 'quit' (q).".to_string(),
        ));
    }

    fn render(&mut self, world: &mut World) {
        world.despawn_entities(&self.entities);
        self.entities.clear();

        let columns = world.resources.terminal_size.columns as usize;
        let rows = world.resources.terminal_size.rows as usize;
        if columns < 20 || rows < 8 {
            return;
        }

        let padding = 2;
        let text_width = columns.saturating_sub(padding * 2).max(10);

        // Reserve rows: prompt + status + a separator above the prompt.
        let status_row = rows - 1;
        let prompt_row = rows - 2;
        let separator_row = prompt_row.saturating_sub(1);
        let transcript_rows = separator_row;

        let mut wrapped: Vec<(String, TermColor)> = Vec::new();
        for entry in &self.state.transcript {
            render_entry(&mut wrapped, entry, text_width);
        }

        let start_index = wrapped.len().saturating_sub(transcript_rows);
        for (i, (line, color)) in wrapped[start_index..].iter().enumerate() {
            self.draw_text(world, padding, i, line, *color);
        }

        // Separator.
        self.draw_horizontal_rule(world, separator_row, columns);

        // Prompt.
        let prompt = if self.state.game_over.is_some() {
            "(ESC to exit)".to_string()
        } else {
            format!(
                "> {}{}",
                self.input_buffer,
                if self.cursor_visible { "_" } else { " " }
            )
        };
        self.draw_text(world, padding, prompt_row, &prompt, TermColor::Green);

        // Status line.
        let status = self.render_status();
        self.draw_text(world, padding, status_row, &status, TermColor::Magenta);
    }

    fn draw_text(
        &mut self,
        world: &mut World,
        column: usize,
        row: usize,
        text: &str,
        color: TermColor,
    ) {
        let terminal_columns = world.resources.terminal_size.columns as usize;
        for (offset, character) in text.chars().enumerate() {
            let x = column + offset;
            if x >= terminal_columns {
                break;
            }
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: x as f64,
                    row: row as f64,
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
            world.set_z_index(entity, ZIndex(1));
            self.entities.push(entity);
        }
    }

    fn draw_horizontal_rule(&mut self, world: &mut World, row: usize, columns: usize) {
        for column_index in 0..columns {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: column_index as f64,
                    row: row as f64,
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
            self.entities.push(entity);
        }
    }

    fn render_status(&self) -> String {
        let turn = self.state.turn;
        let timer_remaining = self
            .state
            .timers_remaining
            .get(&crate::game::ids::timer_storm())
            .copied()
            .unwrap_or(0);
        let inventory_count = self
            .state
            .item_locations
            .values()
            .filter(|location| matches!(location, crate::data::ItemLocation::Inventory))
            .count();
        format!(
            "turn {turn}  |  storm: {timer_remaining}  |  carrying: {inventory_count}  |  type 'help' for options, 'u' undo, 'q' quit",
        )
    }
}

impl Default for LighthouseState {
    fn default() -> Self {
        Self::new()
    }
}

impl State for LighthouseState {
    fn title(&self) -> &str {
        "The Lantern at Dunmere Point"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        self.ensure_started();
        self.needs_redraw = true;
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        if self.state.game_over.is_some() {
            if matches!(key, KeyCode::Escape) {
                world.resources.should_exit = true;
            }
            return;
        }
        match key {
            KeyCode::Escape => {
                world.resources.should_exit = true;
            }
            KeyCode::Enter => {
                self.submit_input();
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
                self.needs_redraw = true;
            }
            KeyCode::Char(character) => {
                self.input_buffer.push(character);
                self.needs_redraw = true;
            }
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.quit_requested {
            world.resources.should_exit = true;
            return;
        }
        let delta = world.resources.timing.delta_seconds;
        self.cursor_timer += delta;
        if self.cursor_timer >= 0.5 {
            self.cursor_timer = 0.0;
            self.cursor_visible = !self.cursor_visible;
            self.needs_redraw = true;
        }
        if self.needs_redraw {
            self.render(world);
            self.needs_redraw = false;
        }
    }
}

fn render_entry(out: &mut Vec<(String, TermColor)>, entry: &TranscriptEntry, max_width: usize) {
    match entry {
        TranscriptEntry::Narration(text) => {
            for line in wrap(text, max_width) {
                out.push((line, TermColor::White));
            }
            out.push((String::new(), TermColor::White));
        }
        TranscriptEntry::PlayerAction(text) => {
            for line in wrap(text, max_width) {
                out.push((line, TermColor::DarkGrey));
            }
        }
        TranscriptEntry::Dialogue { speaker, text } => {
            let combined = format!("{speaker}: {text}");
            for line in wrap(&combined, max_width) {
                out.push((line, TermColor::Cyan));
            }
            out.push((String::new(), TermColor::White));
        }
        TranscriptEntry::System(text) => {
            for line in wrap(text, max_width) {
                out.push((line, TermColor::Yellow));
            }
        }
        TranscriptEntry::Separator => {
            let rule: String = std::iter::repeat_n('-', max_width.min(48)).collect();
            out.push((rule, TermColor::DarkGrey));
        }
    }
}

fn wrap(text: &str, max_width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            if word.chars().count() > max_width {
                let mut remaining = word;
                while remaining.chars().count() > max_width {
                    let take: String = remaining.chars().take(max_width).collect();
                    lines.push(take);
                    let skip: usize = max_width;
                    remaining = match remaining.char_indices().nth(skip) {
                        Some((index, _)) => &remaining[index..],
                        None => "",
                    };
                }
                current = remaining.to_string();
            } else {
                current = word.to_string();
            }
        } else if current.chars().count() + 1 + word.chars().count() > max_width {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
