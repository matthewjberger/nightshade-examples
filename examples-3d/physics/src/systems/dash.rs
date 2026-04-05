use crate::ecs::{GameWorld, PlayerEvent, PlayerState};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::prelude::*;

pub fn dash_system(game_world: &mut GameWorld, world: &mut World) {
    let Some(player_entity) = game_world.resources.player_entity else {
        return;
    };
    let Some(camera_entity) = game_world.resources.camera_entity else {
        return;
    };

    let grounded = world
        .core
        .get_character_controller(player_entity)
        .is_some_and(|controller| controller.grounded);

    let was_grounded_state = matches!(
        game_world.resources.player_state,
        PlayerState::Grounded
            | PlayerState::GroundDash
            | PlayerState::LeaningLeft
            | PlayerState::LeaningRight
            | PlayerState::Sliding
    );

    if grounded && !was_grounded_state {
        let horizontal_speed = world
            .core
            .get_character_controller(player_entity)
            .map(|controller| {
                nalgebra_glm::length(&nalgebra_glm::vec3(
                    controller.velocity.x,
                    0.0,
                    controller.velocity.z,
                ))
            })
            .unwrap_or(0.0);

        let landed_fast = horizontal_speed > game_world.resources.config.slide_min_speed;
        let land_event = if landed_fast {
            PlayerEvent::SlideLand
        } else {
            PlayerEvent::Land
        };

        if let Some(new_state) = game_world
            .resources
            .player_state
            .process_event(land_event)
        {
            game_world.resources.player_state = new_state;
        }
    } else if !grounded
        && matches!(
            game_world.resources.player_state,
            PlayerState::Grounded | PlayerState::LeaningLeft | PlayerState::LeaningRight
        )
    {
        if let Some(new_state) = game_world
            .resources
            .player_state
            .process_event(PlayerEvent::Jump)
        {
            game_world.resources.player_state = new_state;
        }
    } else if !grounded && game_world.resources.player_state == PlayerState::Sliding {
        if let Some(new_state) = game_world
            .resources
            .player_state
            .process_event(PlayerEvent::BecomeAirborne)
        {
            game_world.resources.player_state = new_state;
            if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
                controller.is_crouching = false;
            }
        }
    } else if !grounded && game_world.resources.player_state == PlayerState::GroundDash
        && let Some(new_state) = game_world
            .resources
            .player_state
            .process_event(PlayerEvent::BecomeAirborne)
    {
        game_world.resources.player_state = new_state;
    }

    let gamepad_dash_pressed = if let Some(gamepad) = query_active_gamepad(world) {
        gamepad.is_pressed(gilrs::Button::East)
    } else {
        false
    };
    let gamepad_dash_just_pressed =
        gamepad_dash_pressed && !game_world.resources.dash_button_was_pressed;
    game_world.resources.dash_button_was_pressed = gamepad_dash_pressed;

    let keyboard = &world.resources.input.keyboard;
    let current_time_ms = world.resources.window.timing.uptime_milliseconds;
    let double_tap_window_ms = 250;

    let movement_keys = [
        (KeyCode::KeyW, nalgebra_glm::vec3(0.0, 0.0, -1.0)),
        (KeyCode::KeyS, nalgebra_glm::vec3(0.0, 0.0, 1.0)),
        (KeyCode::KeyA, nalgebra_glm::vec3(-1.0, 0.0, 0.0)),
        (KeyCode::KeyD, nalgebra_glm::vec3(1.0, 0.0, 0.0)),
    ];

    let mut keyboard_dash_direction = None;
    let any_movement_pressed = movement_keys
        .iter()
        .any(|(key, _)| keyboard.is_key_pressed(*key));

    if !any_movement_pressed {
        game_world.resources.key_was_released = true;
    }

    for &(key, ref local_direction) in &movement_keys {
        if keyboard.is_key_pressed(key) && game_world.resources.key_was_released {
            if game_world.resources.last_tap_key == Some(key)
                && current_time_ms.saturating_sub(game_world.resources.last_tap_time_ms)
                    < double_tap_window_ms
            {
                let camera_rotation = world
                    .core
                    .get_local_transform(camera_entity)
                    .map(|transform| transform.rotation)
                    .unwrap_or(nalgebra_glm::quat_identity());
                let world_direction =
                    nalgebra_glm::quat_rotate_vec3(&camera_rotation, local_direction);
                keyboard_dash_direction = Some(nalgebra_glm::normalize(&nalgebra_glm::vec3(
                    world_direction.x,
                    0.0,
                    world_direction.z,
                )));
                game_world.resources.last_tap_key = None;
            } else {
                game_world.resources.last_tap_key = Some(key);
                game_world.resources.last_tap_time_ms = current_time_ms;
            }
            game_world.resources.key_was_released = false;
            break;
        }
    }

    let jump_pressed = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::Space)
        || query_active_gamepad(world)
            .is_some_and(|gamepad| gamepad.is_pressed(gilrs::Button::South));
    let jump_just_pressed = jump_pressed && !game_world.resources.jump_button_was_pressed;
    game_world.resources.jump_button_was_pressed = jump_pressed;

    if jump_just_pressed {
        let player_state = game_world.resources.player_state;
        let is_airborne = matches!(
            player_state,
            PlayerState::Airborne
                | PlayerState::DoubleJumped
                | PlayerState::AirDash
                | PlayerState::Falling
        );

        if is_airborne {
            let jumped =
                if let Some(new_state) = player_state.process_event(PlayerEvent::DoubleJump) {
                    game_world.resources.player_state = new_state;
                    true
                } else if let Some(new_state) = player_state.process_event(PlayerEvent::Jump) {
                    game_world.resources.player_state = new_state;
                    true
                } else {
                    false
                };

            if jumped
                && let Some(controller) = world.core.get_character_controller_mut(player_entity)
            {
                controller.velocity.y = game_world.resources.config.double_jump_impulse;
            }
        }
    }

    let slide_pressed = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::KeyC)
        || query_active_gamepad(world)
            .is_some_and(|gamepad| gamepad.is_pressed(gilrs::Button::LeftThumb));
    let slide_just_pressed = slide_pressed && !game_world.resources.slide_button_was_pressed;
    game_world.resources.slide_button_was_pressed = slide_pressed;

    if slide_just_pressed && game_world.resources.player_state == PlayerState::Grounded {
        let is_sprinting = world
            .core
            .get_character_controller(player_entity)
            .is_some_and(|controller| controller.is_sprinting);
        let horizontal_speed = world
            .core
            .get_character_controller(player_entity)
            .map(|controller| {
                nalgebra_glm::length(&nalgebra_glm::vec3(
                    controller.velocity.x,
                    0.0,
                    controller.velocity.z,
                ))
            })
            .unwrap_or(0.0);

        if (is_sprinting || horizontal_speed > game_world.resources.config.slide_min_speed)
            && let Some(new_state) = game_world
                .resources
                .player_state
                .process_event(PlayerEvent::Slide)
        {
            game_world.resources.player_state = new_state;
            if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
                controller.is_crouching = true;
            }
        }
    }

    if game_world.resources.player_state == PlayerState::Sliding {
        let config = &game_world.resources.config;
        let slide_friction = config.slide_friction;
        let slide_min_speed = config.slide_min_speed;
        let delta_time = world.resources.window.timing.delta_time;

        if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
            let friction = (-slide_friction * delta_time).exp();
            controller.velocity.x *= friction;
            controller.velocity.z *= friction;
            controller.is_crouching = true;

            let horizontal_speed = nalgebra_glm::length(&nalgebra_glm::vec3(
                controller.velocity.x,
                0.0,
                controller.velocity.z,
            ));

            if horizontal_speed < slide_min_speed && !slide_pressed
                && let Some(new_state) = game_world
                    .resources
                    .player_state
                    .process_event(PlayerEvent::Release)
            {
                game_world.resources.player_state = new_state;
                controller.is_crouching = false;
            }
        }
    }

    let dash_triggered = gamepad_dash_just_pressed || keyboard_dash_direction.is_some();

    if dash_triggered
        && game_world.resources.dash_charges > 0
        && let Some(new_state) = game_world
            .resources
            .player_state
            .process_event(PlayerEvent::Dash)
    {
        game_world.resources.dash_charges -= 1;
        game_world.resources.dash_cooldown_timer = game_world.resources.config.dash_cooldown;
        game_world.resources.player_state = new_state;

        let dash_direction = if let Some(direction) = keyboard_dash_direction {
            direction
        } else if let Some(gamepad) = query_active_gamepad(world) {
            let stick_x = gamepad.value(gilrs::Axis::LeftStickX);
            let stick_y = gamepad.value(gilrs::Axis::LeftStickY);
            let stick_magnitude = (stick_x * stick_x + stick_y * stick_y).sqrt();
            if stick_magnitude > 0.3 {
                let camera_rotation = world
                    .core
                    .get_local_transform(camera_entity)
                    .map(|transform| transform.rotation)
                    .unwrap_or(nalgebra_glm::quat_identity());
                let local_direction = nalgebra_glm::vec3(stick_x, 0.0, -stick_y);
                let world_direction =
                    nalgebra_glm::quat_rotate_vec3(&camera_rotation, &local_direction);
                nalgebra_glm::normalize(&nalgebra_glm::vec3(
                    world_direction.x,
                    0.0,
                    world_direction.z,
                ))
            } else {
                let forward = world
                    .core
                    .get_local_transform(camera_entity)
                    .map(|transform| transform.forward_vector())
                    .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, -1.0));
                nalgebra_glm::normalize(&nalgebra_glm::vec3(forward.x, 0.0, forward.z))
            }
        } else {
            let forward = world
                .core
                .get_local_transform(camera_entity)
                .map(|transform| transform.forward_vector())
                .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, -1.0));
            nalgebra_glm::normalize(&nalgebra_glm::vec3(forward.x, 0.0, forward.z))
        };

        let config = &game_world.resources.config;
        let is_air_dash = new_state == PlayerState::AirDash;
        let impulse = if is_air_dash {
            config.dash_air_impulse
        } else {
            config.dash_impulse
        };

        if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
            controller.velocity.x = dash_direction.x * impulse;
            controller.velocity.z = dash_direction.z * impulse;
            if is_air_dash {
                controller.velocity.y = controller.velocity.y.max(1.0);
            }
        }
    }

    if matches!(
        game_world.resources.player_state,
        PlayerState::GroundDash | PlayerState::AirDash
    ) {
        if grounded && game_world.resources.player_state == PlayerState::GroundDash {
            if let Some(new_state) = game_world
                .resources
                .player_state
                .process_event(PlayerEvent::Land)
            {
                game_world.resources.player_state = new_state;
            }
        } else if game_world.resources.player_state == PlayerState::AirDash {
            let speed = world
                .core
                .get_character_controller(player_entity)
                .map(|controller| {
                    nalgebra_glm::length(&nalgebra_glm::vec3(
                        controller.velocity.x,
                        0.0,
                        controller.velocity.z,
                    ))
                })
                .unwrap_or(0.0);
            if speed < 3.0
                && let Some(new_state) = game_world
                    .resources
                    .player_state
                    .process_event(PlayerEvent::DashEnd)
            {
                game_world.resources.player_state = new_state;
            }
        }
    }

    let delta_time = world.resources.window.timing.delta_time;
    let max_dash_charges = game_world.resources.config.max_dash_charges;
    let dash_cooldown = game_world.resources.config.dash_cooldown;
    if game_world.resources.dash_charges < max_dash_charges {
        game_world.resources.dash_cooldown_timer -= delta_time;
        if game_world.resources.dash_cooldown_timer <= 0.0 {
            game_world.resources.dash_charges += 1;
            if game_world.resources.dash_charges < max_dash_charges {
                game_world.resources.dash_cooldown_timer = dash_cooldown;
            }
        }
    }

    update_dash_hud(game_world, world);
}

