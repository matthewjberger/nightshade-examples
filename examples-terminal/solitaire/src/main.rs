use nightshade::tui::prelude::*;
use rand::Rng;

const CARD_WIDTH: i32 = 4;
const TABLEAU_COUNT: usize = 7;
const FOUNDATION_COUNT: usize = 4;
const STOCK_ROW: i32 = 0;
const TABLEAU_START_ROW: i32 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds,
}

impl Suit {
    fn symbol(self) -> char {
        match self {
            Suit::Spades => 's',
            Suit::Hearts => 'h',
            Suit::Clubs => 'c',
            Suit::Diamonds => 'd',
        }
    }

    fn color(self) -> TermColor {
        match self {
            Suit::Hearts | Suit::Diamonds => TermColor::Red,
            Suit::Spades | Suit::Clubs => TermColor::White,
        }
    }

    fn is_red(self) -> bool {
        matches!(self, Suit::Hearts | Suit::Diamonds)
    }
}

const ALL_SUITS: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Card {
    rank: u8,
    suit: Suit,
    face_up: bool,
}

impl Card {
    fn rank_label(self) -> &'static str {
        match self.rank {
            1 => "A",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "T",
            11 => "J",
            12 => "Q",
            13 => "K",
            _ => "?",
        }
    }

    fn display_padded(self) -> String {
        if self.face_up {
            format!("{}{} ", self.rank_label(), self.suit.symbol())
        } else {
            "## ".to_string()
        }
    }

    fn foreground(self) -> TermColor {
        if self.face_up {
            self.suit.color()
        } else {
            TermColor::Rgb {
                r: 80,
                g: 80,
                b: 200,
            }
        }
    }

    fn background(self) -> TermColor {
        if self.face_up {
            TermColor::Rgb {
                r: 20,
                g: 20,
                b: 30,
            }
        } else {
            TermColor::Rgb {
                r: 30,
                g: 30,
                b: 60,
            }
        }
    }
}

