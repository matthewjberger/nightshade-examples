use crate::constants::INTERACT_RANGE;
use crate::ecs::{DOOR, Door, ENGINE_ENTITY, GameWorld, InputMode};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::prelude::*;

pub struct DoorConfig<'a> {
    pub name: &'a str,
    pub position: Vec3,
    pub locked: bool,
    pub side_door: bool,
    pub swing_reversed: bool,
}

pub fn init_door(
    game_world: &mut GameWorld,
    world: &mut World,
    game_entity: freecs::Entity,
    config: &DoorConfig<'_>,
) {
    let door_width = 1.2;
    let door_height = 2.2;

    let engine_entity = game_world
        .get_engine_entity(game_entity)
        .expect("Door game entity must have EngineEntity")
        .0;

    let hinge_offset = door_width / 2.0;
    let hinge_position = if config.side_door {
        let hinge_z = if config.swing_reversed {
            config.position.z + hinge_offset
        } else {
            config.position.z - hinge_offset
        };
        nalgebra_glm::vec3(config.position.x, door_height / 2.0, hinge_z)
    } else {
        nalgebra_glm::vec3(
            config.position.x - hinge_offset,
            door_height / 2.0,
            config.position.z,
        )
    };

    let door_rb_handle = world
        .core
        .get_rigid_body(engine_entity)
        .and_then(|rb| rb.handle)
        .unwrap_or_else(|| panic!("Door '{}' missing physics handle", config.name));

    let (min_angle, max_angle) = if config.swing_reversed {
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

    game_world.set_door(
        game_entity,
        Door {
            rigid_body_handle: door_rb_handle.into(),
            hinge_position,
            door_half_width: door_width / 2.0,
            current_angle: 0.0,
            angular_velocity: 0.0,
            min_angle,
            max_angle,
            locked: config.locked,
            side_door: config.side_door,
            swing_reversed: config.swing_reversed,
        },
    );
}

pub fn update_manipulated_door(
    game_world: &mut GameWorld,
    world: &mut World,
    camera_position: Vec3,
) {
    let Some(door_game_entity) = game_world.resources.interaction.manipulated_door else {
        return;
    };
    let Some(door) = game_world.get_door_mut(door_game_entity) else {
        return;
    };

    let distance_to_hinge = nalgebra_glm::distance(&camera_position, &door.hinge_position);

    if distance_to_hinge > INTERACT_RANGE * 3.0 {
        game_world.resources.interaction.manipulated_door = None;
        return;
    }

    let dt = world.resources.physics.fixed_timestep;

    let mouse_input = if game_world.resources.input_mode == InputMode::MouseKeyboard {
        world.resources.input.mouse.raw_mouse_delta.x * 0.8
    } else {
        0.0
    };

    let gamepad_input = if game_world.resources.input_mode == InputMode::Gamepad {
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

    let door = game_world.get_door_mut(door_game_entity).unwrap();
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

    apply_door_transform(game_world, world, door_game_entity);
}

pub fn apply_door_transform(
    game_world: &GameWorld,
    world: &mut World,
    game_entity: freecs::Entity,
) {
    let Some(door) = game_world.get_door(game_entity) else {
        return;
    };
    let Some(engine_entity) = game_world.get_engine_entity(game_entity) else {
        return;
    };
    let entity = engine_entity.0;

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

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation.x = new_center_x;
        transform.translation.z = new_center_z;
        transform.rotation =
            nalgebra_glm::quat_angle_axis(door.current_angle, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
    }
    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, entity);

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

pub fn slam_door_closed(
    game_world: &mut GameWorld,
    world: &mut World,
    game_entity: freecs::Entity,
) {
    if let Some(door) = game_world.get_door_mut(game_entity) {
        let closed_angle = door.min_angle + std::f32::consts::FRAC_PI_2;
        door.current_angle = closed_angle;
        door.angular_velocity = 0.0;
    }
    apply_door_transform(game_world, world, game_entity);
}

pub fn update_doors_momentum(game_world: &mut GameWorld, world: &mut World) {
    let dt = world.resources.physics.fixed_timestep;
    let friction = 2.0;

    let door_entities: Vec<freecs::Entity> =
        game_world.query_entities(DOOR | ENGINE_ENTITY).collect();

    let manipulated_door = game_world.resources.interaction.manipulated_door;

    for game_entity in door_entities {
        if manipulated_door == Some(game_entity) {
            continue;
        }

        let Some(door) = game_world.get_door_mut(game_entity) else {
            continue;
        };

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

        apply_door_transform(game_world, world, game_entity);
    }
}
