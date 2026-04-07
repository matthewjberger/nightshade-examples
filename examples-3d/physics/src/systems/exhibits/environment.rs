mod parkour;
pub(super) mod visual;
pub(super) mod walls;

use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub fn spawn_sun_overhead(world: &mut World) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | LIGHT,
        1,
    )[0];

    world
        .core
        .set_name(entity, Name("Sun".to_string()));
    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: nalgebra_glm::Vec3::new(5.0, 10.0, 5.0),
            rotation: nalgebra_glm::quat_angle_axis(
                std::f32::consts::FRAC_PI_4,
                &nalgebra_glm::Vec3::new(0.0, 1.0, 0.0),
            ) * nalgebra_glm::quat_angle_axis(
                -std::f32::consts::FRAC_PI_4,
                &nalgebra_glm::Vec3::new(1.0, 0.0, 0.0),
            ),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world.core.set_light(
        entity,
        Light {
            light_type: LightType::Directional,
            color: nalgebra_glm::vec3(1.0, 0.95, 0.8),
            intensity: 5.0,
            range: 100.0,
            inner_cone_angle: std::f32::consts::PI / 6.0,
            outer_cone_angle: std::f32::consts::PI / 4.0,
            cast_shadows: true,
            shadow_bias: 0.0005,
        },
    );

    entity
}

pub fn spawn_environment(world: &mut World) {
    let floor_material =
        create_textured_material(nalgebra_glm::vec3(0.15, 0.15, 0.18), 0.9, 0.0);
    let wall_material =
        create_textured_material(nalgebra_glm::vec3(0.2, 0.18, 0.16), 0.95, 0.0);
    let platform_material =
        create_textured_material(nalgebra_glm::vec3(0.22, 0.22, 0.25), 0.85, 0.1);
    let accent_material =
        create_textured_material(nalgebra_glm::vec3(0.3, 0.25, 0.2), 0.8, 0.0);

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(0.0, -0.25, 11.0),
        nalgebra_glm::vec3(60.0, 0.5, 52.0),
        floor_material,
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(0.0, 5.0, -15.0),
        nalgebra_glm::vec3(60.0, 10.0, 0.5),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(0.0, 5.0, 37.0),
        nalgebra_glm::vec3(60.0, 10.0, 0.5),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-30.0, 5.0, 11.0),
        nalgebra_glm::vec3(0.5, 10.0, 52.0),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(30.0, 5.0, 11.0),
        nalgebra_glm::vec3(0.5, 10.0, 52.0),
        wall_material,
    );

    parkour::spawn_parkour_course(world, &platform_material, &accent_material);
}
