use nightshade::tui::prelude::*;

const GOAL_FOREGROUND: TermColor = TermColor::Rgb {
    r: 100,
    g: 100,
    b: 255,
};
const GOAL_BACKGROUND: TermColor = TermColor::Rgb { r: 0, g: 0, b: 80 };

const LEVELS: &[&[&str]] = &[
    &[
        "  ##########  ",
        "  #        #  ",
        "  #  ####  #  ",
        "###  #  #  #  ",
        "#    #  # ##  ",
        "#  $  $.  #   ",
        "###  #  # #   ",
        "  #  ####.#   ",
        "  ##      #   ",
        "   #  @#  #   ",
        "   ########   ",
    ],
    &[
        " ############",
        " #          #",
        " # ##..#### #",
        " #  $  $    #",
        "##  $ ##  ###",
        "#  $   #  #  ",
        "#   ## #  #  ",
        "## @#    ##  ",
        " ########    ",
    ],
    &[
        "    ########   ",
        "    #      #   ",
        "    # $$$# #   ",
        " #### #   ##   ",
        " #   $  #  #   ",
        " # ###  #  #   ",
        " # ..# $   #   ",
        " # ..#   ###   ",
        " # ..#####     ",
        " #  @      #   ",
        " ###########   ",
    ],
    &[
        "##############",
        "#            #",
        "#  ## ###### #",
        "#  $   $   # #",
        "# $## # $  # #",
        "#  .. .  ### #",
        "# ###.## #   #",
        "#      @ #   #",
        "##############",
    ],
    &[
        "   ############",
        "   #          #",
        "   # ##  $  # #",
        "   # # $  $ # #",
        "####   ## ### #",
        "#  ##$#       #",
        "# ...   ####  #",
        "# ...#  #  @  #",
        "#  ###  #  ####",
        "##      ####   ",
        " ########      ",
    ],
    &[
        "  ############## ",
        "  #   #    #   # ",
        "  #   $$   $   # ",
        "### #$  ## # ###  ",
        "#  .#  $   # #   ",
        "#  .#  # $ # #   ",
        "#  .#  #   # #   ",
        "#  .# ##$### #   ",
        "#  .   $   @ #   ",
        "#  ######   ##   ",
        "####    #####    ",
    ],
    &[
        "################",
        "#              #",
        "# # ########## #",
        "# #    $   $.# #",
        "# # $$ #.  $.# #",
        "# #  $ # $ #.# #",
        "# # $  # $ #.# #",
        "# ##.### $ #.# #",
        "#  @       #   #",
        "################",
    ],
    &[
        "  ################",
        "  #              #",
        "  # ## ###  #### #",
        "  # #  $ #  #..# #",
        "### # $  #  #..# #",
        "#   #  $ ## #..# #",
        "#   ## $  # #..# #",
        "# $ #  $  #      #",
        "#   #     ########",
        "### # ######      ",
        "  # @      #      ",
        "  ##########      ",
    ],
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
    Floor,
    Wall,
    Goal,
}

#[derive(Clone)]
struct LevelData {
    tiles: Vec<Vec<Tile>>,
    width: usize,
    height: usize,
}

#[derive(Clone)]
struct Snapshot {
    player_column: usize,
    player_row: usize,
    box_positions: Vec<(usize, usize)>,
}

fn parse_level(level_strings: &[&str]) -> (LevelData, usize, usize, Vec<(usize, usize)>) {
    let height = level_strings.len();
    let width = level_strings
        .iter()
        .map(|line| line.len())
        .max()
        .unwrap_or(0);

    let mut tiles = vec![vec![Tile::Floor; width]; height];
    let mut player_column = 0;
    let mut player_row = 0;
    let mut box_positions = Vec::new();

    for (row, line) in level_strings.iter().enumerate() {
        for (column, character) in line.chars().enumerate() {
            match character {
                '#' => {
                    tiles[row][column] = Tile::Wall;
                }
                '.' => {
                    tiles[row][column] = Tile::Goal;
                }
                '@' => {
                    tiles[row][column] = Tile::Floor;
                    player_column = column;
                    player_row = row;
                }
                '+' => {
                    tiles[row][column] = Tile::Goal;
                    player_column = column;
                    player_row = row;
                }
                '$' => {
                    tiles[row][column] = Tile::Floor;
                    box_positions.push((column, row));
                }
                '*' => {
                    tiles[row][column] = Tile::Goal;
                    box_positions.push((column, row));
                }
                _ => {
                    tiles[row][column] = Tile::Floor;
                }
            }
        }
    }

    let level_data = LevelData {
        tiles,
        width,
        height,
    };

    (level_data, player_column, player_row, box_positions)
}

