use nightshade::tui::prelude::*;
use rand::Rng;

const GRID_WIDTH: usize = 13;
const GRID_HEIGHT: usize = 11;
const BOMB_FUSE_DURATION: f64 = 2.0;
const EXPLOSION_DURATION: f64 = 0.4;
const ENEMY_MOVE_INTERVAL: f64 = 0.35;
const ENEMY_COUNT: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Empty,
    IndestructibleWall,
    DestructibleWall,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PowerupKind {
    Range,
    BombCount,
    Speed,
}

struct Bomb {
    column: i32,
    row: i32,
    fuse_timer: f64,
    range: i32,
}

struct Explosion {
    column: i32,
    row: i32,
    timer: f64,
}

struct Enemy {
    column: i32,
    row: i32,
    entity: Entity,
    alive: bool,
}

struct Powerup {
    column: i32,
    row: i32,
    kind: PowerupKind,
    entity: Entity,
}

fn build_grid(rng: &mut impl Rng) -> [[CellKind; GRID_WIDTH]; GRID_HEIGHT] {
    let mut grid = [[CellKind::Empty; GRID_WIDTH]; GRID_HEIGHT];

    for (row, grid_row) in grid.iter_mut().enumerate() {
        for (column, cell) in grid_row.iter_mut().enumerate() {
            if row == 0
                || row == GRID_HEIGHT - 1
                || column == 0
                || column == GRID_WIDTH - 1
                || (row % 2 == 0 && column % 2 == 0)
            {
                *cell = CellKind::IndestructibleWall;
            }
        }
    }

    for (row, grid_row) in grid.iter_mut().enumerate().take(GRID_HEIGHT - 1).skip(1) {
        for (column, cell) in grid_row.iter_mut().enumerate().take(GRID_WIDTH - 1).skip(1) {
            if *cell == CellKind::IndestructibleWall {
                continue;
            }
            if row <= 2 && (column <= 2 || column >= GRID_WIDTH - 3) {
                continue;
            }
            if rng.random_range(0..100) < 65 {
                *cell = CellKind::DestructibleWall;
            }
        }
    }

    grid
}

fn is_in_blast_zone(
    bombs: &[Bomb],
    column: i32,
    row: i32,
    grid: &[[CellKind; GRID_WIDTH]; GRID_HEIGHT],
) -> bool {
    for bomb in bombs {
        if bomb.column == column && bomb.row == row {
            return true;
        }
        for &(delta_column, delta_row) in &[(0, -1), (0, 1), (-1, 0), (1, 0)] {
            for distance in 1..=bomb.range {
                let check_column = bomb.column + delta_column * distance;
                let check_row = bomb.row + delta_row * distance;
                if check_column < 0
                    || check_column >= GRID_WIDTH as i32
                    || check_row < 0
                    || check_row >= GRID_HEIGHT as i32
                {
                    break;
                }
                let cell = grid[check_row as usize][check_column as usize];
                if cell == CellKind::IndestructibleWall {
                    break;
                }
                if check_column == column && check_row == row {
                    return true;
                }
                if cell == CellKind::DestructibleWall {
                    break;
                }
            }
        }
    }
    false
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Bomberman - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "BOMBERMAN";
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
                foreground: TermColor::Red,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let bomb_art = "( B )";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - bomb_art.len() as f64 / 2.0,
                row: center_row - 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: bomb_art.to_string(),
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let controls_line_1 = "Arrow keys / WASD to move";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - controls_line_1.len() as f64 / 2.0,
                row: center_row - 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: controls_line_1.to_string(),
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let controls_line_2 = "SPACE to place bomb";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - controls_line_2.len() as f64 / 2.0,
                row: center_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: controls_line_2.to_string(),
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
    grid: [[CellKind; GRID_WIDTH]; GRID_HEIGHT],
    tilemap_entity: Entity,
    offset_column: i32,
    offset_row: i32,
    player_column: i32,
    player_row: i32,
    player_entity: Entity,
    player_speed: f64,
    player_move_timer: f64,
    player_bomb_range: i32,
    player_max_bombs: i32,
    player_alive: bool,
    player_invincible_timer: f64,
    bombs: Vec<Bomb>,
    bomb_entities: Vec<Entity>,
    explosions: Vec<Explosion>,
    explosion_entities: Vec<Entity>,
    enemies: Vec<Enemy>,
    powerups: Vec<Powerup>,
    enemy_move_timer: f64,
    hud_entities: EntityGroup,
    particles: ParticleEmitter,
    game_over: bool,
    won: bool,
    queued_direction: (i32, i32),
    score: u32,
}

