use crate::ecs::{GameWorld, VELOCITY_FRICTION_JOINT, VelocityFrictionJoint};
use crate::systems::ui::spawn_label;
use nightshade::ecs::physics::joints::{
    FixedJoint, JointAxisDirection, RevoluteJoint, create_fixed_joint, create_revolute_joint,
};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(crate) fn spawn_velocity_friction_joint_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    spawn_label(
        world,
        "Velocity Friction",
        nalgebra_glm::vec3(center.x, 4.0, center.z),
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

    let hinge_height = 3.0;
    let arm_length = 1.2;
    let arm_thickness = 0.1;
    let weight_radius = 0.15;
    let damping_factor = 2.0;

    let bracket_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.3, 0.7);
    let bracket_entity = spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, hinge_height, center.z),
        nalgebra_glm::vec3(0.2, 0.2, 0.2),
        bracket_material,
    );

    let arm_center_y = hinge_height - arm_length / 2.0;
    let arm_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.5, 0.25), 0.6, 0.3);
    let arm_entity = spawn_dynamic_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, arm_center_y, center.z),
        nalgebra_glm::vec3(arm_thickness, arm_length, arm_thickness),
        1.5,
        arm_material,
    );
    crate::systems::spawn::register_grabbable(game_world, arm_entity);

    let weight_y = hinge_height - arm_length;
    let weight_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.5, 0.3), 0.4, 0.5);
    let weight_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        nalgebra_glm::vec3(center.x, weight_y - weight_radius, center.z),
        weight_radius,
        4.0,
        weight_material,
    );
    crate::systems::spawn::register_grabbable(game_world, weight_entity);

    create_revolute_joint(
        world,
        bracket_entity,
        arm_entity,
        RevoluteJoint::new(JointAxisDirection::Z)
            .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.1, 0.0))
            .with_local_anchor2(nalgebra_glm::vec3(0.0, arm_length / 2.0, 0.0)),
    );

    create_fixed_joint(
        world,
        arm_entity,
        weight_entity,
        FixedJoint::new()
            .with_local_anchor1(nalgebra_glm::vec3(0.0, -arm_length / 2.0, 0.0))
            .with_local_anchor2(nalgebra_glm::vec3(0.0, weight_radius, 0.0)),
    );

    let game_entity = game_world.spawn_entities(VELOCITY_FRICTION_JOINT, 1)[0];
    game_world.set_velocity_friction_joint(
        game_entity,
        VelocityFrictionJoint {
            arm_entity,
            damping_factor,
            initialized: false,
        },
    );
}

pub fn setup_velocity_friction_joints(game_world: &mut GameWorld, world: &mut World) {
    let joint_entities: Vec<freecs::Entity> =
        game_world.query_entities(VELOCITY_FRICTION_JOINT).collect();

    for game_entity in &joint_entities {
        let Some(joint_state) = game_world.get_velocity_friction_joint(*game_entity) else {
            continue;
        };
        if joint_state.initialized {
            continue;
        }
        let Some(rigid_body_component) = world.core.get_rigid_body(joint_state.arm_entity) else {
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

        rigid_body.set_angular_damping(joint_state.damping_factor);
    }
    for game_entity in joint_entities {
        if let Some(joint_state) = game_world.get_velocity_friction_joint_mut(game_entity) {
            joint_state.initialized = true;
        }
    }
}
