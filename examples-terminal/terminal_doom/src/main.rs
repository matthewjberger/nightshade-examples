use nightshade::tui::prelude::*;

const MAP_WIDTH: usize = 16;
const MAP_HEIGHT: usize = 16;
const FIELD_OF_VIEW: f64 = std::f64::consts::FRAC_PI_3;
const MOVE_SPEED: f64 = 3.0;
const TURN_SPEED: f64 = 2.5;
const WALL_HEIGHT_SCALE: f64 = 12.0;
const MAX_RAY_DEPTH: f64 = 20.0;
const MINIMAP_SCALE: usize = 1;
const ENEMY_RADIUS: f64 = 0.3;
const STARTING_AMMO: i32 = 20;
const STARTING_HEALTH: i32 = 100;
const ENEMY_DAMAGE: i32 = 8;
const ENEMY_AGGRO_RANGE: f64 = 6.0;
const ENEMY_ATTACK_RANGE: f64 = 1.5;
const ENEMY_MOVE_SPEED: f64 = 1.5;
const ENEMY_ATTACK_COOLDOWN: f64 = 1.5;

const MAP_DATA: [[u8; MAP_WIDTH]; MAP_HEIGHT] = [
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 1, 1, 1, 0, 1, 1, 0, 1, 1, 1, 0, 1, 0, 1],
    [1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1],
    [1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 1, 0, 0, 0, 1],
    [1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1],
    [1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 0, 1, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 1, 0, 1, 0, 1],
    [1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1],
    [1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 1],
    [1, 0, 1, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0, 1, 0, 1],
    [1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
];

#[derive(Clone, Copy, PartialEq)]
enum CellContent {
    Empty,
    Wall,
    Key,
    Exit,
}

fn build_map() -> [[CellContent; MAP_WIDTH]; MAP_HEIGHT] {
    let mut map = [[CellContent::Empty; MAP_WIDTH]; MAP_HEIGHT];
    for row in 0..MAP_HEIGHT {
        for column in 0..MAP_WIDTH {
            if MAP_DATA[row][column] == 1 {
                map[row][column] = CellContent::Wall;
            }
        }
    }
    map[13][5] = CellContent::Key;
    map[14][15] = CellContent::Exit;
    map
}

struct EnemyData {
    position_x: f64,
    position_y: f64,
    alive: bool,
    health: i32,
    attack_timer: f64,
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "DOOM - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let art_lines = [
            r"  ____   ___   ___  __  __ ",
            r" |  _ \ / _ \ / _ \|  \/  |",
            r" | | | | | | | | | | |\/| |",
            r" | |_| | |_| | |_| | |  | |",
            r" |____/ \___/ \___/|_|  |_|",
        ];

        for (line_index, line) in art_lines.iter().enumerate() {
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - line.len() as f64 / 2.0,
                    row: center_row - 5.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: line.to_string(),
                    foreground: TermColor::Red,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }

        let subtitle = "ASCII Raycasting FPS";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - subtitle.len() as f64 / 2.0,
                row: center_row + 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: subtitle.to_string(),
                foreground: TermColor::DarkYellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let controls_lines = [
            "WASD: Move  |  Left/Right: Turn  |  Z: Shoot",
            "Find the Key (K), then reach the Exit (X) to win!",
            "",
            "Press ENTER to start",
            "Press ESC to quit",
        ];

        for (line_index, line) in controls_lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            let color = if line_index >= 3 {
                TermColor::White
            } else {
                TermColor::Grey
            };
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - line.len() as f64 / 2.0,
                    row: center_row + 3.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: line.to_string(),
                    foreground: color,
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
    map: [[CellContent; MAP_WIDTH]; MAP_HEIGHT],
    player_x: f64,
    player_y: f64,
    player_angle: f64,
    health: i32,
    ammo: i32,
    kills: i32,
    has_key: bool,
    viewport_entity: Entity,
    minimap_entity: Entity,
    hud_entities: EntityGroup,
    enemies: Vec<EnemyData>,
    shoot_cooldown: f64,
    flash_timer: f64,
    transition: Option<Box<dyn State>>,
}