impl GameplayState {
    fn new() -> Self {
        let mut rng = rand::rng();
        let grid = build_grid(&mut rng);

        Self {
            grid,
            tilemap_entity: Entity::default(),
            offset_column: 0,
            offset_row: 0,
            player_column: 1,
            player_row: 1,
            player_entity: Entity::default(),
            player_speed: 0.18,
            player_move_timer: 0.0,
            player_bomb_range: 2,
            player_max_bombs: 1,
            player_alive: true,
            player_invincible_timer: 0.0,
            bombs: Vec::new(),
            bomb_entities: Vec::new(),
            explosions: Vec::new(),
            explosion_entities: Vec::new(),
            enemies: Vec::new(),
            powerups: Vec::new(),
            enemy_move_timer: 0.0,
            hud_entities: EntityGroup::new(),
            particles: ParticleEmitter::new(),
            game_over: false,
            won: false,
            queued_direction: (0, 0),
            score: 0,
        }
    }

    fn build_tilemap(&mut self, world: &mut World) {
        let terminal = world.resources.terminal_size;
        self.offset_column = ((terminal.columns as i32 - GRID_WIDTH as i32) / 2).max(0);
        self.offset_row = ((terminal.rows as i32 - GRID_HEIGHT as i32 - 2) / 2).max(0);

        let mut tilemap = Tilemap::new(GRID_WIDTH, GRID_HEIGHT);
        for row in 0..GRID_HEIGHT {
            for column in 0..GRID_WIDTH {
                let cell = self.cell_to_tilemap(row, column);
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

    fn cell_to_tilemap(&self, row: usize, column: usize) -> TilemapCell {
        match self.grid[row][column] {
            CellKind::IndestructibleWall => TilemapCell {
                character: '#',
                foreground: TermColor::DarkGrey,
                background: TermColor::Rgb {
                    r: 40,
                    g: 40,
                    b: 40,
                },
            },
            CellKind::DestructibleWall => TilemapCell {
                character: '=',
                foreground: TermColor::Rgb {
                    r: 180,
                    g: 120,
                    b: 60,
                },
                background: TermColor::Black,
            },
            CellKind::Empty => TilemapCell {
                character: ' ',
                foreground: TermColor::Black,
                background: TermColor::Black,
            },
        }
    }

    fn spawn_player(&mut self, world: &mut World) {
        self.player_entity = EntityBuilder::new()
            .position(Position {
                column: self.offset_column as f64 + self.player_column as f64,
                row: self.offset_row as f64 + self.player_row as f64,
            })
            .sprite(Sprite {
                character: '@',
                foreground: TermColor::Cyan,
                background: TermColor::Black,
            })
            .z_index(ZIndex(5))
            .spawn(world);
    }

    fn spawn_enemies(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        let spawn_positions: Vec<(i32, i32)> = vec![
            (GRID_WIDTH as i32 - 2, 1),
            (1, GRID_HEIGHT as i32 - 2),
            (GRID_WIDTH as i32 - 2, GRID_HEIGHT as i32 - 2),
            (GRID_WIDTH as i32 / 2, 1),
            (GRID_WIDTH as i32 / 2, GRID_HEIGHT as i32 - 2),
        ];

        for index in 0..ENEMY_COUNT {
            let (spawn_column, spawn_row) = if index < spawn_positions.len() {
                spawn_positions[index]
            } else {
                let column = rng.random_range(1..GRID_WIDTH as i32 - 1);
                let row = rng.random_range(1..GRID_HEIGHT as i32 - 1);
                (column, row)
            };

            let final_column;
            let final_row;
            if self.grid[spawn_row as usize][spawn_column as usize] != CellKind::Empty {
                let mut found = false;
                let mut attempt_column = spawn_column;
                let mut attempt_row = spawn_row;
                for delta_column in -2..=2_i32 {
                    for delta_row in -2..=2_i32 {
                        let check_column = spawn_column + delta_column;
                        let check_row = spawn_row + delta_row;
                        if check_column >= 1
                            && check_column < GRID_WIDTH as i32 - 1
                            && check_row >= 1
                            && check_row < GRID_HEIGHT as i32 - 1
                            && self.grid[check_row as usize][check_column as usize]
                                == CellKind::Empty
                            && !(check_column == self.player_column && check_row == self.player_row)
                        {
                            attempt_column = check_column;
                            attempt_row = check_row;
                            found = true;
                            break;
                        }
                    }
                    if found {
                        break;
                    }
                }
                final_column = attempt_column;
                final_row = attempt_row;
            } else {
                final_column = spawn_column;
                final_row = spawn_row;
            }

            if self.grid[final_row as usize][final_column as usize] == CellKind::DestructibleWall {
                self.grid[final_row as usize][final_column as usize] = CellKind::Empty;
                if let Some(tilemap) = world.get_tilemap_mut(self.tilemap_entity) {
                    tilemap.set(
                        final_column as usize,
                        final_row as usize,
                        self.cell_to_tilemap(final_row as usize, final_column as usize),
                    );
                }
            }

            let entity = EntityBuilder::new()
                .position(Position {
                    column: self.offset_column as f64 + final_column as f64,
                    row: self.offset_row as f64 + final_row as f64,
                })
                .sprite(Sprite {
                    character: 'E',
                    foreground: TermColor::Red,
                    background: TermColor::Black,
                })
                .z_index(ZIndex(4))
                .spawn(world);

            self.enemies.push(Enemy {
                column: final_column,
                row: final_row,
                entity,
                alive: true,
            });
        }
    }

    fn update_tilemap_cell(&self, world: &mut World, column: usize, row: usize) {
        if let Some(tilemap) = world.get_tilemap_mut(self.tilemap_entity) {
            tilemap.set(column, row, self.cell_to_tilemap(row, column));
        }
    }

    fn place_bomb(&mut self, world: &mut World) {
        let active_bombs = self.bombs.len() as i32;
        if active_bombs >= self.player_max_bombs {
            return;
        }

        let already_has_bomb = self
            .bombs
            .iter()
            .any(|bomb| bomb.column == self.player_column && bomb.row == self.player_row);
        if already_has_bomb {
            return;
        }

        self.bombs.push(Bomb {
            column: self.player_column,
            row: self.player_row,
            fuse_timer: BOMB_FUSE_DURATION,
            range: self.player_bomb_range,
        });

        let entity = EntityBuilder::new()
            .position(Position {
                column: self.offset_column as f64 + self.player_column as f64,
                row: self.offset_row as f64 + self.player_row as f64,
            })
            .sprite(Sprite {
                character: 'B',
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            })
            .z_index(ZIndex(3))
            .spawn(world);

        self.bomb_entities.push(entity);
    }

    fn detonate_bomb(
        &mut self,
        bomb_index: usize,
        world: &mut World,
        chain_detonations: &mut Vec<usize>,
    ) {
        let bomb = &self.bombs[bomb_index];
        let bomb_column = bomb.column;
        let bomb_row = bomb.row;
        let bomb_range = bomb.range;

        self.spawn_explosion(world, bomb_column, bomb_row);

        let directions: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        for &(delta_column, delta_row) in &directions {
            for distance in 1..=bomb_range {
                let explosion_column = bomb_column + delta_column * distance;
                let explosion_row = bomb_row + delta_row * distance;

                if explosion_column < 0
                    || explosion_column >= GRID_WIDTH as i32
                    || explosion_row < 0
                    || explosion_row >= GRID_HEIGHT as i32
                {
                    break;
                }

                let cell = self.grid[explosion_row as usize][explosion_column as usize];

                if cell == CellKind::IndestructibleWall {
                    break;
                }

                self.spawn_explosion(world, explosion_column, explosion_row);

                if cell == CellKind::DestructibleWall {
                    self.grid[explosion_row as usize][explosion_column as usize] = CellKind::Empty;
                    self.update_tilemap_cell(
                        world,
                        explosion_column as usize,
                        explosion_row as usize,
                    );
                    self.maybe_spawn_powerup(world, explosion_column, explosion_row);
                    break;
                }

                for other_bomb_index in 0..self.bombs.len() {
                    if other_bomb_index == bomb_index {
                        continue;
                    }
                    if chain_detonations.contains(&other_bomb_index) {
                        continue;
                    }
                    if self.bombs[other_bomb_index].column == explosion_column
                        && self.bombs[other_bomb_index].row == explosion_row
                    {
                        chain_detonations.push(other_bomb_index);
                    }
                }
            }
        }
    }

    fn spawn_explosion(&mut self, world: &mut World, column: i32, row: i32) {
        let already_exploding = self
            .explosions
            .iter()
            .any(|explosion| explosion.column == column && explosion.row == row);
        if already_exploding {
            return;
        }

        self.explosions.push(Explosion {
            column,
            row,
            timer: EXPLOSION_DURATION,
        });

        let entity = EntityBuilder::new()
            .position(Position {
                column: self.offset_column as f64 + column as f64,
                row: self.offset_row as f64 + row as f64,
            })
            .sprite(Sprite {
                character: '*',
                foreground: TermColor::Rgb {
                    r: 255,
                    g: 100,
                    b: 0,
                },
                background: TermColor::Rgb {
                    r: 200,
                    g: 50,
                    b: 0,
                },
            })
            .z_index(ZIndex(6))
            .spawn(world);

        self.explosion_entities.push(entity);

        self.particles.emit(
            world,
            self.offset_column as f64 + column as f64,
            self.offset_row as f64 + row as f64,
            2,
            &ParticleConfig {
                characters: vec!['.', ',', '\''],
                colors: vec![
                    TermColor::Rgb {
                        r: 255,
                        g: 200,
                        b: 0,
                    },
                    TermColor::Rgb {
                        r: 255,
                        g: 100,
                        b: 0,
                    },
                    TermColor::Red,
                ],
                lifetime: 0.3,
                speed_min: 1.0,
                speed_max: 4.0,
                spread: std::f64::consts::PI * 2.0,
                direction: 0.0,
                z_index: 7,
            },
        );
    }

    fn maybe_spawn_powerup(&mut self, world: &mut World, column: i32, row: i32) {
        let mut rng = rand::rng();
        if rng.random_range(0..100) >= 30 {
            return;
        }

        let kind_roll = rng.random_range(0..3);
        let kind = match kind_roll {
            0 => PowerupKind::Range,
            1 => PowerupKind::BombCount,
            _ => PowerupKind::Speed,
        };

        let (character, foreground) = match kind {
            PowerupKind::Range => (
                'R',
                TermColor::Rgb {
                    r: 255,
                    g: 80,
                    b: 80,
                },
            ),
            PowerupKind::BombCount => (
                'N',
                TermColor::Rgb {
                    r: 80,
                    g: 80,
                    b: 255,
                },
            ),
            PowerupKind::Speed => (
                'S',
                TermColor::Rgb {
                    r: 80,
                    g: 255,
                    b: 80,
                },
            ),
        };

        let entity = EntityBuilder::new()
            .position(Position {
                column: self.offset_column as f64 + column as f64,
                row: self.offset_row as f64 + row as f64,
            })
            .sprite(Sprite {
                character,
                foreground,
                background: TermColor::Black,
            })
            .z_index(ZIndex(2))
            .spawn(world);

        self.powerups.push(Powerup {
            column,
            row,
            kind,
            entity,
        });
    }

    fn check_powerup_pickup(&mut self, world: &mut World) {
        let mut collected_indices = Vec::new();
        for (powerup_index, powerup) in self.powerups.iter().enumerate() {
            if powerup.column == self.player_column && powerup.row == self.player_row {
                collected_indices.push(powerup_index);
            }
        }

        for &powerup_index in collected_indices.iter().rev() {
            let powerup = &self.powerups[powerup_index];
            match powerup.kind {
                PowerupKind::Range => {
                    self.player_bomb_range += 1;
                    self.score += 100;
                }
                PowerupKind::BombCount => {
                    self.player_max_bombs += 1;
                    self.score += 100;
                }
                PowerupKind::Speed => {
                    self.player_speed = (self.player_speed - 0.03).max(0.08);
                    self.score += 100;
                }
            }
            world.despawn_entities(&[powerup.entity]);
            self.powerups.swap_remove(powerup_index);
        }
    }

    fn move_player(&mut self, world: &mut World) {
        let (delta_column, delta_row) = self.queued_direction;
        if delta_column == 0 && delta_row == 0 {
            return;
        }

        let next_column = self.player_column + delta_column;
        let next_row = self.player_row + delta_row;

        if next_column < 0
            || next_column >= GRID_WIDTH as i32
            || next_row < 0
            || next_row >= GRID_HEIGHT as i32
        {
            return;
        }

        if self.grid[next_row as usize][next_column as usize] != CellKind::Empty {
            return;
        }

        let bomb_blocking = self
            .bombs
            .iter()
            .any(|bomb| bomb.column == next_column && bomb.row == next_row);
        if bomb_blocking {
            return;
        }

        self.player_column = next_column;
        self.player_row = next_row;

        if let Some(position) = world.get_position_mut(self.player_entity) {
            position.column = self.offset_column as f64 + self.player_column as f64;
            position.row = self.offset_row as f64 + self.player_row as f64;
        }

        self.check_powerup_pickup(world);
    }

    fn move_enemies(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        let directions: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

        for enemy_index in 0..self.enemies.len() {
            if !self.enemies[enemy_index].alive {
                continue;
            }

            let enemy_column = self.enemies[enemy_index].column;
            let enemy_row = self.enemies[enemy_index].row;

            let mut valid_moves: Vec<(i32, i32)> = Vec::new();
            for &(delta_column, delta_row) in &directions {
                let next_column = enemy_column + delta_column;
                let next_row = enemy_row + delta_row;

                if next_column < 1
                    || next_column >= GRID_WIDTH as i32 - 1
                    || next_row < 1
                    || next_row >= GRID_HEIGHT as i32 - 1
                {
                    continue;
                }

                if self.grid[next_row as usize][next_column as usize] != CellKind::Empty {
                    continue;
                }

                let bomb_blocking = self
                    .bombs
                    .iter()
                    .any(|bomb| bomb.column == next_column && bomb.row == next_row);
                if bomb_blocking {
                    continue;
                }

                let other_enemy_blocking =
                    self.enemies
                        .iter()
                        .enumerate()
                        .any(|(other_index, other_enemy)| {
                            other_index != enemy_index
                                && other_enemy.alive
                                && other_enemy.column == next_column
                                && other_enemy.row == next_row
                        });
                if other_enemy_blocking {
                    continue;
                }

                valid_moves.push((next_column, next_row));
            }

            let safe_moves: Vec<(i32, i32)> = valid_moves
                .iter()
                .filter(|&&(move_column, move_row)| {
                    !is_in_blast_zone(&self.bombs, move_column, move_row, &self.grid)
                })
                .copied()
                .collect();

            let chosen_moves = if !safe_moves.is_empty() {
                &safe_moves
            } else if !valid_moves.is_empty() {
                &valid_moves
            } else {
                continue;
            };

            let move_index = rng.random_range(0..chosen_moves.len());
            let (new_column, new_row) = chosen_moves[move_index];

            self.enemies[enemy_index].column = new_column;
            self.enemies[enemy_index].row = new_row;

            if let Some(position) = world.get_position_mut(self.enemies[enemy_index].entity) {
                position.column = self.offset_column as f64 + new_column as f64;
                position.row = self.offset_row as f64 + new_row as f64;
            }
        }
    }

    fn check_explosion_hits(&mut self, world: &mut World) {
        for explosion in &self.explosions {
            if self.player_alive
                && self.player_invincible_timer <= 0.0
                && explosion.column == self.player_column
                && explosion.row == self.player_row
            {
                self.player_alive = false;
                self.game_over = true;
            }

            for enemy in &mut self.enemies {
                if enemy.alive && explosion.column == enemy.column && explosion.row == enemy.row {
                    enemy.alive = false;
                    self.score += 200;
                    world.despawn_entities(&[enemy.entity]);

                    self.particles.emit(
                        world,
                        self.offset_column as f64 + enemy.column as f64,
                        self.offset_row as f64 + enemy.row as f64,
                        6,
                        &ParticleConfig {
                            characters: vec!['*', '+', 'x'],
                            colors: vec![TermColor::Red, TermColor::Yellow, TermColor::White],
                            lifetime: 0.5,
                            speed_min: 2.0,
                            speed_max: 6.0,
                            spread: std::f64::consts::PI * 2.0,
                            direction: 0.0,
                            z_index: 8,
                        },
                    );
                }
            }
        }

        for powerup_index in (0..self.powerups.len()).rev() {
            let powerup = &self.powerups[powerup_index];
            let destroyed = self.explosions.iter().any(|explosion| {
                explosion.column == powerup.column && explosion.row == powerup.row
            });
            if destroyed {
                world.despawn_entities(&[self.powerups[powerup_index].entity]);
                self.powerups.swap_remove(powerup_index);
            }
        }
    }

    fn check_enemy_player_collision(&mut self) {
        if !self.player_alive || self.player_invincible_timer > 0.0 {
            return;
        }

        for enemy in &self.enemies {
            if enemy.alive && enemy.column == self.player_column && enemy.row == self.player_row {
                self.player_alive = false;
                self.game_over = true;
                return;
            }
        }
    }

    fn update_bombs(&mut self, world: &mut World, delta: f64) {
        let mut detonation_indices: Vec<usize> = Vec::new();
        for (bomb_index, bomb) in self.bombs.iter_mut().enumerate() {
            bomb.fuse_timer -= delta;
            if bomb.fuse_timer <= 0.0 {
                detonation_indices.push(bomb_index);
            }
        }

        for bomb_index in 0..self.bombs.len() {
            if !detonation_indices.contains(&bomb_index) {
                let bomb = &self.bombs[bomb_index];
                let flash = ((bomb.fuse_timer * 6.0) as i32 % 2 == 0) && bomb.fuse_timer < 1.0;
                if let Some(sprite) = world.get_sprite_mut(self.bomb_entities[bomb_index]) {
                    sprite.foreground = if flash {
                        TermColor::Red
                    } else {
                        TermColor::Yellow
                    };
                }
            }
        }

        let mut all_detonation_indices = detonation_indices.clone();
        let mut process_index = 0;
        while process_index < all_detonation_indices.len() {
            let bomb_index = all_detonation_indices[process_index];
            let mut chain_detonations = Vec::new();
            self.detonate_bomb(bomb_index, world, &mut chain_detonations);
            for chained_index in chain_detonations {
                if !all_detonation_indices.contains(&chained_index) {
                    all_detonation_indices.push(chained_index);
                }
            }
            process_index += 1;
        }

        all_detonation_indices.sort_unstable();
        all_detonation_indices.dedup();
        for &bomb_index in all_detonation_indices.iter().rev() {
            if bomb_index < self.bombs.len() {
                self.bombs.remove(bomb_index);
                let entity = self.bomb_entities.remove(bomb_index);
                world.despawn_entities(&[entity]);
            }
        }
    }

    fn update_explosions(&mut self, world: &mut World, delta: f64) {
        let mut expired_indices: Vec<usize> = Vec::new();
        for (explosion_index, explosion) in self.explosions.iter_mut().enumerate() {
            explosion.timer -= delta;
            if explosion.timer <= 0.0 {
                expired_indices.push(explosion_index);
            } else {
                let brightness = (explosion.timer / EXPLOSION_DURATION * 255.0) as u8;
                if let Some(sprite) = world.get_sprite_mut(self.explosion_entities[explosion_index])
                {
                    sprite.foreground = TermColor::Rgb {
                        r: 255,
                        g: brightness / 2,
                        b: 0,
                    };
                    sprite.background = TermColor::Rgb {
                        r: brightness,
                        g: brightness / 4,
                        b: 0,
                    };
                }
            }
        }

        for &explosion_index in expired_indices.iter().rev() {
            self.explosions.remove(explosion_index);
            let entity = self.explosion_entities.remove(explosion_index);
            world.despawn_entities(&[entity]);
        }
    }

    fn update_hud(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);

        let hud_row = self.offset_row as f64 + GRID_HEIGHT as f64 + 1.0;
        let hud_column = self.offset_column as f64;

        let alive_enemies = self.enemies.iter().filter(|enemy| enemy.alive).count();
        let speed_level = ((0.18 - self.player_speed) / 0.03).round() as i32;

        let line_1 = format!("Score: {}  Enemies: {}", self.score, alive_enemies,);
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
                text: line_1,
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let line_2 = format!(
            "Bombs: {}/{}  Range: {}  Speed: +{}",
            self.bombs.len(),
            self.player_max_bombs,
            self.player_bomb_range,
            speed_level,
        );
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
                text: line_2,
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));
    }

