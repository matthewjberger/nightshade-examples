use nightshade::tui::prelude::*;
use rand::Rng;

const HUD_HEIGHT: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Element {
    Empty,
    Sand,
    Water,
    Wall,
    Fire,
    Wood,
    Oil,
    Steam,
    Acid,
    Lava,
    Stone,
    Plant,
}

impl Element {
    fn all_placeable() -> &'static [Element] {
        &[
            Element::Sand,
            Element::Water,
            Element::Wall,
            Element::Fire,
            Element::Wood,
            Element::Oil,
            Element::Steam,
            Element::Acid,
            Element::Lava,
            Element::Stone,
            Element::Plant,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Element::Empty => "Empty",
            Element::Sand => "Sand",
            Element::Water => "Water",
            Element::Wall => "Wall",
            Element::Fire => "Fire",
            Element::Wood => "Wood",
            Element::Oil => "Oil",
            Element::Steam => "Steam",
            Element::Acid => "Acid",
            Element::Lava => "Lava",
            Element::Stone => "Stone",
            Element::Plant => "Plant",
        }
    }

    fn character(&self) -> char {
        match self {
            Element::Empty => ' ',
            Element::Sand => '.',
            Element::Water => '~',
            Element::Wall => '#',
            Element::Fire => '^',
            Element::Wood => '=',
            Element::Oil => '%',
            Element::Steam => '\'',
            Element::Acid => '!',
            Element::Lava => '*',
            Element::Stone => '@',
            Element::Plant => '&',
        }
    }

    fn foreground(&self) -> TermColor {
        match self {
            Element::Empty => TermColor::Black,
            Element::Sand => TermColor::Rgb {
                r: 220,
                g: 200,
                b: 100,
            },
            Element::Water => TermColor::Rgb {
                r: 60,
                g: 120,
                b: 255,
            },
            Element::Wall => TermColor::Rgb {
                r: 160,
                g: 160,
                b: 160,
            },
            Element::Fire => TermColor::Rgb {
                r: 255,
                g: 160,
                b: 30,
            },
            Element::Wood => TermColor::Rgb {
                r: 140,
                g: 90,
                b: 40,
            },
            Element::Oil => TermColor::Rgb {
                r: 80,
                g: 60,
                b: 20,
            },
            Element::Steam => TermColor::Rgb {
                r: 200,
                g: 210,
                b: 220,
            },
            Element::Acid => TermColor::Rgb {
                r: 120,
                g: 255,
                b: 50,
            },
            Element::Lava => TermColor::Rgb {
                r: 255,
                g: 80,
                b: 20,
            },
            Element::Stone => TermColor::Rgb {
                r: 130,
                g: 130,
                b: 140,
            },
            Element::Plant => TermColor::Rgb {
                r: 30,
                g: 180,
                b: 50,
            },
        }
    }

    fn background(&self) -> TermColor {
        match self {
            Element::Empty => TermColor::Black,
            Element::Sand => TermColor::Rgb {
                r: 90,
                g: 80,
                b: 30,
            },
            Element::Water => TermColor::Rgb {
                r: 10,
                g: 30,
                b: 80,
            },
            Element::Wall => TermColor::Rgb {
                r: 80,
                g: 80,
                b: 80,
            },
            Element::Fire => TermColor::Rgb {
                r: 120,
                g: 30,
                b: 0,
            },
            Element::Wood => TermColor::Rgb {
                r: 60,
                g: 35,
                b: 10,
            },
            Element::Oil => TermColor::Rgb { r: 30, g: 20, b: 5 },
            Element::Steam => TermColor::Rgb {
                r: 50,
                g: 55,
                b: 65,
            },
            Element::Acid => TermColor::Rgb {
                r: 30,
                g: 80,
                b: 10,
            },
            Element::Lava => TermColor::Rgb {
                r: 100,
                g: 20,
                b: 0,
            },
            Element::Stone => TermColor::Rgb {
                r: 60,
                g: 60,
                b: 65,
            },
            Element::Plant => TermColor::Rgb {
                r: 10,
                g: 60,
                b: 15,
            },
        }
    }

    fn is_liquid(&self) -> bool {
        matches!(
            self,
            Element::Water | Element::Oil | Element::Acid | Element::Lava
        )
    }

    fn is_flammable(&self) -> bool {
        matches!(self, Element::Wood | Element::Oil | Element::Plant)
    }
}