impl GameplayState {
    fn new() -> Self {
        let map = build_map();
        Self {
            map,
            player_x: 1.5,
            player_y: 1.5,
            player_angle: 0.0,
            health: STARTING_HEALTH,
            ammo: STARTING_AMMO,
            kills: 0,
            has_key: false,
            viewport_entity: Entity::default(),
            minimap_entity: Entity::default(),
            hud_entities: EntityGroup::new(),
            enemies: vec![
                EnemyData {
                    position_x: 11.5,
                    position_y: 7.5,
                    alive: true,
                    health: 30,
                    attack_timer: 0.0,
                },
                EnemyData {
                    position_x: 11.5,
                    position_y: 9.5,
                    alive: true,
                    health: 30,
                    attack_timer: 0.0,
                },
                EnemyData {
                    position_x: 12.5,
                    position_y: 14.5,
                    alive: true,
                    health: 30,
                    attack_timer: 0.0,
                },
            ],
            shoot_cooldown: 0.0,
            flash_timer: 0.0,
            transition: None,
        }
    }

    fn is_wall(&self, grid_x: i32, grid_y: i32) -> bool {
        if grid_x < 0 || grid_x >= MAP_WIDTH as i32 || grid_y < 0 || grid_y >= MAP_HEIGHT as i32 {
            return true;
        }
        self.map[grid_y as usize][grid_x as usize] == CellContent::Wall
    }

    fn cast_ray(&self, angle: f64) -> (f64, bool) {
        let ray_direction_x = angle.cos();
        let ray_direction_y = angle.sin();

        let mut map_x = self.player_x.floor() as i32;
        let mut map_y = self.player_y.floor() as i32;

        let delta_distance_x = if ray_direction_x.abs() < 1e-10 {
            1e10
        } else {
            (1.0 / ray_direction_x).abs()
        };
        let delta_distance_y = if ray_direction_y.abs() < 1e-10 {
            1e10
        } else {
            (1.0 / ray_direction_y).abs()
        };

        let step_x: i32;
        let step_y: i32;
        let mut side_distance_x: f64;
        let mut side_distance_y: f64;

        if ray_direction_x < 0.0 {
            step_x = -1;
            side_distance_x = (self.player_x - map_x as f64) * delta_distance_x;
        } else {
            step_x = 1;
            side_distance_x = (map_x as f64 + 1.0 - self.player_x) * delta_distance_x;
        }

        if ray_direction_y < 0.0 {
            step_y = -1;
            side_distance_y = (self.player_y - map_y as f64) * delta_distance_y;
        } else {
            step_y = 1;
            side_distance_y = (map_y as f64 + 1.0 - self.player_y) * delta_distance_y;
        }

        for step_count in 0..256 {
            let is_side_step = side_distance_x >= side_distance_y;
            if !is_side_step {
                side_distance_x += delta_distance_x;
                map_x += step_x;
            } else {
                side_distance_y += delta_distance_y;
                map_y += step_y;
            }

            if self.is_wall(map_x, map_y) {
                let perpendicular_distance = if is_side_step {
                    (map_y as f64 - self.player_y + (1.0 - step_y as f64) / 2.0) / ray_direction_y
                } else {
                    (map_x as f64 - self.player_x + (1.0 - step_x as f64) / 2.0) / ray_direction_x
                };
                return (perpendicular_distance.max(0.01), is_side_step);
            }
            let _ = step_count;
        }

        (MAX_RAY_DEPTH, false)
    }

    fn wall_character(distance: f64) -> char {
        if distance < 2.0 {
            '#'
        } else if distance < 5.0 {
            '%'
        } else if distance < 10.0 {
            '+'
        } else {
            '.'
        }
    }

    fn wall_color(distance: f64, is_side_hit: bool) -> TermColor {
        let brightness = ((1.0 - (distance / MAX_RAY_DEPTH)).max(0.0) * 255.0) as u8;
        let adjusted = if is_side_hit {
            (brightness as f64 * 0.6) as u8
        } else {
            brightness
        };
        let red = adjusted;
        let green = (adjusted as f64 * 0.4) as u8;
        let blue = (adjusted as f64 * 0.2) as u8;
        TermColor::Rgb {
            r: red,
            g: green,
            b: blue,
        }
    }

    fn ceiling_color(distance_from_top: f64) -> TermColor {
        let brightness = (distance_from_top * 30.0).min(40.0) as u8;
        TermColor::Rgb {
            r: brightness / 3,
            g: brightness / 3,
            b: brightness,
        }
    }

    fn floor_color(distance_from_bottom: f64) -> TermColor {
        let brightness = (distance_from_bottom * 40.0).min(60.0) as u8;
        TermColor::Rgb {
            r: brightness / 2,
            g: brightness,
            b: brightness / 3,
        }
    }

