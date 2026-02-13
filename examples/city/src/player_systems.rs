use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::world::resources::MouseState;
use nightshade::prelude::*;

use crate::CityDemo;

const STANDING_CAMERA_HEIGHT: f32 = 0.8;
const CROUCHING_CAMERA_HEIGHT: f32 = 0.3;
const LEAN_AMOUNT: f32 = 0.4;
const LEAN_ANGLE: f32 = 0.15;
const LEAN_SPEED: f32 = 8.0;

#[derive(Default)]
pub struct LeanState {
    pub current_lean: f32,
    pub target_lean: f32,
    pub base_rotation: nalgebra_glm::Quat,
}

impl LeanState {
    pub fn new() -> Self {
        Self {
            current_lean: 0.0,
            target_lean: 0.0,
            base_rotation: nalgebra_glm::quat_identity(),
        }
    }
}

pub fn camera_look_system(demo: &mut CityDemo, world: &mut World) {
    let Some(camera_entity) = demo.player_camera_entity else {
        return;
    };

    let right_clicked = world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::RIGHT_CLICKED);

    if !right_clicked {
        if let Some(window_handle) = &world.resources.window.handle {
            let _ = window_handle.set_cursor_grab(window::CursorGrabMode::None);
            window_handle.set_cursor_visible(true);
        }
        return;
    }

    if let Some(window_handle) = &world.resources.window.handle {
        if window_handle
            .set_cursor_grab(window::CursorGrabMode::Locked)
            .is_err()
        {
            let _ = window_handle.set_cursor_grab(window::CursorGrabMode::Confined);
        }
        window_handle.set_cursor_visible(false);
    }

    let raw_delta = world.resources.input.mouse.raw_mouse_delta;
    let mouse_sensitivity = 0.002;
    let delta = raw_delta * mouse_sensitivity;

    let yaw = nalgebra_glm::quat_angle_axis(-delta.x, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
    demo.lean_state.base_rotation = yaw * demo.lean_state.base_rotation;

    let forward = nalgebra_glm::quat_rotate_vec3(
        &demo.lean_state.base_rotation,
        &nalgebra_glm::vec3(0.0, 0.0, -1.0),
    );
    let current_pitch = forward.y.asin();
    let new_pitch = current_pitch - delta.y;

    if new_pitch.abs() <= 85_f32.to_radians() {
        let pitch = nalgebra_glm::quat_angle_axis(-delta.y, &nalgebra_glm::vec3(1.0, 0.0, 0.0));
        demo.lean_state.base_rotation *= pitch;
    }

    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
}

pub fn lean_system(demo: &mut CityDemo, world: &mut World) {
    let Some(camera_entity) = demo.player_camera_entity else {
        return;
    };

    let keyboard = &world.resources.input.keyboard;
    let lean_left = keyboard.is_key_pressed(KeyCode::KeyQ);
    let lean_right = keyboard.is_key_pressed(KeyCode::KeyE);

    demo.lean_state.target_lean = if lean_left && !lean_right {
        -1.0
    } else if lean_right && !lean_left {
        1.0
    } else {
        0.0
    };

    let delta_time = world.resources.window.timing.delta_time;
    let lean_diff = demo.lean_state.target_lean - demo.lean_state.current_lean;
    demo.lean_state.current_lean += lean_diff * (LEAN_SPEED * delta_time).min(1.0);

    let right_vector = nalgebra_glm::quat_rotate_vec3(
        &demo.lean_state.base_rotation,
        &nalgebra_glm::vec3(1.0, 0.0, 0.0),
    );
    let horizontal_right =
        nalgebra_glm::normalize(&nalgebra_glm::vec3(right_vector.x, 0.0, right_vector.z));

    let lean_offset = horizontal_right * (demo.lean_state.current_lean * LEAN_AMOUNT);

    let lean_roll = -demo.lean_state.current_lean * LEAN_ANGLE;
    let roll_quat = nalgebra_glm::quat_angle_axis(lean_roll, &nalgebra_glm::vec3(0.0, 0.0, 1.0));

    let final_rotation = demo.lean_state.base_rotation * roll_quat;

    let Some(camera_transform) = world.get_local_transform_mut(camera_entity) else {
        return;
    };

    camera_transform.translation.x = lean_offset.x;
    camera_transform.translation.z = lean_offset.z;
    camera_transform.rotation = final_rotation;

    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
}

pub fn crouch_camera_system(demo: &CityDemo, world: &mut World) {
    let Some(player_entity) = demo.player_entity else {
        return;
    };
    let Some(camera_entity) = demo.player_camera_entity else {
        return;
    };

    let is_crouching = world
        .get_character_controller(player_entity)
        .map(|cc| cc.is_crouching)
        .unwrap_or(false);

    let target_height = if is_crouching {
        CROUCHING_CAMERA_HEIGHT
    } else {
        STANDING_CAMERA_HEIGHT
    };

    let delta_time = world.resources.window.timing.delta_time;
    if let Some(transform) = world.get_local_transform_mut(camera_entity) {
        let speed = 8.0;
        let diff = target_height - transform.translation.y;
        transform.translation.y += diff * speed * delta_time;
    }

    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
}

pub fn spawn_flashlight(world: &mut World) -> Entity {
    let entity = world.spawn_entities(
        LIGHT | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
        1,
    )[0];

    world.set_light(
        entity,
        Light {
            light_type: LightType::Spot,
            color: nalgebra_glm::vec3(1.0, 0.95, 0.8),
            intensity: 150.0,
            range: 50.0,
            inner_cone_angle: 0.15,
            outer_cone_angle: 0.4,
            cast_shadows: true,
            shadow_bias: 0.0001,
        },
    );

    world.set_local_transform(
        entity,
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    world.set_global_transform(entity, GlobalTransform::default());
    world.set_local_transform_dirty(entity, LocalTransformDirty);

    entity
}

pub fn update_flashlight(demo: &mut CityDemo, world: &mut World) {
    let Some(flashlight_entity) = demo.flashlight_entity else {
        return;
    };
    let Some(camera) = demo.player_camera_entity else {
        return;
    };

    let f_pressed = world.resources.input.keyboard.is_key_pressed(KeyCode::KeyF);

    if f_pressed && !demo.flashlight_key_was_pressed {
        demo.flashlight_on = !demo.flashlight_on;
        if let Some(light) = world.get_light_mut(flashlight_entity) {
            light.intensity = if demo.flashlight_on { 150.0 } else { 0.0 };
        }
    }
    demo.flashlight_key_was_pressed = f_pressed;

    if let Some(camera_transform) = world.get_global_transform(camera).cloned() {
        let camera_position = camera_transform.translation();
        let camera_forward = camera_transform.forward_vector();

        let offset_position = camera_position + camera_forward * 0.5;

        let flashlight_transform = LocalTransform {
            translation: offset_position,
            rotation: world
                .get_local_transform(camera)
                .map(|t| t.rotation)
                .unwrap_or(Quat::identity()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        };

        world.set_local_transform(flashlight_entity, flashlight_transform);
        world.set_local_transform_dirty(flashlight_entity, LocalTransformDirty);
    }
}
