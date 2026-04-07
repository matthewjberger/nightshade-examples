use crate::ecs::{GameWorld, PlayerEvent, PlayerState};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::prelude::*;

pub fn check_fall_reset(game_world: &mut GameWorld, world: &mut World) {
    let Some(player_entity) = game_world.resources.player.entity else {
        return;
    };

    let Some(transform) = world.core.get_local_transform(player_entity) else {
        return;
    };

    if transform.translation.y < -20.0 {
        let spawn_position = nalgebra_glm::vec3(0.0, 1.2, 8.0);

        if let Some(transform) = world.core.get_local_transform_mut(player_entity) {
            transform.translation = spawn_position;
        }

        if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
            controller.velocity = nalgebra_glm::vec3(0.0, 0.0, 0.0);
        }

        if let Some(new_state) = game_world.resources.player.state.process_event(PlayerEvent::Reset) {
            game_world.resources.player.state = new_state;
        }
    }
}

pub fn player_state_system(game_world: &mut GameWorld, world: &mut World) {
    let Some(player_entity) = game_world.resources.player.entity else {
        return;
    };

    let grounded = world
        .core
        .get_character_controller(player_entity)
        .is_some_and(|controller| controller.grounded);

    let is_grounded_state = matches!(
        game_world.resources.player.state,
        PlayerState::Grounded | PlayerState::Sprinting | PlayerState::Crouching
    );

    if grounded && !is_grounded_state {
        if let Some(new_state) = game_world
            .resources
            .player
            .state
            .process_event(PlayerEvent::Land)
        {
            game_world.resources.player.state = new_state;
        }
    } else if !grounded && is_grounded_state
        && let Some(new_state) = game_world
            .resources
            .player
            .state
            .process_event(PlayerEvent::Jump)
    {
        game_world.resources.player.state = new_state;
    }

    let reading_note = game_world.resources.ui.reading_note.is_some();

    let sprint_pressed = !reading_note
        && (world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::ShiftLeft)
            || query_active_gamepad(world)
                .is_some_and(|gamepad| gamepad.is_pressed(gilrs::Button::LeftThumb)));
    let crouch_pressed = !reading_note
        && (world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::ControlLeft)
            || world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::KeyC)
            || query_active_gamepad(world)
                .is_some_and(|gamepad| gamepad.is_pressed(gilrs::Button::East)));
    let jump_pressed = !reading_note
        && (world.resources.input.keyboard.just_pressed(KeyCode::Space)
            || world
                .resources
                .input
                .gamepad
                .just_pressed(gilrs::Button::South));

    if jump_pressed && grounded
        && let Some(new_state) = game_world
            .resources
            .player
            .state
            .process_event(PlayerEvent::Jump)
    {
        game_world.resources.player.state = new_state;
        if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
            controller.velocity.y = controller.jump_impulse;
            controller.can_jump = false;
        }
    }

    match game_world.resources.player.state {
        PlayerState::Grounded => {
            if sprint_pressed {
                if let Some(new_state) = game_world
                    .resources
                    .player
                    .state
                    .process_event(PlayerEvent::Sprint)
                {
                    game_world.resources.player.state = new_state;
                }
            } else if crouch_pressed
                && let Some(new_state) = game_world
                    .resources
                    .player
                    .state
                    .process_event(PlayerEvent::Crouch)
            {
                game_world.resources.player.state = new_state;
            }
        }
        PlayerState::Sprinting => {
            if !sprint_pressed
                && let Some(new_state) = game_world
                    .resources
                    .player
                    .state
                    .process_event(PlayerEvent::Release)
            {
                game_world.resources.player.state = new_state;
            }
        }
        PlayerState::Crouching => {
            if !crouch_pressed
                && let Some(new_state) = game_world
                    .resources
                    .player
                    .state
                    .process_event(PlayerEvent::Release)
            {
                game_world.resources.player.state = new_state;
            }
        }
        PlayerState::Airborne => {}
    }

    if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
        controller.is_crouching = game_world.resources.player.state == PlayerState::Crouching;
        controller.is_sprinting = game_world.resources.player.state == PlayerState::Sprinting;
    }

    update_state_hud(game_world, world);
}

fn update_state_hud(game_world: &GameWorld, world: &mut World) {
    let Some(text_entity) = game_world.resources.ui.player_state_text_entity else {
        return;
    };

    let label = match game_world.resources.player.state {
        PlayerState::Grounded => "GROUNDED",
        PlayerState::Sprinting => "SPRINTING",
        PlayerState::Crouching => "CROUCHING",
        PlayerState::Airborne => "AIRBORNE",
    };
    world.ui_set_text(text_entity, label);

    let color = match game_world.resources.player.state {
        PlayerState::Grounded => nalgebra_glm::Vec4::new(0.6, 0.8, 0.6, 0.8),
        PlayerState::Sprinting => nalgebra_glm::Vec4::new(0.3, 0.9, 1.0, 0.9),
        PlayerState::Crouching => nalgebra_glm::Vec4::new(0.9, 0.6, 0.2, 0.8),
        PlayerState::Airborne => nalgebra_glm::Vec4::new(0.8, 0.8, 0.5, 0.8),
    };
    if let Some(node_color) = world.ui.get_ui_node_color_mut(text_entity) {
        node_color.colors[0] = Some(color);
        node_color.computed_color = color;
    }
}

pub fn build_player_state_hud(world: &mut World) -> Entity {
    let mut tree = UiTreeBuilder::new(world);

    let entity = tree
        .add_node()
        .boundary(
            Vp(nalgebra_glm::Vec2::new(50.0, 100.0))
                + Ab(nalgebra_glm::Vec2::new(-70.0, -30.0)),
            Vp(nalgebra_glm::Vec2::new(50.0, 100.0))
                + Ab(nalgebra_glm::Vec2::new(70.0, -10.0)),
        )
        .with_text("GROUNDED", 11.0)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.6, 0.8, 0.6, 0.8))
        .without_pointer_events()
        .done();

    tree.finish();

    entity
}