#[derive(Clone, Copy)]
struct Cell {
    element: Element,
    updated: bool,
    lifetime: u16,
}

impl Cell {
    fn empty() -> Self {
        Self {
            element: Element::Empty,
            updated: false,
            lifetime: 0,
        }
    }

    fn new(element: Element) -> Self {
        Self {
            element,
            updated: false,
            lifetime: 0,
        }
    }
}

struct FallingSandState {
    grid: Vec<Cell>,
    grid_width: usize,
    grid_height: usize,
    tilemap_entity: Entity,
    hud_entities: Vec<Entity>,
    selected_element: Element,
    brush_size: usize,
    mouse_column: u16,
    mouse_row: u16,
    mouse_held: bool,
    right_mouse_held: bool,
    paused: bool,
    simulation_timer: Timer,
}

impl FallingSandState {
    fn new() -> Self {
        Self {
            grid: Vec::new(),
            grid_width: 0,
            grid_height: 0,
            tilemap_entity: Entity::default(),
            hud_entities: Vec::new(),
            selected_element: Element::Sand,
            brush_size: 2,
            mouse_column: 0,
            mouse_row: 0,
            mouse_held: false,
            right_mouse_held: false,
            paused: false,
            simulation_timer: Timer::repeating(1.0 / 30.0),
        }
    }

    fn index(&self, column: usize, row: usize) -> usize {
        row * self.grid_width + column
    }

    fn in_bounds(&self, column: i32, row: i32) -> bool {
        column >= 0 && column < self.grid_width as i32 && row >= 0 && row < self.grid_height as i32
    }

    fn get(&self, column: i32, row: i32) -> Element {
        if !self.in_bounds(column, row) {
            return Element::Wall;
        }
        let index = self.index(column as usize, row as usize);
        self.grid[index].element
    }

    fn set(&mut self, column: i32, row: i32, element: Element) {
        if !self.in_bounds(column, row) {
            return;
        }
        let index = self.index(column as usize, row as usize);
        self.grid[index] = Cell::new(element);
        self.grid[index].updated = true;
    }

    fn swap(&mut self, column_a: i32, row_a: i32, column_b: i32, row_b: i32) {
        if !self.in_bounds(column_a, row_a) || !self.in_bounds(column_b, row_b) {
            return;
        }
        let index_a = self.index(column_a as usize, row_a as usize);
        let index_b = self.index(column_b as usize, row_b as usize);
        self.grid.swap(index_a, index_b);
        self.grid[index_a].updated = true;
        self.grid[index_b].updated = true;
    }

    fn paint(&mut self, center_column: i32, center_row: i32, element: Element) {
        let radius = self.brush_size as i32;
        for delta_row in -radius..=radius {
            for delta_column in -radius..=radius {
                if delta_column * delta_column + delta_row * delta_row <= radius * radius {
                    let column = center_column + delta_column;
                    let row = center_row + delta_row;
                    if self.in_bounds(column, row) {
                        let index = self.index(column as usize, row as usize);
                        if element == Element::Empty || self.grid[index].element == Element::Empty {
                            self.grid[index] = Cell::new(element);
                        }
                    }
                }
            }
        }
    }

    fn clear(&mut self) {
        for cell in &mut self.grid {
            *cell = Cell::empty();
        }
    }

