use crate::state::HorrorDemo;
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
            intensity: 30.0,
            range: 25.0,
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

    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);

    entity
}

pub fn spawn_ambient_light(world: &mut World) {
    let entity = world.spawn_entities(
        LIGHT | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
        1,
    )[0];

    world.core.set_light(
        entity,
        Light {
            light_type: LightType::Directional,
            color: nalgebra_glm::vec3(0.4, 0.45, 0.5),
            intensity: 0.15,
            range: 100.0,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: false,
            shadow_bias: 0.0,
        },
    );

    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: Vec3::new(0.0, 10.0, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(
                -std::f32::consts::FRAC_PI_2,
                &nalgebra_glm::Vec3::x_axis(),
            ),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
}

pub fn update_flashlight(demo: &mut HorrorDemo, world: &mut World) {
    let Some(flashlight_entity) = demo.flashlight_entity else {
        return;
    };
    let Some(camera) = demo.camera_entity else {
        return;
    };

    let f_pressed = world.resources.input.keyboard.is_key_pressed(KeyCode::KeyF);

    if f_pressed && !demo.flashlight_key_was_pressed {
        demo.flashlight_on = !demo.flashlight_on;
        if let Some(light) = world.core.get_light_mut(flashlight_entity) {
            light.intensity = if demo.flashlight_on { 30.0 } else { 0.0 };
        }
    }
    demo.flashlight_key_was_pressed = f_pressed;

    if let Some(camera_transform) = world.core.get_global_transform(camera).cloned() {
        let camera_position = camera_transform.translation();
        let camera_forward = camera_transform.forward_vector();

        let offset_position = camera_position + camera_forward * 0.5;

        let flashlight_transform = LocalTransform {
            translation: offset_position,
            rotation: world
                .core
                .get_local_transform(camera)
                .map(|t| t.rotation)
                .unwrap_or(Quat::identity()),
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

pub fn update_lantern_light(demo: &HorrorDemo, world: &mut World) {
    let Some(lantern_entity) = demo.lantern_entity else {
        return;
    };
    let Some(light_entity) = demo.lantern_light_entity else {
        return;
    };

    let lantern_position =
        if let Some(global_transform) = world.core.get_global_transform(lantern_entity) {
            global_transform.0.column(3).xyz()
        } else {
            return;
        };

    if let Some(transform) = world.core.get_local_transform_mut(light_entity) {
        transform.translation = lantern_position;
    }
    world.mark_local_transform_dirty(light_entity);
}