fn update_dash_hud(game_world: &mut GameWorld, world: &mut World) {
    if let Some(state_text) = game_world.resources.dash_hud_state_text_entity {
        let label = match game_world.resources.player_state {
            PlayerState::Grounded => "GROUNDED",
            PlayerState::LeaningLeft => "LEAN LEFT",
            PlayerState::LeaningRight => "LEAN RIGHT",
            PlayerState::Sliding => "SLIDING",
            PlayerState::GroundDash => "DASH",
            PlayerState::Airborne => "AIRBORNE",
            PlayerState::DoubleJumped => "DOUBLE JUMP",
            PlayerState::AirDash => "AIR DASH",
            PlayerState::Falling => "FALLING",
        };
        world.ui_set_text(state_text, label);

        let text_color = match game_world.resources.player_state {
            PlayerState::Grounded => nalgebra_glm::Vec4::new(0.6, 0.8, 0.6, 0.8),
            PlayerState::LeaningLeft | PlayerState::LeaningRight => {
                nalgebra_glm::Vec4::new(0.5, 0.7, 0.9, 0.8)
            }
            PlayerState::Sliding => nalgebra_glm::Vec4::new(0.9, 0.6, 0.2, 0.9),
            PlayerState::GroundDash | PlayerState::AirDash => {
                nalgebra_glm::Vec4::new(0.3, 0.9, 1.0, 1.0)
            }
            PlayerState::Airborne => nalgebra_glm::Vec4::new(0.8, 0.8, 0.5, 0.8),
            PlayerState::DoubleJumped => nalgebra_glm::Vec4::new(1.0, 0.7, 0.3, 0.9),
            PlayerState::Falling => nalgebra_glm::Vec4::new(0.7, 0.5, 0.5, 0.7),
        };
        if let Some(node_color) = world.ui.get_ui_node_color_mut(state_text) {
            node_color.colors[0] = Some(text_color);
            node_color.computed_color = text_color;
        }
    }

    let config = &game_world.resources.config;
    let cooldown_fraction = if game_world.resources.dash_charges < config.max_dash_charges {
        1.0 - (game_world.resources.dash_cooldown_timer / config.dash_cooldown).clamp(0.0, 1.0)
    } else {
        1.0
    };

    for (index, &charge_entity) in game_world
        .resources
        .dash_hud_charge_entities
        .iter()
        .enumerate()
    {
        let charged = (index as u32) < game_world.resources.dash_charges;
        let is_next_charge = !charged && (index as u32) == game_world.resources.dash_charges;

        let fill_color = if charged {
            nalgebra_glm::Vec4::new(0.15, 0.5, 0.7, 0.8)
        } else if is_next_charge {
            let brightness = cooldown_fraction * 0.5;
            nalgebra_glm::Vec4::new(0.1 * brightness, 0.3 * brightness, 0.5 * brightness, 0.4)
        } else {
            nalgebra_glm::Vec4::new(0.08, 0.08, 0.1, 0.3)
        };

        if let Some(node_color) = world.ui.get_ui_node_color_mut(charge_entity) {
            node_color.colors[0] = Some(fill_color);
            node_color.computed_color = fill_color;
        }
    }
}