    fn simulate(&mut self) {
        let mut rng = rand::rng();

        for cell in &mut self.grid {
            cell.updated = false;
        }

        for row in (0..self.grid_height).rev() {
            let column_range: Box<dyn Iterator<Item = usize>> = if rng.random_bool(0.5) {
                Box::new(0..self.grid_width)
            } else {
                Box::new((0..self.grid_width).rev())
            };

            for column in column_range {
                let index = self.index(column, row);
                if self.grid[index].updated {
                    continue;
                }

                let element = self.grid[index].element;
                match element {
                    Element::Sand => self.update_sand(column as i32, row as i32, &mut rng),
                    Element::Water => self.update_water(column as i32, row as i32, &mut rng),
                    Element::Fire => self.update_fire(column as i32, row as i32, &mut rng),
                    Element::Oil => self.update_oil(column as i32, row as i32, &mut rng),
                    Element::Steam => self.update_steam(column as i32, row as i32, &mut rng),
                    Element::Acid => self.update_acid(column as i32, row as i32, &mut rng),
                    Element::Lava => self.update_lava(column as i32, row as i32, &mut rng),
                    Element::Plant => self.update_plant(column as i32, row as i32, &mut rng),
                    Element::Stone => self.update_stone(column as i32, row as i32, &mut rng),
                    _ => {}
                }
            }
        }
    }

    fn update_sand(&mut self, column: i32, row: i32, rng: &mut impl Rng) {
        let below = self.get(column, row + 1);
        if below == Element::Empty {
            self.swap(column, row, column, row + 1);
            return;
        }

        if below.is_liquid() {
            self.swap(column, row, column, row + 1);
            return;
        }

        let direction: i32 = if rng.random_bool(0.5) { -1 } else { 1 };
        let diag = self.get(column + direction, row + 1);
        if diag == Element::Empty || diag.is_liquid() {
            self.swap(column, row, column + direction, row + 1);
            return;
        }
        let diag_other = self.get(column - direction, row + 1);
        if diag_other == Element::Empty || diag_other.is_liquid() {
            self.swap(column, row, column - direction, row + 1);
        }
    }

    fn update_water(&mut self, column: i32, row: i32, rng: &mut impl Rng) {
        let below = self.get(column, row + 1);
        if below == Element::Empty {
            self.swap(column, row, column, row + 1);
            return;
        }
        if below == Element::Oil {
            self.swap(column, row, column, row + 1);
            return;
        }

        let direction: i32 = if rng.random_bool(0.5) { -1 } else { 1 };
        let diag = self.get(column + direction, row + 1);
        if diag == Element::Empty {
            self.swap(column, row, column + direction, row + 1);
            return;
        }
        let diag_other = self.get(column - direction, row + 1);
        if diag_other == Element::Empty {
            self.swap(column, row, column - direction, row + 1);
            return;
        }

        let side = self.get(column + direction, row);
        if side == Element::Empty {
            self.swap(column, row, column + direction, row);
            return;
        }
        let side_other = self.get(column - direction, row);
        if side_other == Element::Empty {
            self.swap(column, row, column - direction, row);
        }
    }

    fn update_fire(&mut self, column: i32, row: i32, rng: &mut impl Rng) {
        let index = self.index(column as usize, row as usize);
        self.grid[index].lifetime += 1;

        if self.grid[index].lifetime > 20 + rng.random_range(0..15) {
            if rng.random_bool(0.3) {
                self.set(column, row, Element::Steam);
            } else {
                self.set(column, row, Element::Empty);
            }
            return;
        }

        for delta_row in -1i32..=1 {
            for delta_column in -1i32..=1 {
                if delta_row == 0 && delta_column == 0 {
                    continue;
                }
                let neighbor_column = column + delta_column;
                let neighbor_row = row + delta_row;
                let neighbor = self.get(neighbor_column, neighbor_row);
                if neighbor.is_flammable() && rng.random_bool(0.15) {
                    self.set(neighbor_column, neighbor_row, Element::Fire);
                }
                if neighbor == Element::Water {
                    self.set(column, row, Element::Steam);
                    return;
                }
            }
        }

        if rng.random_bool(0.6) {
            let above = self.get(column, row - 1);
            if above == Element::Empty {
                self.swap(column, row, column, row - 1);
                return;
            }
            let direction: i32 = if rng.random_bool(0.5) { -1 } else { 1 };
            let diag_up = self.get(column + direction, row - 1);
            if diag_up == Element::Empty {
                self.swap(column, row, column + direction, row - 1);
            }
        }
    }

