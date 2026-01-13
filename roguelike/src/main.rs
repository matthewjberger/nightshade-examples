mod combat;
mod display;
mod dungeon;
mod ecs;
mod entities;
mod fov;
mod systems;

use display::{DisplayState, spawn_display, update_display};
use dungeon::{find_random_floor, find_random_floor_away_from, generate_dungeon};
use ecs::{EnemyType, FovMap, GameState, GameWorld, Inventory, ItemType};
use entities::{spawn_enemy, spawn_item, spawn_player};
use fov::compute_fov;
use nightshade::prelude::rand::prelude::*;
use nightshade::prelude::{SystemTime, UNIX_EPOCH, *};
use systems::{PlayerAction, check_victory, run_enemy_turns, try_player_action};

const MAP_WIDTH: i32 = 60;
const MAP_HEIGHT: i32 = 30;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(RoguelikeGame::new())?;
    Ok(())
}

struct RoguelikeGame {
    game_world: GameWorld,
    display: DisplayState,
    initialized: bool,
}

impl RoguelikeGame {
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

        self.generate_level();

        self.game_world
            .resources
            .message_log
            .push("Welcome to the dungeon! Find the stairs to descend.".to_string());
        self.game_world
            .resources
            .message_log
            .push("Use WASD or arrows to move, G to grab items, P for potions.".to_string());
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
        let enemy_count = 3 + depth * 2;

        for _ in 0..enemy_count {
            if let Some((x, y)) = find_random_floor_away_from(
                &self.game_world.resources.map,
                &mut rng,
                player_x,
                player_y,
                5,
            ) {
                let enemy_type = match rng.random_range(0..100) {
                    0..50 => EnemyType::Goblin,
                    50..80 => EnemyType::Orc,
                    _ => EnemyType::Troll,
                };
                spawn_enemy(&mut self.game_world, x, y, enemy_type);
            }
        }

        let item_count = 2 + rng.random_range(0..3);

        for _ in 0..item_count {
            if let Some((x, y)) = find_random_floor_away_from(
                &self.game_world.resources.map,
                &mut rng,
                player_x,
                player_y,
                3,
            ) {
                let item_type = match rng.random_range(0..100) {
                    0..60 => ItemType::HealthPotion,
                    60..80 => ItemType::Sword,
                    _ => ItemType::Shield,
                };
                spawn_item(&mut self.game_world, x, y, item_type);
            }
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
            if last_msg.contains("descend to level") {
                self.generate_level();
            }
        }
    }
}

impl State for RoguelikeGame {
    fn title(&self) -> &str {
        "Roguelike Dungeon Crawler"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.clear_color = [0.05, 0.05, 0.1, 1.0];

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
                    KeyCode::KeyP | KeyCode::Digit1 => Some(PlayerAction::UsePotion),
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
