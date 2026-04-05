use crate::ecs::GameWorld;
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::prelude::*;

pub fn spawn_flashlight(world: &mut World) -> Entity {
    let entity = world.spawn_entities(
        LIGHT
            | LOCAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | GLOBAL_TRANSFORM,
        1,
    )[0];

    world.core.set_light(
        entity,
        Light {
            light_type: LightType::Spot,
            color: nalgebra_glm::vec3(1.0, 0.95, 0.85),
            intensity: 60.0,
            range: 50.0,
            inner_cone_angle: 0.12,
            outer_cone_angle: 0.35,
            cast_shadows: true,
            shadow_bias: 0.0001,
        },
    );

    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);

    entity
}

pub fn update_flashlight(game_world: &mut GameWorld, world: &mut World) {
    let Some(flashlight_entity) = game_world.resources.flashlight_entity else {
        return;
    };
    let Some(camera) = game_world.resources.camera_entity else {
        return;
    };

    let f_pressed = world.resources.input.keyboard.is_key_pressed(KeyCode::KeyF);
    let gamepad_flashlight_pressed = if let Some(gamepad) = query_active_gamepad(world) {
        gamepad.is_pressed(gilrs::Button::DPadDown)
    } else {
        false
    };
    let flashlight_input = f_pressed || gamepad_flashlight_pressed;

    if flashlight_input && !game_world.resources.flashlight_key_was_pressed {
        game_world.resources.flashlight_on = !game_world.resources.flashlight_on;
        if let Some(light) = world.core.get_light_mut(flashlight_entity) {
            light.intensity = if game_world.resources.flashlight_on {
                60.0
            } else {
                0.0
            };
        }
    }
    game_world.resources.flashlight_key_was_pressed = flashlight_input;

    let (light_position, light_rotation) =
        if let Some(weapon) = game_world.resources.weapon_entity
            && let Some(weapon_transform) = world.core.get_global_transform(weapon)
        {
            let muzzle_local = nalgebra_glm::vec4(0.0, 0.005, -0.20, 1.0);
            let muzzle_world = weapon_transform.0 * muzzle_local;
            let rotation = world
                .core
                .get_local_transform(camera)
                .map(|t| t.rotation)
                .unwrap_or(Quat::identity());
            (muzzle_world.xyz(), rotation)
        } else if let Some(camera_transform) = world.core.get_global_transform(camera).cloned() {
            let position = camera_transform.translation() + camera_transform.forward_vector() * 0.3;
            let rotation = world
                .core
                .get_local_transform(camera)
                .map(|t| t.rotation)
                .unwrap_or(Quat::identity());
            (position, rotation)
        } else {
            return;
        };

    {
        let flashlight_transform = LocalTransform {
            translation: light_position,
            rotation: light_rotation,
            scale: Vec3::new(1.0, 1.0, 1.0),
        };

        world
            .core
            .set_local_transform(flashlight_entity, flashlight_transform);
        world
            .core
            .set_local_transform_dirty(flashlight_entity, LocalTransformDirty);
    }
}