    fn update_oil(&mut self, column: i32, row: i32, rng: &mut impl Rng) {
        let below = self.get(column, row + 1);
        if below == Element::Empty {
            self.swap(column, row, column, row + 1);
            return;
        }

        let direction: i32 = if rng.random_bool(0.5) { -1 } else { 1 };
        let diag = self.get(column + direction, row + 1);
        if diag == Element::Empty {
            self.swap(column, row, column + direction, row + 1);
            return;
        }
        let diag_other = self.get(column - direction, row + 1);
        if diag_other == Element::Empty {
            self.swap(column, row, column - direction, row + 1);
            return;
        }

        let side = self.get(column + direction, row);
        if side == Element::Empty {
            self.swap(column, row, column + direction, row);
            return;
        }
        let side_other = self.get(column - direction, row);
        if side_other == Element::Empty {
            self.swap(column, row, column - direction, row);
        }
    }

    fn update_steam(&mut self, column: i32, row: i32, rng: &mut impl Rng) {
        let index = self.index(column as usize, row as usize);
        self.grid[index].lifetime += 1;

        if self.grid[index].lifetime > 60 + rng.random_range(0..40) {
            if rng.random_bool(0.5) {
                self.set(column, row, Element::Water);
            } else {
                self.set(column, row, Element::Empty);
            }
            return;
        }

        let above = self.get(column, row - 1);
        if above == Element::Empty {
            self.swap(column, row, column, row - 1);
            return;
        }

        let direction: i32 = if rng.random_bool(0.5) { -1 } else { 1 };
        let diag_up = self.get(column + direction, row - 1);
        if diag_up == Element::Empty {
            self.swap(column, row, column + direction, row - 1);
            return;
        }

        let side = self.get(column + direction, row);
        if side == Element::Empty {
            self.swap(column, row, column + direction, row);
            return;
        }
        let side_other = self.get(column - direction, row);
        if side_other == Element::Empty {
            self.swap(column, row, column - direction, row);
        }
    }

    fn update_acid(&mut self, column: i32, row: i32, rng: &mut impl Rng) {
        for delta_row in -1i32..=1 {
            for delta_column in -1i32..=1 {
                if delta_row == 0 && delta_column == 0 {
                    continue;
                }
                let neighbor_column = column + delta_column;
                let neighbor_row = row + delta_row;
                let neighbor = self.get(neighbor_column, neighbor_row);
                if neighbor != Element::Empty
                    && neighbor != Element::Acid
                    && neighbor != Element::Wall
                    && rng.random_bool(0.08)
                {
                    self.set(neighbor_column, neighbor_row, Element::Empty);
                    if rng.random_bool(0.5) {
                        self.set(column, row, Element::Empty);
                    }
                    return;
                }
            }
        }

        let below = self.get(column, row + 1);
        if below == Element::Empty {
            self.swap(column, row, column, row + 1);
            return;
        }

        let direction: i32 = if rng.random_bool(0.5) { -1 } else { 1 };
        let diag = self.get(column + direction, row + 1);
        if diag == Element::Empty {
            self.swap(column, row, column + direction, row + 1);
            return;
        }

        let side = self.get(column + direction, row);
        if side == Element::Empty {
            self.swap(column, row, column + direction, row);
            return;
        }
        let side_other = self.get(column - direction, row);
        if side_other == Element::Empty {
            self.swap(column, row, column - direction, row);
        }
    }

