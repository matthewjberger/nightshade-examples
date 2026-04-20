use crate::colorizer::{ColoredLine, Keyword, build_keywords, colorize};
use crate::game;
use nightshade::interactive_fiction::data::{ChoiceAction, TranscriptEntry};
use nightshade::interactive_fiction::engine::Engine;
use nightshade::interactive_fiction::parser as input;
use nightshade::tui::prelude::*;

const NARRATION_COLOR: TermColor = TermColor::White;
const PLAYER_ACTION_COLOR: TermColor = TermColor::DarkGrey;
const DIALOGUE_SPEAKER_COLOR: TermColor = TermColor::Magenta;
const DIALOGUE_TEXT_COLOR: TermColor = TermColor::White;
const SYSTEM_COLOR: TermColor = TermColor::Yellow;
const SEPARATOR_COLOR: TermColor = TermColor::DarkGrey;
const STATUS_COLOR: TermColor = TermColor::Magenta;
const PROMPT_COLOR: TermColor = TermColor::Green;

pub struct ChimeranState {
    engine: Engine,
    state: nightshade::interactive_fiction::data::RuntimeState,
    entities: Vec<Entity>,
    needs_redraw: bool,
    input_buffer: String,
    cursor_visible: bool,
    cursor_timer: f64,
    started: bool,
    undo_stack: std::collections::VecDeque<nightshade::interactive_fiction::data::RuntimeState>,
    quit_requested: bool,
    keywords: Vec<Keyword>,
    display_start: usize,
    log_mode: bool,
    log_offset: usize,
}