    fn check_win_condition(&mut self) {
        let alive_enemies = self.enemies.iter().filter(|enemy| enemy.alive).count();
        if alive_enemies == 0 {
            self.won = true;
            self.game_over = true;
        }
    }

    fn clear_all(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);
        world.despawn_entities(&[self.tilemap_entity, self.player_entity]);
        world.despawn_entities(&self.bomb_entities);
        world.despawn_entities(&self.explosion_entities);

        let enemy_entities: Vec<Entity> = self
            .enemies
            .iter()
            .filter(|enemy| enemy.alive)
            .map(|enemy| enemy.entity)
            .collect();
        world.despawn_entities(&enemy_entities);

        let powerup_entities: Vec<Entity> =
            self.powerups.iter().map(|powerup| powerup.entity).collect();
        world.despawn_entities(&powerup_entities);

        self.particles.despawn_all(world);
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Bomberman - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        self.build_tilemap(world);
        self.spawn_player(world);
        self.spawn_enemies(world);
        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        if key == KeyCode::Escape {
            world.resources.should_exit = true;
            return;
        }
        if self.game_over {
            return;
        }
        match key {
            KeyCode::Up | KeyCode::Char('w') => self.queued_direction = (0, -1),
            KeyCode::Down | KeyCode::Char('s') => self.queued_direction = (0, 1),
            KeyCode::Left | KeyCode::Char('a') => self.queued_direction = (-1, 0),
            KeyCode::Right | KeyCode::Char('d') => self.queued_direction = (1, 0),
            KeyCode::Char(' ') => self.queued_direction = (0, 0),
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.game_over {
            return;
        }

        let delta = world.resources.timing.delta_seconds;
        self.particles.update(world, delta);

        if self.player_invincible_timer > 0.0 {
            self.player_invincible_timer -= delta;
            let blink = (self.player_invincible_timer * 8.0) as i32 % 2 == 0;
            if let Some(sprite) = world.get_sprite_mut(self.player_entity) {
                sprite.foreground = if blink {
                    TermColor::Cyan
                } else {
                    TermColor::DarkCyan
                };
            }
        }

        if world.resources.keyboard.is_pressed(KeyCode::Char(' ')) {
            self.place_bomb(world);
        }

        self.player_move_timer += delta;
        if self.player_move_timer >= self.player_speed {
            self.player_move_timer = 0.0;

            let mut direction = self.queued_direction;
            if direction == (0, 0) {
                if world.resources.keyboard.is_pressed(KeyCode::Up)
                    || world.resources.keyboard.is_pressed(KeyCode::Char('w'))
                {
                    direction = (0, -1);
                } else if world.resources.keyboard.is_pressed(KeyCode::Down)
                    || world.resources.keyboard.is_pressed(KeyCode::Char('s'))
                {
                    direction = (0, 1);
                } else if world.resources.keyboard.is_pressed(KeyCode::Left)
                    || world.resources.keyboard.is_pressed(KeyCode::Char('a'))
                {
                    direction = (-1, 0);
                } else if world.resources.keyboard.is_pressed(KeyCode::Right)
                    || world.resources.keyboard.is_pressed(KeyCode::Char('d'))
                {
                    direction = (1, 0);
                }
            }

            if direction != (0, 0) {
                self.queued_direction = direction;
                self.move_player(world);
            }
        }

        self.enemy_move_timer += delta;
        if self.enemy_move_timer >= ENEMY_MOVE_INTERVAL {
            self.enemy_move_timer = 0.0;
            self.move_enemies(world);
        }

        self.update_bombs(world, delta);
        self.update_explosions(world, delta);
        self.check_explosion_hits(world);
        self.check_enemy_player_collision();
        self.check_win_condition();
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
        "Bomberman - Game Over"
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

        let score_text = format!("Score: {}", self.score);
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - score_text.len() as f64 / 2.0,
                row: center_row - 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: score_text,
                foreground: TermColor::Yellow,
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
                row: center_row + 1.0,
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
                row: center_row + 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: quit_prompt.to_string(),
                foreground: TermColor::Grey,
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