fn render_text_centered(
    world: &mut World,
    text: &str,
    center_column: i32,
    row: i32,
    foreground: TermColor,
    background: TermColor,
    z_index: i32,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let start_column = center_column - text.len() as i32 / 2;
    for (char_index, character) in text.chars().enumerate() {
        let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
        world.set_position(
            entity,
            Position {
                column: (start_column + char_index as i32) as f64,
                row: row as f64,
            },
        );
        world.set_sprite(
            entity,
            Sprite {
                character,
                foreground,
                background,
            },
        );
        world.set_z_index(entity, ZIndex(z_index));
        entities.push(entity);
    }
    entities
}

fn despawn_entity_list(world: &mut World, entities: &mut Vec<Entity>) {
    if !entities.is_empty() {
        world.despawn_entities(entities);
        entities.clear();
    }
}

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Sokoban - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r" ____        _         _                 ",
            r"/ ___|  ___ | | _____ | |__   __ _ _ __  ",
            r"\___ \ / _ \| |/ / _ \| '_ \ / _` | '_ \ ",
            r" ___) | (_) |   < (_) | |_) | (_| | | | |",
            r"|____/ \___/|_|\_\___/|_.__/ \__,_|_| |_|",
        ];

        let title_start_row = center_row - 7;

        for (line_index, line) in title_lines.iter().enumerate() {
            let start_column = center_column - line.len() as i32 / 2;
            for (char_index, character) in line.chars().enumerate() {
                if character != ' ' {
                    let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                    world.set_position(
                        entity,
                        Position {
                            column: (start_column + char_index as i32) as f64,
                            row: (title_start_row + line_index as i32) as f64,
                        },
                    );
                    world.set_sprite(
                        entity,
                        Sprite {
                            character,
                            foreground: TermColor::Rgb {
                                r: 220,
                                g: 180,
                                b: 80,
                            },
                            background: TermColor::Black,
                        },
                    );
                    world.set_z_index(entity, ZIndex(10));
                }
            }
        }

        let decorative = "# @ $ . * #";
        let decorative_start = center_column - decorative.len() as i32 / 2;
        for (char_index, character) in decorative.chars().enumerate() {
            if character == ' ' {
                continue;
            }
            let foreground = match character {
                '@' => TermColor::Green,
                '$' => TermColor::DarkYellow,
                '.' => GOAL_FOREGROUND,
                '*' => TermColor::Yellow,
                '#' => TermColor::DarkGrey,
                _ => TermColor::White,
            };
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (decorative_start + char_index as i32) as f64,
                    row: (title_start_row + 7) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }

        render_text_centered(
            world,
            "Press ENTER to start",
            center_column,
            title_start_row + 10,
            TermColor::White,
            TermColor::Black,
            10,
        );

        render_text_centered(
            world,
            "Press ESC to quit",
            center_column,
            title_start_row + 12,
            TermColor::Grey,
            TermColor::Black,
            10,
        );
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
            let all_entities: Vec<Entity> = world.query_entities(POSITION | SPRITE).collect();
            world.despawn_entities(&all_entities);
            return Some(Box::new(GameplayState::new(0)));
        }
        None
    }
}

