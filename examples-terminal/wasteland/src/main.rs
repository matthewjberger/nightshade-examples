use nightshade::tui::prelude::*;
use rand::Rng;

const MAP_SIZE: usize = 24;
const VIEW_RADIUS: f64 = 8.0;
const PLAYER_MAX_HP: i32 = 30;
const PLAYER_MAX_AP: i32 = 8;
const ATTACK_AP_COST: i32 = 2;
const HUD_HEIGHT: i32 = 2;
const MSG_HEIGHT: i32 = 4;
const EDGE_SCROLL_MARGIN: f64 = 2.0;
const EDGE_SCROLL_SPEED: f64 = 16.0;
const WALK_TILES_PER_SECOND: f64 = 6.0;
const ENEMY_ANIM_DURATION: f64 = 0.3;
const MAX_SCROLL_OFFSET: f64 = 30.0;

const TILE_FLOOR: u8 = 0;
const TILE_WALL: u8 = 1;
const TILE_RUBBLE: u8 = 2;
const TILE_WATER: u8 = 3;
const TILE_CRATE: u8 = 4;
const TILE_EXIT: u8 = 5;

fn iso_to_screen(gx: i32, gy: i32) -> (i32, i32) {
    ((gx - gy) * 2, gx + gy)
}

fn tile_appearance(tile: u8, gx: i32, gy: i32) -> (char, char, TermColor, TermColor) {
    match tile {
        TILE_FLOOR => {
            if (gx + gy) % 2 == 0 {
                (
                    '.',
                    ' ',
                    TermColor::Rgb {
                        r: 130,
                        g: 110,
                        b: 70,
                    },
                    TermColor::Rgb {
                        r: 50,
                        g: 40,
                        b: 20,
                    },
                )
            } else {
                (
                    ' ',
                    '.',
                    TermColor::Rgb {
                        r: 110,
                        g: 90,
                        b: 55,
                    },
                    TermColor::Rgb {
                        r: 45,
                        g: 35,
                        b: 18,
                    },
                )
            }
        }
        TILE_WALL => (
            '█',
            '█',
            TermColor::Rgb {
                r: 90,
                g: 85,
                b: 80,
            },
            TermColor::Rgb {
                r: 50,
                g: 48,
                b: 45,
            },
        ),
        TILE_RUBBLE => (
            '░',
            '░',
            TermColor::Rgb {
                r: 110,
                g: 95,
                b: 70,
            },
            TermColor::Rgb {
                r: 50,
                g: 40,
                b: 20,
            },
        ),
        TILE_WATER => (
            '~',
            '~',
            TermColor::Rgb {
                r: 60,
                g: 120,
                b: 200,
            },
            TermColor::Rgb {
                r: 15,
                g: 30,
                b: 80,
            },
        ),
        TILE_CRATE => (
            '[',
            ']',
            TermColor::Rgb {
                r: 200,
                g: 170,
                b: 50,
            },
            TermColor::Rgb {
                r: 70,
                g: 50,
                b: 15,
            },
        ),
        TILE_EXIT => (
            '<',
            '>',
            TermColor::Rgb {
                r: 50,
                g: 255,
                b: 50,
            },
            TermColor::Rgb { r: 0, g: 50, b: 0 },
        ),
        _ => (' ', ' ', TermColor::Black, TermColor::Black),
    }
}

fn is_walkable(tile: u8) -> bool {
    matches!(tile, TILE_FLOOR | TILE_RUBBLE | TILE_CRATE | TILE_EXIT)
}

fn compute_fov(
    player_gx: i32,
    player_gy: i32,
    map: &[[u8; MAP_SIZE]; MAP_SIZE],
    visible: &mut [[bool; MAP_SIZE]; MAP_SIZE],
    seen: &mut [[bool; MAP_SIZE]; MAP_SIZE],
) {
    for row in visible.iter_mut() {
        for cell in row.iter_mut() {
            *cell = false;
        }
    }

    for gy in 0..MAP_SIZE as i32 {
        for gx in 0..MAP_SIZE as i32 {
            let dx = gx - player_gx;
            let dy = gy - player_gy;
            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            if dist > VIEW_RADIUS {
                continue;
            }

            let steps = dist.ceil().max(1.0) as i32;
            let mut blocked = false;

            for step in 1..steps {
                let t = step as f64 / steps as f64;
                let check_x = (player_gx as f64 + dx as f64 * t).round() as i32;
                let check_y = (player_gy as f64 + dy as f64 * t).round() as i32;
                if check_x == gx && check_y == gy {
                    continue;
                }
                if check_x >= 0
                    && check_x < MAP_SIZE as i32
                    && check_y >= 0
                    && check_y < MAP_SIZE as i32
                    && map[check_y as usize][check_x as usize] == TILE_WALL
                {
                    blocked = true;
                    break;
                }
            }

            if !blocked {
                visible[gy as usize][gx as usize] = true;
                seen[gy as usize][gx as usize] = true;
            }
        }
    }
}

const MAP_LAYOUT: [&str; MAP_SIZE] = [
    "WWWWWWWWWWWWWWWWWWWWWWWW",
    "W..r........r.........W",
    "W..WWWW.......WWWWWW..W",
    "W..W..W.......W....W..W",
    "W..W..W..c....W..c.W..W",
    "W..WWWW.......WWWWWW..W",
    "W......r..............W",
    "W.........WWWWWW......W",
    "W.........W....W...c..W",
    "W...c.....W....W......W",
    "W.........WWWWWW......W",
    "W....r................W",
    "W..........r..........W",
    "W..WWWW..........WWWW.W",
    "W..W..W..........W..W.W",
    "W..W..W....c.....W..W.W",
    "W..WWWW..........WWWW.W",
    "W.....r...............W",
    "W........wwww.........W",
    "W........wwww.........W",
    "W........wwww.........W",
    "W......................W",
    "W..........c.........EW",
    "WWWWWWWWWWWWWWWWWWWWWWWW",
];

