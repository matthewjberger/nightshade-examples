mod data;
mod ecs;
mod events;
mod game;
mod systems;
mod types;

use ecs::World as GameWorld;
use game::{
    GamePhase, InputAction, ShopAction, apply_shop_action, handle_phase_input, handle_shop_input,
    toggle_camera_mode,
};
use nightshade::prelude::*;
use systems::{camera, farming, init, player, social, terrain, time, trees, ui, visuals};

const SKY_HDR: &[u8] = include_bytes!("../../../assets/sky/moonrise.hdr");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(FarmingGame::default())
}

#[derive(Default)]
struct FarmingGame {
    game: GameWorld,
    phase: GamePhase,
    ui: ui::FarmingUi,
}

impl FarmingGame {
    fn check_and_recreate_visuals(&mut self, world: &mut World) {
        let needs_recreate = match self.game.resources.visuals.camera {
            Some(camera) => world.get_local_transform(camera).is_none(),
            None => true,
        };

        if needs_recreate {
            init::recreate_visuals(&mut self.game, world);
        }
    }

    fn handle_input_action(&mut self, action: InputAction) {
        match action {
            InputAction::Pause => self.phase = GamePhase::Paused,
            InputAction::Resume => self.phase = GamePhase::Playing,
            InputAction::StartGame => self.phase = GamePhase::Playing,
            InputAction::ToggleCamera => toggle_camera_mode(&mut self.game),
            InputAction::None => {}
        }
    }

    fn run_input_phase(&mut self, world: &mut World) -> player::InputState {
        let input = player::gather_input(&self.game, world);
        trees::update_target(&mut self.game);
        camera::update_rotation(&mut self.game, world);
        input
    }

    fn run_simulation_phase(&mut self, world: &mut World, input: &player::InputState) {
        let in_shop = self.game.resources.shop.is_some();
        let in_dialogue = self.game.resources.dialogue.is_some();

        if !in_shop && !in_dialogue {
            if input.action_pressed
                && let Some(event) = player::try_use_tool(&mut self.game, world)
            {
                visuals::spawn_popup(
                    &mut self.game,
                    world,
                    event.position,
                    &format!("+1 {}", event.item_name),
                );
            }
            if input.interact_pressed
                && player::can_interact(&self.game)
                && social::try_interact(&mut self.game)
            {
                player::set_interaction_cooldown(&mut self.game, 0.3);
            }
            player::update_movement(&mut self.game, world, input);
        } else if in_dialogue && input.interact_pressed && player::can_interact(&self.game) {
            social::advance_dialogue(&mut self.game);
            player::set_interaction_cooldown(&mut self.game, 0.2);
        }

        player::update_stamina(&mut self.game, world);
        player::update_cooldowns(&mut self.game, world);

        if let Some(event) = time::advance(&mut self.game, world) {
            farming::process_day_change(&mut self.game, world);
            social::reset_daily_flags(&mut self.game);

            let player_pos = player::get_player_position(&self.game);
            let message = if let Some(season) = event.new_season {
                format!("Day {} - {:?}", event.new_day, season)
            } else {
                format!("Day {}", event.new_day)
            };
            visuals::spawn_popup(&mut self.game, world, player_pos, &message);
        }

        terrain::update_chunks(&mut self.game, world);

        for event in trees::update(&mut self.game, world) {
            visuals::spawn_popup(
                &mut self.game,
                world,
                event.position,
                &format!("+{} Wood", event.wood),
            );
        }
    }

    fn run_visual_phase(&mut self, world: &mut World) {
        camera::update(&self.game, world);
        visuals::update_tool(&self.game, world);
        time::update_sun(&self.game, world);
        terrain::update_grass(&self.game, world);
        terrain::update_ground(&self.game, world);
        visuals::update_popups(&mut self.game, world);
        nightshade::ecs::text::systems::sync_text_meshes_system(world);
    }

    fn run_playing_systems(&mut self, world: &mut World) {
        let input = self.run_input_phase(world);
        self.run_simulation_phase(world, &input);
        self.run_visual_phase(world);
    }
}

impl State for FarmingGame {
    fn title(&self) -> &str {
        "Meadow Fields"
    }

    fn initialize(&mut self, world: &mut World) {
        load_hdr_skybox(world, SKY_HDR.to_vec());
        init::initialize(&mut self.game, world);
        self.ui.build(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        self.check_and_recreate_visuals(world);
        visuals::update_popups(&mut self.game, world);

        match self.phase {
            GamePhase::Playing => self.run_playing_systems(world),
            _ => camera::update(&self.game, world),
        }

        if let Some(new_phase) = self.ui.update(&self.game, self.phase, world) {
            self.phase = new_phase;
        }
    }

    fn on_keyboard_input(&mut self, _world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        if self.game.resources.shop.is_some() {
            let action = handle_shop_input(key);
            if matches!(action, ShopAction::Transact) {
                social::try_shop_transaction(&mut self.game);
            } else {
                apply_shop_action(&mut self.game, action);
            }
            return;
        }

        let in_dialogue = self.game.resources.dialogue.is_some();
        let action = handle_phase_input(key, self.phase, in_dialogue);
        self.handle_input_action(action);
    }

    fn on_gamepad_event(&mut self, _world: &mut World, event: gilrs::Event) {
        let gilrs::EventType::ButtonPressed(button, _) = event.event else {
            return;
        };

        match (button, self.phase) {
            (gilrs::Button::Start | gilrs::Button::South, GamePhase::MainMenu) => {
                self.phase = GamePhase::Playing;
            }
            (gilrs::Button::Start, GamePhase::Playing) => {
                self.phase = GamePhase::Paused;
            }
            (gilrs::Button::Start, GamePhase::Paused) => {
                self.phase = GamePhase::Playing;
            }
            (gilrs::Button::Select, GamePhase::Playing) => {
                toggle_camera_mode(&mut self.game);
            }
            _ => {}
        }
    }
}
