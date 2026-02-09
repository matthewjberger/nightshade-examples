use nightshade::tui::prelude::*;
use rand::Rng;

const HEX_COLUMNS: usize = 12;
const HEX_ROWS: usize = 10;
const HEX_CHAR_WIDTH: usize = 4;
const HEX_CHAR_HEIGHT: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Terrain {
    Plains,
    Forest,
    Mountain,
    Water,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Faction {
    Player,
    Enemy,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum UnitKind {
    Warrior,
    Archer,
    Cavalry,
}

#[derive(Clone)]
struct Unit {
    kind: UnitKind,
    faction: Faction,
    hex_column: usize,
    hex_row: usize,
    health: i32,
    max_health: i32,
    attack: i32,
    movement: i32,
    remaining_movement: i32,
    attack_range: i32,
    has_attacked: bool,
}

impl Unit {
    fn new(kind: UnitKind, faction: Faction, hex_column: usize, hex_row: usize) -> Self {
        let (health, attack, movement, attack_range) = match kind {
            UnitKind::Warrior => (5, 3, 2, 1),
            UnitKind::Archer => (3, 2, 2, 2),
            UnitKind::Cavalry => (4, 2, 4, 1),
        };
        Self {
            kind,
            faction,
            hex_column,
            hex_row,
            health,
            max_health: health,
            attack,
            movement,
            remaining_movement: movement,
            attack_range,
            has_attacked: false,
        }
    }

    fn character(&self) -> char {
        match self.kind {
            UnitKind::Warrior => 'W',
            UnitKind::Archer => 'A',
            UnitKind::Cavalry => 'C',
        }
    }

    fn cost(&self) -> i32 {
        match self.kind {
            UnitKind::Warrior => 3,
            UnitKind::Archer => 2,
            UnitKind::Cavalry => 4,
        }
    }
}

struct Town {
    faction: Faction,
    hex_column: usize,
    hex_row: usize,
}

fn hex_to_screen(hex_column: usize, hex_row: usize) -> (usize, usize) {
    let screen_column = hex_column * HEX_CHAR_WIDTH + if hex_row % 2 == 1 { 2 } else { 0 };
    let screen_row = hex_row * HEX_CHAR_HEIGHT;
    (screen_column, screen_row)
}

fn hex_distance(column_a: usize, row_a: usize, column_b: usize, row_b: usize) -> i32 {
    let (ax, ay, az) = offset_to_cube(column_a as i32, row_a as i32);
    let (bx, by, bz) = offset_to_cube(column_b as i32, row_b as i32);
    ((ax - bx).abs() + (ay - by).abs() + (az - bz).abs()) / 2
}

fn offset_to_cube(column: i32, row: i32) -> (i32, i32, i32) {
    let x = column - (row - (row & 1)) / 2;
    let z = row;
    let y = -x - z;
    (x, y, z)
}

fn hex_neighbors(column: usize, row: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    let even = row.is_multiple_of(2);

    let offsets: [(i32, i32); 6] = if even {
        [(-1, -1), (0, -1), (-1, 0), (1, 0), (-1, 1), (0, 1)]
    } else {
        [(0, -1), (1, -1), (-1, 0), (1, 0), (0, 1), (1, 1)]
    };

    for (delta_column, delta_row) in offsets {
        let new_column = column as i32 + delta_column;
        let new_row = row as i32 + delta_row;
        if new_column >= 0
            && new_column < HEX_COLUMNS as i32
            && new_row >= 0
            && new_row < HEX_ROWS as i32
        {
            result.push((new_column as usize, new_row as usize));
        }
    }
    result
}

fn terrain_color(terrain: Terrain) -> TermColor {
    match terrain {
        Terrain::Plains => TermColor::Rgb {
            r: 50,
            g: 120,
            b: 50,
        },
        Terrain::Forest => TermColor::Rgb {
            r: 20,
            g: 80,
            b: 20,
        },
        Terrain::Mountain => TermColor::Rgb {
            r: 100,
            g: 100,
            b: 100,
        },
        Terrain::Water => TermColor::Rgb {
            r: 30,
            g: 60,
            b: 140,
        },
    }
}

fn terrain_character(terrain: Terrain) -> char {
    match terrain {
        Terrain::Plains => '.',
        Terrain::Forest => 'T',
        Terrain::Mountain => '^',
        Terrain::Water => '~',
    }
}

fn terrain_foreground(terrain: Terrain) -> TermColor {
    match terrain {
        Terrain::Plains => TermColor::Rgb {
            r: 80,
            g: 160,
            b: 80,
        },
        Terrain::Forest => TermColor::Rgb {
            r: 40,
            g: 140,
            b: 40,
        },
        Terrain::Mountain => TermColor::Rgb {
            r: 160,
            g: 160,
            b: 160,
        },
        Terrain::Water => TermColor::Rgb {
            r: 60,
            g: 100,
            b: 200,
        },
    }
}

fn is_passable(terrain: Terrain) -> bool {
    matches!(terrain, Terrain::Plains | Terrain::Forest)
}

fn defense_bonus(terrain: Terrain) -> i32 {
    match terrain {
        Terrain::Forest => 1,
        _ => 0,
    }
}

fn generate_terrain() -> Vec<Vec<Terrain>> {
    let mut rng = rand::rng();
    let mut grid = vec![vec![Terrain::Plains; HEX_COLUMNS]; HEX_ROWS];

    for (row, grid_row) in grid.iter_mut().enumerate().take(HEX_ROWS) {
        for (column, cell) in grid_row.iter_mut().enumerate().take(HEX_COLUMNS) {
            if (column == 1 && row == 1) || (column == HEX_COLUMNS - 2 && row == HEX_ROWS - 2) {
                continue;
            }
            if hex_distance(column, row, 1, 1) <= 1
                || hex_distance(column, row, HEX_COLUMNS - 2, HEX_ROWS - 2) <= 1
            {
                continue;
            }

            let roll: f64 = rng.random();
            if roll < 0.15 {
                *cell = Terrain::Forest;
            } else if roll < 0.22 {
                *cell = Terrain::Mountain;
            } else if roll < 0.28 {
                *cell = Terrain::Water;
            }
        }
    }

    grid
}

fn find_reachable_hexes(
    start_column: usize,
    start_row: usize,
    max_movement: i32,
    terrain: &[Vec<Terrain>],
    units: &[Unit],
    faction: Faction,
) -> Vec<(usize, usize)> {
    let mut reachable = Vec::new();
    let mut visited = vec![vec![false; HEX_COLUMNS]; HEX_ROWS];
    let mut costs = vec![vec![i32::MAX; HEX_COLUMNS]; HEX_ROWS];
    let mut frontier: Vec<(usize, usize, i32)> = Vec::new();

    visited[start_row][start_column] = true;
    costs[start_row][start_column] = 0;
    frontier.push((start_column, start_row, 0));

    while let Some((current_column, current_row, current_cost)) = frontier.pop() {
        for (neighbor_column, neighbor_row) in hex_neighbors(current_column, current_row) {
            if !is_passable(terrain[neighbor_row][neighbor_column]) {
                continue;
            }

            let occupied_by_friendly = units.iter().any(|unit| {
                unit.hex_column == neighbor_column
                    && unit.hex_row == neighbor_row
                    && unit.faction == faction
                    && unit.health > 0
            });
            if occupied_by_friendly
                && !(neighbor_column == start_column && neighbor_row == start_row)
            {
                continue;
            }

            let move_cost = 1;
            let new_cost = current_cost + move_cost;
            if new_cost <= max_movement && new_cost < costs[neighbor_row][neighbor_column] {
                costs[neighbor_row][neighbor_column] = new_cost;
                if !visited[neighbor_row][neighbor_column] {
                    visited[neighbor_row][neighbor_column] = true;
                    reachable.push((neighbor_column, neighbor_row));
                }
                frontier.push((neighbor_column, neighbor_row, new_cost));
            }
        }
    }

    reachable
}

fn find_attackable_hexes(
    unit_column: usize,
    unit_row: usize,
    attack_range: i32,
    units: &[Unit],
    unit_faction: Faction,
) -> Vec<usize> {
    let mut targets = Vec::new();
    for (target_index, target) in units.iter().enumerate() {
        if target.faction == unit_faction || target.health <= 0 {
            continue;
        }
        let distance = hex_distance(unit_column, unit_row, target.hex_column, target.hex_row);
        if distance <= attack_range {
            targets.push(target_index);
        }
    }
    targets
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Select,
    Move,
    Attack,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TurnOwner {
    Player,
    Ai,
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Hex Strategy - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "HEX STRATEGY";
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
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let subtitle = "A Turn-Based Hex Wargame";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - subtitle.len() as f64 / 2.0,
                row: center_row - 3.0,
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

        let rules1 = "Capture the enemy town to win!";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - rules1.len() as f64 / 2.0,
                row: center_row - 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: rules1.to_string(),
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let rules2 = "W=Warrior(3atk,5hp) A=Archer(2atk,3hp,rng2) C=Cavalry(2atk,4hp,4mv)";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - rules2.len() as f64 / 2.0,
                row: center_row + 0.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: rules2.to_string(),
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
                row: center_row + 3.0,
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

        let quit = "ESC to quit";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - quit.len() as f64 / 2.0,
                row: center_row + 5.0,
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
            return Some(Box::new(GameplayState::new()));
        }
        None
    }
}

struct GameplayState {
    terrain: Vec<Vec<Terrain>>,
    units: Vec<Unit>,
    towns: Vec<Town>,
    tilemap_entity: Entity,
    offset_column: usize,
    offset_row: usize,
    cursor_column: usize,
    cursor_row: usize,
    selected_unit_index: Option<usize>,
    phase: Phase,
    turn_owner: TurnOwner,
    turn_number: u32,
    player_gold: i32,
    enemy_gold: i32,
    reachable_hexes: Vec<(usize, usize)>,
    attackable_units: Vec<usize>,
    hud_entities: EntityGroup,
    highlight_entities: EntityGroup,
    unit_entities: EntityGroup,
    particles: ParticleEmitter,
    messages: Vec<String>,
    ai_timer: Timer,
    ai_action_pending: bool,
    ai_unit_index: usize,
    game_result: Option<Faction>,
    recruit_mode: bool,
    recruit_selection: usize,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            terrain: Vec::new(),
            units: Vec::new(),
            towns: Vec::new(),
            tilemap_entity: Entity::default(),
            offset_column: 2,
            offset_row: 1,
            cursor_column: 1,
            cursor_row: 1,
            selected_unit_index: None,
            phase: Phase::Select,
            turn_owner: TurnOwner::Player,
            turn_number: 1,
            player_gold: 5,
            enemy_gold: 5,
            reachable_hexes: Vec::new(),
            attackable_units: Vec::new(),
            hud_entities: EntityGroup::new(),
            highlight_entities: EntityGroup::new(),
            unit_entities: EntityGroup::new(),
            particles: ParticleEmitter::new(),
            messages: vec!["Your turn. Select a unit with arrow keys + Enter.".to_string()],
            ai_timer: Timer::once(0.4),
            ai_action_pending: false,
            ai_unit_index: 0,
            game_result: None,
            recruit_mode: false,
            recruit_selection: 0,
        }
    }

    fn tilemap_width(&self) -> usize {
        HEX_COLUMNS * HEX_CHAR_WIDTH + 3
    }

    fn tilemap_height(&self) -> usize {
        HEX_ROWS * HEX_CHAR_HEIGHT + 1
    }

    fn build_tilemap(&mut self, world: &mut World) {
        let width = self.tilemap_width();
        let height = self.tilemap_height();
        let mut tilemap = Tilemap::new(width, height);

        for hex_row in 0..HEX_ROWS {
            for hex_column in 0..HEX_COLUMNS {
                let (screen_column, screen_row) = hex_to_screen(hex_column, hex_row);
                let terrain = self.terrain[hex_row][hex_column];
                let background = terrain_color(terrain);
                let foreground = terrain_foreground(terrain);
                let character = terrain_character(terrain);

                for local_row in 0..HEX_CHAR_HEIGHT {
                    for local_column in 0..HEX_CHAR_WIDTH {
                        let tile_column = screen_column + local_column;
                        let tile_row = screen_row + local_row;
                        if tile_column < width && tile_row < height {
                            tilemap.set(
                                tile_column,
                                tile_row,
                                TilemapCell {
                                    character,
                                    foreground,
                                    background,
                                },
                            );
                        }
                    }
                }
            }
        }

        self.tilemap_entity = EntityBuilder::new()
            .position(Position {
                column: self.offset_column as f64,
                row: self.offset_row as f64,
            })
            .tilemap(tilemap)
            .z_index(ZIndex(0))
            .spawn(world);
    }

    fn render_units(&mut self, world: &mut World) {
        self.unit_entities.despawn_all(world);

        for town in &self.towns {
            let (screen_column, screen_row) = hex_to_screen(town.hex_column, town.hex_row);
            let foreground = match town.faction {
                Faction::Player => TermColor::Cyan,
                Faction::Enemy => TermColor::Red,
            };

            let entity = self
                .unit_entities
                .spawn_one(world, POSITION | SPRITE | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: (self.offset_column + screen_column + 1) as f64,
                    row: (self.offset_row + screen_row) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: 'H',
                    foreground,
                    background: terrain_color(self.terrain[town.hex_row][town.hex_column]),
                },
            );
            world.set_z_index(entity, ZIndex(2));

            let entity = self
                .unit_entities
                .spawn_one(world, POSITION | SPRITE | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: (self.offset_column + screen_column + 2) as f64,
                    row: (self.offset_row + screen_row) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: 'Q',
                    foreground,
                    background: terrain_color(self.terrain[town.hex_row][town.hex_column]),
                },
            );
            world.set_z_index(entity, ZIndex(2));
        }

        for (unit_index, unit) in self.units.iter().enumerate() {
            if unit.health <= 0 {
                continue;
            }

            let (screen_column, screen_row) = hex_to_screen(unit.hex_column, unit.hex_row);
            let foreground = match unit.faction {
                Faction::Player => TermColor::Blue,
                Faction::Enemy => TermColor::Red,
            };
            let is_selected = self.selected_unit_index == Some(unit_index);
            let background = if is_selected {
                TermColor::Yellow
            } else {
                terrain_color(self.terrain[unit.hex_row][unit.hex_column])
            };

            let entity = self
                .unit_entities
                .spawn_one(world, POSITION | SPRITE | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: (self.offset_column + screen_column + 1) as f64,
                    row: (self.offset_row + screen_row) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: unit.character(),
                    foreground,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(3));

            let health_bar_length = HEX_CHAR_WIDTH.min(unit.max_health as usize);
            let filled = ((unit.health as f64 / unit.max_health as f64) * health_bar_length as f64)
                .ceil() as usize;

            for bar_index in 0..health_bar_length {
                let bar_char = if bar_index < filled { '=' } else { '-' };
                let bar_color = if unit.health * 3 > unit.max_health * 2 {
                    TermColor::Green
                } else if unit.health * 3 > unit.max_health {
                    TermColor::Yellow
                } else {
                    TermColor::Red
                };
                let entity = self
                    .unit_entities
                    .spawn_one(world, POSITION | SPRITE | Z_INDEX);
                world.set_position(
                    entity,
                    Position {
                        column: (self.offset_column + screen_column + bar_index) as f64,
                        row: (self.offset_row + screen_row + 1) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character: bar_char,
                        foreground: bar_color,
                        background: terrain_color(self.terrain[unit.hex_row][unit.hex_column]),
                    },
                );
                world.set_z_index(entity, ZIndex(3));
            }

            let can_still_act = unit.remaining_movement > 0 || !unit.has_attacked;
            if unit.faction == Faction::Player
                && self.turn_owner == TurnOwner::Player
                && !can_still_act
            {
                let entity = self
                    .unit_entities
                    .spawn_one(world, POSITION | SPRITE | Z_INDEX);
                world.set_position(
                    entity,
                    Position {
                        column: (self.offset_column + screen_column + 2) as f64,
                        row: (self.offset_row + screen_row) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character: 'z',
                        foreground: TermColor::DarkGrey,
                        background: terrain_color(self.terrain[unit.hex_row][unit.hex_column]),
                    },
                );
                world.set_z_index(entity, ZIndex(4));
            }
        }
    }

    fn render_highlights(&mut self, world: &mut World) {
        self.highlight_entities.despawn_all(world);
        let width = self.tilemap_width();
        let height = self.tilemap_height();

        let (cursor_screen_column, cursor_screen_row) =
            hex_to_screen(self.cursor_column, self.cursor_row);
        for local_row in 0..HEX_CHAR_HEIGHT {
            for local_column in 0..HEX_CHAR_WIDTH {
                let tile_column = cursor_screen_column + local_column;
                let tile_row = cursor_screen_row + local_row;
                if tile_column < width && tile_row < height {
                    let entity = self
                        .highlight_entities
                        .spawn_one(world, POSITION | SPRITE | Z_INDEX);
                    world.set_position(
                        entity,
                        Position {
                            column: (self.offset_column + tile_column) as f64,
                            row: (self.offset_row + tile_row) as f64,
                        },
                    );
                    world.set_sprite(
                        entity,
                        Sprite {
                            character: ' ',
                            foreground: TermColor::White,
                            background: TermColor::Rgb {
                                r: 80,
                                g: 80,
                                b: 40,
                            },
                        },
                    );
                    world.set_z_index(entity, ZIndex(1));
                }
            }
        }

        if self.phase == Phase::Move {
            for &(reachable_column, reachable_row) in &self.reachable_hexes {
                let (screen_column, screen_row) = hex_to_screen(reachable_column, reachable_row);
                let entity = self
                    .highlight_entities
                    .spawn_one(world, POSITION | SPRITE | Z_INDEX);
                world.set_position(
                    entity,
                    Position {
                        column: (self.offset_column + screen_column + 1) as f64,
                        row: (self.offset_row + screen_row) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character: '+',
                        foreground: TermColor::Green,
                        background: terrain_color(self.terrain[reachable_row][reachable_column]),
                    },
                );
                world.set_z_index(entity, ZIndex(1));
            }
        }

        if self.phase == Phase::Attack {
            for &target_index in &self.attackable_units {
                if let Some(target) = self.units.get(target_index)
                    && target.health > 0
                {
                    let (screen_column, screen_row) =
                        hex_to_screen(target.hex_column, target.hex_row);
                    for local_row in 0..HEX_CHAR_HEIGHT {
                        for local_column in 0..HEX_CHAR_WIDTH {
                            let tile_column = screen_column + local_column;
                            let tile_row = screen_row + local_row;
                            if tile_column < width && tile_row < height {
                                let entity = self
                                    .highlight_entities
                                    .spawn_one(world, POSITION | SPRITE | Z_INDEX);
                                world.set_position(
                                    entity,
                                    Position {
                                        column: (self.offset_column + tile_column) as f64,
                                        row: (self.offset_row + tile_row) as f64,
                                    },
                                );
                                world.set_sprite(
                                    entity,
                                    Sprite {
                                        character: ' ',
                                        foreground: TermColor::White,
                                        background: TermColor::Rgb {
                                            r: 120,
                                            g: 30,
                                            b: 30,
                                        },
                                    },
                                );
                                world.set_z_index(entity, ZIndex(1));
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_hud(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);

        let hud_column = self.offset_column as f64 + self.tilemap_width() as f64 + 2.0;
        let hud_row = self.offset_row as f64;

        let turn_text = format!(
            "Turn: {}  ({})",
            self.turn_number,
            if self.turn_owner == TurnOwner::Player {
                "Player"
            } else {
                "AI"
            },
        );
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: hud_column,
                row: hud_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: turn_text,
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(15));

        let gold_text = format!("Gold: {}  Enemy: {}", self.player_gold, self.enemy_gold);
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: hud_column,
                row: hud_row + 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: gold_text,
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(15));

        let cursor_terrain = self.terrain[self.cursor_row][self.cursor_column];
        let terrain_name = match cursor_terrain {
            Terrain::Plains => "Plains",
            Terrain::Forest => "Forest (+1 def)",
            Terrain::Mountain => "Mountain (impassable)",
            Terrain::Water => "Water (impassable)",
        };
        let cursor_text = format!(
            "Cursor: ({},{}) {}",
            self.cursor_column, self.cursor_row, terrain_name
        );
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: hud_column,
                row: hud_row + 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: cursor_text,
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(15));

        if let Some(unit_at_cursor) = self.unit_at(self.cursor_column, self.cursor_row) {
            let unit = &self.units[unit_at_cursor];
            let kind_name = match unit.kind {
                UnitKind::Warrior => "Warrior",
                UnitKind::Archer => "Archer",
                UnitKind::Cavalry => "Cavalry",
            };
            let faction_name = match unit.faction {
                Faction::Player => "Player",
                Faction::Enemy => "Enemy",
            };
            let unit_text = format!(
                "{} {} HP:{}/{} ATK:{} MV:{}/{}",
                faction_name,
                kind_name,
                unit.health,
                unit.max_health,
                unit.attack,
                unit.remaining_movement,
                unit.movement,
            );
            let entity = self
                .hud_entities
                .spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: hud_column,
                    row: hud_row + 4.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: unit_text,
                    foreground: if unit.faction == Faction::Player {
                        TermColor::Blue
                    } else {
                        TermColor::Red
                    },
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(15));
        }

        let phase_text = match self.phase {
            Phase::Select => "Select a unit (Enter)",
            Phase::Move => "Move unit (Enter/Esc)",
            Phase::Attack => "Attack target (Enter/Esc)",
        };
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: hud_column,
                row: hud_row + 6.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: phase_text.to_string(),
                foreground: TermColor::Cyan,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(15));

        let controls_start = hud_row + 8.0;
        let controls = [
            "Arrow keys: move cursor",
            "Enter: confirm",
            "Esc: cancel/deselect",
            "Space: end turn",
            "R: recruit at town",
        ];
        for (line_index, control_text) in controls.iter().enumerate() {
            let entity = self
                .hud_entities
                .spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: hud_column,
                    row: controls_start + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: control_text.to_string(),
                    foreground: TermColor::DarkGrey,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(15));
        }

        if self.recruit_mode {
            let recruit_row = controls_start + controls.len() as f64 + 1.0;
            let entity = self
                .hud_entities
                .spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: hud_column,
                    row: recruit_row,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: "--- RECRUIT ---".to_string(),
                    foreground: TermColor::Yellow,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(15));

            let recruit_options = [
                ("Warrior", UnitKind::Warrior),
                ("Archer", UnitKind::Archer),
                ("Cavalry", UnitKind::Cavalry),
            ];
            for (option_index, (name, kind)) in recruit_options.iter().enumerate() {
                let cost = Unit::new(*kind, Faction::Player, 0, 0).cost();
                let marker = if option_index == self.recruit_selection {
                    "> "
                } else {
                    "  "
                };
                let text = format!("{}{} ({}g)", marker, name, cost);
                let can_afford = self.player_gold >= cost;
                let entity = self
                    .hud_entities
                    .spawn_one(world, POSITION | LABEL | Z_INDEX);
                world.set_position(
                    entity,
                    Position {
                        column: hud_column,
                        row: recruit_row + 1.0 + option_index as f64,
                    },
                );
                world.set_label(
                    entity,
                    Label {
                        text,
                        foreground: if can_afford {
                            TermColor::White
                        } else {
                            TermColor::DarkGrey
                        },
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(15));
            }
        }

        let message_row = self.offset_row as f64 + self.tilemap_height() as f64 + 1.0;
        let visible_messages: Vec<&String> = self.messages.iter().rev().take(3).collect();
        for (message_index, message) in visible_messages.iter().rev().enumerate() {
            let entity = self
                .hud_entities
                .spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: self.offset_column as f64,
                    row: message_row + message_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: (*message).clone(),
                    foreground: TermColor::White,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(15));
        }
    }

    fn unit_at(&self, hex_column: usize, hex_row: usize) -> Option<usize> {
        self.units.iter().position(|unit| {
            unit.hex_column == hex_column && unit.hex_row == hex_row && unit.health > 0
        })
    }

    fn player_town_position(&self) -> Option<(usize, usize)> {
        self.towns
            .iter()
            .find(|town| town.faction == Faction::Player)
            .map(|town| (town.hex_column, town.hex_row))
    }

    fn enemy_town_position(&self) -> Option<(usize, usize)> {
        self.towns
            .iter()
            .find(|town| town.faction == Faction::Enemy)
            .map(|town| (town.hex_column, town.hex_row))
    }

    fn add_message(&mut self, message: String) {
        self.messages.push(message);
        if self.messages.len() > 20 {
            self.messages.remove(0);
        }
    }

    fn begin_player_turn(&mut self) {
        self.turn_owner = TurnOwner::Player;
        self.phase = Phase::Select;
        self.selected_unit_index = None;
        self.reachable_hexes.clear();
        self.attackable_units.clear();
        self.recruit_mode = false;

        let player_town_count = self
            .towns
            .iter()
            .filter(|town| town.faction == Faction::Player)
            .count() as i32;
        self.player_gold += player_town_count;

        for unit in &mut self.units {
            if unit.faction == Faction::Player && unit.health > 0 {
                unit.remaining_movement = unit.movement;
                unit.has_attacked = false;
            }
        }

        self.add_message(format!(
            "Turn {}. Your turn! (+{} gold)",
            self.turn_number, player_town_count,
        ));
    }

    fn begin_ai_turn(&mut self) {
        self.turn_owner = TurnOwner::Ai;
        self.phase = Phase::Select;
        self.selected_unit_index = None;
        self.reachable_hexes.clear();
        self.attackable_units.clear();
        self.recruit_mode = false;

        let enemy_town_count = self
            .towns
            .iter()
            .filter(|town| town.faction == Faction::Enemy)
            .count() as i32;
        self.enemy_gold += enemy_town_count;

        for unit in &mut self.units {
            if unit.faction == Faction::Enemy && unit.health > 0 {
                unit.remaining_movement = unit.movement;
                unit.has_attacked = false;
            }
        }

        self.ai_unit_index = 0;
        self.ai_action_pending = true;
        self.ai_timer.reset();

        self.add_message("Enemy turn...".to_string());
    }

    fn end_player_turn(&mut self) {
        self.selected_unit_index = None;
        self.reachable_hexes.clear();
        self.attackable_units.clear();
        self.recruit_mode = false;
        self.turn_number += 1;
        self.begin_ai_turn();
    }

    fn handle_select_unit(&mut self) {
        let unit_index = self.unit_at(self.cursor_column, self.cursor_row);
        if let Some(index) = unit_index {
            let unit = &self.units[index];
            if unit.faction == Faction::Player && unit.health > 0 {
                let can_act = unit.remaining_movement > 0 || !unit.has_attacked;
                if can_act {
                    self.selected_unit_index = Some(index);

                    if self.units[index].remaining_movement > 0 {
                        self.phase = Phase::Move;
                        self.reachable_hexes = find_reachable_hexes(
                            self.units[index].hex_column,
                            self.units[index].hex_row,
                            self.units[index].remaining_movement,
                            &self.terrain,
                            &self.units,
                            Faction::Player,
                        );
                    } else {
                        self.phase = Phase::Attack;
                        self.attackable_units = find_attackable_hexes(
                            self.units[index].hex_column,
                            self.units[index].hex_row,
                            self.units[index].attack_range,
                            &self.units,
                            Faction::Player,
                        );
                        if self.attackable_units.is_empty() {
                            self.add_message("No targets in range.".to_string());
                            self.phase = Phase::Select;
                            self.selected_unit_index = None;
                        }
                    }
                }
            }
        }
    }

    fn handle_move_unit(&mut self, world: &mut World) {
        let unit_index = match self.selected_unit_index {
            Some(index) => index,
            None => return,
        };

        let target_column = self.cursor_column;
        let target_row = self.cursor_row;

        if !self.reachable_hexes.contains(&(target_column, target_row)) {
            return;
        }

        let occupied = self.units.iter().any(|other| {
            other.hex_column == target_column && other.hex_row == target_row && other.health > 0
        });
        if occupied {
            return;
        }

        let distance = hex_distance(
            self.units[unit_index].hex_column,
            self.units[unit_index].hex_row,
            target_column,
            target_row,
        );

        self.units[unit_index].hex_column = target_column;
        self.units[unit_index].hex_row = target_row;
        self.units[unit_index].remaining_movement =
            (self.units[unit_index].remaining_movement - distance).max(0);

        self.check_town_capture(target_column, target_row, Faction::Player, world);

        if !self.units[unit_index].has_attacked {
            self.attackable_units = find_attackable_hexes(
                self.units[unit_index].hex_column,
                self.units[unit_index].hex_row,
                self.units[unit_index].attack_range,
                &self.units,
                Faction::Player,
            );
            if !self.attackable_units.is_empty() {
                self.phase = Phase::Attack;
            } else {
                self.phase = Phase::Select;
                self.selected_unit_index = None;
            }
        } else {
            self.phase = Phase::Select;
            self.selected_unit_index = None;
        }

        self.reachable_hexes.clear();
    }

    fn handle_attack_unit(&mut self, world: &mut World) {
        let unit_index = match self.selected_unit_index {
            Some(index) => index,
            None => return,
        };

        let target_unit_index = self.unit_at(self.cursor_column, self.cursor_row);
        let target_index = match target_unit_index {
            Some(index) if self.attackable_units.contains(&index) => index,
            _ => return,
        };

        self.execute_attack(unit_index, target_index, world);
        self.units[unit_index].has_attacked = true;

        if self.units[unit_index].remaining_movement > 0 {
            self.phase = Phase::Move;
            self.reachable_hexes = find_reachable_hexes(
                self.units[unit_index].hex_column,
                self.units[unit_index].hex_row,
                self.units[unit_index].remaining_movement,
                &self.terrain,
                &self.units,
                Faction::Player,
            );
            self.attackable_units.clear();
        } else {
            self.phase = Phase::Select;
            self.selected_unit_index = None;
            self.attackable_units.clear();
        }
    }

    fn execute_attack(&mut self, attacker_index: usize, defender_index: usize, world: &mut World) {
        let attacker_attack = self.units[attacker_index].attack;
        let defender_terrain =
            self.terrain[self.units[defender_index].hex_row][self.units[defender_index].hex_column];
        let terrain_def = defense_bonus(defender_terrain);
        let damage = (attacker_attack - terrain_def).max(1);

        let attacker_kind = match self.units[attacker_index].kind {
            UnitKind::Warrior => "Warrior",
            UnitKind::Archer => "Archer",
            UnitKind::Cavalry => "Cavalry",
        };
        let defender_kind = match self.units[defender_index].kind {
            UnitKind::Warrior => "Warrior",
            UnitKind::Archer => "Archer",
            UnitKind::Cavalry => "Cavalry",
        };
        let attacker_faction = if self.units[attacker_index].faction == Faction::Player {
            "Your"
        } else {
            "Enemy"
        };

        self.units[defender_index].health -= damage;

        let (screen_column, screen_row) = hex_to_screen(
            self.units[defender_index].hex_column,
            self.units[defender_index].hex_row,
        );
        self.particles.emit(
            world,
            (self.offset_column + screen_column + 1) as f64,
            (self.offset_row + screen_row) as f64,
            4,
            &ParticleConfig {
                characters: vec!['*', '+', '!'],
                colors: vec![TermColor::Red, TermColor::Yellow, TermColor::DarkRed],
                lifetime: 0.4,
                speed_min: 1.0,
                speed_max: 4.0,
                spread: std::f64::consts::PI * 2.0,
                direction: 0.0,
                z_index: 8,
            },
        );

        if self.units[defender_index].health <= 0 {
            self.add_message(format!(
                "{} {} attacks {} for {} damage - KILLED!",
                attacker_faction, attacker_kind, defender_kind, damage,
            ));
        } else {
            self.add_message(format!(
                "{} {} attacks {} for {} damage. ({}hp left)",
                attacker_faction,
                attacker_kind,
                defender_kind,
                damage,
                self.units[defender_index].health,
            ));
        }
    }

    fn check_town_capture(
        &mut self,
        hex_column: usize,
        hex_row: usize,
        capturing_faction: Faction,
        _world: &mut World,
    ) {
        let mut captured = false;
        for town in &mut self.towns {
            if town.hex_column == hex_column
                && town.hex_row == hex_row
                && town.faction != capturing_faction
            {
                let old_faction = town.faction;
                let defenders_present = self.units.iter().any(|unit| {
                    unit.hex_column == hex_column
                        && unit.hex_row == hex_row
                        && unit.faction == old_faction
                        && unit.health > 0
                });
                if !defenders_present {
                    town.faction = capturing_faction;
                    captured = true;
                }
            }
        }
        if captured {
            self.game_result = Some(capturing_faction);
            let faction_name = match capturing_faction {
                Faction::Player => "Player",
                Faction::Enemy => "Enemy",
            };
            self.add_message(format!("{} captured a town!", faction_name));
        }
    }

    fn try_recruit(&mut self, kind: UnitKind) {
        let player_town = match self.player_town_position() {
            Some(position) => position,
            None => return,
        };

        let unit = Unit::new(kind, Faction::Player, 0, 0);
        let cost = unit.cost();

        if self.player_gold < cost {
            self.add_message("Not enough gold!".to_string());
            return;
        }

        let occupied = self.units.iter().any(|existing| {
            existing.hex_column == player_town.0
                && existing.hex_row == player_town.1
                && existing.health > 0
        });
        if occupied {
            let mut placed = false;
            for (neighbor_column, neighbor_row) in hex_neighbors(player_town.0, player_town.1) {
                if !is_passable(self.terrain[neighbor_row][neighbor_column]) {
                    continue;
                }
                let neighbor_occupied = self.units.iter().any(|existing| {
                    existing.hex_column == neighbor_column
                        && existing.hex_row == neighbor_row
                        && existing.health > 0
                });
                if !neighbor_occupied {
                    let mut new_unit =
                        Unit::new(kind, Faction::Player, neighbor_column, neighbor_row);
                    new_unit.remaining_movement = 0;
                    new_unit.has_attacked = true;
                    self.units.push(new_unit);
                    self.player_gold -= cost;
                    placed = true;
                    let kind_name = match kind {
                        UnitKind::Warrior => "Warrior",
                        UnitKind::Archer => "Archer",
                        UnitKind::Cavalry => "Cavalry",
                    };
                    self.add_message(format!("Recruited {} for {} gold.", kind_name, cost));
                    break;
                }
            }
            if !placed {
                self.add_message("No space to recruit!".to_string());
            }
        } else {
            let mut new_unit = Unit::new(kind, Faction::Player, player_town.0, player_town.1);
            new_unit.remaining_movement = 0;
            new_unit.has_attacked = true;
            self.units.push(new_unit);
            self.player_gold -= cost;
            let kind_name = match kind {
                UnitKind::Warrior => "Warrior",
                UnitKind::Archer => "Archer",
                UnitKind::Cavalry => "Cavalry",
            };
            self.add_message(format!("Recruited {} for {} gold.", kind_name, cost));
        }

        self.recruit_mode = false;
    }

    fn run_ai(&mut self, world: &mut World) {
        let enemy_unit_indices: Vec<usize> = self
            .units
            .iter()
            .enumerate()
            .filter(|(_, unit)| unit.faction == Faction::Enemy && unit.health > 0)
            .map(|(index, _)| index)
            .collect();

        if self.ai_unit_index >= enemy_unit_indices.len() {
            self.ai_try_recruit();
            self.ai_action_pending = false;
            self.turn_number += 1;
            self.begin_player_turn();
            return;
        }

        let unit_index = enemy_unit_indices[self.ai_unit_index];
        self.ai_unit_index += 1;

        let unit_column = self.units[unit_index].hex_column;
        let unit_row = self.units[unit_index].hex_row;
        let unit_movement = self.units[unit_index].remaining_movement;
        let unit_attack_range = self.units[unit_index].attack_range;

        let attackable_before_move = find_attackable_hexes(
            unit_column,
            unit_row,
            unit_attack_range,
            &self.units,
            Faction::Enemy,
        );

        if !attackable_before_move.is_empty() {
            let mut best_target = attackable_before_move[0];
            let mut lowest_health = self.units[best_target].health;
            for &target_index in &attackable_before_move {
                if self.units[target_index].health < lowest_health {
                    lowest_health = self.units[target_index].health;
                    best_target = target_index;
                }
            }
            self.execute_attack(unit_index, best_target, world);
            self.units[unit_index].has_attacked = true;
        }

        if unit_movement > 0 {
            let reachable = find_reachable_hexes(
                self.units[unit_index].hex_column,
                self.units[unit_index].hex_row,
                self.units[unit_index].remaining_movement,
                &self.terrain,
                &self.units,
                Faction::Enemy,
            );

            let player_town = self.player_town_position();

            let mut best_hex: Option<(usize, usize)> = None;
            let mut best_score = i32::MIN;

            for &(reachable_column, reachable_row) in &reachable {
                let occupied = self.units.iter().any(|other| {
                    other.hex_column == reachable_column
                        && other.hex_row == reachable_row
                        && other.health > 0
                });
                if occupied {
                    continue;
                }

                let mut score: i32 = 0;

                let targets_from_here = find_attackable_hexes(
                    reachable_column,
                    reachable_row,
                    unit_attack_range,
                    &self.units,
                    Faction::Enemy,
                );
                if !targets_from_here.is_empty() && !self.units[unit_index].has_attacked {
                    score += 20;
                    let mut weakest_health = i32::MAX;
                    for &target_index in &targets_from_here {
                        if self.units[target_index].health < weakest_health {
                            weakest_health = self.units[target_index].health;
                        }
                    }
                    score += (10 - weakest_health).max(0);
                }

                if let Some((town_column, town_row)) = player_town {
                    let current_dist = hex_distance(
                        self.units[unit_index].hex_column,
                        self.units[unit_index].hex_row,
                        town_column,
                        town_row,
                    );
                    let new_dist =
                        hex_distance(reachable_column, reachable_row, town_column, town_row);
                    score += (current_dist - new_dist) * 5;

                    if reachable_column == town_column && reachable_row == town_row {
                        score += 100;
                    }
                }

                if score > best_score {
                    best_score = score;
                    best_hex = Some((reachable_column, reachable_row));
                }
            }

            if let Some((move_column, move_row)) = best_hex {
                let distance = hex_distance(
                    self.units[unit_index].hex_column,
                    self.units[unit_index].hex_row,
                    move_column,
                    move_row,
                );
                self.units[unit_index].hex_column = move_column;
                self.units[unit_index].hex_row = move_row;
                self.units[unit_index].remaining_movement =
                    (self.units[unit_index].remaining_movement - distance).max(0);

                self.check_town_capture(move_column, move_row, Faction::Enemy, world);
            }
        }

        if !self.units[unit_index].has_attacked {
            let attackable_after_move = find_attackable_hexes(
                self.units[unit_index].hex_column,
                self.units[unit_index].hex_row,
                self.units[unit_index].attack_range,
                &self.units,
                Faction::Enemy,
            );
            if !attackable_after_move.is_empty() {
                let mut best_target = attackable_after_move[0];
                let mut lowest_health = self.units[best_target].health;
                for &target_index in &attackable_after_move {
                    if self.units[target_index].health < lowest_health {
                        lowest_health = self.units[target_index].health;
                        best_target = target_index;
                    }
                }
                self.execute_attack(unit_index, best_target, world);
                self.units[unit_index].has_attacked = true;
            }
        }
    }

    fn ai_try_recruit(&mut self) {
        let enemy_town = match self.enemy_town_position() {
            Some(position) => position,
            None => return,
        };

        while self.enemy_gold >= 2 {
            let kind = if self.enemy_gold >= 4 {
                let mut rng = rand::rng();
                let roll: f64 = rng.random();
                if roll < 0.3 {
                    UnitKind::Cavalry
                } else if roll < 0.6 {
                    UnitKind::Warrior
                } else {
                    UnitKind::Archer
                }
            } else if self.enemy_gold >= 3 {
                let mut rng = rand::rng();
                if rng.random_bool(0.5) {
                    UnitKind::Warrior
                } else {
                    UnitKind::Archer
                }
            } else {
                UnitKind::Archer
            };

            let cost = Unit::new(kind, Faction::Enemy, 0, 0).cost();
            if self.enemy_gold < cost {
                break;
            }

            let town_occupied = self.units.iter().any(|unit| {
                unit.hex_column == enemy_town.0 && unit.hex_row == enemy_town.1 && unit.health > 0
            });

            if !town_occupied {
                let mut new_unit = Unit::new(kind, Faction::Enemy, enemy_town.0, enemy_town.1);
                new_unit.remaining_movement = 0;
                new_unit.has_attacked = true;
                self.units.push(new_unit);
                self.enemy_gold -= cost;
            } else {
                let mut placed = false;
                for (neighbor_column, neighbor_row) in hex_neighbors(enemy_town.0, enemy_town.1) {
                    if !is_passable(self.terrain[neighbor_row][neighbor_column]) {
                        continue;
                    }
                    let neighbor_occupied = self.units.iter().any(|existing| {
                        existing.hex_column == neighbor_column
                            && existing.hex_row == neighbor_row
                            && existing.health > 0
                    });
                    if !neighbor_occupied {
                        let mut new_unit =
                            Unit::new(kind, Faction::Enemy, neighbor_column, neighbor_row);
                        new_unit.remaining_movement = 0;
                        new_unit.has_attacked = true;
                        self.units.push(new_unit);
                        self.enemy_gold -= cost;
                        placed = true;
                        break;
                    }
                }
                if !placed {
                    break;
                }
            }
        }
    }

    fn clean_dead_units(&mut self) {
        self.units.retain(|unit| unit.health > 0);
    }

    fn clear_all(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);
        self.highlight_entities.despawn_all(world);
        self.unit_entities.despawn_all(world);
        self.particles.despawn_all(world);
        if self.tilemap_entity != Entity::default() {
            world.despawn_entities(&[self.tilemap_entity]);
        }
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Hex Strategy - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        self.terrain = generate_terrain();

        self.towns.push(Town {
            faction: Faction::Player,
            hex_column: 1,
            hex_row: 1,
        });
        self.towns.push(Town {
            faction: Faction::Enemy,
            hex_column: HEX_COLUMNS - 2,
            hex_row: HEX_ROWS - 2,
        });

        let player_kinds = [UnitKind::Warrior, UnitKind::Archer, UnitKind::Cavalry];
        for &kind in &player_kinds {
            let neighbors = hex_neighbors(1, 1);
            let mut placed = false;
            for &(neighbor_column, neighbor_row) in &neighbors {
                if !is_passable(self.terrain[neighbor_row][neighbor_column]) {
                    continue;
                }
                let already_taken = self
                    .units
                    .iter()
                    .any(|unit| unit.hex_column == neighbor_column && unit.hex_row == neighbor_row);
                if !already_taken {
                    self.units.push(Unit::new(
                        kind,
                        Faction::Player,
                        neighbor_column,
                        neighbor_row,
                    ));
                    placed = true;
                    break;
                }
            }
            if !placed {
                self.units.push(Unit::new(kind, Faction::Player, 1, 1));
            }
        }

        let enemy_kinds = [UnitKind::Warrior, UnitKind::Archer, UnitKind::Cavalry];
        let enemy_town_column = HEX_COLUMNS - 2;
        let enemy_town_row = HEX_ROWS - 2;
        for &kind in &enemy_kinds {
            let neighbors = hex_neighbors(enemy_town_column, enemy_town_row);
            let mut placed = false;
            for &(neighbor_column, neighbor_row) in &neighbors {
                if !is_passable(self.terrain[neighbor_row][neighbor_column]) {
                    continue;
                }
                let already_taken = self
                    .units
                    .iter()
                    .any(|unit| unit.hex_column == neighbor_column && unit.hex_row == neighbor_row);
                if !already_taken {
                    self.units.push(Unit::new(
                        kind,
                        Faction::Enemy,
                        neighbor_column,
                        neighbor_row,
                    ));
                    placed = true;
                    break;
                }
            }
            if !placed {
                self.units.push(Unit::new(
                    kind,
                    Faction::Enemy,
                    enemy_town_column,
                    enemy_town_row,
                ));
            }
        }

        self.build_tilemap(world);
        self.render_units(world);
        self.render_highlights(world);
        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        if self.turn_owner != TurnOwner::Player || self.game_result.is_some() {
            return;
        }

        if self.recruit_mode {
            match key {
                KeyCode::Up => {
                    if self.recruit_selection > 0 {
                        self.recruit_selection -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.recruit_selection < 2 {
                        self.recruit_selection += 1;
                    }
                }
                KeyCode::Enter => {
                    let kind = match self.recruit_selection {
                        0 => UnitKind::Warrior,
                        1 => UnitKind::Archer,
                        _ => UnitKind::Cavalry,
                    };
                    self.try_recruit(kind);
                    self.render_units(world);
                    self.render_highlights(world);
                    self.update_hud(world);
                }
                KeyCode::Escape => {
                    self.recruit_mode = false;
                    self.update_hud(world);
                }
                _ => {}
            }
            self.update_hud(world);
            return;
        }

        match key {
            KeyCode::Up => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                }
            }
            KeyCode::Down => {
                if self.cursor_row < HEX_ROWS - 1 {
                    self.cursor_row += 1;
                }
            }
            KeyCode::Left => {
                if self.cursor_column > 0 {
                    self.cursor_column -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_column < HEX_COLUMNS - 1 {
                    self.cursor_column += 1;
                }
            }
            KeyCode::Enter => {
                match self.phase {
                    Phase::Select => self.handle_select_unit(),
                    Phase::Move => self.handle_move_unit(world),
                    Phase::Attack => self.handle_attack_unit(world),
                }
                self.clean_dead_units();
            }
            KeyCode::Escape => {
                self.selected_unit_index = None;
                self.phase = Phase::Select;
                self.reachable_hexes.clear();
                self.attackable_units.clear();
            }
            KeyCode::Char(' ') => {
                self.end_player_turn();
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if let Some((town_column, town_row)) = self.player_town_position()
                    && self.towns.iter().any(|town| {
                        town.hex_column == town_column
                            && town.hex_row == town_row
                            && town.faction == Faction::Player
                    })
                {
                    self.recruit_mode = true;
                    self.recruit_selection = 0;
                }
            }
            _ => {}
        }

        self.render_units(world);
        self.render_highlights(world);
        self.update_hud(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;
        self.particles.update(world, delta);

        if self.game_result.is_some() {
            return;
        }

        if self.turn_owner == TurnOwner::Ai && self.ai_action_pending && self.ai_timer.tick(delta) {
            self.run_ai(world);
            self.clean_dead_units();
            self.ai_timer.reset();
            self.render_units(world);
            self.render_highlights(world);
            self.update_hud(world);
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if let Some(winner) = self.game_result {
            self.clear_all(world);
            return Some(Box::new(EndScreenState {
                winner,
                turn_count: self.turn_number,
                entities: EntityGroup::new(),
                restart: false,
            }));
        }
        None
    }
}

