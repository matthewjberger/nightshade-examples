use nightshade::tui::prelude::*;
use rand::Rng;
use std::collections::HashMap;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 50;
const FOV_RADIUS: i32 = 8;
const MIN_ROOM_SIZE: i32 = 4;
const MAX_ROOM_SIZE: i32 = 12;
const BSP_MIN_LEAF: i32 = 8;
const HUD_HEIGHT: i32 = 1;
const LOG_HEIGHT: i32 = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
    Wall,
    Floor,
    Stairs,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnemyKind {
    Rat,
    Goblin,
    Orc,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ItemKind {
    HealthPotion,
    Sword,
    Shield,
    Gold,
}

#[derive(Clone, Copy)]
struct ActorStats {
    hp: i32,
    max_hp: i32,
    attack: i32,
    defense: i32,
}

#[derive(Clone, Copy)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x1: x,
            y1: y,
            x2: x + width,
            y2: y + height,
        }
    }

    fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}

struct BspLeaf {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    room: Option<Rect>,
    left: Option<Box<BspLeaf>>,
    right: Option<Box<BspLeaf>>,
}

impl BspLeaf {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            room: None,
            left: None,
            right: None,
        }
    }

    fn split(&mut self, rng: &mut impl Rng) -> bool {
        if self.left.is_some() || self.right.is_some() {
            return false;
        }

        let split_horizontal =
            if self.width > self.height && (self.width as f32 / self.height as f32) >= 1.25 {
                false
            } else if self.height > self.width && (self.height as f32 / self.width as f32) >= 1.25 {
                true
            } else {
                rng.random_bool(0.5)
            };

        let max = if split_horizontal {
            self.height - BSP_MIN_LEAF
        } else {
            self.width - BSP_MIN_LEAF
        };

        if max < BSP_MIN_LEAF {
            return false;
        }

        let split_pos = rng.random_range(BSP_MIN_LEAF..=max);

        if split_horizontal {
            self.left = Some(Box::new(BspLeaf::new(
                self.x, self.y, self.width, split_pos,
            )));
            self.right = Some(Box::new(BspLeaf::new(
                self.x,
                self.y + split_pos,
                self.width,
                self.height - split_pos,
            )));
        } else {
            self.left = Some(Box::new(BspLeaf::new(
                self.x,
                self.y,
                split_pos,
                self.height,
            )));
            self.right = Some(Box::new(BspLeaf::new(
                self.x + split_pos,
                self.y,
                self.width - split_pos,
                self.height,
            )));
        }

        true
    }

    fn generate(&mut self, rng: &mut impl Rng) {
        if self.split(rng) {
            if let Some(left) = self.left.as_mut() {
                left.generate(rng);
            }
            if let Some(right) = self.right.as_mut() {
                right.generate(rng);
            }
        } else {
            let room_width = rng.random_range(MIN_ROOM_SIZE..=(self.width - 2).min(MAX_ROOM_SIZE));
            let room_height =
                rng.random_range(MIN_ROOM_SIZE..=(self.height - 2).min(MAX_ROOM_SIZE));
            let room_x = self.x + rng.random_range(1..=(self.width - room_width - 1).max(1));
            let room_y = self.y + rng.random_range(1..=(self.height - room_height - 1).max(1));
            self.room = Some(Rect::new(room_x, room_y, room_width, room_height));
        }
    }

    fn get_rooms(&self) -> Vec<Rect> {
        let mut rooms = Vec::new();
        if let Some(room) = self.room {
            rooms.push(room);
        }
        if let Some(left) = &self.left {
            rooms.extend(left.get_rooms());
        }
        if let Some(right) = &self.right {
            rooms.extend(right.get_rooms());
        }
        rooms
    }

    fn get_room_center(&self) -> Option<(i32, i32)> {
        if let Some(room) = self.room {
            return Some(room.center());
        }
        if let Some(left) = &self.left
            && let Some(center) = left.get_room_center()
        {
            return Some(center);
        }
        if let Some(right) = &self.right
            && let Some(center) = right.get_room_center()
        {
            return Some(center);
        }
        None
    }

    fn create_corridors(&self, tiles: &mut [Tile]) {
        if let (Some(left), Some(right)) = (&self.left, &self.right) {
            left.create_corridors(tiles);
            right.create_corridors(tiles);

            if let (Some(left_center), Some(right_center)) =
                (left.get_room_center(), right.get_room_center())
            {
                carve_corridor(
                    tiles,
                    left_center.0,
                    left_center.1,
                    right_center.0,
                    right_center.1,
                );
            }
        }
    }
}

fn carve_corridor(tiles: &mut [Tile], x1: i32, y1: i32, x2: i32, y2: i32) {
    let mut current_x = x1;
    let mut current_y = y1;

    while current_x != x2 {
        let index = (current_y * MAP_WIDTH + current_x) as usize;
        if index < tiles.len() && tiles[index] == Tile::Wall {
            tiles[index] = Tile::Floor;
        }
        current_x += if current_x < x2 { 1 } else { -1 };
    }

    while current_y != y2 {
        let index = (current_y * MAP_WIDTH + current_x) as usize;
        if index < tiles.len() && tiles[index] == Tile::Wall {
            tiles[index] = Tile::Floor;
        }
        current_y += if current_y < y2 { 1 } else { -1 };
    }

    let index = (current_y * MAP_WIDTH + current_x) as usize;
    if index < tiles.len() && tiles[index] == Tile::Wall {
        tiles[index] = Tile::Floor;
    }
}

