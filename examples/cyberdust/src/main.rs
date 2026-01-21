mod combat;
mod display;
mod dungeon;
mod ecs;
mod entities;
mod fov;
mod systems;

use display::{DisplayState, spawn_display, update_display};
use dungeon::{find_random_floor, find_random_floor_away_from, generate_dungeon};
use ecs::{EnemyType, FovMap, GameState, GameStats, GameWorld, Inventory, ItemType};
use entities::{spawn_enemy, spawn_item, spawn_player};
use fov::compute_fov;
use nightshade::prelude::rand::prelude::*;
use nightshade::prelude::{SystemTime, UNIX_EPOCH, *};
use systems::{PlayerAction, check_victory, run_enemy_turns, try_player_action};

const MAP_WIDTH: i32 = 60;
const MAP_HEIGHT: i32 = 30;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(CyberdustGame::new())?;
    Ok(())
}

struct CyberdustGame {
    game_world: GameWorld,
    display: DisplayState,
    initialized: bool,
}

impl CyberdustGame {
    fn new() -> Self {
        Self {
            game_world: GameWorld::default(),
            display: DisplayState::default(),
            initialized: false,
        }
    }

    fn new_game(&mut self) {
        self.game_world.resources.rng_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(12345);

        self.game_world.resources.current_depth = 1;
        self.game_world.resources.game_state = GameState::Playing;
        self.game_world.resources.message_log.clear();
        self.game_world.resources.inventory = Inventory::default();
        self.game_world.resources.stats = GameStats::default();

        self.generate_level();

        self.game_world.resources.message_log.push(
            "Welcome to the neon underground. Find the data port to jack deeper.".to_string(),
        );
        self.game_world
            .resources
            .message_log
            .push("WASD: move | G: grab | P: stim | E: EMP | Shift+>: jack in".to_string());
    }

    fn generate_level(&mut self) {
        let seed =
            self.game_world.resources.rng_seed + self.game_world.resources.current_depth as u64;
        let mut rng = StdRng::seed_from_u64(seed);

        let entities_to_despawn: Vec<_> = self.game_world.query_entities(ecs::POSITION).collect();

        self.game_world.despawn_entities(&entities_to_despawn);

        self.game_world.resources.player_entity = None;

        self.game_world.resources.map = generate_dungeon(MAP_WIDTH, MAP_HEIGHT, seed);
        self.game_world.resources.fov_map = FovMap::new(MAP_WIDTH, MAP_HEIGHT);

        let Some((player_x, player_y)) =
            find_random_floor(&self.game_world.resources.map, &mut rng)
        else {
            return;
        };

        spawn_player(&mut self.game_world, player_x, player_y);

        compute_fov(
            &mut self.game_world.resources.fov_map,
            &self.game_world.resources.map,
            player_x,
            player_y,
        );

        let depth = self.game_world.resources.current_depth;
        if depth > self.game_world.resources.stats.max_depth_reached {
            self.game_world.resources.stats.max_depth_reached = depth;
        }

        let enemy_count = 4 + depth * 2;

        for _ in 0..enemy_count {
            if let Some((x, y)) = find_random_floor_away_from(
                &self.game_world.resources.map,
                &mut rng,
                player_x,
                player_y,
                5,
            ) {
                let enemy_type = self.roll_enemy_type(&mut rng, depth);
                spawn_enemy(&mut self.game_world, x, y, enemy_type);
            }
        }

        let item_count = 3 + rng.random_range(0..3);

        for _ in 0..item_count {
            if let Some((x, y)) = find_random_floor_away_from(
                &self.game_world.resources.map,
                &mut rng,
                player_x,
                player_y,
                3,
            ) {
                let item_type = self.roll_item_type(&mut rng, depth);
                spawn_item(&mut self.game_world, x, y, item_type);
            }
        }
    }

    fn roll_enemy_type(&self, rng: &mut StdRng, depth: u32) -> EnemyType {
        let roll = rng.random_range(0..100);
        match depth {
            1 => match roll {
                0..70 => EnemyType::StreetPunk,
                70..90 => EnemyType::Drone,
                _ => EnemyType::CorpoGuard,
            },
            2 => match roll {
                0..40 => EnemyType::StreetPunk,
                40..60 => EnemyType::Drone,
                60..85 => EnemyType::CorpoGuard,
                _ => EnemyType::Netrunner,
            },
            3 => match roll {
                0..25 => EnemyType::StreetPunk,
                25..40 => EnemyType::Drone,
                40..65 => EnemyType::CorpoGuard,
                65..85 => EnemyType::Netrunner,
                _ => EnemyType::Cyborg,
            },
            4 => match roll {
                0..15 => EnemyType::StreetPunk,
                15..30 => EnemyType::Drone,
                30..50 => EnemyType::CorpoGuard,
                50..75 => EnemyType::Netrunner,
                _ => EnemyType::Cyborg,
            },
            _ => match roll {
                0..10 => EnemyType::StreetPunk,
                10..20 => EnemyType::Drone,
                20..40 => EnemyType::CorpoGuard,
                40..65 => EnemyType::Netrunner,
                _ => EnemyType::Cyborg,
            },
        }
    }

