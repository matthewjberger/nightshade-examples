use crate::constants::INTERACT_RANGE;
use crate::ecs::{
    ENGINE_ENTITY, EngineEntity, GameWorld, InputMode, Interactable, InteractionKind, LEVER, Lever,
    LeverAction,
};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::physics::*;
use nightshade::ecs::world::commands::{find_entity_by_name, spawn_material};
use nightshade::prelude::*;

pub fn init_lever(
    game_world: &mut GameWorld,
    world: &mut World,
    game_entity: freecs::Entity,
    name: &str,
    position: Vec3,
    action: LeverAction,
) {
    let arm_half_length = 0.2;
    let handle_radius = 0.04;

    let pivot_entity = find_entity_by_name(world, &format!("{}_Pivot", name))
        .unwrap_or_else(|| panic!("Lever pivot '{}' not found in map", name));

    game_world.set_engine_entity(game_entity, EngineEntity(pivot_entity));
    let light_entity = find_entity_by_name(world, &format!("{}_Light", name))
        .unwrap_or_else(|| panic!("Lever light '{}' not found in map", name));

    let pivot_position = position;
    let collider_half_length = arm_half_length + handle_radius;
    let collider_center_offset = collider_half_length;
    let collider_world_position = nalgebra_glm::vec3(
        pivot_position.x,
        pivot_position.y,
        pivot_position.z + collider_center_offset,
    );

    let collider_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | BOUNDING_VOLUME
            | nightshade::ecs::world::RIGID_BODY
            | nightshade::ecs::world::COLLIDER,
        1,
    )[0];

    if let Some(entity_name) = world.core.get_name_mut(collider_entity) {
        entity_name.0 = format!("{}_Collider", name);
    }

    let hitbox_size = 0.15;
    if let Some(transform) = world.core.get_local_transform_mut(collider_entity) {
        transform.translation = collider_world_position;
        transform.scale = nalgebra_glm::vec3(
            hitbox_size * 2.0,
            hitbox_size * 2.0,
            collider_half_length * 2.0,
        );
    }

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(collider_entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(collider_entity) {
        *rigid_body = RigidBodyComponent::new_kinematic().with_translation(
            collider_world_position.x,
            collider_world_position.y,
            collider_world_position.z,
        );
    }

    if let Some(collider) = world.core.get_collider_mut(collider_entity) {
        *collider = ColliderComponent::new_cuboid(hitbox_size, hitbox_size, collider_half_length)
            .with_friction(0.5);
    }

    world.spawn_physics_body(collider_entity);
    let collider_rb_handle = world
        .core
        .get_rigid_body(collider_entity)
        .and_then(|rb| rb.handle)
        .expect("Lever collider missing physics handle after spawn")
        .into();

    world.core.add_components(light_entity, LIGHT);
    if let Some(light) = world.core.get_light_mut(light_entity) {
        *light = Light {
            light_type: LightType::Point,
            color: nalgebra_glm::vec3(1.0, 0.2, 0.2),
            intensity: 0.0,
            range: 4.0,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: false,
            shadow_bias: 0.0,
        };
    }

    let light_material_name = format!("{}_Light_Material", name);
    let light_fixture_material = Material {
        base_color: [0.2, 0.1, 0.1, 1.0],
        emissive_factor: [0.0, 0.0, 0.0],
        roughness: 0.3,
        metallic: 0.8,
        ..Default::default()
    };
    spawn_material(
        world,
        light_entity,
        light_material_name.clone(),
        light_fixture_material,
    );

    game_world.set_lever(
        game_entity,
        Lever {
            collider_entity,
            collider_rb_handle,
            pivot_position,
            arm_half_length: collider_half_length,
            current_angle: -std::f32::consts::FRAC_PI_4,
            angular_velocity: 0.0,
            min_angle: -std::f32::consts::FRAC_PI_4,
            max_angle: std::f32::consts::FRAC_PI_3,
            action,
            light_entity,
            light_material_name,
            activated: false,
        },
    );

    game_world.set_interactable(
        game_entity,
        Interactable {
            kind: InteractionKind::Lever,
            match_entity: collider_entity,
            range: crate::constants::INTERACT_RANGE,
        },
    );

    apply_lever_transform(game_world, world, game_entity);
}