    fn render_viewport(&self, world: &mut World) {
        let terminal = world.resources.terminal_size;
        let viewport_width = terminal.columns as usize;
        let viewport_height = (terminal.rows as usize).saturating_sub(2);
        if viewport_width == 0 || viewport_height == 0 {
            return;
        }

        let mut tilemap = Tilemap::new(viewport_width, viewport_height);

        let mut wall_distances: Vec<f64> = vec![MAX_RAY_DEPTH; viewport_width];

        for (screen_column, wall_distance_entry) in wall_distances.iter_mut().enumerate() {
            let camera_x = 2.0 * screen_column as f64 / viewport_width as f64 - 1.0;
            let ray_angle = self.player_angle + camera_x * (FIELD_OF_VIEW / 2.0);

            let (distance, is_side_hit) = self.cast_ray(ray_angle);
            *wall_distance_entry = distance;

            let wall_height = if distance > 0.01 {
                ((WALL_HEIGHT_SCALE / distance) * viewport_height as f64 / 24.0) as i32
            } else {
                viewport_height as i32
            };

            let half_height = wall_height / 2;
            let center_row = viewport_height as i32 / 2;
            let wall_top = (center_row - half_height).max(0) as usize;
            let wall_bottom = (center_row + half_height).min(viewport_height as i32 - 1) as usize;

            for row in 0..wall_top {
                let distance_from_top = (wall_top - row) as f64 / viewport_height as f64;
                tilemap.set(
                    screen_column,
                    row,
                    TilemapCell {
                        character: ' ',
                        foreground: TermColor::Black,
                        background: Self::ceiling_color(distance_from_top),
                    },
                );
            }

            let wall_char = Self::wall_character(distance);
            let wall_fg = Self::wall_color(distance, is_side_hit);
            let wall_bg = TermColor::Black;
            for row in wall_top..=wall_bottom {
                if row < viewport_height {
                    tilemap.set(
                        screen_column,
                        row,
                        TilemapCell {
                            character: wall_char,
                            foreground: wall_fg,
                            background: wall_bg,
                        },
                    );
                }
            }

            for row in (wall_bottom + 1)..viewport_height {
                let distance_from_bottom = (row - wall_bottom) as f64 / viewport_height as f64;
                tilemap.set(
                    screen_column,
                    row,
                    TilemapCell {
                        character: ' ',
                        foreground: TermColor::Black,
                        background: Self::floor_color(distance_from_bottom),
                    },
                );
            }
        }

        self.render_enemies_to_viewport(
            &mut tilemap,
            viewport_width,
            viewport_height,
            &wall_distances,
        );
        self.render_key_to_viewport(
            &mut tilemap,
            viewport_width,
            viewport_height,
            &wall_distances,
        );
        self.render_exit_to_viewport(
            &mut tilemap,
            viewport_width,
            viewport_height,
            &wall_distances,
        );

        if self.flash_timer > 0.0 {
            let flash_column = viewport_width / 2;
            let flash_row = viewport_height / 2;
            for delta_column in 0..3 {
                for delta_row in 0..2 {
                    let column = flash_column.wrapping_add(delta_column).wrapping_sub(1);
                    let row = flash_row.wrapping_add(delta_row).wrapping_sub(1);
                    if column < viewport_width && row < viewport_height {
                        tilemap.set(
                            column,
                            row,
                            TilemapCell {
                                character: '*',
                                foreground: TermColor::Yellow,
                                background: TermColor::DarkYellow,
                            },
                        );
                    }
                }
            }
        }

        let crosshair_column = viewport_width / 2;
        let crosshair_row = viewport_height / 2;
        if self.flash_timer <= 0.0
            && crosshair_column < viewport_width
            && crosshair_row < viewport_height
        {
            tilemap.set(
                crosshair_column,
                crosshair_row,
                TilemapCell {
                    character: '+',
                    foreground: TermColor::White,
                    background: TermColor::Black,
                },
            );
        }

        world.set_tilemap(self.viewport_entity, tilemap);
    }

    fn sprite_screen_column(
        &self,
        sprite_x: f64,
        sprite_y: f64,
        viewport_width: usize,
    ) -> Option<(i32, f64)> {
        let relative_x = sprite_x - self.player_x;
        let relative_y = sprite_y - self.player_y;
        let distance = (relative_x * relative_x + relative_y * relative_y).sqrt();
        if distance < 0.1 {
            return None;
        }
        let sprite_angle = relative_y.atan2(relative_x);
        let mut angle_difference = sprite_angle - self.player_angle;
        while angle_difference > std::f64::consts::PI {
            angle_difference -= 2.0 * std::f64::consts::PI;
        }
        while angle_difference < -std::f64::consts::PI {
            angle_difference += 2.0 * std::f64::consts::PI;
        }
        let half_fov = FIELD_OF_VIEW / 2.0;
        if angle_difference.abs() > half_fov + 0.1 {
            return None;
        }
        let screen_fraction = (angle_difference / half_fov + 1.0) / 2.0;
        let screen_x = (screen_fraction * viewport_width as f64) as i32;
        Some((screen_x, distance))
    }

