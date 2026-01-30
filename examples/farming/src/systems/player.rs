use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::{MouseState, TouchGesture};
use nightshade::prelude::*;

use crate::ecs::{CameraMode, World as GameWorld};
use crate::events::CropHarvestedEvent;
use crate::types::{
    GRAVITY, JUMP_VELOCITY, PLAYER_RADIUS, PLAYER_SPEED, STAMINA_REGEN_RATE, TOOL_STAMINA_COST,
    ToolType,
};

pub struct InputState {
    pub movement: Vec3,
    pub action_pressed: bool,
    pub interact_pressed: bool,
    pub jump_pressed: bool,
    pub hotbar_slot: Option<usize>,
    pub sprint_held: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            movement: Vec3::zeros(),
            action_pressed: false,
            interact_pressed: false,
            jump_pressed: false,
            hotbar_slot: None,
            sprint_held: false,
        }
    }
}

pub fn gather_input(game: &GameWorld, world: &mut World) -> InputState {
    let mut input = InputState::default();
    let mut forward: f32 = 0.0;
    let mut right: f32 = 0.0;

    let kb = &world.resources.input.keyboard;
    if kb.is_key_pressed(KeyCode::KeyW) || kb.is_key_pressed(KeyCode::ArrowUp) {
        forward += 1.0;
    }
    if kb.is_key_pressed(KeyCode::KeyS) || kb.is_key_pressed(KeyCode::ArrowDown) {
        forward -= 1.0;
    }
    if kb.is_key_pressed(KeyCode::KeyA) || kb.is_key_pressed(KeyCode::ArrowLeft) {
        right -= 1.0;
    }
    if kb.is_key_pressed(KeyCode::KeyD) || kb.is_key_pressed(KeyCode::ArrowRight) {
        right += 1.0;
    }
    if kb.is_key_pressed(KeyCode::Space) {
        input.jump_pressed = true;
    }
    if kb.is_key_pressed(KeyCode::KeyE) {
        input.interact_pressed = true;
    }
    if kb.is_key_pressed(KeyCode::ShiftLeft) || kb.is_key_pressed(KeyCode::ShiftRight) {
        input.sprint_held = true;
    }

    let hotbar_keys = [
        kb.is_key_pressed(KeyCode::Digit1),
        kb.is_key_pressed(KeyCode::Digit2),
        kb.is_key_pressed(KeyCode::Digit3),
        kb.is_key_pressed(KeyCode::Digit4),
        kb.is_key_pressed(KeyCode::Digit5),
        kb.is_key_pressed(KeyCode::Digit6),
        kb.is_key_pressed(KeyCode::Digit7),
        kb.is_key_pressed(KeyCode::Digit8),
        kb.is_key_pressed(KeyCode::Digit9),
        kb.is_key_pressed(KeyCode::Digit0),
    ];
    for (idx, pressed) in hotbar_keys.iter().enumerate() {
        if *pressed {
            input.hotbar_slot = Some(idx);
        }
    }

    if world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::LEFT_CLICKED)
    {
        input.action_pressed = true;
    }

    if let TouchGesture::SingleDrag { delta } = world.resources.input.touch.gesture {
        right += delta.x * 0.02;
        forward -= delta.y * 0.02;
    }

    if let Some(gamepad) = query_active_gamepad(world) {
        const DEADZONE: f32 = 0.15;
        let lx = gamepad.value(gilrs::Axis::LeftStickX);
        let ly = gamepad.value(gilrs::Axis::LeftStickY);
        if lx.abs() > DEADZONE {
            right += lx;
        }
        if ly.abs() > DEADZONE {
            forward += ly;
        }
        if gamepad.is_pressed(gilrs::Button::DPadUp) {
            forward += 1.0;
        }
        if gamepad.is_pressed(gilrs::Button::DPadDown) {
            forward -= 1.0;
        }
        if gamepad.is_pressed(gilrs::Button::DPadLeft) {
            right -= 1.0;
        }
        if gamepad.is_pressed(gilrs::Button::DPadRight) {
            right += 1.0;
        }
        if gamepad.is_pressed(gilrs::Button::South) {
            input.action_pressed = true;
        }
        if gamepad.is_pressed(gilrs::Button::West) {
            input.interact_pressed = true;
        }
        if gamepad.is_pressed(gilrs::Button::LeftThumb) {
            input.sprint_held = true;
        }
    }

    let mut movement = Vec3::zeros();
    match game.resources.camera_mode {
        CameraMode::TopDown => {
            movement.x = right;
            movement.z = -forward;
        }
        CameraMode::ThirdPerson => {
            let yaw = game.resources.camera_yaw;
            let forward_x = -yaw.sin();
            let forward_z = -yaw.cos();
            let right_x = yaw.cos();
            let right_z = -yaw.sin();
            movement.x = forward * forward_x + right * right_x;
            movement.z = forward * forward_z + right * right_z;
        }
    }
    if nalgebra_glm::length(&movement) > 1.0 {
        movement = nalgebra_glm::normalize(&movement);
    }
    input.movement = movement;
    input
}