fn create_shuffled_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(52);
    for &suit in &ALL_SUITS {
        for rank in 1..=13u8 {
            deck.push(Card {
                rank,
                suit,
                face_up: false,
            });
        }
    }
    let mut rng = rand::rng();
    for index in (1..deck.len()).rev() {
        let swap_index = rng.random_range(0..=index);
        deck.swap(index, swap_index);
    }
    deck
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PileLocation {
    Stock,
    Waste,
    Foundation(usize),
    Tableau(usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CursorLocation {
    Stock,
    Waste,
    Foundation(usize),
    Tableau(usize, usize),
}

struct Selection {
    location: PileLocation,
    card_index: usize,
    card_count: usize,
}

struct PileDisplayInfo {
    last_card: Option<Card>,
    pile_length: usize,
    screen_column: i32,
    screen_row: i32,
    empty_label: &'static str,
    is_cursor: bool,
    is_selected: bool,
    show_count: bool,
}

struct GameBoard {
    stock: Vec<Card>,
    waste: Vec<Card>,
    foundations: [Vec<Card>; FOUNDATION_COUNT],
    tableau: [Vec<Card>; TABLEAU_COUNT],
}

impl GameBoard {
    fn new() -> Self {
        let mut deck = create_shuffled_deck();
        let mut tableau: [Vec<Card>; TABLEAU_COUNT] = Default::default();

        for (column_index, column) in tableau.iter_mut().enumerate() {
            for card_position in 0..=column_index {
                if let Some(mut card) = deck.pop() {
                    if card_position == column_index {
                        card.face_up = true;
                    }
                    column.push(card);
                }
            }
        }

        Self {
            stock: deck,
            waste: Vec::new(),
            foundations: Default::default(),
            tableau,
        }
    }

    fn draw_from_stock(&mut self) {
        if self.stock.is_empty() {
            while let Some(mut card) = self.waste.pop() {
                card.face_up = false;
                self.stock.push(card);
            }
        } else if let Some(mut card) = self.stock.pop() {
            card.face_up = true;
            self.waste.push(card);
        }
    }

    fn can_place_on_foundation(&self, card: Card, foundation_index: usize) -> bool {
        let foundation = &self.foundations[foundation_index];
        if foundation.is_empty() {
            card.rank == 1
        } else {
            let top = foundation.last().unwrap();
            top.suit == card.suit && card.rank == top.rank + 1
        }
    }

    fn can_place_on_tableau(&self, card: Card, tableau_index: usize) -> bool {
        let column = &self.tableau[tableau_index];
        if column.is_empty() {
            card.rank == 13
        } else {
            let top = column.last().unwrap();
            top.face_up && top.suit.is_red() != card.suit.is_red() && card.rank + 1 == top.rank
        }
    }

    fn auto_foundation_candidate(&self, card: Card) -> Option<usize> {
        for foundation_index in 0..FOUNDATION_COUNT {
            if self.can_place_on_foundation(card, foundation_index) {
                let foundation = &self.foundations[foundation_index];
                if foundation.is_empty() && card.rank == 1 {
                    return Some(foundation_index);
                }
                if !foundation.is_empty() {
                    let top = foundation.last().unwrap();
                    if top.suit == card.suit && card.rank == top.rank + 1 {
                        return Some(foundation_index);
                    }
                }
            }
        }
        None
    }

    fn try_move_to_foundation(&mut self, source: PileLocation, card_index: usize) -> bool {
        let source_card = match source {
            PileLocation::Waste => self.waste.last().copied(),
            PileLocation::Tableau(tableau_index) => {
                let column = &self.tableau[tableau_index];
                if card_index < column.len() && card_index == column.len() - 1 {
                    Some(column[card_index])
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(card) = source_card
            && let Some(foundation_index) = self.auto_foundation_candidate(card)
        {
            match source {
                PileLocation::Waste => {
                    self.waste.pop();
                }
                PileLocation::Tableau(tableau_index) => {
                    self.tableau[tableau_index].pop();
                    self.flip_top_tableau(tableau_index);
                }
                _ => {}
            }
            self.foundations[foundation_index].push(card);
            return true;
        }
        false
    }

    fn try_move_cards(
        &mut self,
        source: PileLocation,
        source_card_index: usize,
        source_card_count: usize,
        destination: PileLocation,
    ) -> bool {
        match (source, destination) {
            (PileLocation::Waste, PileLocation::Tableau(destination_index)) => {
                if source_card_count != 1 {
                    return false;
                }
                if let Some(&card) = self.waste.last()
                    && self.can_place_on_tableau(card, destination_index)
                {
                    let card = self.waste.pop().unwrap();
                    self.tableau[destination_index].push(card);
                    return true;
                }
                false
            }
            (PileLocation::Waste, PileLocation::Foundation(foundation_index)) => {
                if source_card_count != 1 {
                    return false;
                }
                if let Some(&card) = self.waste.last()
                    && self.can_place_on_foundation(card, foundation_index)
                {
                    let card = self.waste.pop().unwrap();
                    self.foundations[foundation_index].push(card);
                    return true;
                }
                false
            }
            (PileLocation::Tableau(source_index), PileLocation::Tableau(destination_index)) => {
                if source_index == destination_index {
                    return false;
                }
                let column = &self.tableau[source_index];
                if source_card_index >= column.len() {
                    return false;
                }
                let moving_card = column[source_card_index];
                if !moving_card.face_up {
                    return false;
                }
                if !self.can_place_on_tableau(moving_card, destination_index) {
                    return false;
                }
                let cards_to_move: Vec<Card> = self.tableau[source_index]
                    .drain(source_card_index..)
                    .collect();
                self.tableau[destination_index].extend(cards_to_move);
                self.flip_top_tableau(source_index);
                true
            }
            (PileLocation::Tableau(source_index), PileLocation::Foundation(foundation_index)) => {
                let column = &self.tableau[source_index];
                if source_card_count != 1 || source_card_index != column.len().saturating_sub(1) {
                    return false;
                }
                if let Some(&card) = column.last()
                    && self.can_place_on_foundation(card, foundation_index)
                {
                    let card = self.tableau[source_index].pop().unwrap();
                    self.foundations[foundation_index].push(card);
                    self.flip_top_tableau(source_index);
                    return true;
                }
                false
            }
            (
                PileLocation::Foundation(source_foundation),
                PileLocation::Tableau(destination_index),
            ) => {
                if let Some(&card) = self.foundations[source_foundation].last()
                    && self.can_place_on_tableau(card, destination_index)
                {
                    let card = self.foundations[source_foundation].pop().unwrap();
                    self.tableau[destination_index].push(card);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn flip_top_tableau(&mut self, tableau_index: usize) {
        if let Some(card) = self.tableau[tableau_index].last_mut() {
            card.face_up = true;
        }
    }

    fn is_won(&self) -> bool {
        self.foundations
            .iter()
            .all(|foundation| foundation.len() == 13)
    }
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Solitaire - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title_lines = [
            r" ____        _ _ _        _          ",
            r"/ ___|  ___ | (_) |_ __ _(_)_ __ ___ ",
            r"\___ \ / _ \| | | __/ _` | | '__/ _ \",
            r" ___) | (_) | | | || (_| | | | |  __/",
            r"|____/ \___/|_|_|\__\__,_|_|_|  \___|",
        ];

        let title_start_row = center_row - 7.0;

        for (line_index, line) in title_lines.iter().enumerate() {
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - line.len() as f64 / 2.0,
                    row: title_start_row + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: line.to_string(),
                    foreground: TermColor::Rgb {
                        r: 100,
                        g: 200,
                        b: 100,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }

        let card_display = "As  Kh  Qc  Jd";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - card_display.len() as f64 / 2.0,
                row: title_start_row + 7.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: card_display.to_string(),
                foreground: TermColor::Rgb {
                    r: 200,
                    g: 200,
                    b: 200,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let subtitle = "Klondike Solitaire";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - subtitle.len() as f64 / 2.0,
                row: title_start_row + 9.0,
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
                row: title_start_row + 11.0,
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

        let quit_hint = "Press ESC to quit";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - quit_hint.len() as f64 / 2.0,
                row: title_start_row + 13.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: quit_hint.to_string(),
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
    board: GameBoard,
    entities: EntityGroup,
    cursor: CursorLocation,
    selection: Option<Selection>,
    board_offset_column: i32,
    board_offset_row: i32,
    needs_redraw: bool,
    transition_to_win: bool,
    message: String,
    message_timer: f64,
    moves_count: u32,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            board: GameBoard::new(),
            entities: EntityGroup::new(),
            cursor: CursorLocation::Stock,
            selection: None,
            board_offset_column: 0,
            board_offset_row: 0,
            needs_redraw: true,
            transition_to_win: false,
            message: String::new(),
            message_timer: 0.0,
            moves_count: 0,
        }
    }

    fn pile_screen_column(&self, pile_index: usize) -> i32 {
        self.board_offset_column + pile_index as i32 * (CARD_WIDTH + 1)
    }

    fn handle_select(&mut self) {
        match self.cursor {
            CursorLocation::Stock => {
                self.board.draw_from_stock();
                self.selection = None;
                self.needs_redraw = true;
            }
            CursorLocation::Waste => {
                if self.board.waste.is_empty() {
                    return;
                }
                if self.selection.is_some() {
                    self.selection = None;
                    self.needs_redraw = true;
                    return;
                }
                let card_index = self.board.waste.len() - 1;
                self.selection = Some(Selection {
                    location: PileLocation::Waste,
                    card_index,
                    card_count: 1,
                });
                self.needs_redraw = true;
            }
            CursorLocation::Foundation(foundation_index) => {
                if let Some(selection) = self.selection.take() {
                    let destination = PileLocation::Foundation(foundation_index);
                    if self.board.try_move_cards(
                        selection.location,
                        selection.card_index,
                        selection.card_count,
                        destination,
                    ) {
                        self.moves_count += 1;
                    }
                    self.needs_redraw = true;
                } else {
                    if self.board.foundations[foundation_index].is_empty() {
                        return;
                    }
                    let card_index = self.board.foundations[foundation_index].len() - 1;
                    self.selection = Some(Selection {
                        location: PileLocation::Foundation(foundation_index),
                        card_index,
                        card_count: 1,
                    });
                    self.needs_redraw = true;
                }
            }
            CursorLocation::Tableau(column_index, row_index) => {
                if let Some(selection) = self.selection.take() {
                    let destination = PileLocation::Tableau(column_index);
                    if self.board.try_move_cards(
                        selection.location,
                        selection.card_index,
                        selection.card_count,
                        destination,
                    ) {
                        self.moves_count += 1;
                    }
                    self.needs_redraw = true;
                } else {
                    let column = &self.board.tableau[column_index];
                    if column.is_empty() {
                        return;
                    }
                    let clamped_row = row_index.min(column.len().saturating_sub(1));
                    if !column[clamped_row].face_up {
                        return;
                    }
                    let card_count = column.len() - clamped_row;
                    self.selection = Some(Selection {
                        location: PileLocation::Tableau(column_index),
                        card_index: clamped_row,
                        card_count,
                    });
                    self.needs_redraw = true;
                }
            }
        }
    }

    fn handle_double_select(&mut self) {
        match self.cursor {
            CursorLocation::Waste => {
                if !self.board.waste.is_empty() {
                    let card_index = self.board.waste.len() - 1;
                    if self
                        .board
                        .try_move_to_foundation(PileLocation::Waste, card_index)
                    {
                        self.moves_count += 1;
                        self.selection = None;
                        self.needs_redraw = true;
                    }
                }
            }
            CursorLocation::Tableau(column_index, _row_index) => {
                let column = &self.board.tableau[column_index];
                if !column.is_empty() {
                    let card_index = column.len() - 1;
                    if self
                        .board
                        .try_move_to_foundation(PileLocation::Tableau(column_index), card_index)
                    {
                        self.moves_count += 1;
                        self.selection = None;
                        self.needs_redraw = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn move_cursor_left(&mut self) {
        self.cursor = match self.cursor {
            CursorLocation::Stock => CursorLocation::Stock,
            CursorLocation::Waste => CursorLocation::Stock,
            CursorLocation::Foundation(0) => CursorLocation::Waste,
            CursorLocation::Foundation(index) => CursorLocation::Foundation(index - 1),
            CursorLocation::Tableau(0, row) => CursorLocation::Tableau(0, row),
            CursorLocation::Tableau(column, row) => CursorLocation::Tableau(column - 1, row),
        };
        self.clamp_tableau_cursor_row();
        self.needs_redraw = true;
    }

    fn move_cursor_right(&mut self) {
        self.cursor = match self.cursor {
            CursorLocation::Stock => CursorLocation::Waste,
            CursorLocation::Waste => CursorLocation::Foundation(0),
            CursorLocation::Foundation(index) if index < FOUNDATION_COUNT - 1 => {
                CursorLocation::Foundation(index + 1)
            }
            CursorLocation::Foundation(_) => CursorLocation::Foundation(FOUNDATION_COUNT - 1),
            CursorLocation::Tableau(column, row) if column < TABLEAU_COUNT - 1 => {
                CursorLocation::Tableau(column + 1, row)
            }
            other => other,
        };
        self.clamp_tableau_cursor_row();
        self.needs_redraw = true;
    }

    fn move_cursor_up(&mut self) {
        self.cursor = match self.cursor {
            CursorLocation::Stock => CursorLocation::Stock,
            CursorLocation::Waste => CursorLocation::Waste,
            CursorLocation::Foundation(_) => {
                CursorLocation::Foundation(self.foundation_index_for_cursor())
            }
            CursorLocation::Tableau(column, 0) => {
                if column <= 1 {
                    if column == 0 {
                        CursorLocation::Stock
                    } else {
                        CursorLocation::Waste
                    }
                } else {
                    CursorLocation::Foundation((column - 3).min(FOUNDATION_COUNT - 1))
                }
            }
            CursorLocation::Tableau(column, row) => CursorLocation::Tableau(column, row - 1),
        };
        self.clamp_tableau_cursor_row();
        self.needs_redraw = true;
    }

    fn move_cursor_down(&mut self) {
        self.cursor = match self.cursor {
            CursorLocation::Stock => CursorLocation::Tableau(0, 0),
            CursorLocation::Waste => CursorLocation::Tableau(1, 0),
            CursorLocation::Foundation(index) => CursorLocation::Tableau(index + 3, 0),
            CursorLocation::Tableau(column, row) => {
                let max_row = self.board.tableau[column].len().saturating_sub(1);
                CursorLocation::Tableau(column, (row + 1).min(max_row))
            }
        };
        self.clamp_tableau_cursor_row();
        self.needs_redraw = true;
    }

    fn foundation_index_for_cursor(&self) -> usize {
        match self.cursor {
            CursorLocation::Foundation(index) => index,
            _ => 0,
        }
    }

    fn clamp_tableau_cursor_row(&mut self) {
        if let CursorLocation::Tableau(column, row) = self.cursor {
            let column_len = self.board.tableau[column].len();
            if column_len == 0 {
                self.cursor = CursorLocation::Tableau(column, 0);
            } else {
                let max_row = column_len - 1;
                if row > max_row {
                    self.cursor = CursorLocation::Tableau(column, max_row);
                }
            }
        }
    }

    fn screen_pos_to_cursor(&self, screen_column: u16, screen_row: u16) -> Option<CursorLocation> {
        let relative_column = screen_column as i32 - self.board_offset_column;
        let relative_row = screen_row as i32 - self.board_offset_row;

        if (STOCK_ROW..STOCK_ROW + 2).contains(&relative_row) {
            let stock_column_start = 0;
            let stock_column_end = CARD_WIDTH;
            if relative_column >= stock_column_start && relative_column < stock_column_end {
                return Some(CursorLocation::Stock);
            }

            let waste_column_start = CARD_WIDTH + 1;
            let waste_column_end = waste_column_start + CARD_WIDTH;
            if relative_column >= waste_column_start && relative_column < waste_column_end {
                return Some(CursorLocation::Waste);
            }

            for foundation_index in 0..FOUNDATION_COUNT {
                let foundation_pile_index = 3 + foundation_index;
                let foundation_column_start = foundation_pile_index as i32 * (CARD_WIDTH + 1);
                let foundation_column_end = foundation_column_start + CARD_WIDTH;
                if relative_column >= foundation_column_start
                    && relative_column < foundation_column_end
                {
                    return Some(CursorLocation::Foundation(foundation_index));
                }
            }
        }

        if relative_row >= TABLEAU_START_ROW {
            let tableau_row_index = (relative_row - TABLEAU_START_ROW) as usize;
            for column_index in 0..TABLEAU_COUNT {
                let column_start = column_index as i32 * (CARD_WIDTH + 1);
                let column_end = column_start + CARD_WIDTH;
                if relative_column >= column_start && relative_column < column_end {
                    return Some(CursorLocation::Tableau(column_index, tableau_row_index));
                }
            }
        }

        None
    }

    fn render_board(&mut self, world: &mut World) {
        if !self.needs_redraw {
            return;
        }
        self.needs_redraw = false;
        self.entities.despawn_all(world);

        let top_row_screen = self.board_offset_row + STOCK_ROW;

        let stock_info = PileDisplayInfo {
            last_card: self.board.stock.last().copied(),
            pile_length: self.board.stock.len(),
            screen_column: self.pile_screen_column(0),
            screen_row: top_row_screen,
            empty_label: "[S]",
            is_cursor: self.is_cursor_at(CursorLocation::Stock),
            is_selected: self.is_selected(PileLocation::Stock),
            show_count: true,
        };
        Self::render_pile_top_inline(&mut self.entities, world, &stock_info);

        let waste_info = PileDisplayInfo {
            last_card: self.board.waste.last().copied(),
            pile_length: self.board.waste.len(),
            screen_column: self.pile_screen_column(1),
            screen_row: top_row_screen,
            empty_label: "   ",
            is_cursor: self.is_cursor_at(CursorLocation::Waste),
            is_selected: self.is_selected(PileLocation::Waste),
            show_count: false,
        };
        Self::render_pile_top_inline(&mut self.entities, world, &waste_info);

        let foundation_labels: [&str; FOUNDATION_COUNT] = [" s ", " h ", " c ", " d "];
        for (foundation_index, &label) in foundation_labels.iter().enumerate() {
            let info = PileDisplayInfo {
                last_card: self.board.foundations[foundation_index].last().copied(),
                pile_length: self.board.foundations[foundation_index].len(),
                screen_column: self.pile_screen_column(3 + foundation_index),
                screen_row: top_row_screen,
                empty_label: label,
                is_cursor: self.is_cursor_at(CursorLocation::Foundation(foundation_index)),
                is_selected: self.is_selected(PileLocation::Foundation(foundation_index)),
                show_count: false,
            };
            Self::render_pile_top_inline(&mut self.entities, world, &info);
        }

        let separator_row = self.board_offset_row + STOCK_ROW + 2;
        let total_width = TABLEAU_COUNT as i32 * (CARD_WIDTH + 1) - 1;
        let separator_text: String = (0..total_width).map(|_| '-').collect();
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: self.board_offset_column as f64,
                row: separator_row as f64,
            },
        );
        world.set_label(
            entity,
            Label {
                text: separator_text,
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(1));

        for column_index in 0..TABLEAU_COUNT {
            let screen_column = self.pile_screen_column(column_index);
            let column = &self.board.tableau[column_index];

            if column.is_empty() {
                let is_cursor_here = matches!(self.cursor, CursorLocation::Tableau(cursor_column, _) if cursor_column == column_index);
                let foreground = if is_cursor_here {
                    TermColor::Yellow
                } else {
                    TermColor::DarkGrey
                };
                let background = if is_cursor_here {
                    TermColor::Rgb { r: 50, g: 50, b: 0 }
                } else {
                    TermColor::Black
                };
                let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
                world.set_position(
                    entity,
                    Position {
                        column: screen_column as f64,
                        row: (self.board_offset_row + TABLEAU_START_ROW) as f64,
                    },
                );
                world.set_label(
                    entity,
                    Label {
                        text: " _ ".to_string(),
                        foreground,
                        background,
                    },
                );
                world.set_z_index(entity, ZIndex(1));
                continue;
            }

            for (card_row_index, card) in column.iter().enumerate() {
                let screen_row = self.board_offset_row + TABLEAU_START_ROW + card_row_index as i32;
                let is_cursor_here = matches!(
                    self.cursor,
                    CursorLocation::Tableau(cursor_column, cursor_row)
                    if cursor_column == column_index && cursor_row == card_row_index
                );
                let is_selected =
                    self.is_card_in_selection(PileLocation::Tableau(column_index), card_row_index);

                let mut foreground = card.foreground();
                let mut background = card.background();

                if is_selected {
                    background = TermColor::Rgb {
                        r: 60,
                        g: 60,
                        b: 20,
                    };
                }
                if is_cursor_here {
                    background = TermColor::Rgb { r: 50, g: 50, b: 0 };
                    if !card.face_up {
                        foreground = TermColor::Yellow;
                    }
                }

                let display = card.display_padded();
                let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
                world.set_position(
                    entity,
                    Position {
                        column: screen_column as f64,
                        row: screen_row as f64,
                    },
                );
                world.set_label(
                    entity,
                    Label {
                        text: display,
                        foreground,
                        background,
                    },
                );
                world.set_z_index(entity, ZIndex(2 + card_row_index as i32));
            }
        }

        let hud_row = self.board_offset_row - 2;
        let moves_text = format!("Moves: {}", self.moves_count);
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: self.board_offset_column as f64,
                row: hud_row as f64,
            },
        );
        world.set_label(
            entity,
            Label {
                text: moves_text,
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(1));

        let foundation_count: usize = self
            .board
            .foundations
            .iter()
            .map(|foundation| foundation.len())
            .sum();
        let progress_text = format!("Foundation: {}/52", foundation_count);
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: (self.board_offset_column + 14) as f64,
                row: hud_row as f64,
            },
        );
        world.set_label(
            entity,
            Label {
                text: progress_text,
                foreground: TermColor::Rgb {
                    r: 100,
                    g: 200,
                    b: 100,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(1));

        let max_tableau_height = self
            .board
            .tableau
            .iter()
            .map(|column| column.len())
            .max()
            .unwrap_or(0);
        let footer_row = self.board_offset_row + TABLEAU_START_ROW + max_tableau_height as i32 + 1;
        let controls_text =
            "Arrows: Move | Enter: Select/Place | F: Auto-Foundation | R: Restart | ESC: Quit";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: self.board_offset_column as f64,
                row: footer_row.max(self.board_offset_row + 15) as f64,
            },
        );
        world.set_label(
            entity,
            Label {
                text: controls_text.to_string(),
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(1));

        if !self.message.is_empty() {
            let message_row = hud_row - 1;
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: self.board_offset_column as f64,
                    row: message_row as f64,
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

    fn render_pile_top_inline(
        entities: &mut EntityGroup,
        world: &mut World,
        info: &PileDisplayInfo,
    ) {
        let (text, foreground, mut background) = if let Some(card) = info.last_card {
            (card.display_padded(), card.foreground(), card.background())
        } else {
            (
                format!("{} ", info.empty_label),
                TermColor::DarkGrey,
                TermColor::Black,
            )
        };

        if info.is_selected {
            background = TermColor::Rgb {
                r: 60,
                g: 60,
                b: 20,
            };
        }
        if info.is_cursor {
            background = TermColor::Rgb { r: 50, g: 50, b: 0 };
        }

        let entity = entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: info.screen_column as f64,
                row: info.screen_row as f64,
            },
        );
        world.set_label(
            entity,
            Label {
                text,
                foreground,
                background,
            },
        );
        world.set_z_index(entity, ZIndex(2));

        if info.show_count && info.pile_length > 0 {
            let count_text = format!("({})", info.pile_length);
            let entity = entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: info.screen_column as f64,
                    row: (info.screen_row + 1) as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: count_text,
                    foreground: TermColor::DarkGrey,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(2));
        }
    }

    fn is_cursor_at(&self, location: CursorLocation) -> bool {
        match (self.cursor, location) {
            (CursorLocation::Stock, CursorLocation::Stock) => true,
            (CursorLocation::Waste, CursorLocation::Waste) => true,
            (
                CursorLocation::Foundation(cursor_index),
                CursorLocation::Foundation(target_index),
            ) => cursor_index == target_index,
            _ => false,
        }
    }

    fn is_selected(&self, location: PileLocation) -> bool {
        if let Some(ref selection) = self.selection {
            selection.location == location
        } else {
            false
        }
    }

    fn is_card_in_selection(&self, pile: PileLocation, card_index: usize) -> bool {
        if let Some(ref selection) = self.selection {
            if selection.location == pile {
                card_index >= selection.card_index
                    && card_index < selection.card_index + selection.card_count
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Solitaire - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        let total_width = TABLEAU_COUNT as i32 * (CARD_WIDTH + 1) - 1;
        self.board_offset_column = ((terminal.columns as i32 - total_width) / 2).max(1);
        self.board_offset_row = 3;

        self.needs_redraw = true;
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }

        match key {
            KeyCode::Left | KeyCode::Char('a') => self.move_cursor_left(),
            KeyCode::Right | KeyCode::Char('d') => self.move_cursor_right(),
            KeyCode::Up | KeyCode::Char('w') => self.move_cursor_up(),
            KeyCode::Down | KeyCode::Char('s') => self.move_cursor_down(),
            KeyCode::Enter | KeyCode::Char(' ') => self.handle_select(),
            KeyCode::Char('f') => self.handle_double_select(),
            KeyCode::Escape | KeyCode::Char('q') => {
                self.selection = None;
                if matches!(key, KeyCode::Escape) {
                    world.resources.should_exit = true;
                }
                self.needs_redraw = true;
            }
            KeyCode::Char('r') => {
                self.board = GameBoard::new();
                self.selection = None;
                self.cursor = CursorLocation::Stock;
                self.moves_count = 0;
                self.message.clear();
                self.needs_redraw = true;
            }
            _ => {}
        }
    }

    fn on_mouse_input(
        &mut self,
        _world: &mut World,
        button: MouseButton,
        column: u16,
        row: u16,
        pressed: bool,
    ) {
        if !pressed {
            return;
        }
        if button != MouseButton::Left {
            return;
        }

        if let Some(target) = self.screen_pos_to_cursor(column, row) {
            self.cursor = target;
            self.clamp_tableau_cursor_row();
            self.handle_select();
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.message_timer > 0.0 {
            self.message_timer -= world.resources.timing.delta_seconds;
            if self.message_timer <= 0.0 {
                self.message.clear();
                self.needs_redraw = true;
            }
        }

        if self.board.is_won() {
            self.transition_to_win = true;
        }

        self.render_board(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.transition_to_win {
            self.entities.despawn_all(world);
            return Some(Box::new(WinState {
                entities: EntityGroup::new(),
                restart: false,
                moves_count: self.moves_count,
            }));
        }
        None
    }
}

struct WinState {
    entities: EntityGroup,
    restart: bool,
    moves_count: u32,
}

impl State for WinState {
    fn title(&self) -> &str {
        "Solitaire - You Win!"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let win_lines = [
            r"__   __            __        ___       _ ",
            r"\ \ / /__  _   _  \ \      / (_)_ __ | |",
            r" \ V / _ \| | | |  \ \ /\ / /| | '_ \| |",
            r"  | | (_) | |_| |   \ V  V / | | | | |_|",
            r"  |_|\___/ \__,_|    \_/\_/  |_|_| |_(_)",
        ];

        let title_start_row = center_row - 6.0;

        for (line_index, line) in win_lines.iter().enumerate() {
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - line.len() as f64 / 2.0,
                    row: title_start_row + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: line.to_string(),
                    foreground: TermColor::Rgb {
                        r: 100,
                        g: 255,
                        b: 100,
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }

        let moves_text = format!("Completed in {} moves", self.moves_count);
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - moves_text.len() as f64 / 2.0,
                row: title_start_row + 7.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: moves_text,
                foreground: TermColor::Rgb {
                    r: 255,
                    g: 255,
                    b: 100,
                },
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
                row: title_start_row + 9.0,
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

        let quit_hint = "Press ESC to quit";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - quit_hint.len() as f64 / 2.0,
                row: title_start_row + 11.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: quit_hint.to_string(),
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
        entities: EntityGroup::new(),
        start_game: false,
    }))
}