fn generate_dungeon() -> (Vec<Tile>, Vec<Rect>, (i32, i32)) {
    let mut rng = rand::rng();
    let mut tiles = vec![Tile::Wall; (MAP_WIDTH * MAP_HEIGHT) as usize];

    let mut root = BspLeaf::new(0, 0, MAP_WIDTH, MAP_HEIGHT);
    root.generate(&mut rng);

    let rooms = root.get_rooms();
    for room in &rooms {
        for y in room.y1..room.y2 {
            for x in room.x1..room.x2 {
                if x > 0 && x < MAP_WIDTH - 1 && y > 0 && y < MAP_HEIGHT - 1 {
                    tiles[(y * MAP_WIDTH + x) as usize] = Tile::Floor;
                }
            }
        }
    }

    root.create_corridors(&mut tiles);

    let player_start = if let Some(first_room) = rooms.first() {
        first_room.center()
    } else {
        (MAP_WIDTH / 2, MAP_HEIGHT / 2)
    };

    if let Some(last_room) = rooms.last() {
        let (stair_x, stair_y) = last_room.center();
        tiles[(stair_y * MAP_WIDTH + stair_x) as usize] = Tile::Stairs;
    }

    (tiles, rooms, player_start)
}

fn compute_fov(
    player_x: i32,
    player_y: i32,
    tiles: &[Tile],
    visible: &mut Vec<bool>,
    explored: &mut Vec<bool>,
) {
    for value in visible.iter_mut() {
        *value = false;
    }

    visible[(player_y * MAP_WIDTH + player_x) as usize] = true;
    explored[(player_y * MAP_WIDTH + player_x) as usize] = true;

    let mut context = FovContext {
        origin_x: player_x,
        origin_y: player_y,
        tiles,
        visible,
        explored,
    };

    for octant in 0..8 {
        cast_light(&mut context, 1, 1.0, 0.0, octant);
    }
}

fn transform_octant(octant: u8, row: i32, col: i32) -> (i32, i32) {
    match octant {
        0 => (col, -row),
        1 => (row, -col),
        2 => (row, col),
        3 => (col, row),
        4 => (-col, row),
        5 => (-row, col),
        6 => (-row, -col),
        7 => (-col, -row),
        _ => (col, -row),
    }
}

struct FovContext<'a> {
    origin_x: i32,
    origin_y: i32,
    tiles: &'a [Tile],
    visible: &'a mut Vec<bool>,
    explored: &'a mut Vec<bool>,
}

fn cast_light(
    context: &mut FovContext<'_>,
    row: i32,
    mut start_slope: f64,
    end_slope: f64,
    octant: u8,
) {
    if start_slope < end_slope || row > FOV_RADIUS {
        return;
    }

    let mut blocked = false;
    let mut next_start_slope = start_slope;

    for current_row in row..=FOV_RADIUS {
        if blocked {
            break;
        }

        let delta_y = -current_row;
        for delta_x in -current_row..=0 {
            let left_slope = (delta_x as f64 - 0.5) / (delta_y as f64 + 0.5);
            let right_slope = (delta_x as f64 + 0.5) / (delta_y as f64 - 0.5);

            if start_slope < right_slope {
                continue;
            }
            if end_slope > left_slope {
                break;
            }

            let (map_offset_x, map_offset_y) = transform_octant(octant, current_row, delta_x);
            let map_x = context.origin_x + map_offset_x;
            let map_y = context.origin_y + map_offset_y;

            if !(0..MAP_WIDTH).contains(&map_x) || !(0..MAP_HEIGHT).contains(&map_y) {
                continue;
            }

            let distance_squared = map_offset_x * map_offset_x + map_offset_y * map_offset_y;
            let index = (map_y * MAP_WIDTH + map_x) as usize;

            if distance_squared <= FOV_RADIUS * FOV_RADIUS {
                context.visible[index] = true;
                context.explored[index] = true;
            }

            let is_wall = context.tiles[index] == Tile::Wall;

            if blocked {
                if is_wall {
                    next_start_slope = right_slope;
                } else {
                    blocked = false;
                    start_slope = next_start_slope;
                }
            } else if is_wall && current_row < FOV_RADIUS {
                blocked = true;
                cast_light(context, current_row + 1, start_slope, left_slope, octant);
                next_start_slope = right_slope;
            }
        }
    }
}

struct DamageEvent {
    attacker_name: String,
    defender_name: String,
    damage: i32,
    defender_killed: bool,
}

struct PickupEvent {
    item_name: String,
    item_kind: ItemKind,
}