impl ChimeranState {
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
        let keywords = build_keywords(&engine);
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
            keywords,
            display_start: 0,
            log_mode: false,
            log_offset: 0,
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
        self.engine.pick(&mut self.state, index);
        self.needs_redraw = true;
    }

    fn examine_all(&mut self) {
        if self.state.game_over.is_some() {
            return;
        }
        if self.exploit_window_open() {
            self.debit_counter(1);
        }
        self.engine.apply_all_examine(&mut self.state);
        self.needs_redraw = true;
    }

    fn exploit_window_open(&self) -> bool {
        matches!(
            self.state
                .flags
                .get(&crate::game::ids::flag_exploit_window_open()),
            Some(nightshade::interactive_fiction::data::Value::Bool(true))
        )
    }

    fn debit_counter(&mut self, amount: i64) {
        let key = crate::game::ids::stat_exploit_counter();
        let current = self.state.stats.get(&key).copied().unwrap_or(0);
        self.state.stats.insert(key, current - amount);
    }

    fn pick_all_matching<F>(&mut self, predicate: F)
    where
        F: Fn(&ChoiceAction) -> bool,
    {
        if self.state.game_over.is_some() {
            return;
        }
        loop {
            let choices = self.engine.available_choices(&self.state);
            let next = choices.iter().position(|choice| predicate(&choice.action));
            match next {
                Some(index) => {
                    self.engine.pick(&mut self.state, index);
                    if self.state.game_over.is_some() {
                        break;
                    }
                }
                None => break,
            }
        }
        self.needs_redraw = true;
    }

    fn submit_input(&mut self) {
        let raw = self.input_buffer.trim().to_string();
        self.input_buffer.clear();
        self.needs_redraw = true;

        if raw == "/log" {
            self.log_mode = !self.log_mode;
            self.log_offset = 0;
            return;
        }

        if raw == "/map" {
            let map = game::map::render(&self.state);
            self.state.push_transcript(TranscriptEntry::System(map));
            return;
        }

        let room_before = self.state.current_room.clone();
        let transcript_len_before = self.state.transcript.len();

        let typed_noun = input::extract_noun(&raw);

        let choices = self.engine.available_choices(&self.state);
        let parsed = input::parse(&self.engine, &self.state, &choices, &raw);

        let parse_accepted = !matches!(
            parsed,
            input::Parsed::NoMatch | input::Parsed::Ambiguous | input::Parsed::Empty
        );
        let updates_last_noun = parse_accepted
            && !matches!(
                parsed,
                input::Parsed::DescribeRoom
                    | input::Parsed::Undo
                    | input::Parsed::Quit
                    | input::Parsed::Help
            );
        if updates_last_noun && let Some(noun) = typed_noun {
            self.state.last_noun = Some(noun);
        }

        let produces_output = !matches!(
            parsed,
            input::Parsed::Empty | input::Parsed::Quit | input::Parsed::Undo
        );
        if produces_output {
            self.snapshot();
        }

        if !matches!(parsed, input::Parsed::Empty) && !raw.is_empty() {
            self.state
                .push_transcript(TranscriptEntry::PlayerAction(format!("> {raw}")));
        }

        match parsed {
            input::Parsed::Empty => {}
            input::Parsed::Quit => {
                self.quit_requested = true;
            }
            input::Parsed::Undo => self.undo(),
            input::Parsed::Help => self.show_help(&choices),
            input::Parsed::Choose(index) => self.pick_index(index),
            input::Parsed::TakeAll => {
                self.pick_all_matching(|action| matches!(action, ChoiceAction::Take(_)))
            }
            input::Parsed::DropAll => {
                self.pick_all_matching(|action| matches!(action, ChoiceAction::Drop(_)))
            }
            input::Parsed::ExamineAll => self.examine_all(),
            input::Parsed::DescribeRoom => self.engine.describe_current_room(&mut self.state),
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
            input::Parsed::Refuse(line) => {
                self.state.push_transcript(TranscriptEntry::Narration(line));
            }
        }

        if self.state.current_room != room_before {
            self.display_start = transcript_len_before;
        }
    }

    fn show_help(&mut self, choices: &[nightshade::interactive_fiction::data::Choice]) {
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
            "Also: 'undo' (u), 'quit' (q), 'take all', 'drop all'.".to_string(),
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

        let status_row = 0;
        let transcript_start_row = if self.log_mode { 0 } else { 1 };
        let transcript_rows_available = if self.log_mode {
            rows.saturating_sub(1)
        } else {
            rows.saturating_sub(2)
        };

        let skip = if self.log_mode { 0 } else { self.display_start };
        let mut wrapped: Vec<ColoredLine> = Vec::new();
        for entry in self.state.transcript.iter().skip(skip) {
            render_entry(&mut wrapped, entry, text_width, &self.keywords);
        }

        let tail_start = wrapped.len().saturating_sub(transcript_rows_available);
        let start_index = if self.log_mode {
            tail_start.saturating_sub(self.log_offset)
        } else {
            tail_start
        };
        let end_index = if self.log_mode {
            (start_index + transcript_rows_available).min(wrapped.len())
        } else {
            wrapped.len()
        };
        let visible = &wrapped[start_index..end_index];
        for (row_offset, line) in visible.iter().enumerate() {
            self.draw_colored_line(world, padding, transcript_start_row + row_offset, line);
        }

        let prompt_row = (transcript_start_row + visible.len()).min(rows.saturating_sub(1));
        let prompt = if self.state.game_over.is_some() {
            "(ESC to exit)".to_string()
        } else if self.log_mode {
            format!(
                "-- LOG (↑/↓ scroll, PgUp/PgDn page, Enter/Esc exit, offset +{}) --",
                self.log_offset
            )
        } else {
            format!(
                "> {}{}",
                self.input_buffer,
                if self.cursor_visible { "_" } else { " " }
            )
        };
        self.draw_text(world, padding, prompt_row, &prompt, PROMPT_COLOR);

        if !self.log_mode {
            let status = self.render_status();
            self.draw_text(world, padding, status_row, &status, STATUS_COLOR);
        }
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
            self.draw_char(world, x, row, character, color);
        }
    }

    fn draw_colored_line(
        &mut self,
        world: &mut World,
        column: usize,
        row: usize,
        line: &ColoredLine,
    ) {
        let terminal_columns = world.resources.terminal_size.columns as usize;
        for (offset, (character, color)) in line.iter().enumerate() {
            let x = column + offset;
            if x >= terminal_columns {
                break;
            }
            self.draw_char(world, x, row, *character, *color);
        }
    }

    fn draw_char(
        &mut self,
        world: &mut World,
        column: usize,
        row: usize,
        ch: char,
        color: TermColor,
    ) {
        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
        world.set_position(
            entity,
            Position {
                column: column as f64,
                row: row as f64,
            },
        );
        world.set_sprite(
            entity,
            Sprite {
                character: ch,
                foreground: color,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(1));
        self.entities.push(entity);
    }

    fn render_status(&self) -> String {
        let turn = self.state.turn;
        let cycle = self
            .state
            .stats
            .get(&crate::game::ids::stat_cycle())
            .copied()
            .unwrap_or(0);
        let counter = self
            .state
            .stats
            .get(&crate::game::ids::stat_exploit_counter())
            .copied()
            .unwrap_or(0);
        let exploit_on = matches!(
            self.state
                .flags
                .get(&crate::game::ids::flag_exploit_window_open()),
            Some(nightshade::interactive_fiction::data::Value::Bool(true))
        );
        if exploit_on {
            format!(
                "turn {turn}  |  cycle {cycle}  |  [SUBSTRATE WINDOW: {counter} actions remaining]  |  'help' 'u' 'q'",
            )
        } else {
            let unread = game::mail::unread_count(&self.state);
            let mail_segment = if unread > 0 {
                format!("  |  {unread} unread mail")
            } else {
                String::new()
            };
            format!(
                "turn {turn}  |  cycle {cycle}{mail_segment}  |  type 'help' for options, 'u' undo, 'q' quit",
            )
        }
    }
}

