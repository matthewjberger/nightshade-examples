use nightshade::tui::prelude::*;

const MAP_WIDTH: usize = 70;
const MAP_HEIGHT: usize = 28;
const TILE_GRASS: u8 = 0;
const TILE_PATH: u8 = 1;
const TILE_TOWER: u8 = 2;
const TOWER_RANGE: f64 = 5.0;

const LAYER_ENEMY: u32 = 1;
const LAYER_PROJECTILE: u32 = 2;

struct PathSegment {
    column: usize,
    row: usize,
}

fn build_path() -> Vec<PathSegment> {
    let waypoints: Vec<(usize, usize)> = vec![
        (0, 3),
        (15, 3),
        (15, 10),
        (3, 10),
        (3, 18),
        (22, 18),
        (22, 7),
        (35, 7),
        (35, 25),
        (12, 25),
        (12, 14),
        (45, 14),
        (45, 3),
        (55, 3),
        (55, 18),
        (40, 18),
        (40, 25),
        (62, 25),
        (62, 10),
        (69, 10),
    ];

    let mut path = Vec::new();
    for window_index in 0..waypoints.len() - 1 {
        let (start_column, start_row) = waypoints[window_index];
        let (end_column, end_row) = waypoints[window_index + 1];

        let delta_column = (end_column as i32 - start_column as i32).signum();
        let delta_row = (end_row as i32 - start_row as i32).signum();

        let mut current_column = start_column as i32;
        let mut current_row = start_row as i32;

        while current_column != end_column as i32 || current_row != end_row as i32 {
            path.push(PathSegment {
                column: current_column as usize,
                row: current_row as usize,
            });
            if current_column != end_column as i32 {
                current_column += delta_column;
            } else {
                current_row += delta_row;
            }
        }
    }
    let last = waypoints.last().unwrap();
    path.push(PathSegment {
        column: last.0,
        row: last.1,
    });
    path
}

fn build_map(path: &[PathSegment]) -> [[u8; MAP_WIDTH]; MAP_HEIGHT] {
    let mut map = [[TILE_GRASS; MAP_WIDTH]; MAP_HEIGHT];
    for segment in path {
        if segment.row < MAP_HEIGHT && segment.column < MAP_WIDTH {
            map[segment.row][segment.column] = TILE_PATH;
        }
    }
    map
}

struct EnemyData {
    entity: Entity,
    path_index: usize,
    speed: f64,
    progress: f64,
    health: i32,
    max_health: i32,
    health_bar_entity: Entity,
}

struct TowerData {
    entity: Entity,
    grid_column: usize,
    grid_row: usize,
    fire_timer: Timer,
    tower_type: usize,
}

struct ProjectileData {
    entity: Entity,
    target: Entity,
    speed: f64,
    damage: i32,
}

#[derive(Clone)]
struct TowerType {
    name: String,
    character: char,
    color: TermColor,
    fire_rate: f64,
    damage: i32,
    cost: u32,
}

fn tower_types() -> Vec<TowerType> {
    vec![
        TowerType {
            name: "Arrow".to_string(),
            character: 'A',
            color: TermColor::Green,
            fire_rate: 0.8,
            damage: 10,
            cost: 50,
        },
        TowerType {
            name: "Cannon".to_string(),
            character: 'C',
            color: TermColor::Red,
            fire_rate: 1.5,
            damage: 30,
            cost: 100,
        },
        TowerType {
            name: "Magic".to_string(),
            character: 'M',
            color: TermColor::Magenta,
            fire_rate: 0.5,
            damage: 8,
            cost: 75,
        },
    ]
}

struct WaveConfig {
    enemy_count: usize,
    health: i32,
    speed: f64,
    spawn_interval: f64,
}

fn wave_config(wave_number: u32) -> WaveConfig {
    WaveConfig {
        enemy_count: 5 + wave_number as usize * 3,
        health: 30 + wave_number as i32 * 15,
        speed: 3.0 + wave_number as f64 * 0.5,
        spawn_interval: (1.2 - wave_number as f64 * 0.1).max(0.3),
    }
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Tower Defense - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "TOWER DEFENSE";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - title.len() as f64 / 2.0,
                row: center_row - 4.0,
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

        let subtitle = "Click to place towers, defend your base!";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - subtitle.len() as f64 / 2.0,
                row: center_row - 2.0,
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

        let prompt = "Press ENTER to start";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - prompt.len() as f64 / 2.0,
                row: center_row + 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: prompt.to_string(),
                foreground: TermColor::Grey,
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
                row: center_row + 3.0,
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
    map: [[u8; MAP_WIDTH]; MAP_HEIGHT],
    path: Vec<PathSegment>,
    tilemap_entity: Entity,
    offset_column: i32,
    offset_row: i32,
    enemies: Vec<EnemyData>,
    towers: Vec<TowerData>,
    projectiles: Vec<ProjectileData>,
    particles: ParticleEmitter,
    gold: u32,
    lives: u32,
    wave_number: u32,
    wave_active: bool,
    enemies_spawned: usize,
    enemies_to_spawn: usize,
    spawn_timer: Timer,
    tower_menu: Menu,
    hud_entities: EntityGroup,
    cursor_entity: Entity,
    cursor_column: u16,
    cursor_row: u16,
    game_over: bool,
    all_types: Vec<TowerType>,
}