struct TitleScreenState {
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Roguelike"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let title_lines = [
            r"  ____                        _ _ _        ",
            r" |  _ \ ___   __ _ _   _  ___| (_) | _____ ",
            r" | |_) / _ \ / _` | | | |/ _ \ | | |/ / _ \",
            r" |  _ < (_) | (_| | |_| |  __/ | |   <  __/",
            r" |_| \_\___/ \__, |\__,_|\___|_|_|_|\_\___|",
            r"             |___/                         ",
        ];

        let subtitle = "A Dungeon Crawler";
        let prompt = "Press ENTER to begin";
        let quit_hint = "Press ESC to quit";

        let title_start_row = center_row - 6;

        for (line_index, line) in title_lines.iter().enumerate() {
            let start_col = center_column - line.len() as i32 / 2;
            for (char_index, character) in line.chars().enumerate() {
                if character != ' ' {
                    let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                    world.set_position(
                        entity,
                        Position {
                            column: (start_col + char_index as i32) as f64,
                            row: (title_start_row + line_index as i32) as f64,
                        },
                    );
                    world.set_sprite(
                        entity,
                        Sprite {
                            character,
                            foreground: TermColor::Red,
                            background: TermColor::Black,
                        },
                    );
                    world.set_z_index(entity, ZIndex(10));
                }
            }
        }

        let subtitle_start = center_column - subtitle.len() as i32 / 2;
        for (char_index, character) in subtitle.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (subtitle_start + char_index as i32) as f64,
                    row: (title_start_row + 7) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: TermColor::Yellow,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }

        let prompt_start = center_column - prompt.len() as i32 / 2;
        for (char_index, character) in prompt.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (prompt_start + char_index as i32) as f64,
                    row: (title_start_row + 10) as f64,
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
        }

        let quit_start = center_column - quit_hint.len() as i32 / 2;
        for (char_index, character) in quit_hint.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (quit_start + char_index as i32) as f64,
                    row: (title_start_row + 12) as f64,
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
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Enter => {
                self.start_game = true;
            }
            KeyCode::Escape | KeyCode::Char('q') => {
                world.resources.should_exit = true;
            }
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.start_game {
            let all_entities: Vec<Entity> = world.query_entities(POSITION | SPRITE).collect();
            world.despawn_entities(&all_entities);
            return Some(Box::new(GameplayState::new()));
        }
        None
    }
}