    fn render_enemies_to_viewport(
        &self,
        tilemap: &mut Tilemap,
        viewport_width: usize,
        viewport_height: usize,
        wall_distances: &[f64],
    ) {
        for enemy in &self.enemies {
            if !enemy.alive {
                continue;
            }
            let Some((screen_x, distance)) =
                self.sprite_screen_column(enemy.position_x, enemy.position_y, viewport_width)
            else {
                continue;
            };
            if distance > MAX_RAY_DEPTH {
                continue;
            }
            let sprite_height = if distance > 0.1 {
                ((WALL_HEIGHT_SCALE * 0.7 / distance) * viewport_height as f64 / 24.0) as i32
            } else {
                viewport_height as i32
            };
            let half_height = sprite_height / 2;
            let center_row = viewport_height as i32 / 2;
            let sprite_top = (center_row - half_height).max(0);
            let sprite_bottom = (center_row + half_height).min(viewport_height as i32 - 1);
            let sprite_width = (sprite_height / 2).max(1);
            let half_width = sprite_width / 2;

            let brightness = ((1.0 - distance / MAX_RAY_DEPTH).max(0.0) * 255.0) as u8;

            for sprite_column in (screen_x - half_width)..=(screen_x + half_width) {
                if sprite_column < 0 || sprite_column >= viewport_width as i32 {
                    continue;
                }
                let column_index = sprite_column as usize;
                if distance >= wall_distances[column_index] {
                    continue;
                }
                for row in sprite_top..=sprite_bottom {
                    if row >= 0 && (row as usize) < viewport_height {
                        tilemap.set(
                            column_index,
                            row as usize,
                            TilemapCell {
                                character: 'E',
                                foreground: TermColor::Rgb {
                                    r: brightness,
                                    g: 0,
                                    b: 0,
                                },
                                background: TermColor::Black,
                            },
                        );
                    }
                }
            }
        }
    }

    fn render_key_to_viewport(
        &self,
        tilemap: &mut Tilemap,
        viewport_width: usize,
        viewport_height: usize,
        wall_distances: &[f64],
    ) {
        if self.has_key {
            return;
        }
        let key_x = 5.5;
        let key_y = 13.5;
        let Some((screen_x, distance)) = self.sprite_screen_column(key_x, key_y, viewport_width)
        else {
            return;
        };
        if distance > MAX_RAY_DEPTH {
            return;
        }
        let sprite_height = if distance > 0.1 {
            ((WALL_HEIGHT_SCALE * 0.5 / distance) * viewport_height as f64 / 24.0) as i32
        } else {
            viewport_height as i32
        };
        let half_height = sprite_height / 2;
        let center_row = viewport_height as i32 / 2;
        let sprite_top = (center_row - half_height).max(0);
        let sprite_bottom = (center_row + half_height).min(viewport_height as i32 - 1);

        let brightness = ((1.0 - distance / MAX_RAY_DEPTH).max(0.0) * 255.0) as u8;

        if screen_x >= 0 && screen_x < viewport_width as i32 {
            let column_index = screen_x as usize;
            if distance < wall_distances[column_index] {
                for row in sprite_top..=sprite_bottom {
                    if row >= 0 && (row as usize) < viewport_height {
                        tilemap.set(
                            column_index,
                            row as usize,
                            TilemapCell {
                                character: 'K',
                                foreground: TermColor::Rgb {
                                    r: brightness,
                                    g: brightness,
                                    b: 0,
                                },
                                background: TermColor::Black,
                            },
                        );
                    }
                }
            }
        }
    }