fn create_map() -> [[u8; MAP_SIZE]; MAP_SIZE] {
    let mut map = [[TILE_FLOOR; MAP_SIZE]; MAP_SIZE];
    for (row, line) in MAP_LAYOUT.iter().enumerate() {
        for (col, character) in line.chars().enumerate() {
            if col >= MAP_SIZE || row >= MAP_SIZE {
                continue;
            }
            map[row][col] = match character {
                'W' => TILE_WALL,
                'r' => TILE_RUBBLE,
                'w' => TILE_WATER,
                'c' => TILE_CRATE,
                'E' => TILE_EXIT,
                _ => TILE_FLOOR,
            };
        }
    }
    map
}

#[derive(Clone, Copy, PartialEq)]
enum CreatureKind {
    Raider,
    MutantRat,
    Robot,
}

struct CreatureData {
    gx: i32,
    gy: i32,
    entity: Entity,
    kind: CreatureKind,
    hp: i32,
    damage_min: i32,
    damage_max: i32,
    moves_per_turn: i32,
    alive: bool,
}

struct ItemData {
    gx: i32,
    gy: i32,
    entity: Entity,
    heal_amount: i32,
    picked_up: bool,
}

#[derive(PartialEq)]
enum GamePhase {
    PlayerIdle,
    PlayerWalking,
    EnemyAnimating,
}