struct GameplayState {
    tiles: Vec<Tile>,
    visible: Vec<bool>,
    explored: Vec<bool>,
    tile_entities: Vec<Entity>,
    player_entity: Entity,
    player_x: i32,
    player_y: i32,
    player_stats: ActorStats,
    player_base_attack: i32,
    player_base_defense: i32,
    attack_bonus: i32,
    defense_bonus: i32,
    enemy_entities: Vec<Entity>,
    enemy_positions: HashMap<Entity, (i32, i32)>,
    enemy_stats: HashMap<Entity, ActorStats>,
    enemy_kinds: HashMap<Entity, EnemyKind>,
    item_entities: Vec<Entity>,
    item_positions: HashMap<Entity, (i32, i32)>,
    item_kinds: HashMap<Entity, ItemKind>,
    hud_entities: Vec<Entity>,
    log_entities: Vec<Entity>,
    messages: Vec<String>,
    gold: u32,
    depth: u32,
    turn_taken: bool,
    game_over: bool,
    rooms: Vec<Rect>,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            tiles: Vec::new(),
            visible: Vec::new(),
            explored: Vec::new(),
            tile_entities: Vec::new(),
            player_entity: Entity::default(),
            player_x: 0,
            player_y: 0,
            player_stats: ActorStats {
                hp: 30,
                max_hp: 30,
                attack: 3,
                defense: 1,
            },
            player_base_attack: 3,
            player_base_defense: 1,
            attack_bonus: 0,
            defense_bonus: 0,
            enemy_entities: Vec::new(),
            enemy_positions: HashMap::new(),
            enemy_stats: HashMap::new(),
            enemy_kinds: HashMap::new(),
            item_entities: Vec::new(),
            item_positions: HashMap::new(),
            item_kinds: HashMap::new(),
            hud_entities: Vec::new(),
            log_entities: Vec::new(),
            messages: Vec::new(),
            gold: 0,
            depth: 1,
            turn_taken: false,
            game_over: false,
            rooms: Vec::new(),
        }
    }

    fn generate_level(&mut self, world: &mut World) {
        self.clear_entities(world);

        let (tiles, rooms, player_start) = generate_dungeon();
        self.tiles = tiles;
        self.rooms = rooms;
        self.visible = vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize];
        self.explored = vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize];
        self.player_x = player_start.0;
        self.player_y = player_start.1;

        self.spawn_tile_entities(world);
        self.spawn_player(world);
        self.spawn_enemies(world);
        self.spawn_items(world);
        self.update_camera(world);
        compute_fov(
            self.player_x,
            self.player_y,
            &self.tiles,
            &mut self.visible,
            &mut self.explored,
        );
        self.update_tile_visibility(world);
        self.update_entity_visibility(world);
        self.update_hud(world);
        self.update_log(world);

        self.add_message(format!("You descend to depth {}.", self.depth));
    }

    fn clear_entities(&mut self, world: &mut World) {
        if !self.tile_entities.is_empty() {
            world.despawn_entities(&self.tile_entities);
            self.tile_entities.clear();
        }
        if self.player_entity != Entity::default() {
            world.despawn_entities(&[self.player_entity]);
            self.player_entity = Entity::default();
        }
        if !self.enemy_entities.is_empty() {
            world.despawn_entities(&self.enemy_entities);
            self.enemy_entities.clear();
        }
        self.enemy_positions.clear();
        self.enemy_stats.clear();
        self.enemy_kinds.clear();
        if !self.item_entities.is_empty() {
            world.despawn_entities(&self.item_entities);
            self.item_entities.clear();
        }
        self.item_positions.clear();
        self.item_kinds.clear();
        if !self.hud_entities.is_empty() {
            world.despawn_entities(&self.hud_entities);
            self.hud_entities.clear();
        }
        if !self.log_entities.is_empty() {
            world.despawn_entities(&self.log_entities);
            self.log_entities.clear();
        }
    }

    fn spawn_tile_entities(&mut self, world: &mut World) {
        let entities = world.spawn_entities(
            POSITION | SPRITE | VISIBILITY | Z_INDEX,
            (MAP_WIDTH * MAP_HEIGHT) as usize,
        );
        for (index, &entity) in entities.iter().enumerate() {
            let x = (index as i32) % MAP_WIDTH;
            let y = (index as i32) / MAP_WIDTH;
            let tile = self.tiles[index];

            world.set_position(
                entity,
                Position {
                    column: x as f64,
                    row: y as f64,
                },
            );
            world.set_z_index(entity, ZIndex(0));

            let sprite = match tile {
                Tile::Wall => Sprite {
                    character: '#',
                    foreground: TermColor::DarkGrey,
                    background: TermColor::Black,
                },
                Tile::Floor => Sprite {
                    character: '.',
                    foreground: TermColor::Grey,
                    background: TermColor::Black,
                },
                Tile::Stairs => Sprite {
                    character: '>',
                    foreground: TermColor::Cyan,
                    background: TermColor::Black,
                },
            };
            world.set_sprite(entity, sprite);
            world.set_visibility(entity, Visibility { visible: false });
        }
        self.tile_entities = entities;
    }

    fn spawn_player(&mut self, world: &mut World) {
        self.player_entity =
            world.spawn_entities(POSITION | SPRITE | VISIBILITY | Z_INDEX | NAME, 1)[0];
        world.set_position(
            self.player_entity,
            Position {
                column: self.player_x as f64,
                row: self.player_y as f64,
            },
        );
        world.set_sprite(
            self.player_entity,
            Sprite {
                character: '@',
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_visibility(self.player_entity, Visibility { visible: true });
        world.set_z_index(self.player_entity, ZIndex(2));
        world.set_name(self.player_entity, Name("Player".to_string()));
    }

    fn spawn_enemies(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        let enemy_count = 3 + self.depth * 2;

        for _ in 0..enemy_count {
            if self.rooms.len() < 2 {
                break;
            }
            let room_index = rng.random_range(1..self.rooms.len());
            let room = self.rooms[room_index];
            let x = rng.random_range(room.x1 + 1..room.x2 - 1);
            let y = rng.random_range(room.y1 + 1..room.y2 - 1);

            if x == self.player_x && y == self.player_y {
                continue;
            }

            if self
                .enemy_positions
                .values()
                .any(|&(ex, ey)| ex == x && ey == y)
            {
                continue;
            }

            let kind_roll: f64 = rng.random();
            let kind = if self.depth >= 3 && kind_roll < 0.2 {
                EnemyKind::Orc
            } else if self.depth >= 2 && kind_roll < 0.5 {
                EnemyKind::Goblin
            } else {
                EnemyKind::Rat
            };

            let (character, foreground, name_str, stats) = match kind {
                EnemyKind::Rat => (
                    'r',
                    TermColor::DarkYellow,
                    "Rat",
                    ActorStats {
                        hp: 3 + self.depth as i32,
                        max_hp: 3 + self.depth as i32,
                        attack: 1,
                        defense: 0,
                    },
                ),
                EnemyKind::Goblin => (
                    'g',
                    TermColor::Green,
                    "Goblin",
                    ActorStats {
                        hp: 8 + self.depth as i32,
                        max_hp: 8 + self.depth as i32,
                        attack: 3,
                        defense: 1,
                    },
                ),
                EnemyKind::Orc => (
                    'O',
                    TermColor::Red,
                    "Orc",
                    ActorStats {
                        hp: 15 + self.depth as i32 * 2,
                        max_hp: 15 + self.depth as i32 * 2,
                        attack: 5,
                        defense: 2,
                    },
                ),
            };

            let entity =
                world.spawn_entities(POSITION | SPRITE | VISIBILITY | Z_INDEX | NAME, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: x as f64,
                    row: y as f64,
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
            world.set_visibility(entity, Visibility { visible: false });
            world.set_z_index(entity, ZIndex(2));
            world.set_name(entity, Name(name_str.to_string()));

            self.enemy_entities.push(entity);
            self.enemy_positions.insert(entity, (x, y));
            self.enemy_stats.insert(entity, stats);
            self.enemy_kinds.insert(entity, kind);
        }
    }

    fn spawn_items(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        let item_count = 4 + self.depth;

        for _ in 0..item_count {
            if self.rooms.is_empty() {
                break;
            }
            let room_index = rng.random_range(0..self.rooms.len());
            let room = self.rooms[room_index];
            let x = rng.random_range(room.x1 + 1..room.x2 - 1);
            let y = rng.random_range(room.y1 + 1..room.y2 - 1);

            if x == self.player_x && y == self.player_y {
                continue;
            }

            if self
                .item_positions
                .values()
                .any(|&(ix, iy)| ix == x && iy == y)
            {
                continue;
            }

            if self
                .enemy_positions
                .values()
                .any(|&(ex, ey)| ex == x && ey == y)
            {
                continue;
            }

            let kind_roll: f64 = rng.random();
            let kind = if kind_roll < 0.35 {
                ItemKind::Gold
            } else if kind_roll < 0.6 {
                ItemKind::HealthPotion
            } else if kind_roll < 0.8 {
                ItemKind::Sword
            } else {
                ItemKind::Shield
            };

            let (character, foreground, name_str) = match kind {
                ItemKind::HealthPotion => ('!', TermColor::Magenta, "Health Potion"),
                ItemKind::Sword => ('|', TermColor::Cyan, "Sword"),
                ItemKind::Shield => (']', TermColor::Blue, "Shield"),
                ItemKind::Gold => ('$', TermColor::Yellow, "Gold"),
            };

            let entity =
                world.spawn_entities(POSITION | SPRITE | VISIBILITY | Z_INDEX | NAME, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: x as f64,
                    row: y as f64,
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
            world.set_visibility(entity, Visibility { visible: false });
            world.set_z_index(entity, ZIndex(1));
            world.set_name(entity, Name(name_str.to_string()));

            self.item_entities.push(entity);
            self.item_positions.insert(entity, (x, y));
            self.item_kinds.insert(entity, kind);
        }
    }

    fn update_camera(&mut self, world: &mut World) {
        let terminal = world.resources.terminal_size;
        let viewport_width = terminal.columns as i32;
        let viewport_height = terminal.rows as i32 - HUD_HEIGHT - LOG_HEIGHT;

        let mut cam_x = self.player_x - viewport_width / 2;
        let mut cam_y = self.player_y - viewport_height / 2;

        cam_x = cam_x.clamp(0, (MAP_WIDTH - viewport_width).max(0));
        cam_y = cam_y.clamp(0, (MAP_HEIGHT - viewport_height).max(0));

        world.resources.camera.offset_column = cam_x as f64;
        world.resources.camera.offset_row = cam_y as f64;
    }

    fn update_tile_visibility(&mut self, world: &mut World) {
        for (index, &entity) in self.tile_entities.iter().enumerate() {
            let is_visible = self.visible[index];
            let is_explored = self.explored[index];

            if is_visible {
                world.set_visibility(entity, Visibility { visible: true });
                let tile = self.tiles[index];
                let sprite = match tile {
                    Tile::Wall => Sprite {
                        character: '#',
                        foreground: TermColor::DarkGrey,
                        background: TermColor::Black,
                    },
                    Tile::Floor => Sprite {
                        character: '.',
                        foreground: TermColor::Grey,
                        background: TermColor::Black,
                    },
                    Tile::Stairs => Sprite {
                        character: '>',
                        foreground: TermColor::Cyan,
                        background: TermColor::Black,
                    },
                };
                world.set_sprite(entity, sprite);
            } else if is_explored {
                world.set_visibility(entity, Visibility { visible: true });
                let tile = self.tiles[index];
                let sprite = match tile {
                    Tile::Wall => Sprite {
                        character: '#',
                        foreground: TermColor::Rgb {
                            r: 40,
                            g: 40,
                            b: 40,
                        },
                        background: TermColor::Black,
                    },
                    Tile::Floor => Sprite {
                        character: '.',
                        foreground: TermColor::Rgb {
                            r: 50,
                            g: 50,
                            b: 50,
                        },
                        background: TermColor::Black,
                    },
                    Tile::Stairs => Sprite {
                        character: '>',
                        foreground: TermColor::Rgb {
                            r: 50,
                            g: 80,
                            b: 80,
                        },
                        background: TermColor::Black,
                    },
                };
                world.set_sprite(entity, sprite);
            } else {
                world.set_visibility(entity, Visibility { visible: false });
            }
        }
    }

    fn update_entity_visibility(&mut self, world: &mut World) {
        for &entity in &self.enemy_entities {
            if let Some(&(x, y)) = self.enemy_positions.get(&entity) {
                let index = (y * MAP_WIDTH + x) as usize;
                let is_visible = index < self.visible.len() && self.visible[index];
                world.set_visibility(
                    entity,
                    Visibility {
                        visible: is_visible,
                    },
                );
            }
        }

        for &entity in &self.item_entities {
            if let Some(&(x, y)) = self.item_positions.get(&entity) {
                let index = (y * MAP_WIDTH + x) as usize;
                let is_visible = index < self.visible.len() && self.visible[index];
                world.set_visibility(
                    entity,
                    Visibility {
                        visible: is_visible,
                    },
                );
            }
        }
    }

    fn try_move_player(&mut self, delta_x: i32, delta_y: i32, world: &mut World) -> bool {
        let new_x = self.player_x + delta_x;
        let new_y = self.player_y + delta_y;

        if !(0..MAP_WIDTH).contains(&new_x) || !(0..MAP_HEIGHT).contains(&new_y) {
            return false;
        }

        let tile_index = (new_y * MAP_WIDTH + new_x) as usize;
        if self.tiles[tile_index] == Tile::Wall {
            return false;
        }

        let enemy_at_target: Option<Entity> = self
            .enemy_positions
            .iter()
            .find(|&(_, &(ex, ey))| ex == new_x && ey == new_y)
            .map(|(&entity, _)| entity);

        if let Some(target_entity) = enemy_at_target {
            self.attack_enemy(target_entity, world);
            return true;
        }

        self.player_x = new_x;
        self.player_y = new_y;
        world.set_position(
            self.player_entity,
            Position {
                column: new_x as f64,
                row: new_y as f64,
            },
        );

        self.check_item_pickup(world);

        if self.tiles[tile_index] == Tile::Stairs {
            self.descend(world);
        }

        true
    }

    fn attack_enemy(&mut self, target: Entity, world: &mut World) {
        let target_stats = match self.enemy_stats.get(&target) {
            Some(stats) => *stats,
            None => return,
        };

        let damage = (self.player_stats.attack - target_stats.defense).max(1);
        let target_name = world
            .get_name(target)
            .map(|name| name.0.clone())
            .unwrap_or_else(|| "enemy".to_string());

        let new_hp = target_stats.hp - damage;
        let killed = new_hp <= 0;

        world.resources.event_bus.publish_app_event(DamageEvent {
            attacker_name: "You".to_string(),
            defender_name: target_name,
            damage,
            defender_killed: killed,
        });

        if killed {
            self.remove_enemy(target, world);
        } else if let Some(stats) = self.enemy_stats.get_mut(&target) {
            stats.hp = new_hp;
        }
    }

    fn remove_enemy(&mut self, entity: Entity, world: &mut World) {
        world.despawn_entities(&[entity]);
        self.enemy_entities.retain(|&existing| existing != entity);
        self.enemy_positions.remove(&entity);
        self.enemy_stats.remove(&entity);
        self.enemy_kinds.remove(&entity);
    }

    fn check_item_pickup(&mut self, world: &mut World) {
        let items_here: Vec<Entity> = self
            .item_positions
            .iter()
            .filter(|&(_, &(x, y))| x == self.player_x && y == self.player_y)
            .map(|(&entity, _)| entity)
            .collect();

        for entity in items_here {
            if let Some(&kind) = self.item_kinds.get(&entity) {
                let item_name = world
                    .get_name(entity)
                    .map(|name| name.0.clone())
                    .unwrap_or_else(|| "item".to_string());

                world.resources.event_bus.publish_app_event(PickupEvent {
                    item_name,
                    item_kind: kind,
                });

                match kind {
                    ItemKind::HealthPotion => {
                        let heal = 10;
                        self.player_stats.hp =
                            (self.player_stats.hp + heal).min(self.player_stats.max_hp);
                    }
                    ItemKind::Sword => {
                        self.attack_bonus += 2;
                        self.player_stats.attack = self.player_base_attack + self.attack_bonus;
                    }
                    ItemKind::Shield => {
                        self.defense_bonus += 1;
                        self.player_stats.defense = self.player_base_defense + self.defense_bonus;
                    }
                    ItemKind::Gold => {
                        self.gold += rng_range(5, 15);
                    }
                }

                world.despawn_entities(&[entity]);
                self.item_entities.retain(|&existing| existing != entity);
                self.item_positions.remove(&entity);
                self.item_kinds.remove(&entity);
            }
        }
    }

    fn descend(&mut self, world: &mut World) {
        self.depth += 1;
        self.add_message("You descend deeper...".to_string());
        self.generate_level(world);
    }

    fn run_enemy_turns(&mut self, world: &mut World) {
        let mut rng = rand::rng();

        let enemy_data: Vec<(Entity, (i32, i32), EnemyKind)> = self
            .enemy_entities
            .iter()
            .filter_map(|&entity| {
                let pos = self.enemy_positions.get(&entity)?;
                let kind = self.enemy_kinds.get(&entity)?;
                Some((entity, *pos, *kind))
            })
            .collect();

        for (entity, (enemy_x, enemy_y), kind) in enemy_data {
            let (delta_x, delta_y) = match kind {
                EnemyKind::Rat => {
                    let direction = rng.random_range(0..4);
                    match direction {
                        0 => (0, -1),
                        1 => (0, 1),
                        2 => (-1, 0),
                        _ => (1, 0),
                    }
                }
                EnemyKind::Goblin | EnemyKind::Orc => {
                    let distance_x = (self.player_x - enemy_x).abs();
                    let distance_y = (self.player_y - enemy_y).abs();
                    let manhattan = distance_x + distance_y;

                    if manhattan <= 6 {
                        let move_x = (self.player_x - enemy_x).signum();
                        let move_y = (self.player_y - enemy_y).signum();
                        if distance_x >= distance_y {
                            (move_x, 0)
                        } else {
                            (0, move_y)
                        }
                    } else {
                        let direction = rng.random_range(0..4);
                        match direction {
                            0 => (0, -1),
                            1 => (0, 1),
                            2 => (-1, 0),
                            _ => (1, 0),
                        }
                    }
                }
            };

            let new_x = enemy_x + delta_x;
            let new_y = enemy_y + delta_y;

            if !(0..MAP_WIDTH).contains(&new_x) || !(0..MAP_HEIGHT).contains(&new_y) {
                continue;
            }

            let tile_index = (new_y * MAP_WIDTH + new_x) as usize;
            if tile_index >= self.tiles.len() || self.tiles[tile_index] == Tile::Wall {
                continue;
            }

            if new_x == self.player_x && new_y == self.player_y {
                self.enemy_attacks_player(entity, world);
                continue;
            }

            let blocked_by_enemy = self
                .enemy_positions
                .iter()
                .any(|(&other_entity, &(ox, oy))| {
                    other_entity != entity && ox == new_x && oy == new_y
                });

            if blocked_by_enemy {
                continue;
            }

            self.enemy_positions.insert(entity, (new_x, new_y));
            world.set_position(
                entity,
                Position {
                    column: new_x as f64,
                    row: new_y as f64,
                },
            );
        }
    }

    fn enemy_attacks_player(&mut self, enemy_entity: Entity, world: &mut World) {
        let enemy_stats = match self.enemy_stats.get(&enemy_entity) {
            Some(stats) => *stats,
            None => return,
        };

        let damage = (enemy_stats.attack - self.player_stats.defense).max(1);
        let enemy_name = world
            .get_name(enemy_entity)
            .map(|name| name.0.clone())
            .unwrap_or_else(|| "enemy".to_string());

        self.player_stats.hp -= damage;
        let killed = self.player_stats.hp <= 0;

        world.resources.event_bus.publish_app_event(DamageEvent {
            attacker_name: format!("The {}", enemy_name),
            defender_name: "you".to_string(),
            damage,
            defender_killed: killed,
        });

        if killed {
            self.game_over = true;
        }
    }

    fn add_message(&mut self, message: String) {
        self.messages.push(message);
        if self.messages.len() > 50 {
            self.messages.remove(0);
        }
    }

    fn update_hud(&mut self, world: &mut World) {
        if !self.hud_entities.is_empty() {
            world.despawn_entities(&self.hud_entities);
            self.hud_entities.clear();
        }

        let terminal = world.resources.terminal_size;
        let cam_x = world.resources.camera.offset_column as i32;
        let cam_y = world.resources.camera.offset_row as i32;

        let hud_text = format!(
            "HP: {}/{}  Atk: {}  Def: {}  Gold: {}  Depth: {}",
            self.player_stats.hp,
            self.player_stats.max_hp,
            self.player_stats.attack,
            self.player_stats.defense,
            self.gold,
            self.depth,
        );

        let screen_row = 0;
        for (char_index, character) in hud_text.chars().enumerate() {
            if char_index >= terminal.columns as usize {
                break;
            }
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (cam_x + char_index as i32) as f64,
                    row: (cam_y + screen_row) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: TermColor::Yellow,
                    background: TermColor::Rgb {
                        r: 20,
                        g: 20,
                        b: 40,
                    },
                },
            );
            world.set_z_index(entity, ZIndex(10));
            self.hud_entities.push(entity);
        }

        for fill_index in hud_text.len()..terminal.columns as usize {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (cam_x + fill_index as i32) as f64,
                    row: (cam_y + screen_row) as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: ' ',
                    foreground: TermColor::White,
                    background: TermColor::Rgb {
                        r: 20,
                        g: 20,
                        b: 40,
                    },
                },
            );
            world.set_z_index(entity, ZIndex(10));
            self.hud_entities.push(entity);
        }
    }

    fn update_log(&mut self, world: &mut World) {
        if !self.log_entities.is_empty() {
            world.despawn_entities(&self.log_entities);
            self.log_entities.clear();
        }

        let terminal = world.resources.terminal_size;
        let cam_x = world.resources.camera.offset_column as i32;
        let cam_y = world.resources.camera.offset_row as i32;
        let log_start_row = terminal.rows as i32 - LOG_HEIGHT;

        let recent_messages: Vec<String> = self
            .messages
            .iter()
            .rev()
            .take(LOG_HEIGHT as usize)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        for (line_index, message) in recent_messages.iter().enumerate() {
            let row = log_start_row + line_index as i32;

            for fill_index in 0..terminal.columns as usize {
                let character = message.chars().nth(fill_index).unwrap_or(' ');
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (cam_x + fill_index as i32) as f64,
                        row: (cam_y + row) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: if fill_index < message.len() {
                            TermColor::White
                        } else {
                            TermColor::Black
                        },
                        background: TermColor::Rgb {
                            r: 20,
                            g: 20,
                            b: 20,
                        },
                    },
                );
                world.set_z_index(entity, ZIndex(10));
                self.log_entities.push(entity);
            }
        }

        for line_index in recent_messages.len()..LOG_HEIGHT as usize {
            let row = log_start_row + line_index as i32;
            for fill_index in 0..terminal.columns as usize {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (cam_x + fill_index as i32) as f64,
                        row: (cam_y + row) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character: ' ',
                        foreground: TermColor::Black,
                        background: TermColor::Rgb {
                            r: 20,
                            g: 20,
                            b: 20,
                        },
                    },
                );
                world.set_z_index(entity, ZIndex(10));
                self.log_entities.push(entity);
            }
        }
    }
}

