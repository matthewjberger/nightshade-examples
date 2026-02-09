use nightshade::tui::prelude::*;
use rand::Rng;

const MAP_WIDTH: usize = 40;
const MAP_HEIGHT: usize = 22;

const TILE_GROUND: u8 = 0;
const TILE_WALL: u8 = 1;
const TILE_FARM: u8 = 2;
const TILE_BEDROOM: u8 = 3;
const TILE_STOCKPILE: u8 = 4;

const DAY_LENGTH: f64 = 120.0;
const NIGHT_START: f64 = 80.0;

const HUNGER_INTERVAL: f64 = 30.0;
const REST_INTERVAL: f64 = 60.0;
const FARM_PRODUCE_INTERVAL: f64 = 15.0;
const EVENT_INTERVAL: f64 = 60.0;

const COLONIST_SPEED: f64 = 4.0;

#[derive(Clone, Copy, PartialEq)]
enum ZoneType {
    Farm,
    Bedroom,
    Stockpile,
    Wall,
}

impl ZoneType {
    fn label(self) -> &'static str {
        match self {
            ZoneType::Farm => "Farm (F)",
            ZoneType::Bedroom => "Bedroom (B)",
            ZoneType::Stockpile => "Stockpile (S)",
            ZoneType::Wall => "Wall (W)",
        }
    }

    fn character(self) -> char {
        match self {
            ZoneType::Farm => 'F',
            ZoneType::Bedroom => 'B',
            ZoneType::Stockpile => 'S',
            ZoneType::Wall => 'W',
        }
    }

    fn tile_id(self) -> u8 {
        match self {
            ZoneType::Farm => TILE_FARM,
            ZoneType::Bedroom => TILE_BEDROOM,
            ZoneType::Stockpile => TILE_STOCKPILE,
            ZoneType::Wall => TILE_WALL,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ColonistTask {
    Idle,
    GoingToFood { target_column: i32, target_row: i32 },
    Eating,
    GoingToBed { target_column: i32, target_row: i32 },
    Sleeping,
    GoingToFarm { target_column: i32, target_row: i32 },
    Farming,
    GoingToBuild { target_column: i32, target_row: i32 },
    Building,
}

struct ColonistData {
    entity: Entity,
    grid_column: f64,
    grid_row: f64,
    hunger: f64,
    rest: f64,
    task: ColonistTask,
    path: Vec<(i32, i32)>,
    path_index: usize,
    move_progress: f64,
    alive: bool,
    work_timer: f64,
}

struct RaiderData {
    entity: Entity,
    grid_column: f64,
    grid_row: f64,
    health: i32,
    path: Vec<(i32, i32)>,
    path_index: usize,
    move_progress: f64,
    attack_timer: f64,
    alive: bool,
}

struct BuildJob {
    column: usize,
    row: usize,
    claimed: bool,
}

struct PathResult {
    steps: Vec<(i32, i32)>,
    destination: (i32, i32),
}

struct MessageLog {
    messages: Vec<(String, TermColor)>,
}

impl MessageLog {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    fn add(&mut self, text: String, color: TermColor) {
        self.messages.push((text, color));
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }

    fn recent(&self, count: usize) -> &[(String, TermColor)] {
        let start = self.messages.len().saturating_sub(count);
        &self.messages[start..]
    }
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Colony Sim - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "C O L O N Y";
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

        let art = [
            "    ╔══════════╗    ",
            "    ║ ☼ HOME ☼ ║    ",
            "    ╚══════════╝    ",
            "   c  c  c  c  c   ",
        ];
        for (line_index, line) in art.iter().enumerate() {
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - line.len() as f64 / 2.0,
                    row: center_row - 2.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: line.to_string(),
                    foreground: TermColor::DarkYellow,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }

        let subtitle = "A colony management simulation";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - subtitle.len() as f64 / 2.0,
                row: center_row + 3.0,
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

        let prompt = "Press ENTER to begin";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - prompt.len() as f64 / 2.0,
                row: center_row + 5.0,
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

        let controls = "1-4: zones | Click: place | Space: pause | +/-: speed";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - controls.len() as f64 / 2.0,
                row: center_row + 7.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: controls.to_string(),
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
    tilemap_entity: Entity,
    offset_column: i32,
    offset_row: i32,
    colonists: Vec<ColonistData>,
    raiders: Vec<RaiderData>,
    build_jobs: Vec<BuildJob>,
    particles: ParticleEmitter,
    food: i32,
    day_number: u32,
    day_time: f64,
    selected_zone: ZoneType,
    paused: bool,
    speed_multiplier: f64,
    farm_timer: Timer,
    event_timer: Timer,
    hud_entities: EntityGroup,
    message_entities: EntityGroup,
    cursor_entity: Entity,
    messages: MessageLog,
    game_over: bool,
    is_night: bool,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            map: [[TILE_GROUND; MAP_WIDTH]; MAP_HEIGHT],
            tilemap_entity: Entity::default(),
            offset_column: 0,
            offset_row: 0,
            colonists: Vec::new(),
            raiders: Vec::new(),
            build_jobs: Vec::new(),
            particles: ParticleEmitter::new(),
            food: 20,
            day_number: 1,
            day_time: 0.0,
            selected_zone: ZoneType::Farm,
            paused: false,
            speed_multiplier: 1.0,
            farm_timer: Timer::repeating(FARM_PRODUCE_INTERVAL),
            event_timer: Timer::repeating(EVENT_INTERVAL),
            hud_entities: EntityGroup::new(),
            message_entities: EntityGroup::new(),
            cursor_entity: Entity::default(),
            messages: MessageLog::new(),
            game_over: false,
            is_night: false,
        }
    }

    fn build_tilemap(&mut self, world: &mut World) {
        let terminal = world.resources.terminal_size;
        self.offset_column = ((terminal.columns as i32 - MAP_WIDTH as i32) / 2).max(0);
        self.offset_row = 2;

        self.refresh_tilemap(world);
    }

    fn refresh_tilemap(&self, world: &mut World) {
        let mut tilemap = Tilemap::new(MAP_WIDTH, MAP_HEIGHT);
        for row in 0..MAP_HEIGHT {
            for column in 0..MAP_WIDTH {
                let cell = self.tile_cell(column, row);
                tilemap.set(column, row, cell);
            }
        }

        if world.get_tilemap(self.tilemap_entity).is_some() {
            world.set_tilemap(self.tilemap_entity, tilemap);
        }
    }

    fn tile_cell(&self, column: usize, row: usize) -> TilemapCell {
        let tile = self.map[row][column];
        let night_dim = self.is_night;

        match tile {
            TILE_WALL => TilemapCell {
                character: 'W',
                foreground: if night_dim {
                    TermColor::DarkGrey
                } else {
                    TermColor::Grey
                },
                background: if night_dim {
                    TermColor::Rgb {
                        r: 20,
                        g: 20,
                        b: 25,
                    }
                } else {
                    TermColor::Rgb {
                        r: 60,
                        g: 55,
                        b: 50,
                    }
                },
            },
            TILE_FARM => TilemapCell {
                character: 'F',
                foreground: if night_dim {
                    TermColor::DarkGreen
                } else {
                    TermColor::Green
                },
                background: if night_dim {
                    TermColor::Rgb {
                        r: 10,
                        g: 20,
                        b: 10,
                    }
                } else {
                    TermColor::Rgb {
                        r: 30,
                        g: 50,
                        b: 20,
                    }
                },
            },
            TILE_BEDROOM => TilemapCell {
                character: 'B',
                foreground: if night_dim {
                    TermColor::DarkBlue
                } else {
                    TermColor::Blue
                },
                background: if night_dim {
                    TermColor::Rgb {
                        r: 10,
                        g: 10,
                        b: 30,
                    }
                } else {
                    TermColor::Rgb {
                        r: 25,
                        g: 25,
                        b: 60,
                    }
                },
            },
            TILE_STOCKPILE => TilemapCell {
                character: 'S',
                foreground: if night_dim {
                    TermColor::DarkYellow
                } else {
                    TermColor::Yellow
                },
                background: if night_dim {
                    TermColor::Rgb {
                        r: 25,
                        g: 20,
                        b: 10,
                    }
                } else {
                    TermColor::Rgb {
                        r: 50,
                        g: 40,
                        b: 15,
                    }
                },
            },
            _ => TilemapCell {
                character: '.',
                foreground: if night_dim {
                    TermColor::Rgb {
                        r: 30,
                        g: 35,
                        b: 25,
                    }
                } else {
                    TermColor::Rgb {
                        r: 70,
                        g: 80,
                        b: 50,
                    }
                },
                background: if night_dim {
                    TermColor::Rgb { r: 8, g: 12, b: 8 }
                } else {
                    TermColor::Rgb {
                        r: 20,
                        g: 35,
                        b: 15,
                    }
                },
            },
        }
    }

    fn spawn_colonists(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        for _ in 0..5 {
            let grid_column = rng.random_range(2..MAP_WIDTH as i32 - 2);
            let grid_row = rng.random_range(2..MAP_HEIGHT as i32 - 2);

            let entity = EntityBuilder::new()
                .position(Position {
                    column: self.offset_column as f64 + grid_column as f64,
                    row: self.offset_row as f64 + grid_row as f64,
                })
                .sprite(Sprite {
                    character: 'c',
                    foreground: TermColor::Cyan,
                    background: TermColor::Rgb {
                        r: 20,
                        g: 35,
                        b: 15,
                    },
                })
                .z_index(ZIndex(5))
                .spawn(world);

            self.colonists.push(ColonistData {
                entity,
                grid_column: grid_column as f64,
                grid_row: grid_row as f64,
                hunger: 0.0,
                rest: 0.0,
                task: ColonistTask::Idle,
                path: Vec::new(),
                path_index: 0,
                move_progress: 0.0,
                alive: true,
                work_timer: 0.0,
            });
        }
    }

    fn find_tile_positions(&self, tile_type: u8) -> Vec<(i32, i32)> {
        let mut positions = Vec::new();
        for row in 0..MAP_HEIGHT {
            for column in 0..MAP_WIDTH {
                if self.map[row][column] == tile_type {
                    positions.push((column as i32, row as i32));
                }
            }
        }
        positions
    }

    fn is_walkable(&self, column: i32, row: i32) -> bool {
        if column < 0 || column >= MAP_WIDTH as i32 || row < 0 || row >= MAP_HEIGHT as i32 {
            return false;
        }
        self.map[row as usize][column as usize] != TILE_WALL
    }

    fn find_path_to_nearest(
        &self,
        from_column: i32,
        from_row: i32,
        targets: &[(i32, i32)],
    ) -> Option<PathResult> {
        let mut best_path: Option<Vec<(i32, i32)>> = None;
        let mut best_target = (0, 0);

        for &(target_column, target_row) in targets {
            let map = &self.map;
            let path = astar(
                (from_column, from_row),
                (target_column, target_row),
                |column, row| {
                    if column < 0
                        || column >= MAP_WIDTH as i32
                        || row < 0
                        || row >= MAP_HEIGHT as i32
                    {
                        return false;
                    }
                    map[row as usize][column as usize] != TILE_WALL
                },
                false,
            );

            if let Some(path) = path {
                let is_shorter = best_path
                    .as_ref()
                    .is_none_or(|best| path.len() < best.len());
                if is_shorter {
                    best_path = Some(path);
                    best_target = (target_column, target_row);
                }
            }
        }

        best_path.map(|path| PathResult {
            steps: path,
            destination: best_target,
        })
    }

    fn assign_colonist_task(&mut self, colonist_index: usize) {
        let colonist = &self.colonists[colonist_index];
        let from_column = colonist.grid_column.round() as i32;
        let from_row = colonist.grid_row.round() as i32;

        if colonist.hunger >= HUNGER_INTERVAL && self.food > 0 {
            let stockpiles = self.find_tile_positions(TILE_STOCKPILE);
            if let Some(result) = self.find_path_to_nearest(from_column, from_row, &stockpiles) {
                let colonist = &mut self.colonists[colonist_index];
                colonist.task = ColonistTask::GoingToFood {
                    target_column: result.destination.0,
                    target_row: result.destination.1,
                };
                colonist.path = result.steps;
                colonist.path_index = 0;
                colonist.move_progress = 0.0;
                return;
            }
        }

        if colonist.rest >= REST_INTERVAL || self.is_night {
            let bedrooms = self.find_tile_positions(TILE_BEDROOM);
            if let Some(result) = self.find_path_to_nearest(from_column, from_row, &bedrooms) {
                let colonist = &mut self.colonists[colonist_index];
                colonist.task = ColonistTask::GoingToBed {
                    target_column: result.destination.0,
                    target_row: result.destination.1,
                };
                colonist.path = result.steps;
                colonist.path_index = 0;
                colonist.move_progress = 0.0;
                return;
            }
        }

        let unclaimed_build = self.build_jobs.iter().position(|job| !job.claimed);
        if let Some(job_index) = unclaimed_build {
            let job_column = self.build_jobs[job_index].column as i32;
            let job_row = self.build_jobs[job_index].row as i32;

            let adjacent_targets: Vec<(i32, i32)> = [
                (job_column - 1, job_row),
                (job_column + 1, job_row),
                (job_column, job_row - 1),
                (job_column, job_row + 1),
            ]
            .into_iter()
            .filter(|&(column, row)| self.is_walkable(column, row))
            .collect();

            if let Some(result) =
                self.find_path_to_nearest(from_column, from_row, &adjacent_targets)
            {
                self.build_jobs[job_index].claimed = true;
                let colonist = &mut self.colonists[colonist_index];
                colonist.task = ColonistTask::GoingToBuild {
                    target_column: job_column,
                    target_row: job_row,
                };
                colonist.path = result.steps;
                colonist.path_index = 0;
                colonist.move_progress = 0.0;
                return;
            }
        }

        let farms = self.find_tile_positions(TILE_FARM);
        if let Some(result) = self.find_path_to_nearest(from_column, from_row, &farms) {
            let colonist = &mut self.colonists[colonist_index];
            colonist.task = ColonistTask::GoingToFarm {
                target_column: result.destination.0,
                target_row: result.destination.1,
            };
            colonist.path = result.steps;
            colonist.path_index = 0;
            colonist.move_progress = 0.0;
        }
    }

    fn update_colonists(&mut self, world: &mut World, delta: f64) {
        let effective_delta = delta * self.speed_multiplier;

        for colonist_index in 0..self.colonists.len() {
            if !self.colonists[colonist_index].alive {
                continue;
            }

            self.colonists[colonist_index].hunger += effective_delta;
            self.colonists[colonist_index].rest += effective_delta;

            if self.colonists[colonist_index].hunger >= HUNGER_INTERVAL * 3.0 {
                self.colonists[colonist_index].alive = false;
                let entity = self.colonists[colonist_index].entity;
                let column = self.offset_column as f64 + self.colonists[colonist_index].grid_column;
                let row = self.offset_row as f64 + self.colonists[colonist_index].grid_row;
                self.particles.emit(
                    world,
                    column,
                    row,
                    5,
                    &ParticleConfig {
                        characters: vec!['x', '.', '+'],
                        colors: vec![TermColor::Red, TermColor::DarkRed],
                        lifetime: 0.8,
                        speed_min: 1.0,
                        speed_max: 3.0,
                        spread: std::f64::consts::PI * 2.0,
                        direction: 0.0,
                        z_index: 8,
                    },
                );
                world.despawn_entities(&[entity]);
                self.messages
                    .add("A colonist has starved!".to_string(), TermColor::Red);
                continue;
            }

            match self.colonists[colonist_index].task {
                ColonistTask::Idle => {
                    self.assign_colonist_task(colonist_index);
                }
                ColonistTask::GoingToFood {
                    target_column,
                    target_row,
                } => {
                    if self.move_colonist_along_path(colonist_index, effective_delta) {
                        let current_column =
                            self.colonists[colonist_index].grid_column.round() as i32;
                        let current_row = self.colonists[colonist_index].grid_row.round() as i32;
                        if current_column == target_column
                            && current_row == target_row
                            && self.food > 0
                        {
                            self.colonists[colonist_index].task = ColonistTask::Eating;
                            self.colonists[colonist_index].work_timer = 0.0;
                        } else {
                            self.colonists[colonist_index].task = ColonistTask::Idle;
                        }
                    }
                }
                ColonistTask::Eating => {
                    self.colonists[colonist_index].work_timer += effective_delta;
                    if self.colonists[colonist_index].work_timer >= 2.0 {
                        if self.food > 0 {
                            self.food -= 1;
                            self.colonists[colonist_index].hunger = 0.0;
                        }
                        self.colonists[colonist_index].task = ColonistTask::Idle;
                    }
                }
                ColonistTask::GoingToBed {
                    target_column,
                    target_row,
                } => {
                    if self.move_colonist_along_path(colonist_index, effective_delta) {
                        let current_column =
                            self.colonists[colonist_index].grid_column.round() as i32;
                        let current_row = self.colonists[colonist_index].grid_row.round() as i32;
                        if current_column == target_column && current_row == target_row {
                            self.colonists[colonist_index].task = ColonistTask::Sleeping;
                            self.colonists[colonist_index].work_timer = 0.0;
                        } else {
                            self.colonists[colonist_index].task = ColonistTask::Idle;
                        }
                    }
                }
                ColonistTask::Sleeping => {
                    self.colonists[colonist_index].work_timer += effective_delta;
                    if self.colonists[colonist_index].work_timer >= 5.0 {
                        self.colonists[colonist_index].rest = 0.0;
                        self.colonists[colonist_index].task = ColonistTask::Idle;
                    }
                }
                ColonistTask::GoingToFarm {
                    target_column,
                    target_row,
                } => {
                    if self.move_colonist_along_path(colonist_index, effective_delta) {
                        let current_column =
                            self.colonists[colonist_index].grid_column.round() as i32;
                        let current_row = self.colonists[colonist_index].grid_row.round() as i32;
                        if current_column == target_column && current_row == target_row {
                            self.colonists[colonist_index].task = ColonistTask::Farming;
                            self.colonists[colonist_index].work_timer = 0.0;
                        } else {
                            self.colonists[colonist_index].task = ColonistTask::Idle;
                        }
                    }
                }
                ColonistTask::Farming => {
                    self.colonists[colonist_index].work_timer += effective_delta;
                    if self.colonists[colonist_index].work_timer >= 8.0 {
                        self.colonists[colonist_index].task = ColonistTask::Idle;
                        self.colonists[colonist_index].work_timer = 0.0;
                    }

                    if self.colonists[colonist_index].hunger >= HUNGER_INTERVAL && self.food > 0 {
                        self.colonists[colonist_index].task = ColonistTask::Idle;
                    }
                    if self.colonists[colonist_index].rest >= REST_INTERVAL {
                        self.colonists[colonist_index].task = ColonistTask::Idle;
                    }
                }
                ColonistTask::GoingToBuild {
                    target_column,
                    target_row,
                } => {
                    if self.move_colonist_along_path(colonist_index, effective_delta) {
                        let current_column =
                            self.colonists[colonist_index].grid_column.round() as i32;
                        let current_row = self.colonists[colonist_index].grid_row.round() as i32;
                        let adjacent = (current_column - target_column).abs()
                            + (current_row - target_row).abs()
                            <= 1;
                        if adjacent {
                            self.colonists[colonist_index].task = ColonistTask::Building;
                            self.colonists[colonist_index].work_timer = 0.0;
                        } else {
                            self.colonists[colonist_index].task = ColonistTask::Idle;
                            if let Some(job) = self.build_jobs.iter_mut().find(|job| {
                                job.column == target_column as usize
                                    && job.row == target_row as usize
                            }) {
                                job.claimed = false;
                            }
                        }
                    }
                }
                ColonistTask::Building => {
                    self.colonists[colonist_index].work_timer += effective_delta;
                    if self.colonists[colonist_index].work_timer >= 5.0 {
                        self.colonists[colonist_index].task = ColonistTask::Idle;
                        self.colonists[colonist_index].work_timer = 0.0;
                    }

                    if self.colonists[colonist_index].hunger >= HUNGER_INTERVAL && self.food > 0 {
                        self.colonists[colonist_index].task = ColonistTask::Idle;
                    }
                }
            }

            let character = match self.colonists[colonist_index].task {
                ColonistTask::Sleeping => 'z',
                ColonistTask::Eating | ColonistTask::GoingToFood { .. } => {
                    if self.colonists[colonist_index].hunger >= HUNGER_INTERVAL {
                        '!'
                    } else {
                        'c'
                    }
                }
                _ => {
                    if self.colonists[colonist_index].hunger >= HUNGER_INTERVAL {
                        '!'
                    } else {
                        'c'
                    }
                }
            };

            let foreground = match self.colonists[colonist_index].task {
                ColonistTask::Sleeping => TermColor::Blue,
                _ => {
                    if self.colonists[colonist_index].hunger >= HUNGER_INTERVAL {
                        TermColor::Red
                    } else {
                        TermColor::Cyan
                    }
                }
            };

            let entity = self.colonists[colonist_index].entity;
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.character = character;
                sprite.foreground = foreground;
            }

            let screen_column =
                self.offset_column as f64 + self.colonists[colonist_index].grid_column;
            let screen_row = self.offset_row as f64 + self.colonists[colonist_index].grid_row;
            if let Some(position) = world.get_position_mut(entity) {
                position.column = screen_column;
                position.row = screen_row;
            }
        }
    }

    fn move_colonist_along_path(&mut self, colonist_index: usize, delta: f64) -> bool {
        let colonist = &mut self.colonists[colonist_index];
        if colonist.path.is_empty() || colonist.path_index >= colonist.path.len() - 1 {
            return true;
        }

        colonist.move_progress += COLONIST_SPEED * delta;

        while colonist.move_progress >= 1.0 && colonist.path_index < colonist.path.len() - 1 {
            colonist.move_progress -= 1.0;
            colonist.path_index += 1;
            let (target_column, target_row) = colonist.path[colonist.path_index];
            colonist.grid_column = target_column as f64;
            colonist.grid_row = target_row as f64;
        }

        if colonist.path_index >= colonist.path.len() - 1 {
            let (final_column, final_row) = colonist.path[colonist.path.len() - 1];
            colonist.grid_column = final_column as f64;
            colonist.grid_row = final_row as f64;
            return true;
        }

        let current = colonist.path[colonist.path_index];
        let next_index = (colonist.path_index + 1).min(colonist.path.len() - 1);
        let next = colonist.path[next_index];
        let progress = colonist.move_progress.min(1.0);
        colonist.grid_column = current.0 as f64 + (next.0 as f64 - current.0 as f64) * progress;
        colonist.grid_row = current.1 as f64 + (next.1 as f64 - current.1 as f64) * progress;

        false
    }

    fn update_farms(&mut self, delta: f64) {
        let effective_delta = delta * self.speed_multiplier;
        if self.farm_timer.tick(effective_delta) {
            let farm_count = self.find_tile_positions(TILE_FARM).len();
            let working_on_farms = self
                .colonists
                .iter()
                .filter(|colonist| colonist.alive && matches!(colonist.task, ColonistTask::Farming))
                .count();

            let production = (farm_count.min(working_on_farms)) as i32;
            if production > 0 {
                self.food += production;
                self.messages.add(
                    format!("Farms produced {} food", production),
                    TermColor::Green,
                );
            }
        }
    }

    fn update_day_cycle(&mut self, delta: f64) {
        let effective_delta = delta * self.speed_multiplier;
        self.day_time += effective_delta;

        let was_night = self.is_night;
        self.is_night = self.day_time >= NIGHT_START;

        if self.day_time >= DAY_LENGTH {
            self.day_time -= DAY_LENGTH;
            self.day_number += 1;
            self.is_night = false;
            self.messages
                .add(format!("Day {} begins", self.day_number), TermColor::Yellow);
        }

        if self.is_night && !was_night {
            self.messages
                .add("Night falls...".to_string(), TermColor::DarkBlue);
        }
        if !self.is_night && was_night {
            self.messages
                .add("The sun rises.".to_string(), TermColor::Yellow);
        }
    }

    fn update_events(&mut self, world: &mut World, delta: f64) {
        let effective_delta = delta * self.speed_multiplier;
        if !self.event_timer.tick(effective_delta) {
            return;
        }

        let mut rng = rand::rng();
        let event_type = rng.random_range(0..3);

        match event_type {
            0 => {
                self.spawn_raiders(world);
            }
            1 => {
                let bonus = rng.random_range(3..8);
                self.food += bonus;
                self.messages.add(
                    format!("Bountiful harvest! +{} food", bonus),
                    TermColor::Green,
                );
                self.particles.emit(
                    world,
                    self.offset_column as f64 + MAP_WIDTH as f64 / 2.0,
                    self.offset_row as f64 + MAP_HEIGHT as f64 / 2.0,
                    8,
                    &ParticleConfig {
                        characters: vec!['*', '.', '+'],
                        colors: vec![TermColor::Green, TermColor::Yellow],
                        lifetime: 1.0,
                        speed_min: 1.0,
                        speed_max: 4.0,
                        spread: std::f64::consts::PI * 2.0,
                        direction: 0.0,
                        z_index: 8,
                    },
                );
            }
            _ => {
                let bonus = rng.random_range(2..5);
                self.food += bonus;
                self.messages
                    .add(format!("Traders arrive! +{} food", bonus), TermColor::Cyan);
            }
        }
    }

    fn spawn_raiders(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        let raider_count = rng.random_range(1..4);

        self.messages.add(
            format!("Raiders attack! {} hostiles approaching!", raider_count),
            TermColor::Red,
        );

        for _ in 0..raider_count {
            let (start_column, start_row) = match rng.random_range(0..4) {
                0 => (0, rng.random_range(0..MAP_HEIGHT as i32)),
                1 => (MAP_WIDTH as i32 - 1, rng.random_range(0..MAP_HEIGHT as i32)),
                2 => (rng.random_range(0..MAP_WIDTH as i32), 0),
                _ => (rng.random_range(0..MAP_WIDTH as i32), MAP_HEIGHT as i32 - 1),
            };

            let entity = EntityBuilder::new()
                .position(Position {
                    column: self.offset_column as f64 + start_column as f64,
                    row: self.offset_row as f64 + start_row as f64,
                })
                .sprite(Sprite {
                    character: 'R',
                    foreground: TermColor::Red,
                    background: TermColor::Rgb {
                        r: 20,
                        g: 35,
                        b: 15,
                    },
                })
                .z_index(ZIndex(6))
                .spawn(world);

            self.raiders.push(RaiderData {
                entity,
                grid_column: start_column as f64,
                grid_row: start_row as f64,
                health: 3,
                path: Vec::new(),
                path_index: 0,
                move_progress: 0.0,
                attack_timer: 0.0,
                alive: true,
            });
        }
    }

    fn update_raiders(&mut self, world: &mut World, delta: f64) {
        let effective_delta = delta * self.speed_multiplier;
        let raider_speed = 2.5;

        let colonist_positions: Vec<(f64, f64)> = self
            .colonists
            .iter()
            .filter(|colonist| colonist.alive)
            .map(|colonist| (colonist.grid_column, colonist.grid_row))
            .collect();

        if colonist_positions.is_empty() {
            return;
        }

        let mut kills = Vec::new();

        for raider_index in 0..self.raiders.len() {
            if !self.raiders[raider_index].alive {
                continue;
            }

            let raider_column = self.raiders[raider_index].grid_column;
            let raider_row = self.raiders[raider_index].grid_row;

            let mut nearest_colonist_index = None;
            let mut nearest_distance = f64::MAX;

            for (colonist_index, colonist) in self.colonists.iter().enumerate() {
                if !colonist.alive {
                    continue;
                }
                let distance_column = colonist.grid_column - raider_column;
                let distance_row = colonist.grid_row - raider_row;
                let distance =
                    (distance_column * distance_column + distance_row * distance_row).sqrt();
                if distance < nearest_distance {
                    nearest_distance = distance;
                    nearest_colonist_index = Some(colonist_index);
                }
            }

            if nearest_distance < 1.5 {
                self.raiders[raider_index].attack_timer += effective_delta;
                if self.raiders[raider_index].attack_timer >= 3.0 {
                    self.raiders[raider_index].attack_timer = 0.0;
                    if let Some(target_index) = nearest_colonist_index {
                        kills.push(target_index);
                    }
                }
                continue;
            }

            if let Some(target_index) = nearest_colonist_index {
                let target_column = self.colonists[target_index].grid_column.round() as i32;
                let target_row = self.colonists[target_index].grid_row.round() as i32;
                let from_column = raider_column.round() as i32;
                let from_row = raider_row.round() as i32;

                if self.raiders[raider_index].path.is_empty()
                    || self.raiders[raider_index].path_index
                        >= self.raiders[raider_index].path.len() - 1
                {
                    let map = &self.map;
                    let path = astar(
                        (from_column, from_row),
                        (target_column, target_row),
                        |column, row| {
                            if column < 0
                                || column >= MAP_WIDTH as i32
                                || row < 0
                                || row >= MAP_HEIGHT as i32
                            {
                                return false;
                            }
                            map[row as usize][column as usize] != TILE_WALL
                        },
                        false,
                    );

                    if let Some(path) = path {
                        self.raiders[raider_index].path = path;
                        self.raiders[raider_index].path_index = 0;
                        self.raiders[raider_index].move_progress = 0.0;
                    }
                }

                let raider = &mut self.raiders[raider_index];
                if !raider.path.is_empty() && raider.path_index < raider.path.len() - 1 {
                    raider.move_progress += raider_speed * effective_delta;

                    while raider.move_progress >= 1.0 && raider.path_index < raider.path.len() - 1 {
                        raider.move_progress -= 1.0;
                        raider.path_index += 1;
                        let (next_column, next_row) = raider.path[raider.path_index];
                        raider.grid_column = next_column as f64;
                        raider.grid_row = next_row as f64;
                    }

                    if raider.path_index < raider.path.len() - 1 {
                        let current = raider.path[raider.path_index];
                        let next = raider.path[(raider.path_index + 1).min(raider.path.len() - 1)];
                        let progress = raider.move_progress.min(1.0);
                        raider.grid_column =
                            current.0 as f64 + (next.0 as f64 - current.0 as f64) * progress;
                        raider.grid_row =
                            current.1 as f64 + (next.1 as f64 - current.1 as f64) * progress;
                    }
                }
            }

            let raider = &self.raiders[raider_index];
            let screen_column = self.offset_column as f64 + raider.grid_column;
            let screen_row = self.offset_row as f64 + raider.grid_row;
            if let Some(position) = world.get_position_mut(raider.entity) {
                position.column = screen_column;
                position.row = screen_row;
            }
        }

        kills.sort_unstable();
        kills.dedup();
        for &colonist_index in kills.iter().rev() {
            if colonist_index < self.colonists.len() && self.colonists[colonist_index].alive {
                self.colonists[colonist_index].alive = false;
                let entity = self.colonists[colonist_index].entity;
                let column = self.offset_column as f64 + self.colonists[colonist_index].grid_column;
                let row = self.offset_row as f64 + self.colonists[colonist_index].grid_row;
                self.particles.emit(
                    world,
                    column,
                    row,
                    6,
                    &ParticleConfig {
                        characters: vec!['*', 'x', '.'],
                        colors: vec![TermColor::Red, TermColor::DarkRed, TermColor::Yellow],
                        lifetime: 0.6,
                        speed_min: 1.0,
                        speed_max: 4.0,
                        spread: std::f64::consts::PI * 2.0,
                        direction: 0.0,
                        z_index: 8,
                    },
                );
                world.despawn_entities(&[entity]);
                self.messages.add(
                    "A colonist was killed by raiders!".to_string(),
                    TermColor::Red,
                );
            }
        }

        self.resolve_raider_colonist_combat(world);
    }

    fn resolve_raider_colonist_combat(&mut self, world: &mut World) {
        let mut raider_kills = Vec::new();

        for raider_index in 0..self.raiders.len() {
            if !self.raiders[raider_index].alive {
                continue;
            }

            let raider_column = self.raiders[raider_index].grid_column.round() as i32;
            let raider_row = self.raiders[raider_index].grid_row.round() as i32;

            let nearby_colonist_count = self
                .colonists
                .iter()
                .filter(|colonist| {
                    if !colonist.alive {
                        return false;
                    }
                    let distance_column =
                        (colonist.grid_column.round() as i32 - raider_column).abs();
                    let distance_row = (colonist.grid_row.round() as i32 - raider_row).abs();
                    distance_column <= 2 && distance_row <= 2
                })
                .count();

            if nearby_colonist_count >= 2 {
                self.raiders[raider_index].health -= 1;
                if self.raiders[raider_index].health <= 0 {
                    raider_kills.push(raider_index);
                }
            }
        }

        for &raider_index in raider_kills.iter().rev() {
            let raider = &mut self.raiders[raider_index];
            raider.alive = false;
            let entity = raider.entity;
            let column = self.offset_column as f64 + raider.grid_column;
            let row = self.offset_row as f64 + raider.grid_row;
            self.particles.emit(
                world,
                column,
                row,
                4,
                &ParticleConfig {
                    characters: vec!['*', '+'],
                    colors: vec![TermColor::Red, TermColor::Yellow],
                    lifetime: 0.5,
                    speed_min: 1.0,
                    speed_max: 3.0,
                    spread: std::f64::consts::PI * 2.0,
                    direction: 0.0,
                    z_index: 8,
                },
            );
            world.despawn_entities(&[entity]);
            self.messages
                .add("Colonists defeated a raider!".to_string(), TermColor::Green);
        }
    }

    fn update_build_jobs(&mut self, world: &mut World) {
        let mut completed = Vec::new();

        for colonist in &self.colonists {
            if !colonist.alive {
                continue;
            }
            if !matches!(colonist.task, ColonistTask::Building) {
                continue;
            }
            if colonist.work_timer < 5.0 {
                continue;
            }

            let colonist_column = colonist.grid_column.round() as i32;
            let colonist_row = colonist.grid_row.round() as i32;

            for (job_index, job) in self.build_jobs.iter().enumerate() {
                let distance_column = (job.column as i32 - colonist_column).abs();
                let distance_row = (job.row as i32 - colonist_row).abs();
                if distance_column + distance_row <= 1 && !completed.contains(&job_index) {
                    completed.push(job_index);
                }
            }
        }

        completed.sort_unstable();
        for &job_index in completed.iter().rev() {
            if job_index < self.build_jobs.len() {
                let job = self.build_jobs.remove(job_index);
                self.map[job.row][job.column] = TILE_WALL;
                self.messages.add(
                    format!("Wall built at ({}, {})", job.column, job.row),
                    TermColor::Grey,
                );
            }
        }

        if !completed.is_empty() {
            self.refresh_tilemap(world);
        }
    }

    fn check_game_over(&mut self) {
        let alive_count = self
            .colonists
            .iter()
            .filter(|colonist| colonist.alive)
            .count();
        if alive_count == 0 {
            self.game_over = true;
            self.messages
                .add("All colonists have perished...".to_string(), TermColor::Red);
        }
    }

    fn update_hud(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);

        let alive_count = self
            .colonists
            .iter()
            .filter(|colonist| colonist.alive)
            .count();
        let time_of_day = if self.is_night { "Night" } else { "Day" };
        let speed_text = match self.speed_multiplier as i32 {
            1 => "1x",
            2 => "2x",
            3 => "3x",
            _ => "1x",
        };
        let pause_text = if self.paused { " [PAUSED]" } else { "" };

        let hud_line = format!(
            "Food: {}  Colonists: {}  Day: {}  Time: {} ({:.0}s)  Speed: {}{}  Zone: [{}]",
            self.food,
            alive_count,
            self.day_number,
            time_of_day,
            self.day_time,
            speed_text,
            pause_text,
            self.selected_zone.label(),
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
        world.set_z_index(entity, ZIndex(15));

        let working_count = self
            .colonists
            .iter()
            .filter(|colonist| {
                colonist.alive
                    && matches!(
                        colonist.task,
                        ColonistTask::Farming | ColonistTask::Building
                    )
            })
            .count();
        let sleeping_count = self
            .colonists
            .iter()
            .filter(|colonist| colonist.alive && matches!(colonist.task, ColonistTask::Sleeping))
            .count();
        let hungry_count = self
            .colonists
            .iter()
            .filter(|colonist| colonist.alive && colonist.hunger >= HUNGER_INTERVAL)
            .count();

        let status_line = format!(
            "Working: {}  Sleeping: {}  Hungry: {}  Raiders: {}  1:Farm 2:Bed 3:Stock 4:Wall  Click:Place  +/-:Speed  Space:Pause",
            working_count,
            sleeping_count,
            hungry_count,
            self.raiders.iter().filter(|raider| raider.alive).count(),
        );

        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 0.0,
                row: 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: status_line,
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(15));
    }

    fn update_messages_display(&mut self, world: &mut World) {
        self.message_entities.despawn_all(world);

        let terminal = world.resources.terminal_size;
        let message_row_start = (self.offset_row + MAP_HEIGHT as i32 + 1) as f64;
        let available_lines = (terminal.rows as f64 - message_row_start).max(1.0) as usize;
        let recent = self.messages.recent(available_lines.min(4));

        for (line_index, (text, color)) in recent.iter().enumerate() {
            let entity = self
                .message_entities
                .spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: self.offset_column as f64,
                    row: message_row_start + line_index as f64,
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
            world.set_z_index(entity, ZIndex(15));
        }
    }

    fn place_zone(&mut self, grid_column: usize, grid_row: usize, world: &mut World) {
        if grid_column >= MAP_WIDTH || grid_row >= MAP_HEIGHT {
            return;
        }

        if self.map[grid_row][grid_column] != TILE_GROUND {
            return;
        }

        match self.selected_zone {
            ZoneType::Wall => {
                let already_queued = self
                    .build_jobs
                    .iter()
                    .any(|job| job.column == grid_column && job.row == grid_row);
                if !already_queued {
                    self.build_jobs.push(BuildJob {
                        column: grid_column,
                        row: grid_row,
                        claimed: false,
                    });
                    self.messages.add(
                        format!(
                            "Wall construction queued at ({}, {})",
                            grid_column, grid_row
                        ),
                        TermColor::Grey,
                    );

                    if let Some(tilemap) = world.get_tilemap_mut(self.tilemap_entity) {
                        tilemap.set(
                            grid_column,
                            grid_row,
                            TilemapCell {
                                character: 'w',
                                foreground: TermColor::DarkGrey,
                                background: if self.is_night {
                                    TermColor::Rgb {
                                        r: 15,
                                        g: 15,
                                        b: 20,
                                    }
                                } else {
                                    TermColor::Rgb {
                                        r: 40,
                                        g: 35,
                                        b: 30,
                                    }
                                },
                            },
                        );
                    }
                }
            }
            other => {
                self.map[grid_row][grid_column] = other.tile_id();
                self.messages.add(
                    format!(
                        "{} placed at ({}, {})",
                        other.label(),
                        grid_column,
                        grid_row
                    ),
                    TermColor::White,
                );
                self.refresh_tilemap(world);
            }
        }
    }

    fn cleanup_dead(&mut self, world: &mut World) {
        let mut raider_removals = Vec::new();
        for (raider_index, raider) in self.raiders.iter().enumerate() {
            if !raider.alive {
                raider_removals.push(raider_index);
            }
        }
        for &raider_index in raider_removals.iter().rev() {
            self.raiders.swap_remove(raider_index);
        }

        let mut colonist_removals = Vec::new();
        for (colonist_index, colonist) in self.colonists.iter().enumerate() {
            if !colonist.alive {
                colonist_removals.push(colonist_index);
            }
        }
        for &colonist_index in colonist_removals.iter().rev() {
            self.colonists.swap_remove(colonist_index);
        }

        let _ = world;
    }

    fn clear_all(&mut self, world: &mut World) {
        for colonist in &self.colonists {
            if colonist.alive {
                world.despawn_entities(&[colonist.entity]);
            }
        }
        self.colonists.clear();

        for raider in &self.raiders {
            if raider.alive {
                world.despawn_entities(&[raider.entity]);
            }
        }
        self.raiders.clear();

        self.particles.despawn_all(world);
        self.hud_entities.despawn_all(world);
        self.message_entities.despawn_all(world);
        world.despawn_entities(&[self.tilemap_entity, self.cursor_entity]);
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Colony Sim - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        self.tilemap_entity = EntityBuilder::new()
            .position(Position {
                column: self.offset_column as f64,
                row: self.offset_row as f64,
            })
            .tilemap(Tilemap::new(MAP_WIDTH, MAP_HEIGHT))
            .z_index(ZIndex(0))
            .spawn(world);

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

        self.spawn_colonists(world);

        self.messages
            .add("Welcome to your new colony!".to_string(), TermColor::Yellow);
        self.messages.add(
            "Place zones and manage your colonists.".to_string(),
            TermColor::Grey,
        );

        self.update_hud(world);
        self.update_messages_display(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Char('1') => {
                self.selected_zone = ZoneType::Farm;
            }
            KeyCode::Char('2') => {
                self.selected_zone = ZoneType::Bedroom;
            }
            KeyCode::Char('3') => {
                self.selected_zone = ZoneType::Stockpile;
            }
            KeyCode::Char('4') => {
                self.selected_zone = ZoneType::Wall;
            }
            KeyCode::Char(' ') => {
                self.paused = !self.paused;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.speed_multiplier = (self.speed_multiplier + 1.0).min(3.0);
            }
            KeyCode::Char('-') => {
                self.speed_multiplier = (self.speed_multiplier - 1.0).max(1.0);
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
            self.place_zone(grid_column as usize, grid_row as usize, world);
        }
    }

    fn on_mouse_move(&mut self, world: &mut World, column: u16, row: u16) {
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
            let can_place = self.map[grid_row as usize][grid_column as usize] == TILE_GROUND;
            if let Some(sprite) = world.get_sprite_mut(self.cursor_entity) {
                sprite.character = self.selected_zone.character();
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

        if self.paused {
            self.update_hud(world);
            self.update_messages_display(world);
            return;
        }

        let delta = world.resources.timing.delta_seconds;

        self.update_day_cycle(delta);
        self.update_colonists(world, delta);
        self.update_farms(delta);
        self.update_raiders(world, delta);
        self.update_build_jobs(world);
        self.update_events(world, delta);
        self.particles.update(world, delta);
        self.cleanup_dead(world);
        self.check_game_over();
        self.refresh_tilemap(world);
        self.update_hud(world);
        self.update_messages_display(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            self.clear_all(world);
            return Some(Box::new(GameOverState {
                day_reached: self.day_number,
                food_remaining: self.food,
                entities: EntityGroup::new(),
                restart: false,
            }));
        }
        None
    }
}

struct GameOverState {
    day_reached: u32,
    food_remaining: i32,
    entities: EntityGroup,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Colony Sim - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let day_text = format!("Days survived: {}", self.day_reached);
        let food_text = format!("Food remaining: {}", self.food_remaining);
        let lines: Vec<(String, TermColor)> = vec![
            ("COLONY LOST".to_string(), TermColor::Red),
            (String::new(), TermColor::Black),
            (
                "All colonists have perished.".to_string(),
                TermColor::DarkYellow,
            ),
            (String::new(), TermColor::Black),
            (day_text, TermColor::Yellow),
            (food_text, TermColor::Yellow),
            (String::new(), TermColor::Black),
            ("Press R to restart".to_string(), TermColor::White),
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
                    row: center_row - 5.0 + line_index as f64,
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
