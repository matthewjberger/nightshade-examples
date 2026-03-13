use crate::constants::{INTERACT_RANGE, WALL_THICKNESS};
use crate::state::{DoorState, HorrorDemo, InputMode};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::world::commands::find_entity_by_name;
use nightshade::prelude::*;

pub fn spawn_doors(demo: &mut HorrorDemo, world: &mut World) {
    let t = WALL_THICKNESS;
    init_door(
        demo,
        world,
        "Door_Entry",
        nalgebra_glm::vec3(0.0, 0.0, t / 2.0),
        false,
        false,
        false,
    );
    init_door(
        demo,
        world,
        "Door_Storage",
        nalgebra_glm::vec3(6.0 + t / 2.0, 0.0, -16.0),
        false,
        true,
        false,
    );
    init_door(
        demo,
        world,
        "Door_Generator",
        nalgebra_glm::vec3(-6.0 - t / 2.0, 0.0, -16.0),
        false,
        true,
        true,
    );
    init_door(
        demo,
        world,
        "Door_Exit",
        nalgebra_glm::vec3(0.0, 0.0, -22.0 - t / 2.0),
        true,
        false,
        false,
    );
    demo.exit_door_index = demo.doors.len() - 1;
}

fn init_door(
    demo: &mut HorrorDemo,
    world: &mut World,
    name: &str,
    position: Vec3,
    locked: bool,
    side_door: bool,
    swing_reversed: bool,
) {
    let door_width = 1.2;
    let door_height = 2.2;

    let door_entity = find_entity_by_name(world, name)
        .unwrap_or_else(|| panic!("Door entity '{}' not found in map", name));

    let hinge_offset = door_width / 2.0;
    let hinge_position = if side_door {
        let hinge_z = if swing_reversed {
            position.z + hinge_offset
        } else {
            position.z - hinge_offset
        };
        nalgebra_glm::vec3(position.x, door_height / 2.0, hinge_z)
    } else {
        nalgebra_glm::vec3(position.x - hinge_offset, door_height / 2.0, position.z)
    };

    let door_rb_handle = world
        .core.get_rigid_body(door_entity)
        .and_then(|rb| rb.handle)
        .unwrap_or_else(|| panic!("Door '{}' missing physics handle", name));

    let (min_angle, max_angle) = if swing_reversed {
        (
            -std::f32::consts::FRAC_PI_2 * 0.1,
            std::f32::consts::FRAC_PI_2,
        )
    } else {
        (
            -std::f32::consts::FRAC_PI_2,
            std::f32::consts::FRAC_PI_2 * 0.1,
        )
    };

    demo.doors.push(DoorState {
        entity: door_entity,
        rigid_body_handle: door_rb_handle.into(),
        hinge_position,
        door_half_width: door_width / 2.0,
        current_angle: 0.0,
        angular_velocity: 0.0,
        min_angle,
        max_angle,
        locked,
        side_door,
        swing_reversed,
    });
}

