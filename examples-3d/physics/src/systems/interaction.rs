mod buttons;
mod doors;
mod drawers;
mod levers;
mod notes;
mod picking;
mod shooting_input;
mod wheels;

pub use doors::update_doors_momentum;
pub use drawers::update_drawers_momentum;
pub use levers::{apply_lever_transform, update_levers_momentum};
pub use notes::note_reading_system;
pub use picking::update_interaction_prompt;
pub use wheels::update_wheels_momentum;

use crate::ecs::{GameWorld, InteractableKind};
use buttons::{release_button, update_pressed_button};
use doors::update_manipulated_door;
use drawers::update_manipulated_drawer;
use levers::update_manipulated_lever;
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::ecs::picking::{PickingOptions, pick_entities};
use nightshade::prelude::*;
use wheels::update_manipulated_wheel;

pub fn interaction_system(game_world: &mut GameWorld, world: &mut World) {
    let (left_clicked, _left_just_pressed, right_clicked, scroll_delta) =
        if world.resources.input.input_mode == InputMode::MouseKeyboard {
            let mouse = &world.resources.input.mouse;
            (
                mouse.state.contains(MouseState::LEFT_CLICKED),
                mouse.state.contains(MouseState::LEFT_JUST_PRESSED),
                mouse.state.contains(MouseState::RIGHT_CLICKED),
                mouse.wheel_delta.y,
            )
        } else {
            (false, false, false, 0.0)
        };

    let (gamepad_rt_held, gamepad_lt_held, gamepad_dpad_distance) =
        if world.resources.input.input_mode == InputMode::Gamepad {
            if let Some(gamepad) = query_active_gamepad(world) {
                let rt_axis_value = gamepad.value(gilrs::Axis::RightZ);
                let lt_axis_value = gamepad.value(gilrs::Axis::LeftZ);
                let rt_button = gamepad.is_pressed(gilrs::Button::RightTrigger2);
                let lt_button = gamepad.is_pressed(gilrs::Button::LeftTrigger2);
                let rt_held = rt_axis_value > 0.5 || rt_button;
                let lt_held = lt_axis_value > 0.5 || lt_button;
                let dpad_up = gamepad.is_pressed(gilrs::Button::DPadUp);
                let dpad_down = gamepad.is_pressed(gilrs::Button::DPadDown);
                let dpad_distance: f32 = if dpad_up {
                    1.0
                } else if dpad_down {
                    -1.0
                } else {
                    0.0
                };
                (rt_held, lt_held, dpad_distance)
            } else {
                (false, false, 0.0)
            }
        } else {
            (false, false, 0.0)
        };

    #[cfg(feature = "openxr")]
    let (xr_interact_held, xr_shoot_trigger_held, xr_throw_grip_held, xr_thumbstick_y) = {
        if let Some(xr_input) = &world.resources.xr.input {
            (
                xr_input.left_trigger_pressed(),
                xr_input.right_trigger_pressed(),
                xr_input.left_grip_pressed(),
                xr_input.right_thumbstick.y,
            )
        } else {
            (false, false, false, 0.0)
        }
    };

    #[cfg(not(feature = "openxr"))]
    let (xr_interact_held, xr_shoot_trigger_held, xr_throw_grip_held, xr_thumbstick_y) =
        (false, false, false, 0.0_f32);
    let _ = xr_throw_grip_held;

    let interact_held = left_clicked || gamepad_lt_held || xr_interact_held;
    let throw_pressed = right_clicked || gamepad_rt_held || xr_throw_grip_held;

    let keyboard_shoot_pressed = if world.resources.input.input_mode == InputMode::MouseKeyboard {
        right_clicked
    } else {
        false
    };
    let shoot_pressed = keyboard_shoot_pressed || gamepad_rt_held || xr_shoot_trigger_held;

    let delta_time = world.resources.window.timing.delta_time;
    #[cfg(feature = "openxr")]
    let xr_distance_delta = if xr_thumbstick_y.abs() > 0.1 {
        xr_thumbstick_y * delta_time * 3.0
    } else {
        0.0
    };
    #[cfg(not(feature = "openxr"))]
    let xr_distance_delta = 0.0_f32;
    let _ = xr_thumbstick_y;

    let effective_scroll_delta = if world.resources.input.input_mode == InputMode::Gamepad
        && gamepad_dpad_distance.abs() > 0.0
    {
        gamepad_dpad_distance * delta_time * 3.0
    } else if xr_distance_delta.abs() > 0.0 {
        xr_distance_delta
    } else {
        scroll_delta
    };

    let Some(camera_entity) = game_world.resources.player.camera_entity else {
        return;
    };
    let Some(camera_transform) = world.core.get_global_transform(camera_entity) else {
        return;
    };

    let camera_position = camera_transform.translation();
    let camera_forward = camera_transform.forward_vector();

    shooting_input::handle_shooting_input(
        game_world,
        world,
        shoot_pressed,
        camera_position,
        camera_forward,
    );

    if !interact_held {
        if let Some((button_entity, InteractableKind::Button)) =
            &game_world.resources.interaction.manipulated
        {
            release_button(game_world, world, *button_entity);
        }
        nightshade::ecs::physics::grab::release_grab_physics(world);
        game_world.resources.interaction.manipulated = None;
        game_world.resources.interaction.require_interact_release = false;
        return;
    }

    if game_world.resources.interaction.require_interact_release {
        return;
    }

    if world.resources.physics.grab.is_holding() {
        update_grabbed_object(game_world, world, effective_scroll_delta);

        let throw_direction = {
            #[cfg(feature = "openxr")]
            {
                if let Some(xr_input) = &world.resources.xr.input
                    && let Some(fwd) = xr_input.left_hand_aim_direction()
                {
                    fwd
                } else {
                    camera_forward
                }
            }
            #[cfg(not(feature = "openxr"))]
            {
                camera_forward
            }
        };

        if throw_pressed {
            throw_grabbed_object(game_world, world, throw_direction);
            game_world.resources.interaction.require_interact_release = true;
        }
        return;
    }

    if let Some((entity, kind)) = game_world.resources.interaction.manipulated.clone() {
        match kind {
            InteractableKind::Door => update_manipulated_door(game_world, world, camera_position),
            InteractableKind::Drawer => {
                update_manipulated_drawer(game_world, world, camera_position)
            }
            InteractableKind::Lever => update_manipulated_lever(game_world, world, camera_position),
            InteractableKind::Wheel => update_manipulated_wheel(game_world, world, camera_position),
            InteractableKind::Button => update_pressed_button(game_world, world, entity),
            _ => {}
        }
        return;
    }

    let config = &game_world.resources.config;
    let options = PickingOptions {
        max_distance: config.grab_range,
        ignore_invisible: true,
    };

    #[cfg(feature = "openxr")]
    let pick_results = {
        if let Some(xr_input) = &world.resources.xr.input {
            if let (Some(origin), Some(direction)) = (
                xr_input.left_hand_position(),
                xr_input.left_hand_aim_direction(),
            ) {
                picking::pick_entities_from_ray(world, origin, direction, options)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    };

    #[cfg(not(feature = "openxr"))]
    let pick_results = {
        let viewport_size = world
            .resources
            .window
            .cached_viewport_size
            .unwrap_or((800, 600));
        let screen_pos =
            nalgebra_glm::vec2(viewport_size.0 as f32 / 2.0, viewport_size.1 as f32 / 2.0);
        if world.resources.input.input_mode == InputMode::Gamepad {
            picking::pick_entities_cone(world, screen_pos, config.interact_cone_radius, options)
        } else {
            pick_entities(world, screen_pos, options)
        }
    };

    picking::try_start_interaction(game_world, world, &pick_results);
}

fn update_grabbed_object(game_world: &GameWorld, world: &mut World, scroll_delta: f32) {
    let scroll_speed = game_world.resources.config.scroll_distance_speed;
    world
        .resources
        .physics
        .grab
        .adjust_distance(scroll_delta * scroll_speed);
}

fn throw_grabbed_object(game_world: &mut GameWorld, world: &mut World, camera_forward: Vec3) {
    let throw_strength = game_world.resources.config.throw_strength;
    nightshade::ecs::physics::grab::throw_grab_physics(world, camera_forward, throw_strength);
}
