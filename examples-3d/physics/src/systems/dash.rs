use crate::ecs::{GameWorld, PlayerEvent, PlayerState};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::prelude::*;

const DEFAULT_FRICTION_RATE: f32 = 8.0;
const DEFAULT_ABOVE_MAX_FRICTION_RATE: f32 = 1.5;

fn set_friction(world: &mut World, entity: Entity, rate: f32, above_max_rate: f32) {
    if let Some(controller) = world.core.get_character_controller_mut(entity) {
        controller.friction_rate = rate;
        controller.above_max_friction_rate = above_max_rate;
    }
}

fn restore_default_friction(world: &mut World, entity: Entity) {
    set_friction(world, entity, DEFAULT_FRICTION_RATE, DEFAULT_ABOVE_MAX_FRICTION_RATE);
}

pub fn dash_system(game_world: &mut GameWorld, world: &mut World) {
    let Some(player_entity) = game_world.resources.player.entity else {
        return;
    };
    let Some(camera_entity) = game_world.resources.player.camera_entity else {
        return;
    };

    sync_grounded_state(game_world, world, player_entity);
    handle_jump(game_world, world, player_entity);
    handle_slide(game_world, world, player_entity);
    handle_dash(game_world, world, player_entity, camera_entity);
    recharge_dash(game_world, world);
    update_dash_hud(game_world, world);
}

fn sync_grounded_state(game_world: &mut GameWorld, world: &mut World, player_entity: Entity) {
    let grounded = world
        .core
        .get_character_controller(player_entity)
        .is_some_and(|controller| controller.grounded);

    let was_grounded_state = matches!(
        game_world.resources.player.state,
        PlayerState::Grounded
            | PlayerState::GroundDash
            | PlayerState::LeaningLeft
            | PlayerState::LeaningRight
            | PlayerState::Sliding
    );

    if grounded && !was_grounded_state {
        if let Some(new_state) = game_world
            .resources
            .player
            .state
            .process_event(PlayerEvent::Land)
        {
            game_world.resources.player.state = new_state;
        }
    } else if !grounded
        && matches!(
            game_world.resources.player.state,
            PlayerState::Grounded | PlayerState::LeaningLeft | PlayerState::LeaningRight
        )
    {
        if let Some(new_state) = game_world
            .resources
            .player
            .state
            .process_event(PlayerEvent::Jump)
        {
            game_world.resources.player.state = new_state;
        }
    } else if !grounded && game_world.resources.player.state == PlayerState::Sliding {
        if let Some(new_state) = game_world
            .resources
            .player
            .state
            .process_event(PlayerEvent::BecomeAirborne)
        {
            game_world.resources.player.state = new_state;
            restore_default_friction(world, player_entity);
        }
    } else if !grounded && game_world.resources.player.state == PlayerState::GroundDash
        && let Some(new_state) = game_world
            .resources
            .player
            .state
            .process_event(PlayerEvent::BecomeAirborne)
    {
        game_world.resources.player.state = new_state;
        restore_default_friction(world, player_entity);
    }
}

