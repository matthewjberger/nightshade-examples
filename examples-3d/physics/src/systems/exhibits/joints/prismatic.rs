use crate::ecs::{GameWorld, PrismaticSlider, PRISMATIC_SLIDER};
use crate::systems::ui::spawn_label;
use nightshade::ecs::physics::joints::{JointAxisDirection, JointLimits, PrismaticJoint, create_prismatic_joint};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(crate) fn spawn_prismatic_joint_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    spawn_label(
        world,
        "Prismatic Joint",
        nalgebra_glm::vec3(center.x, 2.5, center.z),
        TextProperties {
            font_size: 24.0,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 0.03,
            outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        },
    );

    let rail_y = 1.5;
    let rail_half_height = 0.075;
    let slider_half_height = 0.15;
    let slider_y = rail_y + rail_half_height + slider_half_height;

    let rail_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
    let rail_entity = spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, rail_y, center.z),
        nalgebra_glm::vec3(3.0, 0.15, 0.15),
        rail_material,
    );

    let slider_material = create_textured_material(nalgebra_glm::vec3(0.8, 0.8, 0.3), 0.5, 0.4);
    let slider_entity = spawn_dynamic_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x - 1.0, slider_y, center.z),
        nalgebra_glm::vec3(0.3, 0.3, 0.3),
        1.0,
        slider_material,
    );
    crate::systems::spawn::register_grabbable(game_world, slider_entity);

    create_prismatic_joint(
        world,
        rail_entity,
        slider_entity,
        PrismaticJoint::new(JointAxisDirection::X)
            .with_local_anchor1(nalgebra_glm::vec3(
                0.0,
                rail_half_height + slider_half_height,
                0.0,
            ))
            .with_local_anchor2(nalgebra_glm::vec3(0.0, 0.0, 0.0))
            .with_limits(JointLimits::new(-1.3, 1.3)),
    );

    let game_entity = game_world.spawn_entities(PRISMATIC_SLIDER, 1)[0];
    game_world.set_prismatic_slider(
        game_entity,
        PrismaticSlider {
            entity: slider_entity,
            time_accumulator: 0.0,
        },
    );
}

pub fn update_prismatic_sliders(game_world: &mut GameWorld, world: &mut World) {
    let dt = world.resources.window.timing.delta_time;

    let slider_entities: Vec<freecs::Entity> =
        game_world.query_entities(PRISMATIC_SLIDER).collect();

    let grabbed = world.resources.physics.grab.entity;

    for game_entity in slider_entities {
        let Some(slider) = game_world.get_prismatic_slider_mut(game_entity) else {
            continue;
        };

        if grabbed == Some(slider.entity) {
            continue;
        }

        slider.time_accumulator += dt;

        let target_velocity = (slider.time_accumulator * 1.5).sin() * 2.0;
        let entity = slider.entity;

        let Some(rigid_body_component) = world.core.get_rigid_body(entity) else {
            continue;
        };
        let Some(handle) = rigid_body_component.handle else {
            continue;
        };
        let Some(rigid_body) = world
            .resources
            .physics
            .rigid_body_set
            .get_mut(handle.into())
        else {
            continue;
        };

        let current_vel = rigid_body.linvel();
        rigid_body.set_linvel(
            rapier3d::math::Vector::new(target_velocity, current_vel.y, current_vel.z),
            true,
        );
    }
}