struct EnemySnapshot {
    creature_index: usize,
    old_gx: i32,
    old_gy: i32,
    new_gx: i32,
    new_gy: i32,
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
        if self.messages.len() > 50 {
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
        "Wasteland - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        let terminal = world.resources.terminal_size;
        let center_col = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "W A S T E L A N D";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_col - title.len() as f64 / 2.0,
                row: center_row - 6.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: title.to_string(),
                foreground: TermColor::Rgb {
                    r: 200,
                    g: 150,
                    b: 50,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let art = [
            "      ___===___      ",
            "     /  * * *  \\    ",
            "    | WASTELAND |    ",
            "     \\_________/    ",
            "       |     |       ",
            "     __|_____|__     ",
            "    /___________\\   ",
        ];
        for (line_index, line) in art.iter().enumerate() {
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_col - line.len() as f64 / 2.0,
                    row: center_row - 3.0 + line_index as f64,
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

        let subtitle = "An isometric wasteland RPG";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_col - subtitle.len() as f64 / 2.0,
                row: center_row + 5.0,
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
                column: center_col - prompt.len() as f64 / 2.0,
                row: center_row + 7.0,
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

        let controls = "Click: move/attack | Edge scroll | Right-click: recenter | Space: end turn";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_col - controls.len() as f64 / 2.0,
                row: center_row + 9.0,
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
    map: [[u8; MAP_SIZE]; MAP_SIZE],
    visible: [[bool; MAP_SIZE]; MAP_SIZE],
    seen: [[bool; MAP_SIZE]; MAP_SIZE],
    tilemap_entity: Entity,
    player_entity: Entity,
    player_gx: i32,
    player_gy: i32,
    player_hp: i32,
    player_max_hp: i32,
    player_ap: i32,
    player_max_ap: i32,
    player_damage_min: i32,
    player_damage_max: i32,
    creatures: Vec<CreatureData>,
    items: Vec<ItemData>,
    phase: GamePhase,
    messages: MessageLog,
    hud_entities: EntityGroup,
    msg_entities: EntityGroup,
    highlight_entity: Entity,
    game_over: bool,
    game_won: bool,
    kills: u32,
    medkits_used: u32,
    camera_sx: f64,
    camera_sy: f64,
    scroll_offset_sx: f64,
    scroll_offset_sy: f64,
    walk_path: Vec<(i32, i32)>,
    walk_index: usize,
    walk_progress: f64,
    pending_attack_target: Option<usize>,
    enemy_snapshots: Vec<EnemySnapshot>,
    enemy_anim_progress: f64,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            map: create_map(),
            visible: [[false; MAP_SIZE]; MAP_SIZE],
            seen: [[false; MAP_SIZE]; MAP_SIZE],
            tilemap_entity: Entity::default(),
            player_entity: Entity::default(),
            player_gx: 2,
            player_gy: 1,
            player_hp: PLAYER_MAX_HP,
            player_max_hp: PLAYER_MAX_HP,
            player_ap: PLAYER_MAX_AP,
            player_max_ap: PLAYER_MAX_AP,
            player_damage_min: 3,
            player_damage_max: 6,
            creatures: Vec::new(),
            items: Vec::new(),
            phase: GamePhase::PlayerIdle,
            messages: MessageLog::new(),
            hud_entities: EntityGroup::new(),
            msg_entities: EntityGroup::new(),
            highlight_entity: Entity::default(),
            game_over: false,
            game_won: false,
            kills: 0,
            medkits_used: 0,
            camera_sx: 0.0,
            camera_sy: 0.0,
            scroll_offset_sx: 0.0,
            scroll_offset_sy: 0.0,
            walk_path: Vec::new(),
            walk_index: 0,
            walk_progress: 0.0,
            pending_attack_target: None,
            enemy_snapshots: Vec::new(),
            enemy_anim_progress: 0.0,
        }
    }

    fn spawn_creatures(&mut self, world: &mut World) {
        let creature_defs: Vec<(i32, i32, CreatureKind)> = vec![
            (10, 3, CreatureKind::Raider),
            (16, 11, CreatureKind::Raider),
            (15, 19, CreatureKind::Raider),
            (5, 7, CreatureKind::MutantRat),
            (12, 6, CreatureKind::MutantRat),
            (18, 8, CreatureKind::MutantRat),
            (20, 4, CreatureKind::MutantRat),
            (8, 15, CreatureKind::Robot),
        ];

        for (gx, gy, kind) in creature_defs {
            let (character, color, hp, damage_min, damage_max, moves) = match kind {
                CreatureKind::Raider => ('R', TermColor::Red, 15, 3, 5, 1),
                CreatureKind::MutantRat => (
                    'r',
                    TermColor::Rgb {
                        r: 180,
                        g: 50,
                        b: 50,
                    },
                    8,
                    1,
                    3,
                    2,
                ),
                CreatureKind::Robot => (
                    'T',
                    TermColor::Rgb {
                        r: 200,
                        g: 200,
                        b: 50,
                    },
                    25,
                    5,
                    8,
                    1,
                ),
            };

            let entity = EntityBuilder::new()
                .position(Position {
                    column: 0.0,
                    row: 0.0,
                })
                .sprite(Sprite {
                    character,
                    foreground: color,
                    background: TermColor::Black,
                })
                .z_index(ZIndex(3))
                .visibility(Visibility { visible: false })
                .spawn(world);

            self.creatures.push(CreatureData {
                gx,
                gy,
                entity,
                kind,
                hp,
                damage_min,
                damage_max,
                moves_per_turn: moves,
                alive: true,
            });
        }
    }

    fn spawn_items(&mut self, world: &mut World) {
        let item_positions = [(10, 4), (15, 8), (4, 9), (11, 15), (8, 22), (21, 8)];

        for (gx, gy) in item_positions {
            let entity = EntityBuilder::new()
                .position(Position {
                    column: 0.0,
                    row: 0.0,
                })
                .sprite(Sprite {
                    character: '+',
                    foreground: TermColor::Green,
                    background: TermColor::Black,
                })
                .z_index(ZIndex(2))
                .visibility(Visibility { visible: false })
                .spawn(world);

            self.items.push(ItemData {
                gx,
                gy,
                entity,
                heal_amount: 10,
                picked_up: false,
            });
        }
    }

    fn screen_center(&self, world: &World) -> (f64, f64) {
        let terminal = world.resources.terminal_size;
        let center_col = terminal.columns as f64 / 2.0;
        let view_top = HUD_HEIGHT as f64;
        let view_bottom = terminal.rows as f64 - MSG_HEIGHT as f64;
        let center_row = (view_top + view_bottom) / 2.0;
        (center_col, center_row)
    }

    fn player_visual_iso(&self) -> (f64, f64) {
        if self.phase == GamePhase::PlayerWalking
            && self.walk_index > 0
            && self.walk_index < self.walk_path.len()
        {
            let (from_gx, from_gy) = self.walk_path[self.walk_index - 1];
            let (to_gx, to_gy) = self.walk_path[self.walk_index];
            let (from_sx, from_sy) = iso_to_screen(from_gx, from_gy);
            let (to_sx, to_sy) = iso_to_screen(to_gx, to_gy);
            let t = self.walk_progress.clamp(0.0, 1.0);
            (
                from_sx as f64 + (to_sx - from_sx) as f64 * t,
                from_sy as f64 + (to_sy - from_sy) as f64 * t,
            )
        } else {
            let (sx, sy) = iso_to_screen(self.player_gx, self.player_gy);
            (sx as f64, sy as f64)
        }
    }

    fn creature_visual_iso(&self, creature_index: usize) -> (f64, f64) {
        if self.phase == GamePhase::EnemyAnimating
            && let Some(snapshot) = self
                .enemy_snapshots
                .iter()
                .find(|snapshot| snapshot.creature_index == creature_index)
        {
            let (from_sx, from_sy) = iso_to_screen(snapshot.old_gx, snapshot.old_gy);
            let (to_sx, to_sy) = iso_to_screen(snapshot.new_gx, snapshot.new_gy);
            let t = self.enemy_anim_progress.clamp(0.0, 1.0);
            return (
                from_sx as f64 + (to_sx - from_sx) as f64 * t,
                from_sy as f64 + (to_sy - from_sy) as f64 * t,
            );
        }
        let creature = &self.creatures[creature_index];
        let (sx, sy) = iso_to_screen(creature.gx, creature.gy);
        (sx as f64, sy as f64)
    }

    fn update_camera(&mut self, world: &World) {
        let delta = world.resources.timing.delta_seconds;

        if self.phase == GamePhase::PlayerIdle {
            let mouse_col = world.resources.mouse.column as f64;
            let mouse_row = world.resources.mouse.row as f64;
            let cols = world.resources.terminal_size.columns as f64;
            let rows = world.resources.terminal_size.rows as f64;

            if mouse_col <= EDGE_SCROLL_MARGIN {
                self.scroll_offset_sx -= EDGE_SCROLL_SPEED * delta;
            }
            if mouse_col >= cols - EDGE_SCROLL_MARGIN - 1.0 {
                self.scroll_offset_sx += EDGE_SCROLL_SPEED * delta;
            }
            if mouse_row <= EDGE_SCROLL_MARGIN {
                self.scroll_offset_sy -= EDGE_SCROLL_SPEED * delta;
            }
            if mouse_row >= rows - EDGE_SCROLL_MARGIN - 1.0 {
                self.scroll_offset_sy += EDGE_SCROLL_SPEED * delta;
            }

            self.scroll_offset_sx = self
                .scroll_offset_sx
                .clamp(-MAX_SCROLL_OFFSET, MAX_SCROLL_OFFSET);
            self.scroll_offset_sy = self
                .scroll_offset_sy
                .clamp(-MAX_SCROLL_OFFSET, MAX_SCROLL_OFFSET);
        }

        let (player_sx, player_sy) = self.player_visual_iso();
        self.camera_sx = player_sx + self.scroll_offset_sx;
        self.camera_sy = player_sy + self.scroll_offset_sy;
    }

    fn start_walk(&mut self, path: Vec<(i32, i32)>) {
        if path.len() < 2 {
            return;
        }
        self.walk_path = path;
        self.walk_index = 1;
        self.walk_progress = 0.0;
        self.pending_attack_target = None;
        self.phase = GamePhase::PlayerWalking;
        self.scroll_offset_sx = 0.0;
        self.scroll_offset_sy = 0.0;
    }

    fn start_walk_with_attack(&mut self, path: Vec<(i32, i32)>, target_creature: usize) {
        if path.len() < 2 {
            return;
        }
        self.walk_path = path;
        self.walk_index = 1;
        self.walk_progress = 0.0;
        self.pending_attack_target = Some(target_creature);
        self.phase = GamePhase::PlayerWalking;
        self.scroll_offset_sx = 0.0;
        self.scroll_offset_sy = 0.0;
    }

    fn advance_walk(&mut self, delta: f64, world: &mut World) {
        if self.phase != GamePhase::PlayerWalking {
            return;
        }

        self.walk_progress += WALK_TILES_PER_SECOND * delta;

        while self.walk_progress >= 1.0 && self.phase == GamePhase::PlayerWalking {
            self.walk_progress -= 1.0;

            let (gx, gy) = self.walk_path[self.walk_index];
            self.player_gx = gx;
            self.player_gy = gy;
            self.player_ap -= 1;

            self.check_item_pickup(world);
            compute_fov(
                self.player_gx,
                self.player_gy,
                &self.map,
                &mut self.visible,
                &mut self.seen,
            );

            if self.map[gy as usize][gx as usize] == TILE_EXIT {
                self.game_won = true;
                self.messages.add(
                    "You found the exit! You escape the wasteland!".to_string(),
                    TermColor::Green,
                );
                self.finish_walking();
                return;
            }

            if self.game_over {
                self.finish_walking();
                return;
            }

            let at_end = self.walk_index + 1 >= self.walk_path.len();
            let out_of_ap = self.player_ap <= 0;
            let next_blocked = !at_end && {
                let (next_gx, next_gy) = self.walk_path[self.walk_index + 1];
                self.creatures.iter().any(|creature| {
                    creature.alive && creature.gx == next_gx && creature.gy == next_gy
                })
            };

            if at_end || out_of_ap || next_blocked {
                self.walk_progress = 0.0;
                self.finish_walking();
                if self.player_ap <= 0 && !self.game_over && !self.game_won {
                    self.begin_enemy_turn();
                }
                return;
            }

            self.walk_index += 1;
        }
    }

    fn finish_walking(&mut self) {
        self.phase = GamePhase::PlayerIdle;
        self.walk_path.clear();

        if let Some(creature_index) = self.pending_attack_target.take()
            && self.player_ap >= ATTACK_AP_COST
            && creature_index < self.creatures.len()
            && self.creatures[creature_index].alive
        {
            let dx = (self.creatures[creature_index].gx - self.player_gx).abs();
            let dy = (self.creatures[creature_index].gy - self.player_gy).abs();
            if dx <= 1 && dy <= 1 && (dx + dy) > 0 {
                self.attack_creature(creature_index);
                if self.player_ap <= 0 && !self.game_over && !self.game_won {
                    self.begin_enemy_turn();
                }
            }
        }
    }

    fn begin_enemy_turn(&mut self) {
        let old_positions: Vec<(i32, i32)> = self
            .creatures
            .iter()
            .map(|creature| (creature.gx, creature.gy))
            .collect();

        self.execute_enemy_turns();

        self.enemy_snapshots = self
            .creatures
            .iter()
            .enumerate()
            .zip(old_positions.iter())
            .filter(|((_, creature), (old_gx, old_gy))| {
                creature.alive && (creature.gx != *old_gx || creature.gy != *old_gy)
            })
            .map(|((index, creature), (old_gx, old_gy))| EnemySnapshot {
                creature_index: index,
                old_gx: *old_gx,
                old_gy: *old_gy,
                new_gx: creature.gx,
                new_gy: creature.gy,
            })
            .collect();

        if self.enemy_snapshots.is_empty() {
            self.player_ap = self.player_max_ap;
            self.phase = GamePhase::PlayerIdle;
            compute_fov(
                self.player_gx,
                self.player_gy,
                &self.map,
                &mut self.visible,
                &mut self.seen,
            );
        } else {
            self.enemy_anim_progress = 0.0;
            self.phase = GamePhase::EnemyAnimating;
        }
    }

    fn advance_enemy_animation(&mut self, delta: f64) {
        if self.phase != GamePhase::EnemyAnimating {
            return;
        }

        self.enemy_anim_progress += delta / ENEMY_ANIM_DURATION;

        if self.enemy_anim_progress >= 1.0 {
            self.enemy_anim_progress = 1.0;
            self.enemy_snapshots.clear();
            self.player_ap = self.player_max_ap;
            self.phase = GamePhase::PlayerIdle;
            compute_fov(
                self.player_gx,
                self.player_gy,
                &self.map,
                &mut self.visible,
                &mut self.seen,
            );
        }
    }

    fn render_iso_map(&self, world: &mut World) {
        let terminal = world.resources.terminal_size;
        let tilemap_width = terminal.columns as usize;
        let tilemap_height = terminal.rows as usize;
        let mut tilemap = Tilemap::new(tilemap_width, tilemap_height);

        let (center_col, center_row) = self.screen_center(world);

        for gy in 0..MAP_SIZE as i32 {
            for gx in 0..MAP_SIZE as i32 {
                if !self.visible[gy as usize][gx as usize] && !self.seen[gy as usize][gx as usize] {
                    continue;
                }

                let (sx, sy) = iso_to_screen(gx, gy);
                let col = sx as f64 - self.camera_sx + center_col;
                let row = sy as f64 - self.camera_sy + center_row;

                let col_i = col.round() as i32;
                let row_i = row.round() as i32;

                if row_i < HUD_HEIGHT || row_i >= tilemap_height as i32 - MSG_HEIGHT || row_i < 0 {
                    continue;
                }

                let tile = self.map[gy as usize][gx as usize];
                let (character_1, character_2, foreground, background) =
                    tile_appearance(tile, gx, gy);

                let (foreground, background) = if !self.visible[gy as usize][gx as usize] {
                    (
                        TermColor::Rgb {
                            r: 50,
                            g: 50,
                            b: 50,
                        },
                        TermColor::Rgb {
                            r: 15,
                            g: 15,
                            b: 15,
                        },
                    )
                } else {
                    (foreground, background)
                };

                if col_i >= 0 && col_i < tilemap_width as i32 {
                    tilemap.set(
                        col_i as usize,
                        row_i as usize,
                        TilemapCell {
                            character: character_1,
                            foreground,
                            background,
                        },
                    );
                }
                if col_i + 1 >= 0 && (col_i + 1) < tilemap_width as i32 {
                    tilemap.set(
                        (col_i + 1) as usize,
                        row_i as usize,
                        TilemapCell {
                            character: character_2,
                            foreground,
                            background,
                        },
                    );
                }
            }
        }

        world.set_tilemap(self.tilemap_entity, tilemap);
    }

    fn update_entity_positions(&self, world: &mut World) {
        let (center_col, center_row) = self.screen_center(world);

        let (player_iso_sx, player_iso_sy) = self.player_visual_iso();
        let player_col = player_iso_sx - self.camera_sx + center_col;
        let player_row = player_iso_sy - self.camera_sy + center_row;

        if let Some(position) = world.get_position_mut(self.player_entity) {
            position.column = player_col;
            position.row = player_row;
        }

        for (creature_index, creature) in self.creatures.iter().enumerate() {
            if !creature.alive {
                continue;
            }

            let (iso_sx, iso_sy) = self.creature_visual_iso(creature_index);
            let col = iso_sx - self.camera_sx + center_col;
            let row = iso_sy - self.camera_sy + center_row;

            if let Some(position) = world.get_position_mut(creature.entity) {
                position.column = col;
                position.row = row;
            }

            let is_visible = creature.gy >= 0
                && creature.gy < MAP_SIZE as i32
                && creature.gx >= 0
                && creature.gx < MAP_SIZE as i32
                && self.visible[creature.gy as usize][creature.gx as usize];
            if let Some(visibility) = world.get_visibility_mut(creature.entity) {
                visibility.visible = is_visible;
            }
        }

        for item in &self.items {
            if item.picked_up {
                continue;
            }
            let (sx, sy) = iso_to_screen(item.gx, item.gy);
            let col = sx as f64 - self.camera_sx + center_col;
            let row = sy as f64 - self.camera_sy + center_row;

            if let Some(position) = world.get_position_mut(item.entity) {
                position.column = col;
                position.row = row;
            }

            let is_visible = item.gy >= 0
                && item.gy < MAP_SIZE as i32
                && item.gx >= 0
                && item.gx < MAP_SIZE as i32
                && self.visible[item.gy as usize][item.gx as usize];
            if let Some(visibility) = world.get_visibility_mut(item.entity) {
                visibility.visible = is_visible;
            }
        }
    }

    fn update_highlight(&mut self, world: &mut World) {
        if self.phase != GamePhase::PlayerIdle {
            if let Some(visibility) = world.get_visibility_mut(self.highlight_entity) {
                visibility.visible = false;
            }
            return;
        }

        let mouse_col = world.resources.mouse.column as f64;
        let mouse_row = world.resources.mouse.row as f64;
        let (target_gx, target_gy) = self.screen_to_iso_grid(mouse_col, mouse_row, world);

        let in_bounds = target_gx >= 0
            && target_gx < MAP_SIZE as i32
            && target_gy >= 0
            && target_gy < MAP_SIZE as i32;
        let is_visible = in_bounds && self.visible[target_gy as usize][target_gx as usize];

        if is_visible {
            if let Some(visibility) = world.get_visibility_mut(self.highlight_entity) {
                visibility.visible = true;
            }

            let (center_col, center_row) = self.screen_center(world);
            let (sx, sy) = iso_to_screen(target_gx, target_gy);
            let highlight_col = sx as f64 - self.camera_sx + center_col + 1.0;
            let highlight_row = sy as f64 - self.camera_sy + center_row;

            if let Some(position) = world.get_position_mut(self.highlight_entity) {
                position.column = highlight_col;
                position.row = highlight_row;
            }

            let has_enemy = self.creatures.iter().any(|creature| {
                creature.alive && creature.gx == target_gx && creature.gy == target_gy
            });
            let tile = self.map[target_gy as usize][target_gx as usize];
            let color = if has_enemy {
                TermColor::Red
            } else if is_walkable(tile) {
                TermColor::Yellow
            } else {
                TermColor::DarkGrey
            };

            if let Some(sprite) = world.get_sprite_mut(self.highlight_entity) {
                sprite.foreground = color;
            }
        } else if let Some(visibility) = world.get_visibility_mut(self.highlight_entity) {
            visibility.visible = false;
        }
    }

    fn try_keyboard_move(&mut self, dx: i32, dy: i32, world: &mut World) {
        if self.player_ap <= 0 || self.phase != GamePhase::PlayerIdle {
            return;
        }

        let new_gx = self.player_gx + dx;
        let new_gy = self.player_gy + dy;

        if new_gx < 0 || new_gx >= MAP_SIZE as i32 || new_gy < 0 || new_gy >= MAP_SIZE as i32 {
            return;
        }

        if let Some(creature_index) = self
            .creatures
            .iter()
            .position(|creature| creature.alive && creature.gx == new_gx && creature.gy == new_gy)
        {
            self.attack_creature(creature_index);
            if self.player_ap <= 0 && !self.game_over && !self.game_won {
                self.begin_enemy_turn();
            }
            return;
        }

        if !is_walkable(self.map[new_gy as usize][new_gx as usize]) {
            return;
        }

        self.player_gx = new_gx;
        self.player_gy = new_gy;
        self.player_ap -= 1;
        self.scroll_offset_sx = 0.0;
        self.scroll_offset_sy = 0.0;
        self.check_item_pickup(world);

        if self.map[new_gy as usize][new_gx as usize] == TILE_EXIT {
            self.game_won = true;
            self.messages.add(
                "You found the exit! You escape the wasteland!".to_string(),
                TermColor::Green,
            );
        }

        compute_fov(
            self.player_gx,
            self.player_gy,
            &self.map,
            &mut self.visible,
            &mut self.seen,
        );

        if self.player_ap <= 0 && !self.game_over && !self.game_won {
            self.begin_enemy_turn();
        }
    }

    fn try_click_move(&mut self, target_gx: i32, target_gy: i32) {
        if self.phase != GamePhase::PlayerIdle || self.player_ap <= 0 {
            return;
        }

        if target_gx < 0
            || target_gx >= MAP_SIZE as i32
            || target_gy < 0
            || target_gy >= MAP_SIZE as i32
        {
            return;
        }

        if target_gx == self.player_gx && target_gy == self.player_gy {
            return;
        }

        if let Some(creature_index) = self.creatures.iter().position(|creature| {
            creature.alive && creature.gx == target_gx && creature.gy == target_gy
        }) {
            let dx = (target_gx - self.player_gx).abs();
            let dy = (target_gy - self.player_gy).abs();
            if dx <= 1 && dy <= 1 && (dx + dy) > 0 {
                self.attack_creature(creature_index);
                if self.player_ap <= 0 && !self.game_over && !self.game_won {
                    self.begin_enemy_turn();
                }
            } else {
                let map = self.map;
                let creature_positions: Vec<(i32, i32)> = self
                    .creatures
                    .iter()
                    .filter(|creature| creature.alive)
                    .map(|creature| (creature.gx, creature.gy))
                    .collect();

                let adjacent_offsets = [
                    (0, -1),
                    (1, 0),
                    (0, 1),
                    (-1, 0),
                    (-1, -1),
                    (1, -1),
                    (-1, 1),
                    (1, 1),
                ];

                let player_pos = (self.player_gx, self.player_gy);
                let mut best_path: Option<Vec<(i32, i32)>> = None;

                for (offset_gx, offset_gy) in adjacent_offsets {
                    let adjacent_gx = target_gx + offset_gx;
                    let adjacent_gy = target_gy + offset_gy;
                    if adjacent_gx < 0
                        || adjacent_gx >= MAP_SIZE as i32
                        || adjacent_gy < 0
                        || adjacent_gy >= MAP_SIZE as i32
                    {
                        continue;
                    }
                    if !is_walkable(map[adjacent_gy as usize][adjacent_gx as usize]) {
                        continue;
                    }
                    if creature_positions.contains(&(adjacent_gx, adjacent_gy)) {
                        continue;
                    }
                    if adjacent_gx == player_pos.0 && adjacent_gy == player_pos.1 {
                        self.attack_creature(creature_index);
                        if self.player_ap <= 0 && !self.game_over && !self.game_won {
                            self.begin_enemy_turn();
                        }
                        return;
                    }

                    let path = astar(
                        player_pos,
                        (adjacent_gx, adjacent_gy),
                        |gx, gy| {
                            if gx < 0 || gx >= MAP_SIZE as i32 || gy < 0 || gy >= MAP_SIZE as i32 {
                                return false;
                            }
                            if !is_walkable(map[gy as usize][gx as usize]) {
                                return false;
                            }
                            !creature_positions.contains(&(gx, gy))
                        },
                        false,
                    );

                    if let Some(path) = path
                        && best_path
                            .as_ref()
                            .is_none_or(|best| path.len() < best.len())
                    {
                        best_path = Some(path);
                    }
                }

                if let Some(path) = best_path {
                    self.start_walk_with_attack(path, creature_index);
                } else {
                    self.messages
                        .add("No path to that enemy.".to_string(), TermColor::DarkYellow);
                }
            }
            return;
        }

        if !is_walkable(self.map[target_gy as usize][target_gx as usize]) {
            return;
        }

        let map = self.map;
        let creature_positions: Vec<(i32, i32)> = self
            .creatures
            .iter()
            .filter(|creature| creature.alive)
            .map(|creature| (creature.gx, creature.gy))
            .collect();

        let path = astar(
            (self.player_gx, self.player_gy),
            (target_gx, target_gy),
            |gx, gy| {
                if gx < 0 || gx >= MAP_SIZE as i32 || gy < 0 || gy >= MAP_SIZE as i32 {
                    return false;
                }
                if !is_walkable(map[gy as usize][gx as usize]) {
                    return false;
                }
                !creature_positions.contains(&(gx, gy))
            },
            false,
        );

        if let Some(path) = path {
            if path.len() >= 2 {
                self.start_walk(path);
            }
        } else {
            self.messages.add(
                "No path to that location.".to_string(),
                TermColor::DarkYellow,
            );
        }
    }

    fn attack_creature(&mut self, creature_index: usize) {
        if self.player_ap < ATTACK_AP_COST {
            self.messages
                .add("Not enough AP to attack.".to_string(), TermColor::Yellow);
            return;
        }

        self.player_ap -= ATTACK_AP_COST;
        let mut rng = rand::rng();
        let damage = rng.random_range(self.player_damage_min..=self.player_damage_max);
        let creature = &mut self.creatures[creature_index];
        creature.hp -= damage;

        let kind_name = match creature.kind {
            CreatureKind::Raider => "Raider",
            CreatureKind::MutantRat => "Mutant Rat",
            CreatureKind::Robot => "Robot",
        };

        self.messages.add(
            format!("You hit the {} for {} damage!", kind_name, damage),
            TermColor::White,
        );

        if creature.hp <= 0 {
            creature.alive = false;
            self.kills += 1;
            self.messages
                .add(format!("The {} is destroyed!", kind_name), TermColor::Green);
        }
    }

    fn check_item_pickup(&mut self, world: &mut World) {
        for item in &mut self.items {
            if item.picked_up {
                continue;
            }
            if item.gx == self.player_gx && item.gy == self.player_gy {
                item.picked_up = true;
                world.despawn_entities(&[item.entity]);
                self.player_hp = (self.player_hp + item.heal_amount).min(self.player_max_hp);
                self.medkits_used += 1;
                self.messages.add(
                    format!("Picked up medkit! +{} HP", item.heal_amount),
                    TermColor::Green,
                );
            }
        }
    }

    fn execute_enemy_turns(&mut self) {
        let mut rng = rand::rng();
        let player_gx = self.player_gx;
        let player_gy = self.player_gy;
        let map = self.map;

        for creature_index in 0..self.creatures.len() {
            if !self.creatures[creature_index].alive {
                continue;
            }

            let creature_gx = self.creatures[creature_index].gx;
            let creature_gy = self.creatures[creature_index].gy;
            let moves = self.creatures[creature_index].moves_per_turn;

            let dx = (player_gx - creature_gx).abs();
            let dy = (player_gy - creature_gy).abs();

            if dx <= 1 && dy <= 1 && (dx + dy) > 0 {
                let damage = rng.random_range(
                    self.creatures[creature_index].damage_min
                        ..=self.creatures[creature_index].damage_max,
                );
                self.player_hp -= damage;

                let kind_name = match self.creatures[creature_index].kind {
                    CreatureKind::Raider => "Raider",
                    CreatureKind::MutantRat => "Mutant Rat",
                    CreatureKind::Robot => "Robot",
                };
                self.messages.add(
                    format!("The {} hits you for {} damage!", kind_name, damage),
                    TermColor::Red,
                );

                if self.player_hp <= 0 {
                    self.player_hp = 0;
                    self.game_over = true;
                    self.messages
                        .add("You have been killed...".to_string(), TermColor::Red);
                }
                continue;
            }

            let other_positions: Vec<(i32, i32)> = self
                .creatures
                .iter()
                .enumerate()
                .filter(|(other_index, creature)| creature.alive && *other_index != creature_index)
                .map(|(_, creature)| (creature.gx, creature.gy))
                .collect();

            let path = astar(
                (creature_gx, creature_gy),
                (player_gx, player_gy),
                |gx, gy| {
                    if gx < 0 || gx >= MAP_SIZE as i32 || gy < 0 || gy >= MAP_SIZE as i32 {
                        return false;
                    }
                    if !is_walkable(map[gy as usize][gx as usize]) {
                        return false;
                    }
                    if gx == player_gx && gy == player_gy {
                        return true;
                    }
                    !other_positions.contains(&(gx, gy))
                },
                false,
            );

            if let Some(path) = path {
                let max_steps = moves.min(path.len() as i32 - 1);
                for step in 1..=max_steps {
                    let (next_gx, next_gy) = path[step as usize];
                    if next_gx == player_gx && next_gy == player_gy {
                        break;
                    }
                    self.creatures[creature_index].gx = next_gx;
                    self.creatures[creature_index].gy = next_gy;
                }
            }
        }
    }

    fn despawn_dead_creatures(&mut self, world: &mut World) {
        for creature in &mut self.creatures {
            if !creature.alive && creature.entity != Entity::default() {
                world.despawn_entities(&[creature.entity]);
                creature.entity = Entity::default();
            }
        }
    }

    fn screen_to_iso_grid(&self, mouse_col: f64, mouse_row: f64, world: &World) -> (i32, i32) {
        let (center_col, center_row) = self.screen_center(world);

        let iso_sx = mouse_col - center_col + self.camera_sx;
        let iso_sy = mouse_row - center_row + self.camera_sy;

        let half_sx = iso_sx / 2.0;
        let gx = (half_sx + iso_sy) / 2.0;
        let gy = (iso_sy - half_sx) / 2.0;

        (gx.round() as i32, gy.round() as i32)
    }

    fn update_hud(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);

        let hp_filled = (self.player_hp as f64 / self.player_max_hp as f64 * 20.0).round() as usize;
        let hp_bar = format!(
            "HP [{}{}] {}/{}",
            "█".repeat(hp_filled),
            "░".repeat(20 - hp_filled),
            self.player_hp,
            self.player_max_hp,
        );
        let hp_color = if self.player_hp > self.player_max_hp / 2 {
            TermColor::Green
        } else if self.player_hp > self.player_max_hp / 4 {
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
                column: 1.0,
                row: 0.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: hp_bar,
                foreground: hp_color,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(20));

        let ap_filled = self.player_ap.max(0) as usize;
        let ap_empty = (self.player_max_ap - self.player_ap).max(0) as usize;
        let ap_text = format!("AP {}{}", "◆".repeat(ap_filled), "◇".repeat(ap_empty));
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 32.0,
                row: 0.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: ap_text,
                foreground: TermColor::Cyan,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(20));

