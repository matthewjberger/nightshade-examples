use nightshade::tui::prelude::*;
use rand::Rng;

const MAZE_WIDTH: usize = 28;
const MAZE_HEIGHT: usize = 31;
const TILE_WALL: u8 = 1;
const TILE_DOT: u8 = 2;
const TILE_POWER: u8 = 3;
const TILE_EMPTY: u8 = 0;
const TILE_GATE: u8 = 4;

const GHOST_FRIGHTENED_DURATION: f64 = 8.0;
const GHOST_SCATTER_DURATION: f64 = 7.0;
const GHOST_CHASE_DURATION: f64 = 20.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl Direction {
    fn delta(self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
            Self::None => (0, 0),
        }
    }

    fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::None => Self::None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GhostPersonality {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

impl GhostPersonality {
    fn color(self) -> TermColor {
        match self {
            Self::Blinky => TermColor::Red,
            Self::Pinky => TermColor::Rgb {
                r: 255,
                g: 184,
                b: 255,
            },
            Self::Inky => TermColor::Cyan,
            Self::Clyde => TermColor::Rgb {
                r: 255,
                g: 184,
                b: 82,
            },
        }
    }

    fn scatter_target(self) -> (i32, i32) {
        match self {
            Self::Blinky => (MAZE_WIDTH as i32 - 3, 0),
            Self::Pinky => (2, 0),
            Self::Inky => (MAZE_WIDTH as i32 - 1, MAZE_HEIGHT as i32 - 1),
            Self::Clyde => (0, MAZE_HEIGHT as i32 - 1),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GhostMode {
    Chase,
    Scatter,
    Frightened,
}

struct Ghost {
    column: i32,
    row: i32,
    direction: Direction,
    personality: GhostPersonality,
    mode: GhostMode,
    frightened_timer: f64,
    in_house: bool,
    release_timer: f64,
}

fn build_maze() -> [[u8; MAZE_WIDTH]; MAZE_HEIGHT] {
    let layout: [&str; MAZE_HEIGHT] = [
        "1111111111111111111111111111",
        "1222222222222112222222222221",
        "1211112111112112111121111121",
        "1311112111112112111121111131",
        "1211112111112112111121111121",
        "1222222222222222222222222221",
        "1211112112111111211211112121",
        "1211112112111111211211112121",
        "1222222112222112222112222221",
        "1111112111110110111112111111",
        "0000012111110110111112100000",
        "0000012110000000001112100000",
        "0000012110144444101112100000",
        "1111112110100000101112111111",
        "0000002000100000100002000000",
        "1111112110100000101112111111",
        "0000012110111111101112100000",
        "0000012110000000001112100000",
        "0000012110111111101112100000",
        "1111112110111111101112111111",
        "1222222222222112222222222221",
        "1211112111112112111121111121",
        "1211112111112112111121111121",
        "1322112222222002222222112231",
        "1112112112111111211211211121",
        "1112112112111111211211211121",
        "1222222112222112222112222221",
        "1211111111112112111111111121",
        "1211111111112112111111111121",
        "1222222222222222222222222221",
        "1111111111111111111111111111",
    ];

    let mut maze = [[TILE_EMPTY; MAZE_WIDTH]; MAZE_HEIGHT];
    for (row, line) in layout.iter().enumerate() {
        for (column, character) in line.chars().enumerate() {
            if column < MAZE_WIDTH {
                maze[row][column] = match character {
                    '1' => TILE_WALL,
                    '2' => TILE_DOT,
                    '3' => TILE_POWER,
                    '4' => TILE_GATE,
                    _ => TILE_EMPTY,
                };
            }
        }
    }
    maze
}

fn is_walkable(
    maze: &[[u8; MAZE_WIDTH]; MAZE_HEIGHT],
    column: i32,
    row: i32,
    is_ghost: bool,
) -> bool {
    if row < 0 || row >= MAZE_HEIGHT as i32 {
        return false;
    }
    let wrapped_column = ((column % MAZE_WIDTH as i32) + MAZE_WIDTH as i32) as usize % MAZE_WIDTH;
    let tile = maze[row as usize][wrapped_column];
    if tile == TILE_WALL {
        return false;
    }
    if tile == TILE_GATE && !is_ghost {
        return false;
    }
    true
}

fn ghost_target(
    ghost: &Ghost,
    player_column: i32,
    player_row: i32,
    player_direction: Direction,
    blinky_column: i32,
    blinky_row: i32,
) -> (i32, i32) {
    match ghost.mode {
        GhostMode::Frightened => {
            let mut rng = rand::rng();
            (
                rng.random_range(0..MAZE_WIDTH as i32),
                rng.random_range(0..MAZE_HEIGHT as i32),
            )
        }
        GhostMode::Scatter => ghost.personality.scatter_target(),
        GhostMode::Chase => match ghost.personality {
            GhostPersonality::Blinky => (player_column, player_row),
            GhostPersonality::Pinky => {
                let (delta_column, delta_row) = player_direction.delta();
                (player_column + delta_column * 4, player_row + delta_row * 4)
            }
            GhostPersonality::Inky => {
                let (delta_column, delta_row) = player_direction.delta();
                let ahead_column = player_column + delta_column * 2;
                let ahead_row = player_row + delta_row * 2;
                (2 * ahead_column - blinky_column, 2 * ahead_row - blinky_row)
            }
            GhostPersonality::Clyde => {
                let distance_column = ghost.column - player_column;
                let distance_row = ghost.row - player_row;
                let distance_squared =
                    distance_column * distance_column + distance_row * distance_row;
                if distance_squared > 64 {
                    (player_column, player_row)
                } else {
                    ghost.personality.scatter_target()
                }
            }
        },
    }
}

fn choose_ghost_direction(
    maze: &[[u8; MAZE_WIDTH]; MAZE_HEIGHT],
    ghost: &Ghost,
    target: (i32, i32),
) -> Direction {
    let directions = [
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right,
    ];
    let opposite = ghost.direction.opposite();
    let mut best_direction = ghost.direction;
    let mut best_distance = i64::MAX;

    for &direction in &directions {
        if direction == opposite {
            continue;
        }
        let (delta_column, delta_row) = direction.delta();
        let next_column = ghost.column + delta_column;
        let next_row = ghost.row + delta_row;
        if !is_walkable(maze, next_column, next_row, true) {
            continue;
        }

        let distance_column = (next_column - target.0) as i64;
        let distance_row = (next_row - target.1) as i64;
        let distance = distance_column * distance_column + distance_row * distance_row;
        if distance < best_distance {
            best_distance = distance;
            best_direction = direction;
        }
    }

    best_direction
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Pac-Man - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "PAC-MAN";
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

        let pac_art = "C . . . o . . .";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - pac_art.len() as f64 / 2.0,
                row: center_row - 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: pac_art.to_string(),
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let ghost_art = "M  M  M  M";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - ghost_art.len() as f64 / 2.0,
                row: center_row - 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: ghost_art.to_string(),
                foreground: TermColor::Red,
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
                row: center_row + 2.0,
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
                row: center_row + 4.0,
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
    maze: [[u8; MAZE_WIDTH]; MAZE_HEIGHT],
    tilemap_entity: Entity,
    offset_column: i32,
    offset_row: i32,
    player_column: i32,
    player_row: i32,
    player_direction: Direction,
    queued_direction: Direction,
    player_entity: Entity,
    player_mouth_open: bool,
    mouth_timer: f64,
    ghosts: Vec<Ghost>,
    ghost_entities: Vec<Entity>,
    move_timer: f64,
    move_interval: f64,
    ghost_move_timer: f64,
    ghost_move_interval: f64,
    mode_timer: f64,
    is_scatter: bool,
    score: u32,
    lives: u32,
    dots_remaining: u32,
    level: u32,
    entities: EntityGroup,
    game_over: bool,
    won: bool,
    particles: ParticleEmitter,
}

impl GameplayState {
    fn new() -> Self {
        let maze = build_maze();
        let mut dots = 0u32;
        for row in &maze {
            for &cell in row {
                if cell == TILE_DOT || cell == TILE_POWER {
                    dots += 1;
                }
            }
        }

        Self {
            maze,
            tilemap_entity: Entity::default(),
            offset_column: 0,
            offset_row: 0,
            player_column: 14,
            player_row: 23,
            player_direction: Direction::Left,
            queued_direction: Direction::None,
            player_entity: Entity::default(),
            player_mouth_open: true,
            mouth_timer: 0.0,
            ghosts: vec![
                Ghost {
                    column: 14,
                    row: 11,
                    direction: Direction::Left,
                    personality: GhostPersonality::Blinky,
                    mode: GhostMode::Scatter,
                    frightened_timer: 0.0,
                    in_house: false,
                    release_timer: 0.0,
                },
                Ghost {
                    column: 14,
                    row: 14,
                    direction: Direction::Up,
                    personality: GhostPersonality::Pinky,
                    mode: GhostMode::Scatter,
                    frightened_timer: 0.0,
                    in_house: true,
                    release_timer: 3.0,
                },
                Ghost {
                    column: 12,
                    row: 14,
                    direction: Direction::Up,
                    personality: GhostPersonality::Inky,
                    mode: GhostMode::Scatter,
                    frightened_timer: 0.0,
                    in_house: true,
                    release_timer: 6.0,
                },
                Ghost {
                    column: 16,
                    row: 14,
                    direction: Direction::Up,
                    personality: GhostPersonality::Clyde,
                    mode: GhostMode::Scatter,
                    frightened_timer: 0.0,
                    in_house: true,
                    release_timer: 9.0,
                },
            ],
            ghost_entities: Vec::new(),
            move_timer: 0.0,
            move_interval: 0.15,
            ghost_move_timer: 0.0,
            ghost_move_interval: 0.18,
            mode_timer: 0.0,
            is_scatter: true,
            score: 0,
            lives: 3,
            dots_remaining: dots,
            level: 1,
            entities: EntityGroup::new(),
            game_over: false,
            won: false,
            particles: ParticleEmitter::new(),
        }
    }

    fn build_tilemap(&mut self, world: &mut World) {
        let terminal = world.resources.terminal_size;
        self.offset_column = ((terminal.columns as i32 - MAZE_WIDTH as i32) / 2).max(0);
        self.offset_row = ((terminal.rows as i32 - MAZE_HEIGHT as i32 - 2) / 2).max(0);

        let mut tilemap = Tilemap::new(MAZE_WIDTH, MAZE_HEIGHT);
        for row in 0..MAZE_HEIGHT {
            for column in 0..MAZE_WIDTH {
                let cell = match self.maze[row][column] {
                    TILE_WALL => TilemapCell {
                        character: '#',
                        foreground: TermColor::Rgb {
                            r: 33,
                            g: 33,
                            b: 222,
                        },
                        background: TermColor::Black,
                    },
                    TILE_DOT => TilemapCell {
                        character: '.',
                        foreground: TermColor::Rgb {
                            r: 255,
                            g: 183,
                            b: 174,
                        },
                        background: TermColor::Black,
                    },
                    TILE_POWER => TilemapCell {
                        character: 'O',
                        foreground: TermColor::Rgb {
                            r: 255,
                            g: 183,
                            b: 174,
                        },
                        background: TermColor::Black,
                    },
                    TILE_GATE => TilemapCell {
                        character: '-',
                        foreground: TermColor::Rgb {
                            r: 255,
                            g: 184,
                            b: 255,
                        },
                        background: TermColor::Black,
                    },
                    _ => TilemapCell {
                        character: ' ',
                        foreground: TermColor::Black,
                        background: TermColor::Black,
                    },
                };
                tilemap.set(column, row, cell);
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

    fn spawn_characters(&mut self, world: &mut World) {
        self.player_entity = EntityBuilder::new()
            .position(Position {
                column: self.offset_column as f64 + self.player_column as f64,
                row: self.offset_row as f64 + self.player_row as f64,
            })
            .sprite(Sprite {
                character: 'C',
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            })
            .z_index(ZIndex(5))
            .spawn(world);

        for ghost in &self.ghosts {
            let entity = EntityBuilder::new()
                .position(Position {
                    column: self.offset_column as f64 + ghost.column as f64,
                    row: self.offset_row as f64 + ghost.row as f64,
                })
                .sprite(Sprite {
                    character: 'M',
                    foreground: ghost.personality.color(),
                    background: TermColor::Black,
                })
                .z_index(ZIndex(4))
                .spawn(world);
            self.ghost_entities.push(entity);
        }
    }

    fn update_tilemap_cell(&self, world: &mut World, column: usize, row: usize) {
        if let Some(tilemap) = world.get_tilemap_mut(self.tilemap_entity) {
            let cell = match self.maze[row][column] {
                TILE_DOT => TilemapCell {
                    character: '.',
                    foreground: TermColor::Rgb {
                        r: 255,
                        g: 183,
                        b: 174,
                    },
                    background: TermColor::Black,
                },
                TILE_POWER => TilemapCell {
                    character: 'O',
                    foreground: TermColor::Rgb {
                        r: 255,
                        g: 183,
                        b: 174,
                    },
                    background: TermColor::Black,
                },
                _ => TilemapCell {
                    character: ' ',
                    foreground: TermColor::Black,
                    background: TermColor::Black,
                },
            };
            tilemap.set(column, row, cell);
        }
    }

    fn move_player(&mut self, world: &mut World) {
        if self.queued_direction != Direction::None {
            let (delta_column, delta_row) = self.queued_direction.delta();
            let next_column = self.player_column + delta_column;
            let next_row = self.player_row + delta_row;
            if is_walkable(&self.maze, next_column, next_row, false) {
                self.player_direction = self.queued_direction;
                self.queued_direction = Direction::None;
            }
        }

        let (delta_column, delta_row) = self.player_direction.delta();
        let mut next_column = self.player_column + delta_column;
        let next_row = self.player_row + delta_row;

        if next_column < 0 {
            next_column = MAZE_WIDTH as i32 - 1;
        }
        if next_column >= MAZE_WIDTH as i32 {
            next_column = 0;
        }

        if is_walkable(&self.maze, next_column, next_row, false) {
            self.player_column = next_column;
            self.player_row = next_row;

            let tile = self.maze[self.player_row as usize][self.player_column as usize];
            if tile == TILE_DOT {
                self.maze[self.player_row as usize][self.player_column as usize] = TILE_EMPTY;
                self.score += 10;
                self.dots_remaining -= 1;
                self.update_tilemap_cell(
                    world,
                    self.player_column as usize,
                    self.player_row as usize,
                );
            } else if tile == TILE_POWER {
                self.maze[self.player_row as usize][self.player_column as usize] = TILE_EMPTY;
                self.score += 50;
                self.dots_remaining -= 1;
                self.update_tilemap_cell(
                    world,
                    self.player_column as usize,
                    self.player_row as usize,
                );
                for ghost in &mut self.ghosts {
                    if !ghost.in_house {
                        ghost.mode = GhostMode::Frightened;
                        ghost.frightened_timer = GHOST_FRIGHTENED_DURATION;
                        ghost.direction = ghost.direction.opposite();
                    }
                }
            }
        }

        if let Some(position) = world.get_position_mut(self.player_entity) {
            position.column = self.offset_column as f64 + self.player_column as f64;
            position.row = self.offset_row as f64 + self.player_row as f64;
        }

        let player_char = if self.player_mouth_open {
            match self.player_direction {
                Direction::Right => '>',
                Direction::Left => '<',
                Direction::Up => 'V',
                Direction::Down => '^',
                Direction::None => 'C',
            }
        } else {
            'O'
        };
        if let Some(sprite) = world.get_sprite_mut(self.player_entity) {
            sprite.character = player_char;
        }
    }

    fn move_ghosts(&mut self, world: &mut World) {
        let player_column = self.player_column;
        let player_row = self.player_row;
        let player_direction = self.player_direction;
        let blinky_column = self.ghosts.first().map_or(0, |ghost| ghost.column);
        let blinky_row = self.ghosts.first().map_or(0, |ghost| ghost.row);

        for ghost_index in 0..self.ghosts.len() {
            let ghost = &mut self.ghosts[ghost_index];

            if ghost.in_house {
                ghost.release_timer -= self.ghost_move_interval;
                if ghost.release_timer <= 0.0 {
                    ghost.in_house = false;
                    ghost.column = 14;
                    ghost.row = 11;
                    ghost.direction = Direction::Left;
                }
                continue;
            }

            let target = ghost_target(
                ghost,
                player_column,
                player_row,
                player_direction,
                blinky_column,
                blinky_row,
            );
            let new_direction = choose_ghost_direction(&self.maze, ghost, target);
            ghost.direction = new_direction;
            let (delta_column, delta_row) = ghost.direction.delta();
            let mut next_column = ghost.column + delta_column;
            let next_row = ghost.row + delta_row;

            if next_column < 0 {
                next_column = MAZE_WIDTH as i32 - 1;
            }
            if next_column >= MAZE_WIDTH as i32 {
                next_column = 0;
            }

            if is_walkable(&self.maze, next_column, next_row, true) {
                ghost.column = next_column;
                ghost.row = next_row;
            }

            if let Some(position) = world.get_position_mut(self.ghost_entities[ghost_index]) {
                position.column = self.offset_column as f64 + ghost.column as f64;
                position.row = self.offset_row as f64 + ghost.row as f64;
            }

            let ghost_char = if ghost.mode == GhostMode::Frightened {
                'W'
            } else {
                'M'
            };
            let ghost_color = if ghost.mode == GhostMode::Frightened {
                TermColor::Rgb {
                    r: 33,
                    g: 33,
                    b: 255,
                }
            } else {
                ghost.personality.color()
            };
            if let Some(sprite) = world.get_sprite_mut(self.ghost_entities[ghost_index]) {
                sprite.character = ghost_char;
                sprite.foreground = ghost_color;
            }
        }
    }

    fn check_collisions(&mut self, world: &mut World) {
        let mut eaten_ghost_indices = Vec::new();
        for (ghost_index, ghost) in self.ghosts.iter().enumerate() {
            if ghost.in_house {
                continue;
            }
            if ghost.column == self.player_column && ghost.row == self.player_row {
                if ghost.mode == GhostMode::Frightened {
                    eaten_ghost_indices.push(ghost_index);
                    self.score += 200;
                    self.particles.emit(
                        world,
                        self.offset_column as f64 + ghost.column as f64,
                        self.offset_row as f64 + ghost.row as f64,
                        5,
                        &ParticleConfig {
                            characters: vec!['*', '+'],
                            colors: vec![TermColor::Cyan, TermColor::White],
                            lifetime: 0.5,
                            speed_min: 2.0,
                            speed_max: 5.0,
                            spread: std::f64::consts::PI * 2.0,
                            direction: 0.0,
                            z_index: 8,
                        },
                    );
                } else {
                    self.lives = self.lives.saturating_sub(1);
                    if self.lives == 0 {
                        self.game_over = true;
                    } else {
                        self.player_column = 14;
                        self.player_row = 23;
                        self.player_direction = Direction::Left;
                        if let Some(position) = world.get_position_mut(self.player_entity) {
                            position.column = self.offset_column as f64 + 14.0;
                            position.row = self.offset_row as f64 + 23.0;
                        }
                    }
                    return;
                }
            }
        }

        for &ghost_index in eaten_ghost_indices.iter().rev() {
            self.ghosts[ghost_index].column = 14;
            self.ghosts[ghost_index].row = 14;
            self.ghosts[ghost_index].mode = GhostMode::Scatter;
            self.ghosts[ghost_index].frightened_timer = 0.0;
            self.ghosts[ghost_index].in_house = true;
            self.ghosts[ghost_index].release_timer = 3.0;
        }
    }

    fn update_hud(&mut self, world: &mut World) {
        self.entities.despawn_all(world);

        let hud_row = self.offset_row as f64 + MAZE_HEIGHT as f64 + 1.0;
        let score_text = format!(
            "Score: {}  Lives: {}  Level: {}",
            self.score, self.lives, self.level
        );
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: self.offset_column as f64,
                row: hud_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: score_text,
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let dots_text = format!("Dots: {}", self.dots_remaining);
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: self.offset_column as f64 + MAZE_WIDTH as f64 - dots_text.len() as f64,
                row: hud_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: dots_text,
                foreground: TermColor::Rgb {
                    r: 255,
                    g: 183,
                    b: 174,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));
    }

    fn clear_all(&mut self, world: &mut World) {
        self.entities.despawn_all(world);
        world.despawn_entities(&[self.tilemap_entity, self.player_entity]);
        world.despawn_entities(&self.ghost_entities);
        self.ghost_entities.clear();
        self.particles.despawn_all(world);
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Pac-Man - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        self.build_tilemap(world);
        self.spawn_characters(world);
        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, _world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed || self.game_over {
            return;
        }
        match key {
            KeyCode::Up | KeyCode::Char('w') => self.queued_direction = Direction::Up,
            KeyCode::Down | KeyCode::Char('s') => self.queued_direction = Direction::Down,
            KeyCode::Left | KeyCode::Char('a') => self.queued_direction = Direction::Left,
            KeyCode::Right | KeyCode::Char('d') => self.queued_direction = Direction::Right,
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.game_over {
            return;
        }

        let delta = world.resources.timing.delta_seconds;
        self.particles.update(world, delta);

        self.mouth_timer += delta;
        if self.mouth_timer >= 0.15 {
            self.mouth_timer = 0.0;
            self.player_mouth_open = !self.player_mouth_open;
        }

        self.move_timer += delta;
        if self.move_timer >= self.move_interval {
            self.move_timer = 0.0;
            self.move_player(world);
        }

        self.ghost_move_timer += delta;
        if self.ghost_move_timer >= self.ghost_move_interval {
            self.ghost_move_timer = 0.0;
            self.move_ghosts(world);
        }

        self.mode_timer += delta;
        let mode_limit = if self.is_scatter {
            GHOST_SCATTER_DURATION
        } else {
            GHOST_CHASE_DURATION
        };
        if self.mode_timer >= mode_limit {
            self.mode_timer = 0.0;
            self.is_scatter = !self.is_scatter;
            for ghost in &mut self.ghosts {
                if ghost.mode != GhostMode::Frightened {
                    ghost.mode = if self.is_scatter {
                        GhostMode::Scatter
                    } else {
                        GhostMode::Chase
                    };
                }
            }
        }

        for ghost in &mut self.ghosts {
            if ghost.mode == GhostMode::Frightened {
                ghost.frightened_timer -= delta;
                if ghost.frightened_timer <= 0.0 {
                    ghost.mode = if self.is_scatter {
                        GhostMode::Scatter
                    } else {
                        GhostMode::Chase
                    };
                }
            }
        }

        self.check_collisions(world);

        if self.dots_remaining == 0 {
            self.won = true;
            self.game_over = true;
        }

        self.update_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            let score = self.score;
            let won = self.won;
            self.clear_all(world);
            return Some(Box::new(GameOverState {
                score,
                won,
                entities: EntityGroup::new(),
                restart: false,
            }));
        }
        None
    }
}

struct GameOverState {
    score: u32,
    won: bool,
    entities: EntityGroup,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Pac-Man - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let (result_text, result_color) = if self.won {
            (
                "YOU WIN!",
                TermColor::Rgb {
                    r: 80,
                    g: 255,
                    b: 80,
                },
            )
        } else {
            ("GAME OVER", TermColor::Red)
        };

        let lines: Vec<(String, TermColor)> = vec![
            (result_text.to_string(), result_color),
            (String::new(), TermColor::Black),
            (format!("Score: {}", self.score), TermColor::Yellow),
            (String::new(), TermColor::Black),
            ("Press R to play again".to_string(), TermColor::White),
            ("Press ESC to quit".to_string(), TermColor::Grey),
        ];

        for (line_index, (text, color)) in lines.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - text.len() as f64 / 2.0,
                    row: center_row - 3.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: text.clone(),
                    foreground: *color,
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
            KeyCode::Char('r') => self.restart = true,
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