pub fn build_dash_hud(world: &mut World, max_dash_charges: u32) -> (Entity, Entity, Vec<Entity>) {
    let mut tree = UiTreeBuilder::new(world);

    let panel_width = 140.0;
    let panel_height = 50.0;

    let container = tree
        .add_node()
        .boundary(
            Vp(nalgebra_glm::Vec2::new(50.0, 100.0))
                + Ab(nalgebra_glm::Vec2::new(-panel_width / 2.0, -panel_height - 15.0)),
            Vp(nalgebra_glm::Vec2::new(50.0, 100.0))
                + Ab(nalgebra_glm::Vec2::new(panel_width / 2.0, -15.0)),
        )
        .with_rect(6.0, 1.0, nalgebra_glm::Vec4::new(0.3, 0.8, 1.0, 0.3))
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.02, 0.04, 0.08, 0.6))
        .without_pointer_events()
        .entity();

    tree.push_parent(container);

    let state_text = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(0.0, 2.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 20.0)),
        )
        .with_text("GROUNDED", 11.0)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.6, 0.7, 0.8, 0.7))
        .without_pointer_events()
        .done();

    let charge_size = 20.0;
    let gap = 6.0;
    let total_width = charge_size * 2.0 + gap;
    let start_x = (panel_width - total_width) / 2.0;

    let mut charge_entities = Vec::new();
    for charge_index in 0..max_dash_charges {
        let offset_x = start_x + charge_index as f32 * (charge_size + gap);
        let charge = tree
            .add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(offset_x, 24.0)),
                Ab(nalgebra_glm::Vec2::new(
                    offset_x + charge_size,
                    24.0 + charge_size,
                )),
            )
            .with_rect(4.0, 1.5, nalgebra_glm::Vec4::new(0.3, 0.8, 1.0, 0.8))
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.15, 0.5, 0.7, 0.8))
            .without_pointer_events()
            .done();
        charge_entities.push(charge);
    }

    tree.pop_parent();
    tree.finish();

    (container, state_text, charge_entities)
}
