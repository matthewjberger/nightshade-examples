use nightshade::tui::prelude::*;

const GRID_WIDTH: usize = 40;
const GRID_HEIGHT: usize = 20;

const TILE_EMPTY: u8 = 0;
const TILE_WALL: u8 = 1;

#[derive(PartialEq)]
enum PlacementMode {
    Start,
    Goal,
    Wall,
}

struct GameState {
    map: [[u8; GRID_WIDTH]; GRID_HEIGHT],
    tilemap_entity: Entity,
    offset_column: i32,
    offset_row: i32,
    start: Option<(i32, i32)>,
    goal: Option<(i32, i32)>,
    path: Vec<(i32, i32)>,
    placement_mode: PlacementMode,
    allow_diagonal: bool,
    hud_entities: EntityGroup,
    cursor_entity: Entity,
}

impl GameState {
    fn new() -> Self {
        Self {
            map: [[TILE_EMPTY; GRID_WIDTH]; GRID_HEIGHT],
            tilemap_entity: Entity::default(),
            offset_column: 0,
            offset_row: 0,
            start: None,
            goal: None,
            path: Vec::new(),
            placement_mode: PlacementMode::Start,
            allow_diagonal: false,
            hud_entities: EntityGroup::new(),
            cursor_entity: Entity::default(),
        }
    }

    fn recompute_path(&mut self) {
        self.path.clear();

        if let (Some(start), Some(goal)) = (self.start, self.goal) {
            let map = &self.map;
            let result = astar(
                start,
                goal,
                |column, row| {
                    if column < 0
                        || column >= GRID_WIDTH as i32
                        || row < 0
                        || row >= GRID_HEIGHT as i32
                    {
                        return false;
                    }
                    map[row as usize][column as usize] != TILE_WALL
                },
                self.allow_diagonal,
            );

            if let Some(found_path) = result {
                self.path = found_path;
            }
        }
    }

    fn refresh_tilemap(&self, world: &mut World) {
        let mut tilemap = Tilemap::new(GRID_WIDTH, GRID_HEIGHT);

        let path_set: std::collections::HashSet<(i32, i32)> = self.path.iter().copied().collect();

        for row in 0..GRID_HEIGHT {
            for column in 0..GRID_WIDTH {
                let position = (column as i32, row as i32);
                let is_start = self.start == Some(position);
                let is_goal = self.goal == Some(position);
                let is_path = path_set.contains(&position) && !is_start && !is_goal;
                let tile = self.map[row][column];

                let cell = if is_start {
                    TilemapCell {
                        character: 'S',
                        foreground: TermColor::Black,
                        background: TermColor::Green,
                    }
                } else if is_goal {
                    TilemapCell {
                        character: 'G',
                        foreground: TermColor::Black,
                        background: TermColor::Red,
                    }
                } else if is_path {
                    TilemapCell {
                        character: '·',
                        foreground: TermColor::Black,
                        background: TermColor::Yellow,
                    }
                } else {
                    match tile {
                        TILE_WALL => TilemapCell {
                            character: '█',
                            foreground: TermColor::Rgb {
                                r: 80,
                                g: 80,
                                b: 80,
                            },
                            background: TermColor::Rgb {
                                r: 40,
                                g: 40,
                                b: 40,
                            },
                        },
                        _ => TilemapCell {
                            character: '.',
                            foreground: TermColor::Rgb {
                                r: 50,
                                g: 50,
                                b: 50,
                            },
                            background: TermColor::Rgb {
                                r: 20,
                                g: 20,
                                b: 20,
                            },
                        },
                    }
                };

                tilemap.set(column, row, cell);
            }
        }

        world.set_tilemap(self.tilemap_entity, tilemap);
    }

    fn update_hud(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);

        let mode_text = match self.placement_mode {
            PlacementMode::Start => "Place START",
            PlacementMode::Goal => "Place GOAL",
            PlacementMode::Wall => "Place WALLS",
        };

        let diagonal_text = if self.allow_diagonal {
            "Diagonal: ON"
        } else {
            "Diagonal: OFF"
        };

        let path_text = if self.start.is_some() && self.goal.is_some() {
            if self.path.is_empty() {
                "Path: No path found!".to_string()
            } else {
                format!("Path: {} steps", self.path.len() - 1)
            }
        } else {
            "Path: Set start and goal".to_string()
        };