fn rng_range(min: u32, max: u32) -> u32 {
    let mut rng = rand::rng();
    rng.random_range(min..=max)
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Roguelike"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        self.generate_level(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed || self.game_over {
            return;
        }

        let (delta_x, delta_y) = match key {
            KeyCode::Up | KeyCode::Char('w') => (0, -1),
            KeyCode::Down | KeyCode::Char('s') => (0, 1),
            KeyCode::Left | KeyCode::Char('a') => (-1, 0),
            KeyCode::Right | KeyCode::Char('d') => (1, 0),
            KeyCode::Escape | KeyCode::Char('q') => {
                world.resources.should_exit = true;
                return;
            }
            _ => return,
        };

        if self.try_move_player(delta_x, delta_y, world) {
            self.turn_taken = true;
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.game_over {
            return;
        }

        if self.turn_taken {
            self.turn_taken = false;
            self.run_enemy_turns(world);
            self.update_camera(world);
            compute_fov(
                self.player_x,
                self.player_y,
                &self.tiles,
                &mut self.visible,
                &mut self.explored,
            );
            self.update_tile_visibility(world);
            self.update_entity_visibility(world);
            self.update_hud(world);
            self.update_log(world);
        }
    }

    fn handle_event(&mut self, _world: &mut World, message: &Message) {
        if let Message::App { type_name, payload } = message {
            if *type_name == std::any::type_name::<DamageEvent>() {
                if let Some(event) = payload.downcast_ref::<DamageEvent>() {
                    if event.defender_killed {
                        self.add_message(format!(
                            "{} hit the {} for {} damage, killing it!",
                            event.attacker_name, event.defender_name, event.damage
                        ));
                    } else {
                        self.add_message(format!(
                            "{} hit the {} for {} damage.",
                            event.attacker_name, event.defender_name, event.damage
                        ));
                    }
                }
            } else if *type_name == std::any::type_name::<PickupEvent>()
                && let Some(event) = payload.downcast_ref::<PickupEvent>()
            {
                let effect = match event.item_kind {
                    ItemKind::HealthPotion => "Restored health!",
                    ItemKind::Sword => "Attack increased!",
                    ItemKind::Shield => "Defense increased!",
                    ItemKind::Gold => "Shiny!",
                };
                self.add_message(format!("You picked up {}. {}", event.item_name, effect));
            }
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            self.clear_entities(world);
            return Some(Box::new(GameOverState {
                depth: self.depth,
                gold: self.gold,
                restart: false,
            }));
        }
        None
    }
}

struct GameOverState {
    depth: u32,
    gold: u32,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Roguelike - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as i32 / 2;
        let center_row = terminal.rows as i32 / 2;

        let lines = [
            ("GAME OVER", TermColor::Red),
            ("", TermColor::Black),
            (
                &format!("You died on depth {} with {} gold.", self.depth, self.gold),
                TermColor::Yellow,
            ),
            ("", TermColor::Black),
            ("Press R to restart", TermColor::White),
            ("Press ESC to quit", TermColor::Grey),
        ];

        for (line_index, (text, color)) in lines.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let start_col = center_column - text.len() as i32 / 2;
            for (char_index, character) in text.chars().enumerate() {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (start_col + char_index as i32) as f64,
                        row: (center_row - 3 + line_index as i32) as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: *color,
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(10));
            }
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Char('r') => {
                self.restart = true;
            }
            KeyCode::Escape | KeyCode::Char('q') => {
                world.resources.should_exit = true;
            }
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.restart {
            let all_entities: Vec<Entity> = world.query_entities(POSITION | SPRITE).collect();
            world.despawn_entities(&all_entities);
            return Some(Box::new(GameplayState::new()));
        }
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(TitleScreenState { start_game: false }))
}