pub fn update_manipulated_door(demo: &mut HorrorDemo, world: &mut World, camera_position: Vec3) {
    let Some(door_index) = demo.interaction.manipulated_door_index else {
        return;
    };
    let Some(door) = demo.doors.get_mut(door_index) else {
        return;
    };

    let distance_to_hinge = nalgebra_glm::distance(&camera_position, &door.hinge_position);

    if distance_to_hinge > INTERACT_RANGE * 3.0 {
        demo.interaction.manipulated_door_index = None;
        return;
    }

    let dt = world.resources.physics.fixed_timestep;

    let mouse_input = if demo.input_mode == InputMode::MouseKeyboard {
        world.resources.input.mouse.raw_mouse_delta.x * 0.8
    } else {
        0.0
    };

    let gamepad_input = if demo.input_mode == InputMode::Gamepad {
        if let Some(gamepad) = query_active_gamepad(world) {
            let right_stick_y = gamepad.value(gilrs::Axis::RightStickY);
            let deadzone = 0.15;
            if right_stick_y.abs() > deadzone {
                right_stick_y * 3.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    } else {
        0.0
    };

    let torque = mouse_input + gamepad_input;
    let friction = 6.0;

    door.angular_velocity += torque * dt;
    door.angular_velocity -= door.angular_velocity * friction * dt;

    let angle_delta = door.angular_velocity * dt;
    let new_angle = (door.current_angle + angle_delta).clamp(door.min_angle, door.max_angle);

    if (new_angle - door.min_angle).abs() < 0.001 && door.angular_velocity < 0.0 {
        door.angular_velocity = -door.angular_velocity * 0.2;
    }
    if (new_angle - door.max_angle).abs() < 0.001 && door.angular_velocity > 0.0 {
        door.angular_velocity = -door.angular_velocity * 0.2;
    }

    door.current_angle = new_angle;

    apply_door_transform(demo, world, door_index);
}

pub fn apply_door_transform(demo: &HorrorDemo, world: &mut World, door_index: usize) {
    let Some(door) = demo.doors.get(door_index) else {
        return;
    };

    let cos_angle = door.current_angle.cos();
    let sin_angle = door.current_angle.sin();
    let (new_center_x, new_center_z) = if door.side_door {
        if door.swing_reversed {
            (
                door.hinge_position.x - door.door_half_width * sin_angle,
                door.hinge_position.z - door.door_half_width * cos_angle,
            )
        } else {
            (
                door.hinge_position.x + door.door_half_width * sin_angle,
                door.hinge_position.z + door.door_half_width * cos_angle,
            )
        }
    } else {
        (
            door.hinge_position.x + door.door_half_width * cos_angle,
            door.hinge_position.z - door.door_half_width * sin_angle,
        )
    };

    if let Some(transform) = world.core.get_local_transform_mut(door.entity) {
        transform.translation.x = new_center_x;
        transform.translation.z = new_center_z;
        transform.rotation =
            nalgebra_glm::quat_angle_axis(door.current_angle, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
    }
    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, door.entity);

    if let Some(rb) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(door.rigid_body_handle)
    {
        let rotation = rapier3d::na::UnitQuaternion::from_axis_angle(
            &rapier3d::na::Vector3::y_axis(),
            door.current_angle,
        );
        rb.set_translation(
            rapier3d::math::Vector::new(new_center_x, door.hinge_position.y, new_center_z),
            true,
        );
        rb.set_rotation(rotation, true);
    }
}

pub fn slam_door_closed(demo: &mut HorrorDemo, world: &mut World, door_index: usize) {
    if let Some(door) = demo.doors.get_mut(door_index) {
        let closed_angle = door.min_angle + std::f32::consts::FRAC_PI_2;
        door.current_angle = closed_angle;
        door.angular_velocity = 0.0;
    }
    apply_door_transform(demo, world, door_index);
}

pub fn update_doors_momentum(demo: &mut HorrorDemo, world: &mut World) {
    let dt = world.resources.physics.fixed_timestep;
    let friction = 2.0;

    for door_index in 0..demo.doors.len() {
        if demo.interaction.manipulated_door_index == Some(door_index) {
            continue;
        }

        let door = &mut demo.doors[door_index];

        if door.angular_velocity.abs() < 0.01 {
            door.angular_velocity = 0.0;
            continue;
        }

        door.angular_velocity *= (-friction * dt).exp();

        let angle_delta = door.angular_velocity * dt;
        let new_angle = (door.current_angle + angle_delta).clamp(door.min_angle, door.max_angle);

        if (new_angle - door.min_angle).abs() < 0.001 || (new_angle - door.max_angle).abs() < 0.001
        {
            door.angular_velocity = -door.angular_velocity * 0.3;
        }

        door.current_angle = new_angle;

        apply_door_transform(demo, world, door_index);
    }
}
