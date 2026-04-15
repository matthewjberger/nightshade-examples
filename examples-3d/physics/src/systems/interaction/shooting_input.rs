use crate::ecs::GameWorld;
use nightshade::ecs::input::resources::MouseState;
use nightshade::prelude::*;

pub(super) fn handle_shooting_input(
    game_world: &mut GameWorld,
    world: &mut World,
    shoot_pressed: bool,
    camera_position: Vec3,
    camera_forward: Vec3,
) {
    #[cfg(feature = "openxr")]
    let (shoot_origin, shoot_direction) = {
        if let Some(xr_input) = &world.resources.xr.input {
            if let (Some(pos), Some(forward)) = (
                xr_input.right_hand_position(),
                xr_input.right_hand_aim_direction(),
            ) {
                let muzzle_offset = forward * 0.18;
                (pos + muzzle_offset, forward)
            } else {
                (camera_position, camera_forward)
            }
        } else {
            (camera_position, camera_forward)
        }
    };
    #[cfg(not(feature = "openxr"))]
    let (shoot_origin, shoot_direction) = if let Some(weapon) = game_world.resources.weapon.entity
        && let Some(weapon_transform) = world.core.get_global_transform(weapon)
    {
        let muzzle_local = nalgebra_glm::vec4(0.0, 0.005, -0.20, 1.0);
        let muzzle_world = weapon_transform.0 * muzzle_local;
        (muzzle_world.xyz(), camera_forward)
    } else {
        (camera_position, camera_forward)
    };

    let current_time_ms = world.resources.window.timing.uptime_milliseconds;

    let keyboard_shoot_just_pressed =
        if world.resources.input.input_mode == InputMode::MouseKeyboard {
            world
                .resources
                .input
                .mouse
                .state
                .contains(MouseState::RIGHT_JUST_PRESSED)
        } else {
            false
        };
    let gamepad_shoot_just_pressed = world
        .resources
        .input
        .gamepad
        .just_pressed(gilrs::Button::RightTrigger2);
    let shoot_just_pressed = keyboard_shoot_just_pressed || gamepad_shoot_just_pressed;

    if !world.resources.physics.grab.is_holding() {
        if shoot_just_pressed {
            game_world.resources.interaction.shoot_hold_start_ms = Some(current_time_ms);
            game_world.resources.interaction.last_rapid_fire_ms = current_time_ms;
            crate::systems::shooting::shoot_bauble(
                game_world,
                world,
                shoot_origin,
                shoot_direction,
            );
        } else if shoot_pressed {
            if let Some(hold_start) = game_world.resources.interaction.shoot_hold_start_ms {
                let hold_duration = current_time_ms.saturating_sub(hold_start);
                if hold_duration > 200 {
                    let time_since_last_shot = current_time_ms
                        .saturating_sub(game_world.resources.interaction.last_rapid_fire_ms);
                    if time_since_last_shot >= 80 {
                        game_world.resources.interaction.last_rapid_fire_ms = current_time_ms;
                        crate::systems::shooting::shoot_bauble(
                            game_world,
                            world,
                            shoot_origin,
                            shoot_direction,
                        );
                    }
                }
            }
        } else {
            game_world.resources.interaction.shoot_hold_start_ms = None;
        }
    }
}