        let phase_text = match self.phase {
            GamePhase::PlayerIdle | GamePhase::PlayerWalking => "YOUR TURN",
            GamePhase::EnemyAnimating => "ENEMY TURN",
        };
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 55.0,
                row: 0.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: phase_text.to_string(),
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(20));

        let info = format!(
            "Kills: {}  [Click: move | Edge scroll | Right-click: recenter | Space: end turn]",
            self.kills
        );
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 1.0,
                row: 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: info,
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(20));
    }

    fn update_messages(&mut self, world: &mut World) {
        self.msg_entities.despawn_all(world);

        let terminal = world.resources.terminal_size;
        let msg_start_row = terminal.rows as f64 - MSG_HEIGHT as f64;
        let recent = self.messages.recent(MSG_HEIGHT as usize);

        for (line_index, (text, color)) in recent.iter().enumerate() {
            let entity = self
                .msg_entities
                .spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: 1.0,
                    row: msg_start_row + line_index as f64,
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
            world.set_z_index(entity, ZIndex(20));
        }
    }

    fn clear_all(&mut self, world: &mut World) {
        world.despawn_entities(&[
            self.tilemap_entity,
            self.player_entity,
            self.highlight_entity,
        ]);
        for creature in &self.creatures {
            if creature.alive && creature.entity != Entity::default() {
                world.despawn_entities(&[creature.entity]);
            }
        }
        for item in &self.items {
            if !item.picked_up {
                world.despawn_entities(&[item.entity]);
            }
        }
        self.hud_entities.despawn_all(world);
        self.msg_entities.despawn_all(world);
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Wasteland - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        world.resources.camera.offset_column = 0.0;
        world.resources.camera.offset_row = 0.0;

        compute_fov(
            self.player_gx,
            self.player_gy,
            &self.map,
            &mut self.visible,
            &mut self.seen,
        );

        let (initial_sx, initial_sy) = iso_to_screen(self.player_gx, self.player_gy);
        self.camera_sx = initial_sx as f64;
        self.camera_sy = initial_sy as f64;

        self.tilemap_entity = EntityBuilder::new()
            .position(Position {
                column: 0.0,
                row: 0.0,
            })
            .tilemap(Tilemap::new(1, 1))
            .z_index(ZIndex(0))
            .spawn(world);

        self.player_entity = EntityBuilder::new()
            .position(Position {
                column: 0.0,
                row: 0.0,
            })
            .sprite(Sprite {
                character: '@',
                foreground: TermColor::Cyan,
                background: TermColor::Black,
            })
            .z_index(ZIndex(5))
            .spawn(world);

        self.highlight_entity = EntityBuilder::new()
            .position(Position {
                column: 0.0,
                row: 0.0,
            })
            .sprite(Sprite {
                character: '+',
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            })
            .z_index(ZIndex(4))
            .visibility(Visibility { visible: false })
            .spawn(world);

        self.spawn_creatures(world);
        self.spawn_items(world);

        self.messages.add(
            "You awaken in the ruins. Find the exit to escape.".to_string(),
            TermColor::Cyan,
        );
        self.messages.add(
            "Click to move/attack | Edge scroll to look around | Right-click to recenter"
                .to_string(),
            TermColor::DarkGrey,
        );

        self.render_iso_map(world);
        self.update_entity_positions(world);
        self.update_hud(world);
        self.update_messages(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }

        if key == KeyCode::Escape {
            world.resources.should_exit = true;
            return;
        }

        if self.game_over || self.game_won {
            return;
        }

        if self.phase == GamePhase::PlayerWalking {
            self.phase = GamePhase::PlayerIdle;
            self.walk_path.clear();
            self.pending_attack_target = None;
            return;
        }

        if self.phase != GamePhase::PlayerIdle {
            return;
        }

        match key {
            KeyCode::Char('w') => self.try_keyboard_move(0, -1, world),
            KeyCode::Char('s') => self.try_keyboard_move(0, 1, world),
            KeyCode::Char('a') => self.try_keyboard_move(-1, 0, world),
            KeyCode::Char('d') => self.try_keyboard_move(1, 0, world),
            KeyCode::Char('q') => self.try_keyboard_move(-1, -1, world),
            KeyCode::Char('e') => self.try_keyboard_move(1, -1, world),
            KeyCode::Char('z') => self.try_keyboard_move(-1, 1, world),
            KeyCode::Char('c') => self.try_keyboard_move(1, 1, world),
            KeyCode::Char(' ') => {
                self.messages
                    .add("Turn ended.".to_string(), TermColor::Grey);
                self.begin_enemy_turn();
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
        if !pressed || self.game_over || self.game_won {
            return;
        }

        match button {
            MouseButton::Right => {
                if self.phase == GamePhase::PlayerWalking {
                    self.phase = GamePhase::PlayerIdle;
                    self.walk_path.clear();
                    self.pending_attack_target = None;
                }
                self.scroll_offset_sx = 0.0;
                self.scroll_offset_sy = 0.0;
            }
            MouseButton::Left => {
                if self.phase != GamePhase::PlayerIdle {
                    return;
                }
                let (target_gx, target_gy) =
                    self.screen_to_iso_grid(column as f64, row as f64, world);
                self.try_click_move(target_gx, target_gy);
            }
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;

        self.advance_walk(delta, world);
        self.advance_enemy_animation(delta);
        self.update_camera(world);
        self.despawn_dead_creatures(world);
        self.render_iso_map(world);
        self.update_entity_positions(world);
        self.update_highlight(world);
        self.update_hud(world);
        self.update_messages(world);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.game_over {
            self.clear_all(world);
            return Some(Box::new(GameOverState {
                kills: self.kills,
                entities: EntityGroup::new(),
                restart: false,
            }));
        }
        if self.game_won {
            self.clear_all(world);
            return Some(Box::new(WinState {
                kills: self.kills,
                medkits: self.medkits_used,
                hp_remaining: self.player_hp,
                entities: EntityGroup::new(),
                restart: false,
            }));
        }
        None
    }
}

struct GameOverState {
    kills: u32,
    entities: EntityGroup,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Wasteland - Game Over"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        let terminal = world.resources.terminal_size;
        let center_col = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let kills_text = format!("Enemies defeated: {}", self.kills);
        let lines: Vec<(String, TermColor)> = vec![
            ("YOU HAVE PERISHED".to_string(), TermColor::Red),
            (String::new(), TermColor::Black),
            (
                "The wasteland claims another soul...".to_string(),
                TermColor::DarkYellow,
            ),
            (String::new(), TermColor::Black),
            (kills_text, TermColor::Yellow),
            (String::new(), TermColor::Black),
            ("Press R to try again".to_string(), TermColor::White),
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
                    column: center_col - text.len() as f64 / 2.0,
                    row: center_row - 4.0 + line_index as f64,
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

struct WinState {
    kills: u32,
    medkits: u32,
    hp_remaining: i32,
    entities: EntityGroup,
    restart: bool,
}

impl State for WinState {
    fn title(&self) -> &str {
        "Wasteland - Escaped!"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        let terminal = world.resources.terminal_size;
        let center_col = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let kills_text = format!("Enemies defeated: {}", self.kills);
        let medkits_text = format!("Medkits used: {}", self.medkits);
        let hp_text = format!("HP remaining: {}", self.hp_remaining);
        let lines: Vec<(String, TermColor)> = vec![
            ("YOU ESCAPED THE WASTELAND!".to_string(), TermColor::Green),
            (String::new(), TermColor::Black),
            (kills_text, TermColor::Yellow),
            (medkits_text, TermColor::Cyan),
            (hp_text, TermColor::Green),
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
                    column: center_col - text.len() as f64 / 2.0,
                    row: center_row - 4.0 + line_index as f64,
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
