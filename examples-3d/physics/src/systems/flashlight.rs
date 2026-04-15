use crate::ecs::GameWorld;
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::prelude::*;

pub fn update_lantern_light(game_world: &GameWorld, world: &mut World) {
    let Some(lantern_entity) = game_world.resources.lantern_entity else {
        return;
    };
    let Some(light_entity) = game_world.resources.lantern_light_entity else {
        return;
    };

    let lantern_position =
        if let Some(global_transform) = world.core.get_global_transform(lantern_entity) {
            global_transform.translation()
        } else {
            return;
        };

    if let Some(transform) = world.core.get_local_transform_mut(light_entity) {
        transform.translation = lantern_position;
    }
    world.mark_local_transform_dirty(light_entity);
}

pub fn spawn_flashlight(world: &mut World) -> Entity {
    let entity = world.spawn_entities(
        LIGHT | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
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

    entity
}

pub fn update_flashlight(game_world: &mut GameWorld, world: &mut World) {
    let Some(flashlight_entity) = game_world.resources.flashlight.entity else {
        return;
    };
    let Some(camera) = game_world.resources.player.camera_entity else {
        return;
    };

    if game_world.resources.ui.reading_note.is_none() {
        let keyboard_toggle = world.resources.input.keyboard.just_pressed(KeyCode::KeyF);
        let gamepad_toggle = world
            .resources
            .input
            .gamepad
            .just_pressed(gilrs::Button::North);

        if keyboard_toggle || gamepad_toggle {
            game_world.resources.flashlight.on = !game_world.resources.flashlight.on;
            if let Some(light) = world.core.get_light_mut(flashlight_entity) {
                light.intensity = if game_world.resources.flashlight.on {
                    60.0
                } else {
                    0.0
                };
            }
        }
    }

    let (light_position, light_rotation) = if let Some(weapon) = game_world.resources.weapon.entity
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

    world.core.set_local_transform(
        flashlight_entity,
        LocalTransform {
            translation: light_position,
            rotation: light_rotation,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(flashlight_entity, LocalTransformDirty);
}
