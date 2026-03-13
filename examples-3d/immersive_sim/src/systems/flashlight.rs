use crate::state::ImmersiveSim;
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::prelude::*;

pub fn spawn_flashlight(world: &mut World) -> Entity {
    let entity = world.spawn_entities(
        LIGHT | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
        1,
    )[0];

    world.core.set_light(
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

    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    world.core.set_global_transform(entity, GlobalTransform::default());
    world.core.set_local_transform_dirty(entity, LocalTransformDirty);

    entity
}

pub fn update_flashlight(game: &mut ImmersiveSim, world: &mut World) {
    let Some(flashlight_entity) = game.flashlight_entity else {
        return;
    };
    let Some(camera) = game.camera_entity else {
        return;
    };

    let f_pressed = world.resources.input.keyboard.is_key_pressed(KeyCode::KeyF);

    let gamepad_flashlight = query_active_gamepad(world)
        .map(|g| g.is_pressed(gilrs::Button::North))
        .unwrap_or(false);

    let toggle_pressed = f_pressed || gamepad_flashlight;

    if toggle_pressed && !game.flashlight_key_was_pressed {
        game.flashlight_on = !game.flashlight_on;
        if let Some(light) = world.core.get_light_mut(flashlight_entity) {
            light.intensity = if game.flashlight_on { 150.0 } else { 0.0 };
        }
    }
    game.flashlight_key_was_pressed = toggle_pressed;

    if let Some(camera_transform) = world.core.get_global_transform(camera).cloned() {
        let camera_position = camera_transform.translation();
        let camera_forward = camera_transform.forward_vector();

        let offset_position = camera_position + camera_forward * 0.5;

        let flashlight_transform = LocalTransform {
            translation: offset_position,
            rotation: world
                .core.get_local_transform(camera)
                .map(|t| t.rotation)
                .unwrap_or(Quat::identity()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        };

        world.core.set_local_transform(flashlight_entity, flashlight_transform);
        world.core.set_local_transform_dirty(flashlight_entity, LocalTransformDirty);
    }
}