    fn update_lava(&mut self, column: i32, row: i32, rng: &mut impl Rng) {
        for delta_row in -1i32..=1 {
            for delta_column in -1i32..=1 {
                if delta_row == 0 && delta_column == 0 {
                    continue;
                }
                let neighbor_column = column + delta_column;
                let neighbor_row = row + delta_row;
                let neighbor = self.get(neighbor_column, neighbor_row);
                if neighbor.is_flammable() && rng.random_bool(0.4) {
                    self.set(neighbor_column, neighbor_row, Element::Fire);
                }
                if neighbor == Element::Water {
                    self.set(neighbor_column, neighbor_row, Element::Stone);
                    if rng.random_bool(0.3) {
                        self.set(column, row, Element::Stone);
                    }
                    return;
                }
            }
        }

        let below = self.get(column, row + 1);
        if below == Element::Empty {
            self.swap(column, row, column, row + 1);
            return;
        }

        if rng.random_bool(0.3) {
            let direction: i32 = if rng.random_bool(0.5) { -1 } else { 1 };
            let side = self.get(column + direction, row);
            if side == Element::Empty {
                self.swap(column, row, column + direction, row);
                return;
            }
            let side_other = self.get(column - direction, row);
            if side_other == Element::Empty {
                self.swap(column, row, column - direction, row);
            }
        }
    }

    fn update_plant(&mut self, column: i32, row: i32, rng: &mut impl Rng) {
        let mut has_water_neighbor = false;
        for delta_row in -1i32..=1 {
            for delta_column in -1i32..=1 {
                if delta_row == 0 && delta_column == 0 {
                    continue;
                }
                let neighbor = self.get(column + delta_column, row + delta_row);
                if neighbor == Element::Water {
                    has_water_neighbor = true;
                }
            }
        }

        if has_water_neighbor && rng.random_bool(0.05) {
            let directions: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
            let direction = directions[rng.random_range(0..4)];
            let target_column = column + direction.0;
            let target_row = row + direction.1;
            let target = self.get(target_column, target_row);
            if target == Element::Empty || target == Element::Water {
                self.set(target_column, target_row, Element::Plant);
            }
        }
    }

    fn update_stone(&mut self, column: i32, row: i32, _rng: &mut impl Rng) {
        let below = self.get(column, row + 1);
        if below == Element::Empty {
            self.swap(column, row, column, row + 1);
            return;
        }
        if below.is_liquid() {
            self.swap(column, row, column, row + 1);
        }
    }

    fn refresh_tilemap(&self, world: &mut World) {
        let mut tilemap = Tilemap::new(self.grid_width, self.grid_height);
        for row in 0..self.grid_height {
            for column in 0..self.grid_width {
                let index = self.index(column, row);
                let cell = &self.grid[index];
                tilemap.set(
                    column,
                    row,
                    TilemapCell {
                        character: cell.element.character(),
                        foreground: cell.element.foreground(),
                        background: cell.element.background(),
                    },
                );
            }
        }

        if world.get_tilemap(self.tilemap_entity).is_some() {
            world.set_tilemap(self.tilemap_entity, tilemap);
        }
    }

    fn render_hud(&mut self, world: &mut World) {
        world.despawn_entities(&self.hud_entities);
        self.hud_entities.clear();

        let terminal_columns = world.resources.terminal_size.columns as usize;

        let elements = Element::all_placeable();
        let mut element_list = String::new();
        for (index, element) in elements.iter().enumerate() {
            let key = index + 1;
            let key_label = if key <= 9 {
                format!("{}", key)
            } else if key == 10 {
                "0".to_string()
            } else {
                format!("{}", (b'a' + (key - 11) as u8) as char)
            };

            if *element == self.selected_element {
                element_list.push_str(&format!("[{}:{}]", key_label, element.name()));
            } else {
                element_list.push_str(&format!(" {}:{} ", key_label, element.name()));
            }
        }

        let status = if self.paused { "PAUSED" } else { "RUNNING" };
        let row_0 = format!(
            "Falling Sand | {} | Brush: {} | {}",
            status, self.brush_size, element_list,
        );

        let row_1 = "LMB=place RMB=erase +/-=brush C=clear Space=pause Q=quit".to_string();

        self.spawn_hud_line(world, &row_0, 0, terminal_columns);
        self.spawn_hud_line(world, &row_1, 1, terminal_columns);
    }