impl GameplayState {
    fn new() -> Self {
        let path = build_path();
        let map = build_map(&path);
        let types = tower_types();

        let menu_items: Vec<String> = types
            .iter()
            .map(|tower_type| format!("{} ({}g)", tower_type.name, tower_type.cost))
            .collect();

        Self {
            map,
            path,
            tilemap_entity: Entity::default(),
            offset_column: 0,
            offset_row: 0,
            enemies: Vec::new(),
            towers: Vec::new(),
            projectiles: Vec::new(),
            particles: ParticleEmitter::new(),
            gold: 200,
            lives: 20,
            wave_number: 0,
            wave_active: false,
            enemies_spawned: 0,
            enemies_to_spawn: 0,
            spawn_timer: Timer::repeating(1.0),
            tower_menu: Menu::new(menu_items, 0.0, 0.0, MenuColors::default(), 20),
            hud_entities: EntityGroup::new(),
            cursor_entity: Entity::default(),
            cursor_column: 0,
            cursor_row: 0,
            game_over: false,
            all_types: types,
        }
    }

    fn build_tilemap(&mut self, world: &mut World) {
        let terminal = world.resources.terminal_size;
        self.offset_column = ((terminal.columns as i32 - MAP_WIDTH as i32) / 2).max(0);
        self.offset_row = ((terminal.rows as i32 - MAP_HEIGHT as i32 - 4) / 2).max(1);

        let mut tilemap = Tilemap::new(MAP_WIDTH, MAP_HEIGHT);
        for row in 0..MAP_HEIGHT {
            for column in 0..MAP_WIDTH {
                let cell = match self.map[row][column] {
                    TILE_PATH => TilemapCell {
                        character: '.',
                        foreground: TermColor::DarkYellow,
                        background: TermColor::Rgb {
                            r: 40,
                            g: 30,
                            b: 10,
                        },
                    },
                    TILE_TOWER => TilemapCell {
                        character: ' ',
                        foreground: TermColor::White,
                        background: TermColor::Black,
                    },
                    _ => TilemapCell {
                        character: ' ',
                        foreground: TermColor::DarkGreen,
                        background: TermColor::Rgb {
                            r: 10,
                            g: 30,
                            b: 10,
                        },
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

    fn update_hud(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);

        let hud_row = self.offset_row as f64 + MAP_HEIGHT as f64 + 1.0;

        let gold_text = format!(
            "Gold: {}  Lives: {}  Wave: {}",
            self.gold, self.lives, self.wave_number
        );
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
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
                text: gold_text,
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(15));

        let status = if self.wave_active {
            format!(
                "Enemies: {}/{}",
                self.enemies.len(),
                self.enemies_to_spawn - self.enemies_spawned + self.enemies.len()
            )
        } else {
            "Press SPACE for next wave".to_string()
        };
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: self.offset_column as f64,
                row: hud_row + 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: status,
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(15));

        let menu_column = self.offset_column as f64 + MAP_WIDTH as f64 + 2.0;
        self.tower_menu = Menu::new(
            self.all_types
                .iter()
                .map(|tower_type| format!("{} ({}g)", tower_type.name, tower_type.cost))
                .collect(),
            menu_column,
            self.offset_row as f64,
            MenuColors {
                normal_foreground: TermColor::White,
                normal_background: TermColor::Black,
                selected_foreground: TermColor::Black,
                selected_background: TermColor::Yellow,
            },
            20,
        );
        self.tower_menu.select_at(self.tower_menu.selected_index());
        self.tower_menu.render(world);

        let help_text = "Up/Down: select tower | Click: place";
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: menu_column,
                row: self.offset_row as f64 + self.all_types.len() as f64 + 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: help_text.to_string(),
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(15));
    }

    fn try_place_tower(&mut self, world: &mut World, grid_column: usize, grid_row: usize) {
        if grid_column >= MAP_WIDTH || grid_row >= MAP_HEIGHT {
            return;
        }
        if self.map[grid_row][grid_column] != TILE_GRASS {
            return;
        }

        let selected_type = self.tower_menu.selected_index();
        let tower_type = &self.all_types[selected_type];
        if self.gold < tower_type.cost {
            return;
        }

        self.gold -= tower_type.cost;
        self.map[grid_row][grid_column] = TILE_TOWER;

        if let Some(tilemap) = world.get_tilemap_mut(self.tilemap_entity) {
            tilemap.set(
                grid_column,
                grid_row,
                TilemapCell {
                    character: tower_type.character,
                    foreground: tower_type.color,
                    background: TermColor::Rgb {
                        r: 10,
                        g: 30,
                        b: 10,
                    },
                },
            );
        }

        let entity = EntityBuilder::new()
            .position(Position {
                column: self.offset_column as f64 + grid_column as f64,
                row: self.offset_row as f64 + grid_row as f64,
            })
            .z_index(ZIndex(5))
            .spawn(world);

        self.towers.push(TowerData {
            entity,
            grid_column,
            grid_row,
            fire_timer: Timer::repeating(tower_type.fire_rate),
            tower_type: selected_type,
        });
    }

    fn spawn_enemy(&mut self, world: &mut World) {
        let config = wave_config(self.wave_number);
        let start = &self.path[0];

        let entity = EntityBuilder::new()
            .position(Position {
                column: self.offset_column as f64 + start.column as f64,
                row: self.offset_row as f64 + start.row as f64,
            })
            .sprite(Sprite {
                character: 'E',
                foreground: TermColor::Red,
                background: TermColor::Rgb {
                    r: 40,
                    g: 30,
                    b: 10,
                },
            })
            .z_index(ZIndex(3))
            .collider(Collider {
                width: 1,
                height: 1,
                layer: LAYER_ENEMY,
                mask: LAYER_PROJECTILE,
                ..Default::default()
            })
            .spawn(world);

        let health_bar_entity = EntityBuilder::new()
            .position(Position {
                column: self.offset_column as f64 + start.column as f64 - 1.0,
                row: self.offset_row as f64 + start.row as f64 - 1.0,
            })
            .label(Label {
                text: "███".to_string(),
                foreground: TermColor::Green,
                background: TermColor::Black,
            })
            .z_index(ZIndex(4))
            .spawn(world);

        self.enemies.push(EnemyData {
            entity,
            path_index: 0,
            speed: config.speed,
            progress: 0.0,
            health: config.health,
            max_health: config.health,
            health_bar_entity,
        });
    }

    fn update_enemies(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;
        let mut reached_end = Vec::new();

        for (enemy_index, enemy) in self.enemies.iter_mut().enumerate() {
            enemy.progress += enemy.speed * delta;

            while enemy.progress >= 1.0 && enemy.path_index + 1 < self.path.len() {
                enemy.progress -= 1.0;
                enemy.path_index += 1;
            }

            if enemy.path_index >= self.path.len() - 1 && enemy.progress >= 1.0 {
                reached_end.push(enemy_index);
                continue;
            }

            let current = &self.path[enemy.path_index];
            let next_index = (enemy.path_index + 1).min(self.path.len() - 1);
            let next = &self.path[next_index];

            let lerp_column = current.column as f64
                + (next.column as f64 - current.column as f64) * enemy.progress.min(1.0);
            let lerp_row = current.row as f64
                + (next.row as f64 - current.row as f64) * enemy.progress.min(1.0);

            if let Some(position) = world.get_position_mut(enemy.entity) {
                position.column = self.offset_column as f64 + lerp_column;
                position.row = self.offset_row as f64 + lerp_row;
            }

            let health_fraction = enemy.health as f64 / enemy.max_health as f64;
            let bar_length = 3;
            let filled = (health_fraction * bar_length as f64).ceil() as usize;
            let bar: String = format!("{}{}", "█".repeat(filled), "░".repeat(bar_length - filled));
            let bar_color = if health_fraction > 0.6 {
                TermColor::Green
            } else if health_fraction > 0.3 {
                TermColor::Yellow
            } else {
                TermColor::Red
            };

            if let Some(label) = world.get_label_mut(enemy.health_bar_entity) {
                label.text = bar;
                label.foreground = bar_color;
            }
            if let Some(position) = world.get_position_mut(enemy.health_bar_entity) {
                position.column = self.offset_column as f64 + lerp_column - 1.0;
                position.row = self.offset_row as f64 + lerp_row - 1.0;
            }
        }

        for &enemy_index in reached_end.iter().rev() {
            let enemy = self.enemies.swap_remove(enemy_index);
            world.despawn_entities(&[enemy.entity, enemy.health_bar_entity]);
            self.lives = self.lives.saturating_sub(1);
            if self.lives == 0 {
                self.game_over = true;
            }
        }
    }

    fn update_towers(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;
        let mut new_projectiles = Vec::new();

        for tower in &mut self.towers {
            if !tower.fire_timer.tick(delta) {
                continue;
            }

            let tower_column = self.offset_column as f64 + tower.grid_column as f64;
            let tower_row = self.offset_row as f64 + tower.grid_row as f64;

            let mut closest_enemy: Option<(Entity, f64)> = None;
            for enemy in &self.enemies {
                if let Some(enemy_position) = world.get_position(enemy.entity) {
                    let distance_column = enemy_position.column - tower_column;
                    let distance_row = enemy_position.row - tower_row;
                    let distance =
                        (distance_column * distance_column + distance_row * distance_row).sqrt();
                    if distance <= TOWER_RANGE
                        && (closest_enemy.is_none() || distance < closest_enemy.unwrap().1)
                    {
                        closest_enemy = Some((enemy.entity, distance));
                    }
                }
            }

            if let Some((target_entity, _)) = closest_enemy {
                let tower_type = &self.all_types[tower.tower_type];
                let projectile_entity = EntityBuilder::new()
                    .position(Position {
                        column: tower_column,
                        row: tower_row,
                    })
                    .sprite(Sprite {
                        character: '*',
                        foreground: tower_type.color,
                        background: TermColor::Black,
                    })
                    .z_index(ZIndex(4))
                    .collider(Collider {
                        width: 1,
                        height: 1,
                        layer: LAYER_PROJECTILE,
                        mask: LAYER_ENEMY,
                        ..Default::default()
                    })
                    .spawn(world);

                new_projectiles.push(ProjectileData {
                    entity: projectile_entity,
                    target: target_entity,
                    speed: 15.0,
                    damage: tower_type.damage,
                });
            }
        }

        self.projectiles.extend(new_projectiles);
    }

    fn update_projectiles(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;
        let mut to_remove = Vec::new();
        let mut hits: Vec<(usize, Entity, i32, f64, f64)> = Vec::new();

        for (projectile_index, projectile) in self.projectiles.iter().enumerate() {
            let projectile_position = match world.get_position(projectile.entity) {
                Some(position) => *position,
                None => {
                    to_remove.push(projectile_index);
                    continue;
                }
            };

            let target_position = match world.get_position(projectile.target) {
                Some(position) => *position,
                None => {
                    to_remove.push(projectile_index);
                    continue;
                }
            };

            let direction_column = target_position.column - projectile_position.column;
            let direction_row = target_position.row - projectile_position.row;
            let distance =
                (direction_column * direction_column + direction_row * direction_row).sqrt();

            if distance < 0.5 {
                hits.push((
                    projectile_index,
                    projectile.target,
                    projectile.damage,
                    projectile_position.column,
                    projectile_position.row,
                ));
                continue;
            }

            let normalized_column = direction_column / distance;
            let normalized_row = direction_row / distance;

            if let Some(position) = world.get_position_mut(projectile.entity) {
                position.column += normalized_column * projectile.speed * delta;
                position.row += normalized_row * projectile.speed * delta;
            }
        }

        for (projectile_index, target, damage, hit_column, hit_row) in hits.iter().rev() {
            let projectile = &self.projectiles[*projectile_index];
            world.despawn_entities(&[projectile.entity]);

            if let Some(enemy) = self
                .enemies
                .iter_mut()
                .find(|enemy| enemy.entity == *target)
            {
                enemy.health -= damage;
                if enemy.health <= 0 {
                    self.particles.emit(
                        world,
                        *hit_column,
                        *hit_row,
                        5,
                        &ParticleConfig {
                            characters: vec!['*', '+', '.'],
                            colors: vec![TermColor::Red, TermColor::Yellow, TermColor::DarkRed],
                            lifetime: 0.5,
                            speed_min: 2.0,
                            speed_max: 6.0,
                            spread: std::f64::consts::PI * 2.0,
                            direction: 0.0,
                            z_index: 6,
                        },
                    );
                    self.gold += 10 + self.wave_number * 2;
                    world.despawn_entities(&[enemy.entity, enemy.health_bar_entity]);
                    self.enemies.retain(|existing| existing.entity != *target);
                }
            }

            if !to_remove.contains(projectile_index) {
                to_remove.push(*projectile_index);
            }
        }

        to_remove.sort_unstable();
        to_remove.dedup();
        for &projectile_index in to_remove.iter().rev() {
            if projectile_index < self.projectiles.len() {
                let projectile = self.projectiles.swap_remove(projectile_index);
                if world.get_position(projectile.entity).is_some() {
                    world.despawn_entities(&[projectile.entity]);
                }
            }
        }
    }

    fn start_wave(&mut self) {
        self.wave_number += 1;
        let config = wave_config(self.wave_number);
        self.enemies_to_spawn = config.enemy_count;
        self.enemies_spawned = 0;
        self.spawn_timer = Timer::repeating(config.spawn_interval);
        self.wave_active = true;
    }

    fn clear_all(&mut self, world: &mut World) {
        for enemy in &self.enemies {
            world.despawn_entities(&[enemy.entity, enemy.health_bar_entity]);
        }
        self.enemies.clear();

        for tower in &self.towers {
            world.despawn_entities(&[tower.entity]);
        }
        self.towers.clear();

        for projectile in &self.projectiles {
            world.despawn_entities(&[projectile.entity]);
        }
        self.projectiles.clear();

        self.particles.despawn_all(world);
        self.hud_entities.despawn_all(world);
        self.tower_menu.despawn(world);
        world.despawn_entities(&[self.tilemap_entity, self.cursor_entity]);
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Tower Defense - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        self.build_tilemap(world);

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

        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Up => self.tower_menu.up(),
            KeyCode::Down => self.tower_menu.down(),
            KeyCode::Char(' ') => {
                if !self.wave_active {
                    self.start_wave();
                }
            }
            KeyCode::Escape => world.resources.should_exit = true,
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
            && grid_column < MAP_WIDTH as i32
            && grid_row >= 0
            && grid_row < MAP_HEIGHT as i32
        {
            self.try_place_tower(world, grid_column as usize, grid_row as usize);
        }
    }

    fn on_mouse_move(&mut self, world: &mut World, column: u16, row: u16) {
        self.cursor_column = column;
        self.cursor_row = row;

        let grid_column = column as i32 - self.offset_column;
        let grid_row = row as i32 - self.offset_row;

        let in_bounds = grid_column >= 0
            && grid_column < MAP_WIDTH as i32
            && grid_row >= 0
            && grid_row < MAP_HEIGHT as i32;

        if let Some(visibility) = world.get_visibility_mut(self.cursor_entity) {
            visibility.visible = in_bounds;
        }
        if in_bounds {
            if let Some(position) = world.get_position_mut(self.cursor_entity) {
                position.column = column as f64;
                position.row = row as f64;
            }
            let can_place = self.map[grid_row as usize][grid_column as usize] == TILE_GRASS;
            if let Some(sprite) = world.get_sprite_mut(self.cursor_entity) {
                sprite.foreground = if can_place {
                    TermColor::Green
                } else {
                    TermColor::Red
                };
            }
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.game_over {
            return;
        }

        let delta = world.resources.timing.delta_seconds;

        if self.wave_active {
            if self.enemies_spawned < self.enemies_to_spawn && self.spawn_timer.tick(delta) {
                self.spawn_enemy(world);
                self.enemies_spawned += 1;
            }

            if self.enemies_spawned >= self.enemies_to_spawn && self.enemies.is_empty() {
                self.wave_active = false;
            }
        }

        self.update_enemies(world);
        self.update_towers(world);
        self.update_projectiles(world);
        self.particles.update(world, delta);
        self.update_hud(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            self.clear_all(world);
            return Some(Box::new(GameOverState {
                wave_reached: self.wave_number,
                gold_earned: self.gold,
                restart: false,
                entities: EntityGroup::new(),
            }));
        }
        None
    }
}

struct GameOverState {
    wave_reached: u32,
    gold_earned: u32,
    restart: bool,
    entities: EntityGroup,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Tower Defense - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let waves_text = format!("Waves survived: {}", self.wave_reached);
        let gold_text = format!("Gold remaining: {}", self.gold_earned);
        let lines: Vec<(&str, TermColor)> = vec![
            ("GAME OVER", TermColor::Red),
            ("", TermColor::Black),
            (&waves_text, TermColor::Yellow),
            (&gold_text, TermColor::Yellow),
            ("", TermColor::Black),
            ("Press R to restart", TermColor::White),
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
