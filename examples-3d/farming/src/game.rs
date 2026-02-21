use nightshade::prelude::*;

use crate::ecs::{CameraMode, ShopMode, World as GameWorld};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GamePhase {
    #[default]
    MainMenu,
    Playing,
    Paused,
}

pub enum InputAction {
    None,
    ToggleCamera,
    Pause,
    Resume,
    StartGame,
}

pub enum ShopAction {
    None,
    Close,
    SelectUp,
    SelectDown,
    ToggleMode,
    Transact,
}

pub fn handle_shop_input(key: KeyCode) -> ShopAction {
    match key {
        KeyCode::Escape => ShopAction::Close,
        KeyCode::KeyW | KeyCode::ArrowUp => ShopAction::SelectUp,
        KeyCode::KeyS | KeyCode::ArrowDown => ShopAction::SelectDown,
        KeyCode::KeyQ => ShopAction::ToggleMode,
        KeyCode::KeyE | KeyCode::Enter => ShopAction::Transact,
        _ => ShopAction::None,
    }
}

pub fn handle_phase_input(key: KeyCode, phase: GamePhase, in_dialogue: bool) -> InputAction {
    match (key, phase) {
        (KeyCode::Escape, GamePhase::Playing) => InputAction::Pause,
        (KeyCode::Escape, GamePhase::Paused) => InputAction::Resume,
        (KeyCode::Enter | KeyCode::Space, GamePhase::MainMenu) => InputAction::StartGame,
        (KeyCode::Tab, GamePhase::Playing) if !in_dialogue => InputAction::ToggleCamera,
        _ => InputAction::None,
    }
}

pub fn apply_shop_action(game: &mut GameWorld, action: ShopAction) {
    match action {
        ShopAction::Close => {
            game.resources.shop = None;
        }
        ShopAction::SelectUp => {
            if let Some(shop) = &mut game.resources.shop
                && shop.selected > 0
            {
                shop.selected -= 1;
            }
        }
        ShopAction::SelectDown => {
            if let Some(shop) = &mut game.resources.shop
                && shop.selected + 1 < game.resources.shop_items.len()
            {
                shop.selected += 1;
            }
        }
        ShopAction::ToggleMode => {
            if let Some(shop) = &mut game.resources.shop {
                shop.mode = match shop.mode {
                    ShopMode::Buy => ShopMode::Sell,
                    ShopMode::Sell => ShopMode::Buy,
                };
            }
        }
        ShopAction::Transact | ShopAction::None => {}
    }
}

pub fn toggle_camera_mode(game: &mut GameWorld) {
    game.resources.camera_mode = match game.resources.camera_mode {
        CameraMode::TopDown => CameraMode::ThirdPerson,
        CameraMode::ThirdPerson => CameraMode::TopDown,
    };
}