        let hud_line = format!(
            "Pathfinding Demo | {} | {} | {} | Tab: toggle diagonal | C: clear | Click: place | ESC: quit",
            mode_text, diagonal_text, path_text,
        );

        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 0.0,
                row: 0.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: hud_line,
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));
    }

    fn place_at(&mut self, grid_column: usize, grid_row: usize) {
        if grid_column >= GRID_WIDTH || grid_row >= GRID_HEIGHT {
            return;
        }

        match self.placement_mode {
            PlacementMode::Start => {
                if self.map[grid_row][grid_column] == TILE_WALL {
                    return;
                }
                self.start = Some((grid_column as i32, grid_row as i32));
                self.placement_mode = PlacementMode::Goal;
                self.recompute_path();
            }
            PlacementMode::Goal => {
                if self.map[grid_row][grid_column] == TILE_WALL {
                    return;
                }
                if self.start == Some((grid_column as i32, grid_row as i32)) {
                    return;
                }
                self.goal = Some((grid_column as i32, grid_row as i32));
                self.placement_mode = PlacementMode::Wall;
                self.recompute_path();
            }
            PlacementMode::Wall => {
                let position = (grid_column as i32, grid_row as i32);
                if self.start == Some(position) || self.goal == Some(position) {
                    return;
                }
                if self.map[grid_row][grid_column] == TILE_WALL {
                    self.map[grid_row][grid_column] = TILE_EMPTY;
                } else {
                    self.map[grid_row][grid_column] = TILE_WALL;
                }
                self.recompute_path();
            }
        }
    }

    fn clear_all(&mut self) {
        self.map = [[TILE_EMPTY; GRID_WIDTH]; GRID_HEIGHT];
        self.start = None;
        self.goal = None;
        self.path.clear();
        self.placement_mode = PlacementMode::Start;
    }
}

impl State for GameState {
    fn title(&self) -> &str {
        "Pathfinding Demo - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        self.offset_column = ((terminal.columns as i32 - GRID_WIDTH as i32) / 2).max(0);
        self.offset_row = 2;

        self.tilemap_entity = EntityBuilder::new()
            .position(Position {
                column: self.offset_column as f64,
                row: self.offset_row as f64,
            })
            .tilemap(Tilemap::new(GRID_WIDTH, GRID_HEIGHT))
            .z_index(ZIndex(0))
            .spawn(world);

        self.cursor_entity = EntityBuilder::new()
            .position(Position {
                column: 0.0,
                row: 0.0,
            })
            .sprite(Sprite {
                character: '+',
                foreground: TermColor::White,
                background: TermColor::Black,
            })
            .z_index(ZIndex(10))
            .visibility(Visibility { visible: false })
            .spawn(world);

        self.refresh_tilemap(world);
        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Escape => world.resources.should_exit = true,
            KeyCode::Tab => {
                self.allow_diagonal = !self.allow_diagonal;
                self.recompute_path();
                self.refresh_tilemap(world);
                self.update_hud(world);
            }
            KeyCode::Char('c') => {
                self.clear_all();
                self.refresh_tilemap(world);
                self.update_hud(world);
            }
            _ => {}
        }
    }

    fn on_mouse_input(
        &mut self,
        world: &mut World,
        button: MouseButton,
        column: u16,
        row: u16,
        pressed: bool,
    ) {
        if !pressed || button != MouseButton::Left {
            return;
        }

        let grid_column = column as i32 - self.offset_column;
        let grid_row = row as i32 - self.offset_row;

        if grid_column >= 0
            && grid_column < GRID_WIDTH as i32
            && grid_row >= 0
            && grid_row < GRID_HEIGHT as i32
        {
            self.place_at(grid_column as usize, grid_row as usize);
            self.refresh_tilemap(world);
            self.update_hud(world);
        }
    }

    fn on_mouse_move(&mut self, world: &mut World, column: u16, row: u16) {
        let grid_column = column as i32 - self.offset_column;
        let grid_row = row as i32 - self.offset_row;

        let in_bounds = grid_column >= 0
            && grid_column < GRID_WIDTH as i32
            && grid_row >= 0
            && grid_row < GRID_HEIGHT as i32;

        if let Some(visibility) = world.get_visibility_mut(self.cursor_entity) {
            visibility.visible = in_bounds;
        }

        if in_bounds {
            if let Some(position) = world.get_position_mut(self.cursor_entity) {
                position.column = column as f64;
                position.row = row as f64;
            }

            let mode_color = match self.placement_mode {
                PlacementMode::Start => TermColor::Green,
                PlacementMode::Goal => TermColor::Red,
                PlacementMode::Wall => TermColor::Grey,
            };

            if let Some(sprite) = world.get_sprite_mut(self.cursor_entity) {
                sprite.foreground = mode_color;
            }
        }
    }

    fn run_systems(&mut self, _world: &mut World) {}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(GameState::new()))
}