    fn render_exit_to_viewport(
        &self,
        tilemap: &mut Tilemap,
        viewport_width: usize,
        viewport_height: usize,
        wall_distances: &[f64],
    ) {
        let exit_x = 15.5;
        let exit_y = 14.5;
        let Some((screen_x, distance)) = self.sprite_screen_column(exit_x, exit_y, viewport_width)
        else {
            return;
        };
        if distance > MAX_RAY_DEPTH {
            return;
        }
        let sprite_height = if distance > 0.1 {
            ((WALL_HEIGHT_SCALE * 0.6 / distance) * viewport_height as f64 / 24.0) as i32
        } else {
            viewport_height as i32
        };
        let half_height = sprite_height / 2;
        let center_row = viewport_height as i32 / 2;
        let sprite_top = (center_row - half_height).max(0);
        let sprite_bottom = (center_row + half_height).min(viewport_height as i32 - 1);

        let brightness = ((1.0 - distance / MAX_RAY_DEPTH).max(0.0) * 255.0) as u8;
        let color = if self.has_key {
            TermColor::Rgb {
                r: 0,
                g: brightness,
                b: 0,
            }
        } else {
            TermColor::Rgb {
                r: brightness / 2,
                g: brightness / 2,
                b: brightness / 2,
            }
        };

        for delta_column in -1i32..=1 {
            let column = screen_x + delta_column;
            if column >= 0 && column < viewport_width as i32 {
                let column_index = column as usize;
                if distance < wall_distances[column_index] {
                    for row in sprite_top..=sprite_bottom {
                        if row >= 0 && (row as usize) < viewport_height {
                            tilemap.set(
                                column_index,
                                row as usize,
                                TilemapCell {
                                    character: 'X',
                                    foreground: color,
                                    background: TermColor::Black,
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    fn render_minimap(&self, world: &mut World) {
        let minimap_width = MAP_WIDTH * MINIMAP_SCALE;
        let minimap_height = MAP_HEIGHT * MINIMAP_SCALE;
        let mut tilemap = Tilemap::new(minimap_width, minimap_height);

        for map_row in 0..MAP_HEIGHT {
            for map_column in 0..MAP_WIDTH {
                let character = match self.map[map_row][map_column] {
                    CellContent::Wall => '#',
                    CellContent::Key => {
                        if self.has_key {
                            '.'
                        } else {
                            'K'
                        }
                    }
                    CellContent::Exit => 'X',
                    CellContent::Empty => '.',
                };
                let foreground = match self.map[map_row][map_column] {
                    CellContent::Wall => TermColor::Rgb {
                        r: 80,
                        g: 80,
                        b: 80,
                    },
                    CellContent::Key => {
                        if self.has_key {
                            TermColor::Rgb {
                                r: 40,
                                g: 40,
                                b: 40,
                            }
                        } else {
                            TermColor::Yellow
                        }
                    }
                    CellContent::Exit => {
                        if self.has_key {
                            TermColor::Green
                        } else {
                            TermColor::Grey
                        }
                    }
                    CellContent::Empty => TermColor::Rgb {
                        r: 40,
                        g: 40,
                        b: 40,
                    },
                };

                for scale_row in 0..MINIMAP_SCALE {
                    for scale_column in 0..MINIMAP_SCALE {
                        tilemap.set(
                            map_column * MINIMAP_SCALE + scale_column,
                            map_row * MINIMAP_SCALE + scale_row,
                            TilemapCell {
                                character,
                                foreground,
                                background: TermColor::Black,
                            },
                        );
                    }
                }
            }
        }

        let player_map_column = (self.player_x as usize).min(MAP_WIDTH - 1) * MINIMAP_SCALE;
        let player_map_row = (self.player_y as usize).min(MAP_HEIGHT - 1) * MINIMAP_SCALE;
        if player_map_column < minimap_width && player_map_row < minimap_height {
            tilemap.set(
                player_map_column,
                player_map_row,
                TilemapCell {
                    character: '@',
                    foreground: TermColor::Cyan,
                    background: TermColor::Black,
                },
            );
        }

        for enemy in &self.enemies {
            if !enemy.alive {
                continue;
            }
            let enemy_column = (enemy.position_x as usize).min(MAP_WIDTH - 1) * MINIMAP_SCALE;
            let enemy_row = (enemy.position_y as usize).min(MAP_HEIGHT - 1) * MINIMAP_SCALE;
            if enemy_column < minimap_width && enemy_row < minimap_height {
                tilemap.set(
                    enemy_column,
                    enemy_row,
                    TilemapCell {
                        character: 'E',
                        foreground: TermColor::Red,
                        background: TermColor::Black,
                    },
                );
            }
        }

        world.set_tilemap(self.minimap_entity, tilemap);
    }

    fn render_hud(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);
        let terminal = world.resources.terminal_size;
        let hud_row = (terminal.rows as usize).saturating_sub(2) as f64;

        let health_color = if self.health > 60 {
            TermColor::Green
        } else if self.health > 30 {
            TermColor::Yellow
        } else {
            TermColor::Red
        };
        let health_bar_filled =
            ((self.health as f64 / STARTING_HEALTH as f64) * 10.0).round() as usize;
        let health_bar_empty = 10_usize.saturating_sub(health_bar_filled);
        let health_text = format!(
            "HP [{}>{}] {}",
            "=".repeat(health_bar_filled),
            " ".repeat(health_bar_empty),
            self.health,
        );

        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 1.0,
                row: hud_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: health_text,
                foreground: health_color,
                background: TermColor::Rgb {
                    r: 20,
                    g: 20,
                    b: 20,
                },
            },
        );
        world.set_z_index(entity, ZIndex(20));

        let ammo_text = format!("AMMO: {}", self.ammo);
        let ammo_color = if self.ammo > 5 {
            TermColor::Cyan
        } else if self.ammo > 0 {
            TermColor::Yellow
        } else {
            TermColor::Red
        };
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 25.0,
                row: hud_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: ammo_text,
                foreground: ammo_color,
                background: TermColor::Rgb {
                    r: 20,
                    g: 20,
                    b: 20,
                },
            },
        );
        world.set_z_index(entity, ZIndex(20));

        let kills_text = format!("KILLS: {}", self.kills);
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 38.0,
                row: hud_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: kills_text,
                foreground: TermColor::White,
                background: TermColor::Rgb {
                    r: 20,
                    g: 20,
                    b: 20,
                },
            },
        );
        world.set_z_index(entity, ZIndex(20));

        let key_text = if self.has_key { "KEY: YES" } else { "KEY: NO" };
        let key_color = if self.has_key {
            TermColor::Yellow
        } else {
            TermColor::DarkGrey
        };
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 50.0,
                row: hud_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: key_text.to_string(),
                foreground: key_color,
                background: TermColor::Rgb {
                    r: 20,
                    g: 20,
                    b: 20,
                },
            },
        );
        world.set_z_index(entity, ZIndex(20));