impl Default for ChimeranState {
    fn default() -> Self {
        Self::new()
    }
}

impl State for ChimeranState {
    fn title(&self) -> &str {
        "Chimeran"
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
        if self.log_mode {
            match key {
                KeyCode::Escape | KeyCode::Enter => {
                    self.log_mode = false;
                    self.needs_redraw = true;
                }
                KeyCode::Up => {
                    self.log_offset = self.log_offset.saturating_add(1);
                    self.needs_redraw = true;
                }
                KeyCode::Down => {
                    self.log_offset = self.log_offset.saturating_sub(1);
                    self.needs_redraw = true;
                }
                KeyCode::PageUp => {
                    self.log_offset = self.log_offset.saturating_add(10);
                    self.needs_redraw = true;
                }
                KeyCode::PageDown => {
                    self.log_offset = self.log_offset.saturating_sub(10);
                    self.needs_redraw = true;
                }
                _ => {}
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

fn render_entry(
    out: &mut Vec<ColoredLine>,
    entry: &TranscriptEntry,
    max_width: usize,
    keywords: &[Keyword],
) {
    match entry {
        TranscriptEntry::Narration(text) => {
            for line in wrap(text, max_width) {
                out.push(colorize(&line, NARRATION_COLOR, keywords));
            }
            out.push(Vec::new());
        }
        TranscriptEntry::PlayerAction(text) => {
            for line in wrap(text, max_width) {
                out.push(colorize(&line, PLAYER_ACTION_COLOR, keywords));
            }
        }
        TranscriptEntry::Dialogue { speaker, text } => {
            let prefix = format!("{speaker}: ");
            let lines = wrap(&format!("{prefix}{text}"), max_width);
            for (index, line) in lines.iter().enumerate() {
                let mut colored = colorize(line, DIALOGUE_TEXT_COLOR, keywords);
                if index == 0 {
                    let speaker_chars = prefix.chars().count();
                    for (character_index, (_, color)) in
                        colored.iter_mut().enumerate().take(speaker_chars)
                    {
                        *color = DIALOGUE_SPEAKER_COLOR;
                        let _ = character_index;
                    }
                }
                out.push(colored);
            }
            out.push(Vec::new());
        }
        TranscriptEntry::System(text) => {
            for line in wrap(text, max_width) {
                out.push(colorize(&line, SYSTEM_COLOR, keywords));
            }
        }
        TranscriptEntry::Separator => {
            let rule: String = std::iter::repeat_n('-', max_width.min(48)).collect();
            out.push(colorize(&rule, SEPARATOR_COLOR, keywords));
        }
    }
}

fn wrap(text: &str, max_width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
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
    }
    lines
}