    fn roll_item_type(&self, rng: &mut StdRng, depth: u32) -> ItemType {
        let roll = rng.random_range(0..100);
        match depth {
            1 => match roll {
                0..50 => ItemType::StimPack,
                50..70 => ItemType::CredChip,
                70..85 => ItemType::Katana,
                _ => ItemType::CyberArmor,
            },
            2..=3 => match roll {
                0..40 => ItemType::StimPack,
                40..55 => ItemType::CredChip,
                55..70 => ItemType::Katana,
                70..85 => ItemType::CyberArmor,
                85..95 => ItemType::EmpGrenade,
                _ => ItemType::NeuralImplant,
            },
            _ => match roll {
                0..35 => ItemType::StimPack,
                35..50 => ItemType::CredChip,
                50..60 => ItemType::Katana,
                60..70 => ItemType::CyberArmor,
                70..85 => ItemType::EmpGrenade,
                _ => ItemType::NeuralImplant,
            },
        }
    }

    fn handle_player_turn(&mut self, action: PlayerAction) {
        if try_player_action(&mut self.game_world, action)
            && matches!(self.game_world.resources.game_state, GameState::Playing)
        {
            run_enemy_turns(&mut self.game_world);
            check_victory(&mut self.game_world);
        }
    }

    fn check_descend(&mut self) {
        let msg_count = self.game_world.resources.message_log.len();
        if msg_count > 0 {
            let last_msg = &self.game_world.resources.message_log[msg_count - 1];
            if last_msg.contains("jack into level") {
                self.generate_level();
            }
        }
    }
}

impl State for CyberdustGame {
    fn title(&self) -> &str {
        "CYBERDUST // Neon Runner"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.clear_color = [0.02, 0.0, 0.05, 1.0];

        let camera_position = Vec3::new(0.0, 0.0, 5.0);
        let main_camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(main_camera);

        self.display = spawn_display(world);

        self.new_game();
        self.initialized = true;
    }

    fn run_systems(&mut self, world: &mut World) {
        if !self.initialized {
            return;
        }

        self.check_descend();
        update_display(&self.display, &self.game_world, world);
        sync_text_meshes_system(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if !matches!(state, KeyState::Pressed) {
            return;
        }

        match self.game_world.resources.game_state {
            GameState::Playing => {
                let action = match key {
                    KeyCode::KeyW | KeyCode::ArrowUp => Some(PlayerAction::Move { dx: 0, dy: -1 }),
                    KeyCode::KeyS | KeyCode::ArrowDown => Some(PlayerAction::Move { dx: 0, dy: 1 }),
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        Some(PlayerAction::Move { dx: -1, dy: 0 })
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        Some(PlayerAction::Move { dx: 1, dy: 0 })
                    }
                    KeyCode::Space => Some(PlayerAction::Wait),
                    KeyCode::KeyG | KeyCode::Comma => Some(PlayerAction::PickupItem),
                    KeyCode::KeyP | KeyCode::Digit1 => Some(PlayerAction::UseStim),
                    KeyCode::KeyE | KeyCode::Digit2 => Some(PlayerAction::UseEmp),
                    KeyCode::Period => {
                        if world
                            .resources
                            .input
                            .keyboard
                            .is_key_pressed(KeyCode::ShiftLeft)
                            || world
                                .resources
                                .input
                                .keyboard
                                .is_key_pressed(KeyCode::ShiftRight)
                        {
                            Some(PlayerAction::Descend)
                        } else {
                            Some(PlayerAction::Wait)
                        }
                    }
                    KeyCode::Escape => {
                        world.resources.window.should_exit = true;
                        None
                    }
                    _ => None,
                };

                if let Some(action) = action {
                    self.handle_player_turn(action);
                }
            }
            GameState::PlayerDead | GameState::Victory => match key {
                KeyCode::KeyR => {
                    self.new_game();
                }
                KeyCode::Escape => {
                    world.resources.window.should_exit = true;
                }
                _ => {}
            },
        }
    }
}