        let status_row = hud_row + 1.0;
        let status_text = if self.has_key {
            "Find the EXIT (X) to escape!"
        } else {
            "Find the KEY (K) to unlock the exit!"
        };
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 1.0,
                row: status_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: status_text.to_string(),
                foreground: TermColor::Grey,
                background: TermColor::Rgb {
                    r: 20,
                    g: 20,
                    b: 20,
                },
            },
        );
        world.set_z_index(entity, ZIndex(20));

        let fill_width = terminal.columns as usize;
        for fill_row_offset in 0..2 {
            let row_value = hud_row + fill_row_offset as f64;
            let entity = self
                .hud_entities
                .spawn_one(world, POSITION | TILEMAP | Z_INDEX);
            let mut fill_tilemap = Tilemap::new(fill_width, 1);
            for column in 0..fill_width {
                fill_tilemap.set(
                    column,
                    0,
                    TilemapCell {
                        character: ' ',
                        foreground: TermColor::Black,
                        background: TermColor::Rgb {
                            r: 20,
                            g: 20,
                            b: 20,
                        },
                    },
                );
            }
            world.set_position(
                entity,
                Position {
                    column: 0.0,
                    row: row_value,
                },
            );
            world.set_tilemap(entity, fill_tilemap);
            world.set_z_index(entity, ZIndex(15));
        }
    }

    fn try_move(&self, new_x: f64, new_y: f64) -> bool {
        let margin = 0.2;
        let check_positions = [
            (new_x - margin, new_y - margin),
            (new_x + margin, new_y - margin),
            (new_x - margin, new_y + margin),
            (new_x + margin, new_y + margin),
        ];
        for (check_x, check_y) in check_positions {
            let grid_x = check_x.floor() as i32;
            let grid_y = check_y.floor() as i32;
            if self.is_wall(grid_x, grid_y) {
                return false;
            }
        }
        true
    }

    fn update_player_movement(&mut self, world: &World) {
        let delta = world.resources.timing.delta_seconds;
        let keyboard = &world.resources.keyboard;

        if keyboard.is_pressed(KeyCode::Left) {
            self.player_angle -= TURN_SPEED * delta;
        }
        if keyboard.is_pressed(KeyCode::Right) {
            self.player_angle += TURN_SPEED * delta;
        }

        while self.player_angle < 0.0 {
            self.player_angle += 2.0 * std::f64::consts::PI;
        }
        while self.player_angle >= 2.0 * std::f64::consts::PI {
            self.player_angle -= 2.0 * std::f64::consts::PI;
        }

        let forward_x = self.player_angle.cos();
        let forward_y = self.player_angle.sin();
        let strafe_x = (self.player_angle - std::f64::consts::FRAC_PI_2).cos();
        let strafe_y = (self.player_angle - std::f64::consts::FRAC_PI_2).sin();

        let mut move_x = 0.0;
        let mut move_y = 0.0;

        if keyboard.is_pressed(KeyCode::Char('w')) {
            move_x += forward_x;
            move_y += forward_y;
        }
        if keyboard.is_pressed(KeyCode::Char('s')) {
            move_x -= forward_x;
            move_y -= forward_y;
        }
        if keyboard.is_pressed(KeyCode::Char('a')) {
            move_x += strafe_x;
            move_y += strafe_y;
        }
        if keyboard.is_pressed(KeyCode::Char('d')) {
            move_x -= strafe_x;
            move_y -= strafe_y;
        }

        let magnitude = (move_x * move_x + move_y * move_y).sqrt();
        if magnitude > 0.01 {
            move_x /= magnitude;
            move_y /= magnitude;

            let new_x = self.player_x + move_x * MOVE_SPEED * delta;
            let new_y = self.player_y + move_y * MOVE_SPEED * delta;

            if self.try_move(new_x, self.player_y) {
                self.player_x = new_x;
            }
            if self.try_move(self.player_x, new_y) {
                self.player_y = new_y;
            }
        }

        self.check_pickups();
    }

    fn check_pickups(&mut self) {
        let player_grid_x = self.player_x.floor() as usize;
        let player_grid_y = self.player_y.floor() as usize;

        if player_grid_x < MAP_WIDTH && player_grid_y < MAP_HEIGHT {
            if self.map[player_grid_y][player_grid_x] == CellContent::Key && !self.has_key {
                self.has_key = true;
                self.map[player_grid_y][player_grid_x] = CellContent::Empty;
            }

            if self.map[player_grid_y][player_grid_x] == CellContent::Exit && self.has_key {
                self.transition = Some(Box::new(WinState {
                    kills: self.kills,
                    entities: EntityGroup::new(),
                    restart: false,
                }));
            }
        }
    }

    fn shoot(&mut self) {
        if self.ammo <= 0 || self.shoot_cooldown > 0.0 {
            return;
        }

        self.ammo -= 1;
        self.shoot_cooldown = 0.3;
        self.flash_timer = 0.1;

        let ray_direction_x = self.player_angle.cos();
        let ray_direction_y = self.player_angle.sin();

        let mut closest_enemy_index: Option<usize> = None;
        let mut closest_distance = MAX_RAY_DEPTH;

        for (enemy_index, enemy) in self.enemies.iter().enumerate() {
            if !enemy.alive {
                continue;
            }

            let relative_x = enemy.position_x - self.player_x;
            let relative_y = enemy.position_y - self.player_y;
            let distance = (relative_x * relative_x + relative_y * relative_y).sqrt();

            if !(0.1..=MAX_RAY_DEPTH).contains(&distance) {
                continue;
            }

            let dot = relative_x * ray_direction_x + relative_y * ray_direction_y;
            if dot < 0.0 {
                continue;
            }

            let perpendicular_x = relative_x - dot * ray_direction_x;
            let perpendicular_y = relative_y - dot * ray_direction_y;
            let perpendicular_distance =
                (perpendicular_x * perpendicular_x + perpendicular_y * perpendicular_y).sqrt();

            if perpendicular_distance < ENEMY_RADIUS && distance < closest_distance {
                let (wall_distance, _) = self.cast_ray(self.player_angle);
                if distance < wall_distance {
                    closest_distance = distance;
                    closest_enemy_index = Some(enemy_index);
                }
            }
        }

        if let Some(enemy_index) = closest_enemy_index {
            self.enemies[enemy_index].health -= 15;
            if self.enemies[enemy_index].health <= 0 {
                self.enemies[enemy_index].alive = false;
                self.kills += 1;
            }
        }
    }

    fn update_enemies(&mut self, delta: f64) {
        let player_x = self.player_x;
        let player_y = self.player_y;

        for enemy in &mut self.enemies {
            if !enemy.alive {
                continue;
            }

            enemy.attack_timer = (enemy.attack_timer - delta).max(0.0);

            let relative_x = player_x - enemy.position_x;
            let relative_y = player_y - enemy.position_y;
            let distance = (relative_x * relative_x + relative_y * relative_y).sqrt();

            if distance < ENEMY_ATTACK_RANGE && enemy.attack_timer <= 0.0 {
                enemy.attack_timer = ENEMY_ATTACK_COOLDOWN;
            } else if distance < ENEMY_AGGRO_RANGE && distance > ENEMY_ATTACK_RANGE * 0.8 {
                let direction_x = relative_x / distance;
                let direction_y = relative_y / distance;

                let is_blocked = |test_x: f64, test_y: f64| -> bool {
                    let grid_x = test_x.floor() as i32;
                    let grid_y = test_y.floor() as i32;
                    if grid_x < 0
                        || grid_x >= MAP_WIDTH as i32
                        || grid_y < 0
                        || grid_y >= MAP_HEIGHT as i32
                    {
                        return true;
                    }
                    self.map[grid_y as usize][grid_x as usize] == CellContent::Wall
                };

                let candidate_x = enemy.position_x + direction_x * ENEMY_MOVE_SPEED * delta;
                let candidate_y = enemy.position_y + direction_y * ENEMY_MOVE_SPEED * delta;

                let can_move_x = !is_blocked(candidate_x, enemy.position_y);
                let can_move_y = !is_blocked(enemy.position_x, candidate_y);

                if can_move_x {
                    enemy.position_x += direction_x * ENEMY_MOVE_SPEED * delta;
                }
                if can_move_y {
                    enemy.position_y += direction_y * ENEMY_MOVE_SPEED * delta;
                }
            }
        }
    }

    fn apply_enemy_damage(&mut self) {
        for enemy in &self.enemies {
            if !enemy.alive {
                continue;
            }
            let relative_x = self.player_x - enemy.position_x;
            let relative_y = self.player_y - enemy.position_y;
            let distance = (relative_x * relative_x + relative_y * relative_y).sqrt();
            if distance < ENEMY_ATTACK_RANGE
                && enemy.attack_timer > ENEMY_ATTACK_COOLDOWN - 0.05
                && enemy.attack_timer <= ENEMY_ATTACK_COOLDOWN
            {
                self.health -= ENEMY_DAMAGE;
            }
        }
        if self.health <= 0 {
            self.health = 0;
            self.transition = Some(Box::new(GameOverState {
                kills: self.kills,
                entities: EntityGroup::new(),
                restart: false,
            }));
        }
    }

    fn clear_all(&mut self, world: &mut World) {
        if self.viewport_entity != Entity::default() {
            world.despawn_entities(&[self.viewport_entity]);
        }
        if self.minimap_entity != Entity::default() {
            world.despawn_entities(&[self.minimap_entity]);
        }
        self.hud_entities.despawn_all(world);
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "DOOM - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        let viewport_width = terminal.columns as usize;
        let viewport_height = (terminal.rows as usize).saturating_sub(2);

        self.viewport_entity = EntityBuilder::new()
            .position(Position {
                column: 0.0,
                row: 0.0,
            })
            .tilemap(Tilemap::new(viewport_width, viewport_height))
            .z_index(ZIndex(0))
            .spawn(world);

        self.minimap_entity = EntityBuilder::new()
            .position(Position {
                column: (terminal.columns as usize - MAP_WIDTH * MINIMAP_SCALE - 1) as f64,
                row: 1.0,
            })
            .tilemap(Tilemap::new(
                MAP_WIDTH * MINIMAP_SCALE,
                MAP_HEIGHT * MINIMAP_SCALE,
            ))
            .z_index(ZIndex(10))
            .spawn(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Char('z') => self.shoot(),
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;

        self.shoot_cooldown = (self.shoot_cooldown - delta).max(0.0);
        self.flash_timer = (self.flash_timer - delta).max(0.0);

        self.update_player_movement(world);
        self.update_enemies(delta);
        self.apply_enemy_damage();
        self.render_viewport(world);
        self.render_minimap(world);
        self.render_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if let Some(next) = self.transition.take() {
            self.clear_all(world);
            return Some(next);
        }
        None
    }
}