struct GameplayState {
    current_level_index: usize,
    level_data: LevelData,
    player_column: usize,
    player_row: usize,
    box_positions: Vec<(usize, usize)>,
    move_count: u32,
    total_moves: u32,
    undo_stack: Vec<Snapshot>,
    board_offset_column: i32,
    board_offset_row: i32,
    tile_entities: Vec<Entity>,
    dynamic_entities: Vec<Entity>,
    hud_entities: Vec<Entity>,
    level_complete: bool,
    level_complete_timer: f64,
    all_levels_done: bool,
}

impl GameplayState {
    fn new(level_index: usize) -> Self {
        let (level_data, player_column, player_row, box_positions) =
            parse_level(LEVELS[level_index]);

        Self {
            current_level_index: level_index,
            level_data,
            player_column,
            player_row,
            box_positions,
            move_count: 0,
            total_moves: 0,
            undo_stack: Vec::new(),
            board_offset_column: 0,
            board_offset_row: 0,
            tile_entities: Vec::new(),
            dynamic_entities: Vec::new(),
            hud_entities: Vec::new(),
            level_complete: false,
            level_complete_timer: 0.0,
            all_levels_done: false,
        }
    }

    fn with_total_moves(level_index: usize, total_moves: u32) -> Self {
        let mut state = Self::new(level_index);
        state.total_moves = total_moves;
        state
    }

    fn is_wall(&self, column: usize, row: usize) -> bool {
        if row >= self.level_data.height || column >= self.level_data.width {
            return true;
        }
        self.level_data.tiles[row][column] == Tile::Wall
    }

    fn is_goal(&self, column: usize, row: usize) -> bool {
        if row >= self.level_data.height || column >= self.level_data.width {
            return false;
        }
        self.level_data.tiles[row][column] == Tile::Goal
    }

    fn box_at(&self, column: usize, row: usize) -> Option<usize> {
        self.box_positions
            .iter()
            .position(|&(box_column, box_row)| box_column == column && box_row == row)
    }

    fn check_level_complete(&self) -> bool {
        let goal_count = self
            .level_data
            .tiles
            .iter()
            .flatten()
            .filter(|tile| **tile == Tile::Goal)
            .count();
        if goal_count == 0 {
            return false;
        }
        self.box_positions
            .iter()
            .all(|&(box_column, box_row)| self.is_goal(box_column, box_row))
    }

    fn try_move(&mut self, delta_column: i32, delta_row: i32) {
        if self.level_complete {
            return;
        }

        let target_column = self.player_column as i32 + delta_column;
        let target_row = self.player_row as i32 + delta_row;

        if target_column < 0 || target_row < 0 {
            return;
        }

        let target_column = target_column as usize;
        let target_row = target_row as usize;

        if self.is_wall(target_column, target_row) {
            return;
        }

        if let Some(box_index) = self.box_at(target_column, target_row) {
            let push_column = target_column as i32 + delta_column;
            let push_row = target_row as i32 + delta_row;

            if push_column < 0 || push_row < 0 {
                return;
            }

            let push_column = push_column as usize;
            let push_row = push_row as usize;

            if self.is_wall(push_column, push_row) {
                return;
            }

            if self.box_at(push_column, push_row).is_some() {
                return;
            }

            self.undo_stack.push(Snapshot {
                player_column: self.player_column,
                player_row: self.player_row,
                box_positions: self.box_positions.clone(),
            });

            self.box_positions[box_index] = (push_column, push_row);
            self.player_column = target_column;
            self.player_row = target_row;
            self.move_count += 1;
        } else {
            self.undo_stack.push(Snapshot {
                player_column: self.player_column,
                player_row: self.player_row,
                box_positions: self.box_positions.clone(),
            });

            self.player_column = target_column;
            self.player_row = target_row;
            self.move_count += 1;
        }

        if self.check_level_complete() {
            self.level_complete = true;
            self.level_complete_timer = 0.0;
        }
    }

    fn undo(&mut self) {
        if self.level_complete {
            return;
        }

        if let Some(snapshot) = self.undo_stack.pop() {
            self.player_column = snapshot.player_column;
            self.player_row = snapshot.player_row;
            self.box_positions = snapshot.box_positions;
            self.move_count = self.move_count.saturating_sub(1);
        }
    }