fn handle_jump(game_world: &mut GameWorld, world: &mut World, player_entity: Entity) {
    let jump_pressed = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::Space)
        || query_active_gamepad(world)
            .is_some_and(|gamepad| gamepad.is_pressed(gilrs::Button::South));
    game_world.resources.input_actions.jump.update(jump_pressed);

    if !game_world.resources.input_actions.jump.just_pressed() {
        return;
    }

    let player_state = game_world.resources.player.state;

    let is_grounded_action = matches!(
        player_state,
        PlayerState::Sliding | PlayerState::GroundDash
    );
    let is_airborne = matches!(
        player_state,
        PlayerState::Airborne
            | PlayerState::DoubleJumped
            | PlayerState::AirDash
            | PlayerState::Falling
    );

    if is_grounded_action {
        if let Some(new_state) = player_state.process_event(PlayerEvent::Jump) {
            game_world.resources.player.state = new_state;
            restore_default_friction(world, player_entity);
            if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
                controller.velocity.y = controller.jump_impulse;
                controller.can_jump = false;
            }
        }
    } else if is_airborne {
        let jumped =
            if let Some(new_state) = player_state.process_event(PlayerEvent::DoubleJump) {
                game_world.resources.player.state = new_state;
                true
            } else if let Some(new_state) = player_state.process_event(PlayerEvent::Jump) {
                game_world.resources.player.state = new_state;
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

fn handle_slide(game_world: &mut GameWorld, world: &mut World, player_entity: Entity) {
    let slide_pressed = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::KeyC)
        || query_active_gamepad(world)
            .is_some_and(|gamepad| gamepad.is_pressed(gilrs::Button::LeftThumb));
    game_world.resources.input_actions.slide.update(slide_pressed);

    let sprint_held = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::ShiftLeft)
        || query_active_gamepad(world).is_some_and(|gamepad| {
            let stick_x = gamepad.value(gilrs::Axis::LeftStickX);
            let stick_y = gamepad.value(gilrs::Axis::LeftStickY);
            (stick_x * stick_x + stick_y * stick_y).sqrt() > 0.85
        });

    if game_world.resources.input_actions.slide.just_pressed()
        && game_world.resources.player.state == PlayerState::Grounded
        && sprint_held
        && let Some(new_state) = game_world
            .resources
            .player
            .state
            .process_event(PlayerEvent::Slide)
    {
        game_world.resources.player.state = new_state;
        let slide_friction = game_world.resources.config.slide_friction;
        set_friction(world, player_entity, slide_friction, slide_friction);
        if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
            let speed = horizontal_speed(controller.velocity);
            if speed > 0.1 {
                let direction = horizontal_direction(controller.velocity);
                let boosted = speed + game_world.resources.config.slide_boost;
                controller.velocity.x = direction.x * boosted;
                controller.velocity.z = direction.z * boosted;
            }
        }
    }

    if game_world.resources.player.state == PlayerState::Sliding
        && let Some(controller) = world.core.get_character_controller_mut(player_entity)
    {
        let speed = horizontal_speed(controller.velocity);
        let should_end = !game_world.resources.input_actions.slide.held()
            || speed < game_world.resources.config.slide_min_speed;

        if should_end
            && let Some(new_state) = game_world
                .resources
                .player
                .state
                .process_event(PlayerEvent::Release)
        {
            game_world.resources.player.state = new_state;
            restore_default_friction(world, player_entity);
        }
    }
}

fn handle_dash(
    game_world: &mut GameWorld,
    world: &mut World,
    player_entity: Entity,
    camera_entity: Entity,
) {
    let gamepad_dash_pressed = query_active_gamepad(world)
        .is_some_and(|gamepad| gamepad.is_pressed(gilrs::Button::East));
    let keyboard_dash_pressed = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::KeyV);
    game_world
        .resources
        .input_actions
        .dash
        .update(gamepad_dash_pressed || keyboard_dash_pressed);

    let dash_just_pressed = game_world.resources.input_actions.dash.just_pressed();

    if dash_just_pressed
        && game_world.resources.player.dash_charges > 0
        && let Some(new_state) = game_world
            .resources
            .player
            .state
            .process_event(PlayerEvent::Dash)
    {
        game_world.resources.player.dash_charges -= 1;
        game_world.resources.player.dash_cooldown_timer =
            game_world.resources.config.dash_cooldown;
        game_world.resources.player.state = new_state;

        let dash_direction = compute_movement_direction(world, camera_entity);

        let config = &game_world.resources.config;
        let is_air_dash = new_state == PlayerState::AirDash;
        let impulse = if is_air_dash {
            config.dash_air_impulse
        } else {
            config.dash_impulse
        };

        let dash_friction = config.dash_friction;
        set_friction(world, player_entity, dash_friction, dash_friction);

        if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
            controller.velocity.x = dash_direction.x * impulse;
            controller.velocity.z = dash_direction.z * impulse;
            if is_air_dash {
                controller.velocity.y = controller.velocity.y.max(2.0);
            }
        }
    }

    if matches!(
        game_world.resources.player.state,
        PlayerState::GroundDash | PlayerState::AirDash
    ) && !dash_just_pressed
    {
        let grounded = world
            .core
            .get_character_controller(player_entity)
            .is_some_and(|controller| controller.grounded);

        if grounded && game_world.resources.player.state == PlayerState::GroundDash {
            if let Some(new_state) = game_world
                .resources
                .player
                .state
                .process_event(PlayerEvent::Land)
            {
                game_world.resources.player.state = new_state;
                restore_default_friction(world, player_entity);
            }
        } else if game_world.resources.player.state == PlayerState::AirDash {
            let speed = world
                .core
                .get_character_controller(player_entity)
                .map(|controller| horizontal_speed(controller.velocity))
                .unwrap_or(0.0);
            if speed < 2.0
                && let Some(new_state) = game_world
                    .resources
                    .player
                    .state
                    .process_event(PlayerEvent::DashEnd)
            {
                game_world.resources.player.state = new_state;
                restore_default_friction(world, player_entity);
            }
        }
    }
}