pub fn update_movement(game: &mut GameWorld, world: &mut World, input: &InputState) {
    let delta = world.resources.window.timing.delta_time;
    let Some(player_entity) = game.resources.player_entity else {
        return;
    };

    let Some(player) = game.get_player(player_entity) else {
        return;
    };

    let grounded = player.grounded;
    let mut vertical_velocity = player.vertical_velocity;
    let mut height = player.height;
    let mut facing = player.facing;

    if input.jump_pressed && grounded {
        vertical_velocity = JUMP_VELOCITY;
    }

    vertical_velocity -= GRAVITY * delta;
    height += vertical_velocity * delta;

    let mut new_grounded = grounded;
    if height <= 0.0 {
        height = 0.0;
        vertical_velocity = 0.0;
        new_grounded = true;
    } else if input.jump_pressed && grounded {
        new_grounded = false;
    }

    let speed = if input.sprint_held {
        PLAYER_SPEED * 1.5
    } else {
        PLAYER_SPEED
    };
    let velocity = input.movement * speed;

    if nalgebra_glm::length(&velocity) > 0.01 {
        facing = nalgebra_glm::normalize(&velocity);
    }

    let equipped_tool = if let Some(slot) = input.hotbar_slot {
        match slot {
            0 => ToolType::Hoe,
            1 => ToolType::WateringCan,
            2 => ToolType::Axe,
            3 => ToolType::Pickaxe,
            4 => ToolType::Scythe,
            5 => ToolType::Sword,
            _ => ToolType::Hand,
        }
    } else {
        player.equipped_tool
    };

    game.modify_player(player_entity, |p| {
        p.vertical_velocity = vertical_velocity;
        p.height = height;
        p.grounded = new_grounded;
        p.facing = facing;
        p.equipped_tool = equipped_tool;
    });

    if let Some(slot) = input.hotbar_slot {
        game.resources.inventory.selected_slot = slot;
    }

    let current_pos = game
        .get_position(player_entity)
        .map(|p| p.0)
        .unwrap_or(Vec3::zeros());

    let new_position = if nalgebra_glm::length(&velocity) > 0.01 {
        Vec3::new(
            current_pos.x + velocity.x * delta,
            PLAYER_RADIUS + height,
            current_pos.z + velocity.z * delta,
        )
    } else {
        Vec3::new(current_pos.x, PLAYER_RADIUS + height, current_pos.z)
    };

    game.modify_position(player_entity, |pos| pos.0 = new_position);

    if let Some(visual) = game.resources.visuals.player_visual {
        if let Some(transform) = world.get_local_transform_mut(visual) {
            transform.translation = new_position;
        }
        mark_local_transform_dirty(world, visual);
    }
}

pub fn update_stamina(game: &mut GameWorld, world: &World) {
    let delta = world.resources.window.timing.delta_time;
    let Some(player_entity) = game.resources.player_entity else {
        return;
    };

    let (stamina, max_stamina) = game
        .get_player(player_entity)
        .map(|p| (p.stamina, p.max_stamina))
        .unwrap_or((0.0, 0.0));

    if stamina < max_stamina {
        let new_stamina = (stamina + STAMINA_REGEN_RATE * delta).min(max_stamina);
        game.modify_player(player_entity, |p| p.stamina = new_stamina);
    }
}

pub fn update_cooldowns(game: &mut GameWorld, world: &World) {
    let delta = world.resources.window.timing.delta_time;
    let Some(player_entity) = game.resources.player_entity else {
        return;
    };

    game.modify_player(player_entity, |p| {
        p.attack_cooldown = (p.attack_cooldown - delta).max(0.0);
        p.interaction_cooldown = (p.interaction_cooldown - delta).max(0.0);
    });
}

pub fn can_interact(game: &GameWorld) -> bool {
    game.resources
        .player_entity
        .and_then(|e| game.get_player(e))
        .map(|p| p.interaction_cooldown <= 0.0)
        .unwrap_or(false)
}

pub fn set_interaction_cooldown(game: &mut GameWorld, cooldown: f32) {
    if let Some(player_entity) = game.resources.player_entity {
        game.modify_player(player_entity, |p| {
            p.interaction_cooldown = cooldown;
        });
    }
}

pub fn try_use_tool(game: &mut GameWorld, world: &mut World) -> Option<CropHarvestedEvent> {
    let player_entity = game.resources.player_entity?;
    let player = game.get_player(player_entity)?;

    if player.stamina < TOOL_STAMINA_COST || player.attack_cooldown > 0.0 {
        return None;
    }

    let equipped_tool = player.equipped_tool;

    let (success, event) = match equipped_tool {
        ToolType::Axe => (crate::systems::trees::try_chop(game), None),
        ToolType::Hoe | ToolType::WateringCan | ToolType::Scythe | ToolType::Hand => {
            crate::systems::farming::try_use_tool(game, world)
        }
        _ => (false, None),
    };

    if success {
        game.modify_player(player_entity, |p| p.attack_cooldown = 0.3);
    }

    event
}

pub fn get_player_position(game: &GameWorld) -> Vec3 {
    game.resources
        .player_entity
        .and_then(|e| game.get_position(e))
        .map(|p| p.0)
        .unwrap_or(Vec3::zeros())
}

pub fn get_player_facing(game: &GameWorld) -> Vec3 {
    game.resources
        .player_entity
        .and_then(|e| game.get_player(e))
        .map(|p| p.facing)
        .unwrap_or(Vec3::new(0.0, 0.0, 1.0))
}

pub fn get_equipped_tool(game: &GameWorld) -> ToolType {
    game.resources
        .player_entity
        .and_then(|e| game.get_player(e))
        .map(|p| p.equipped_tool)
        .unwrap_or(ToolType::Hand)
}