pub fn update_manipulated_lever(
    game_world: &mut GameWorld,
    world: &mut World,
    camera_position: Vec3,
) {
    let Some(lever_game_entity) = game_world.resources.interaction.manipulated_lever else {
        return;
    };
    let Some(lever) = game_world.get_lever_mut(lever_game_entity) else {
        return;
    };

    let distance = nalgebra_glm::distance(&camera_position, &lever.pivot_position);

    if distance > INTERACT_RANGE * 3.0 {
        game_world.resources.interaction.manipulated_lever = None;
        return;
    }

    let dt = world.resources.physics.fixed_timestep;

    let mouse_input = if game_world.resources.input_mode == InputMode::MouseKeyboard {
        world.resources.input.mouse.raw_mouse_delta.y * 1.5
    } else {
        0.0
    };

    let gamepad_input = if game_world.resources.input_mode == InputMode::Gamepad {
        if let Some(gamepad) = query_active_gamepad(world) {
            let right_stick_y = gamepad.value(gilrs::Axis::RightStickY);
            let deadzone = 0.15;
            if right_stick_y.abs() > deadzone {
                -right_stick_y * 3.0
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
    let friction = 5.0;

    let lever = game_world.get_lever_mut(lever_game_entity).unwrap();
    lever.angular_velocity += torque * dt;
    lever.angular_velocity -= lever.angular_velocity * friction * dt;

    let angle_delta = lever.angular_velocity * dt;
    let new_angle = (lever.current_angle + angle_delta).clamp(lever.min_angle, lever.max_angle);

    if (new_angle - lever.min_angle).abs() < 0.001 && lever.angular_velocity < 0.0 {
        lever.angular_velocity = -lever.angular_velocity * 0.2;
    }
    if (new_angle - lever.max_angle).abs() < 0.001 && lever.angular_velocity > 0.0 {
        lever.angular_velocity = -lever.angular_velocity * 0.2;
    }

    lever.current_angle = new_angle;

    apply_lever_transform(game_world, world, lever_game_entity);
}

pub fn apply_lever_transform(
    game_world: &GameWorld,
    world: &mut World,
    game_entity: freecs::Entity,
) {
    let Some(lever) = game_world.get_lever(game_entity) else {
        return;
    };

    let rotation =
        nalgebra_glm::quat_angle_axis(lever.current_angle, &nalgebra_glm::vec3(1.0, 0.0, 0.0));

    let lever_pivot_position = lever.pivot_position;
    let lever_arm_half_length = lever.arm_half_length;
    let lever_collider_entity = lever.collider_entity;
    let lever_collider_rb_handle = lever.collider_rb_handle;

    if let Some(engine_entity) = game_world.get_engine_entity(game_entity) {
        if let Some(transform) = world.core.get_local_transform_mut(engine_entity.0) {
            transform.rotation = rotation;
        }
        nightshade::ecs::transform::commands::mark_local_transform_dirty(world, engine_entity.0);
    }

    let local_offset = nalgebra_glm::vec3(0.0, 0.0, lever_arm_half_length);
    let rotated_offset = nalgebra_glm::quat_rotate_vec3(&rotation, &local_offset);
    let center_pos = lever_pivot_position + rotated_offset;

    if let Some(transform) = world.core.get_local_transform_mut(lever_collider_entity) {
        transform.translation = center_pos;
        transform.rotation = rotation;
    }
    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, lever_collider_entity);

    if let Some(rb) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(lever_collider_rb_handle)
    {
        use rapier3d::prelude::*;
        let rapier_rotation =
            rapier3d::na::UnitQuaternion::from_quaternion(rapier3d::na::Quaternion::new(
                rotation.w,
                rotation.coords.x,
                rotation.coords.y,
                rotation.coords.z,
            ));
        rb.set_position(
            Isometry::from_parts(
                Translation::new(center_pos.x, center_pos.y, center_pos.z),
                rapier_rotation,
            ),
            true,
        );
    }
}

pub fn update_levers_momentum(game_world: &mut GameWorld, world: &mut World) {
    let dt = world.resources.physics.fixed_timestep;
    let friction = 2.5;

    let lever_entities: Vec<freecs::Entity> =
        game_world.query_entities(LEVER | ENGINE_ENTITY).collect();

    let manipulated_lever = game_world.resources.interaction.manipulated_lever;

    for game_entity in lever_entities {
        if manipulated_lever == Some(game_entity) {
            continue;
        }

        let Some(lever) = game_world.get_lever_mut(game_entity) else {
            continue;
        };

        if lever.angular_velocity.abs() < 0.01 {
            lever.angular_velocity = 0.0;
            continue;
        }

        lever.angular_velocity *= (-friction * dt).exp();

        let angle_delta = lever.angular_velocity * dt;
        let new_angle = (lever.current_angle + angle_delta).clamp(lever.min_angle, lever.max_angle);

        if (new_angle - lever.min_angle).abs() < 0.001
            || (new_angle - lever.max_angle).abs() < 0.001
        {
            lever.angular_velocity = -lever.angular_velocity * 0.3;
        }

        lever.current_angle = new_angle;

        apply_lever_transform(game_world, world, game_entity);
    }
}