fn compute_movement_direction(world: &mut World, camera_entity: Entity) -> Vec3 {
    let camera_rotation = world
        .core
        .get_local_transform(camera_entity)
        .map(|transform| transform.rotation)
        .unwrap_or(nalgebra_glm::quat_identity());

    let keyboard = &world.resources.input.keyboard;
    let mut local_direction = nalgebra_glm::vec3(0.0, 0.0, 0.0);
    if keyboard.is_key_pressed(KeyCode::KeyW) {
        local_direction += nalgebra_glm::vec3(0.0, 0.0, -1.0);
    }
    if keyboard.is_key_pressed(KeyCode::KeyS) {
        local_direction += nalgebra_glm::vec3(0.0, 0.0, 1.0);
    }
    if keyboard.is_key_pressed(KeyCode::KeyA) {
        local_direction += nalgebra_glm::vec3(-1.0, 0.0, 0.0);
    }
    if keyboard.is_key_pressed(KeyCode::KeyD) {
        local_direction += nalgebra_glm::vec3(1.0, 0.0, 0.0);
    }

    if let Some(gamepad) = query_active_gamepad(world) {
        let stick_x = gamepad.value(gilrs::Axis::LeftStickX);
        let stick_y = gamepad.value(gilrs::Axis::LeftStickY);
        let stick_magnitude = (stick_x * stick_x + stick_y * stick_y).sqrt();
        if stick_magnitude > 0.3 {
            local_direction = nalgebra_glm::vec3(stick_x, 0.0, -stick_y);
        }
    }

    if nalgebra_glm::length(&local_direction) > 0.01 {
        let world_direction =
            nalgebra_glm::quat_rotate_vec3(&camera_rotation, &nalgebra_glm::normalize(&local_direction));
        nalgebra_glm::normalize(&nalgebra_glm::vec3(
            world_direction.x,
            0.0,
            world_direction.z,
        ))
    } else {
        let forward =
            nalgebra_glm::quat_rotate_vec3(&camera_rotation, &nalgebra_glm::vec3(0.0, 0.0, -1.0));
        nalgebra_glm::normalize(&nalgebra_glm::vec3(forward.x, 0.0, forward.z))
    }
}

fn horizontal_speed(velocity: Vec3) -> f32 {
    nalgebra_glm::length(&nalgebra_glm::vec3(velocity.x, 0.0, velocity.z))
}

fn horizontal_direction(velocity: Vec3) -> Vec3 {
    let horizontal = nalgebra_glm::vec3(velocity.x, 0.0, velocity.z);
    if nalgebra_glm::length(&horizontal) > 0.01 {
        nalgebra_glm::normalize(&horizontal)
    } else {
        nalgebra_glm::vec3(0.0, 0.0, -1.0)
    }
}

fn recharge_dash(game_world: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time;
    let max_dash_charges = game_world.resources.config.max_dash_charges;
    let dash_cooldown = game_world.resources.config.dash_cooldown;
    if game_world.resources.player.dash_charges < max_dash_charges {
        game_world.resources.player.dash_cooldown_timer -= delta_time;
        if game_world.resources.player.dash_cooldown_timer <= 0.0 {
            game_world.resources.player.dash_charges += 1;
            if game_world.resources.player.dash_charges < max_dash_charges {
                game_world.resources.player.dash_cooldown_timer = dash_cooldown;
            }
        }
    }
}

fn update_dash_hud(game_world: &mut GameWorld, world: &mut World) {
    if let Some(state_text) = game_world.resources.ui.dash_hud_state_text_entity {
        let label = match game_world.resources.player.state {
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

        let text_color = match game_world.resources.player.state {
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
    let cooldown_fraction = if game_world.resources.player.dash_charges < config.max_dash_charges {
        1.0 - (game_world.resources.player.dash_cooldown_timer / config.dash_cooldown)
            .clamp(0.0, 1.0)
    } else {
        1.0
    };

    for (index, &charge_entity) in game_world
        .resources
        .ui
        .dash_hud_charge_entities
        .iter()
        .enumerate()
    {
        let charged = (index as u32) < game_world.resources.player.dash_charges;
        let is_next_charge = !charged && (index as u32) == game_world.resources.player.dash_charges;

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