    fn restart_level(&mut self) {
        let (level_data, player_column, player_row, box_positions) =
            parse_level(LEVELS[self.current_level_index]);
        self.level_data = level_data;
        self.player_column = player_column;
        self.player_row = player_row;
        self.box_positions = box_positions;
        self.move_count = 0;
        self.undo_stack.clear();
        self.level_complete = false;
        self.level_complete_timer = 0.0;
    }

    fn spawn_tile_entities(&mut self, world: &mut World) {
        for row in 0..self.level_data.height {
            for column in 0..self.level_data.width {
                let tile = self.level_data.tiles[row][column];
                let (character, foreground, background) = match tile {
                    Tile::Wall => ('#', TermColor::DarkGrey, TermColor::Black),
                    Tile::Goal => ('.', GOAL_FOREGROUND, TermColor::Black),
                    Tile::Floor => continue,
                };
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (self.board_offset_column + column as i32) as f64,
                        row: (self.board_offset_row + row as i32) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground,
                        background,
                    },
                );
                world.set_z_index(entity, ZIndex(1));
                self.tile_entities.push(entity);
            }
        }
    }

    fn render_dynamic_entities(&mut self, world: &mut World) {
        despawn_entity_list(world, &mut self.dynamic_entities);

        for &(box_column, box_row) in &self.box_positions {
            let on_goal = self.is_goal(box_column, box_row);
            let (character, foreground, background) = if on_goal {
                ('*', TermColor::Yellow, GOAL_BACKGROUND)
            } else {
                ('$', TermColor::DarkYellow, TermColor::Black)
            };
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (self.board_offset_column + box_column as i32) as f64,
                    row: (self.board_offset_row + box_row as i32) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(2));
            self.dynamic_entities.push(entity);
        }

        let player_on_goal = self.is_goal(self.player_column, self.player_row);
        let player_background = if player_on_goal {
            GOAL_BACKGROUND
        } else {
            TermColor::Black
        };
        let player_entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
        world.set_position(
            player_entity,
            Position {
                column: (self.board_offset_column + self.player_column as i32) as f64,
                row: (self.board_offset_row + self.player_row as i32) as f64,
            },
        );
        world.set_sprite(
            player_entity,
            Sprite {
                character: '@',
                foreground: TermColor::Green,
                background: player_background,
            },
        );
        world.set_z_index(player_entity, ZIndex(3));
        self.dynamic_entities.push(player_entity);
    }

    fn render_hud(&mut self, world: &mut World) {
        despawn_entity_list(world, &mut self.hud_entities);

        let hud_row = self.board_offset_row - 2;

        let level_text = format!(
            "Level {}/{}  Moves: {}",
            self.current_level_index + 1,
            LEVELS.len(),
            self.move_count,
        );
        let level_start = self.board_offset_column;
        for (char_index, character) in level_text.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (level_start + char_index as i32) as f64,
                    row: hud_row as f64,
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
            self.hud_entities.push(entity);
        }

        let controls_row = self.board_offset_row + self.level_data.height as i32 + 1;
        let controls_text = "Arrows/WASD: Move  Z: Undo  R: Restart  ESC: Quit";
        let controls_start = self.board_offset_column;
        for (char_index, character) in controls_text.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (controls_start + char_index as i32) as f64,
                    row: controls_row as f64,
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
            self.hud_entities.push(entity);
        }

        if self.level_complete {
            let terminal = world.resources.terminal_size;
            let center_column = terminal.columns as i32 / 2;
            let message_row = self.board_offset_row + self.level_data.height as i32 / 2;
            let message = "Level Complete!";
            let start = center_column - message.len() as i32 / 2;
            for (char_index, character) in message.chars().enumerate() {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (start + char_index as i32) as f64,
                        row: message_row as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: TermColor::Rgb {
                            r: 100,
                            g: 255,
                            b: 100,
                        },
                        background: TermColor::Rgb { r: 0, g: 40, b: 0 },
                    },
                );
                world.set_z_index(entity, ZIndex(20));
                self.hud_entities.push(entity);
            }
        }
    }

    fn clear_all_entities(&mut self, world: &mut World) {
        despawn_entity_list(world, &mut self.tile_entities);
        despawn_entity_list(world, &mut self.dynamic_entities);
        despawn_entity_list(world, &mut self.hud_entities);
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Sokoban - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        self.board_offset_column = (terminal.columns as i32 - self.level_data.width as i32) / 2;
        self.board_offset_row = (terminal.rows as i32 - self.level_data.height as i32) / 2;

        if self.board_offset_column < 0 {
            self.board_offset_column = 0;
        }
        if self.board_offset_row < 2 {
            self.board_offset_row = 2;
        }

        self.spawn_tile_entities(world);
        self.render_dynamic_entities(world);
        self.render_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }

        if self.level_complete {
            return;
        }

        match key {
            KeyCode::Up | KeyCode::Char('w') => self.try_move(0, -1),
            KeyCode::Down | KeyCode::Char('s') => self.try_move(0, 1),
            KeyCode::Left | KeyCode::Char('a') => self.try_move(-1, 0),
            KeyCode::Right | KeyCode::Char('d') => self.try_move(1, 0),
            KeyCode::Char('z') => self.undo(),
            KeyCode::Char('r') => {
                self.restart_level();
                self.clear_all_entities(world);
                self.spawn_tile_entities(world);
            }
            KeyCode::Escape | KeyCode::Char('q') => {
                world.resources.should_exit = true;
            }
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.level_complete {
            self.level_complete_timer += world.resources.timing.delta_seconds;
            if self.level_complete_timer >= 1.5 {
                let next_level = self.current_level_index + 1;
                self.all_levels_done = next_level >= LEVELS.len();
            }
        }

        self.render_dynamic_entities(world);
        self.render_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.level_complete && self.level_complete_timer >= 1.5 {
            let accumulated_moves = self.total_moves + self.move_count;
            self.clear_all_entities(world);

            if self.all_levels_done {
                return Some(Box::new(GameCompleteState {
                    total_moves: accumulated_moves,
                    restart: false,
                }));
            }

            return Some(Box::new(GameplayState::with_total_moves(
                self.current_level_index + 1,
                accumulated_moves,
            )));
        }
        None
    }
}