struct EndScreenState {
    winner: Faction,
    turn_count: u32,
    entities: EntityGroup,
    restart: bool,
}

impl State for EndScreenState {
    fn title(&self) -> &str {
        "Hex Strategy - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let (result_text, result_color) = match self.winner {
            Faction::Player => ("VICTORY!", TermColor::Green),
            Faction::Enemy => ("DEFEAT!", TermColor::Red),
        };

        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - result_text.len() as f64 / 2.0,
                row: center_row - 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: result_text.to_string(),
                foreground: result_color,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let summary = format!("Game ended on turn {}", self.turn_count);
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - summary.len() as f64 / 2.0,
                row: center_row - 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: summary,
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let flavor = match self.winner {
            Faction::Player => "You have captured the enemy stronghold!",
            Faction::Enemy => "The enemy has overrun your town...",
        };
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - flavor.len() as f64 / 2.0,
                row: center_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: flavor.to_string(),
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let restart_prompt = "Press R to play again";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - restart_prompt.len() as f64 / 2.0,
                row: center_row + 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: restart_prompt.to_string(),
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let quit_prompt = "Press ESC to quit";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - quit_prompt.len() as f64 / 2.0,
                row: center_row + 5.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: quit_prompt.to_string(),
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
            KeyCode::Char('r') | KeyCode::Char('R') => self.restart = true,
            KeyCode::Escape => world.resources.should_exit = true,
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