struct WinState {
    kills: i32,
    entities: EntityGroup,
    restart: bool,
}

impl State for WinState {
    fn title(&self) -> &str {
        "DOOM - You Win!"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let kills_text = format!("Enemies defeated: {}", self.kills);
        let lines: Vec<(&str, TermColor)> = vec![
            ("YOU ESCAPED!", TermColor::Green),
            ("", TermColor::Black),
            ("You found the key and reached the exit.", TermColor::White),
            ("", TermColor::Black),
            (&kills_text, TermColor::Yellow),
            ("", TermColor::Black),
            ("Press R to play again", TermColor::White),
            ("Press ESC to quit", TermColor::Grey),
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
                    row: center_row - 4.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: text.to_string(),
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

struct GameOverState {
    kills: i32,
    entities: EntityGroup,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "DOOM - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let kills_text = format!("Enemies defeated: {}", self.kills);
        let lines: Vec<(&str, TermColor)> = vec![
            ("YOU DIED", TermColor::Red),
            ("", TermColor::Black),
            ("The demons got you...", TermColor::DarkRed),
            ("", TermColor::Black),
            (&kills_text, TermColor::Yellow),
            ("", TermColor::Black),
            ("Press R to try again", TermColor::White),
            ("Press ESC to quit", TermColor::Grey),
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
                    row: center_row - 4.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: text.to_string(),
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