    fn spawn_hud_line(
        &mut self,
        world: &mut World,
        text: &str,
        row: usize,
        terminal_columns: usize,
    ) {
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

    fn grid_from_mouse(&self, column: u16, row: u16) -> (i32, i32) {
        (column as i32, row as i32 - HUD_HEIGHT as i32)
    }
}

impl State for FallingSandState {
    fn title(&self) -> &str {
        "Falling Sand"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 60;

        let terminal_columns = world.resources.terminal_size.columns as usize;
        let terminal_rows = world.resources.terminal_size.rows as usize;

        self.grid_width = terminal_columns;
        self.grid_height = if terminal_rows > HUD_HEIGHT {
            terminal_rows - HUD_HEIGHT
        } else {
            10
        };

        self.grid = vec![Cell::empty(); self.grid_width * self.grid_height];

        self.tilemap_entity = EntityBuilder::new()
            .position(Position {
                column: 0.0,
                row: HUD_HEIGHT as f64,
            })
            .tilemap(Tilemap::new(self.grid_width, self.grid_height))
            .z_index(ZIndex(0))
            .spawn(world);

        self.refresh_tilemap(world);
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
            KeyCode::Char(' ') => {
                self.paused = !self.paused;
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.clear();
            }
            KeyCode::Char('+') | KeyCode::Char('=') if self.brush_size < 10 => {
                self.brush_size += 1;
            }
            KeyCode::Char('-') | KeyCode::Char('_') if self.brush_size > 1 => {
                self.brush_size -= 1;
            }
            KeyCode::Char('1') => self.selected_element = Element::Sand,
            KeyCode::Char('2') => self.selected_element = Element::Water,
            KeyCode::Char('3') => self.selected_element = Element::Wall,
            KeyCode::Char('4') => self.selected_element = Element::Fire,
            KeyCode::Char('5') => self.selected_element = Element::Wood,
            KeyCode::Char('6') => self.selected_element = Element::Oil,
            KeyCode::Char('7') => self.selected_element = Element::Steam,
            KeyCode::Char('8') => self.selected_element = Element::Acid,
            KeyCode::Char('9') => self.selected_element = Element::Lava,
            KeyCode::Char('0') => self.selected_element = Element::Stone,
            KeyCode::Char('a') | KeyCode::Char('A') => self.selected_element = Element::Plant,
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
        self.mouse_column = column;
        self.mouse_row = row;

        match button {
            MouseButton::Left => {
                self.mouse_held = pressed;
                if pressed {
                    let (grid_column, grid_row) = self.grid_from_mouse(column, row);
                    self.paint(grid_column, grid_row, self.selected_element);
                }
            }
            MouseButton::Right => {
                self.right_mouse_held = pressed;
                if pressed {
                    let (grid_column, grid_row) = self.grid_from_mouse(column, row);
                    self.paint(grid_column, grid_row, Element::Empty);
                }
            }
            _ => {}
        }
    }

    fn on_mouse_move(&mut self, _world: &mut World, column: u16, row: u16) {
        self.mouse_column = column;
        self.mouse_row = row;

        if self.mouse_held {
            let (grid_column, grid_row) = self.grid_from_mouse(column, row);
            self.paint(grid_column, grid_row, self.selected_element);
        }
        if self.right_mouse_held {
            let (grid_column, grid_row) = self.grid_from_mouse(column, row);
            self.paint(grid_column, grid_row, Element::Empty);
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if !self.paused {
            let delta = world.resources.timing.delta_seconds;
            if self.simulation_timer.tick(delta) {
                self.simulate();
            }
        }

        self.refresh_tilemap(world);
        self.render_hud(world);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(FallingSandState::new()))
}
