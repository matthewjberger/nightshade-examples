use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::prelude::*;

use crate::ecs::{CameraMode, World as GameWorld};
use crate::systems::player::get_player_position;
use crate::types::CAMERA_HEIGHT;

pub fn update(game: &GameWorld, world: &mut World) {
    let player_pos = get_player_position(game);

    let Some(camera_entity) = game.resources.visuals.camera else {
        return;
    };

    let target_position = match game.resources.camera_mode {
        CameraMode::TopDown => Vec3::new(
            player_pos.x,
            player_pos.y + CAMERA_HEIGHT,
            player_pos.z + 0.01,
        ),
        CameraMode::ThirdPerson => {
            let yaw = game.resources.camera_yaw;
            let distance = 10.0;
            let height = 5.0;
            let offset_x = distance * yaw.sin();
            let offset_z = distance * yaw.cos();
            Vec3::new(
                player_pos.x + offset_x,
                player_pos.y + height,
                player_pos.z + offset_z,
            )
        }
    };

    if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
        transform.translation = target_position;
        let look_target = player_pos + Vec3::new(0.0, 1.0, 0.0);
        let direction = nalgebra_glm::normalize(&(look_target - target_position));
        let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&direction, &Vec3::y()));
        let up = nalgebra_glm::cross(&right, &direction);
        transform.rotation =
            nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[right, up, -direction]));
    }
    mark_local_transform_dirty(world, camera_entity);
}

pub fn update_rotation(game: &mut GameWorld, world: &mut World) {
    if game.resources.camera_mode != CameraMode::ThirdPerson {
        return;
    }

    let delta = world.resources.window.timing.delta_time;
    let camera_rotate_speed = 2.5;

    if let Some(gamepad) = query_active_gamepad(world) {
        let rx = gamepad.value(gilrs::Axis::RightStickX);
        const DEADZONE: f32 = 0.15;
        if rx.abs() > DEADZONE {
            game.resources.camera_yaw -= rx * camera_rotate_speed * delta;
        }
    }

    let mouse = &world.resources.input.mouse;
    if mouse.state.contains(MouseState::RIGHT_CLICKED) {
        game.resources.camera_yaw -= mouse.raw_mouse_delta.x * 0.005;
    }

    let kb = &world.resources.input.keyboard;
    if kb.is_key_pressed(KeyCode::KeyZ) {
        game.resources.camera_yaw += camera_rotate_speed * delta;
    }
    if kb.is_key_pressed(KeyCode::KeyC) {
        game.resources.camera_yaw -= camera_rotate_speed * delta;
    }
}