struct GameCompleteState {
    total_moves: u32,
    restart: bool,
}

impl State for GameCompleteState {
    fn title(&self) -> &str {
        "Sokoban - Complete!"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let lines: Vec<(&str, TermColor)> = vec![
            (
                "Congratulations!",
                TermColor::Rgb {
                    r: 100,
                    g: 255,
                    b: 100,
                },
            ),
            ("", TermColor::Black),
            (
                "All levels complete!",
                TermColor::Rgb {
                    r: 255,
                    g: 255,
                    b: 100,
                },
            ),
        ];

        let total_line = format!("Total moves: {}", self.total_moves);

        let all_lines: Vec<(String, TermColor)> = lines
            .iter()
            .map(|(text, color)| (text.to_string(), *color))
            .chain(std::iter::once((
                total_line,
                TermColor::Rgb {
                    r: 100,
                    g: 200,
                    b: 255,
                },
            )))
            .chain(std::iter::once((String::new(), TermColor::Black)))
            .chain(std::iter::once((
                "Press R to play again".to_string(),
                TermColor::White,
            )))
            .chain(std::iter::once((
                "Press ESC to quit".to_string(),
                TermColor::Grey,
            )))
            .collect();

        let start_row = center_row - all_lines.len() as i32 / 2;

        for (line_index, (text, color)) in all_lines.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            render_text_centered(
                world,
                text,
                center_column,
                start_row + line_index as i32,
                *color,
                TermColor::Black,
                10,
            );
        }
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
            let all_entities: Vec<Entity> = world.query_entities(POSITION | SPRITE).collect();
            world.despawn_entities(&all_entities);
            return Some(Box::new(GameplayState::new(0)));
        }
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(TitleScreenState { start_game: false }))
}
