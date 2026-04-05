use crate::ecs::{
    CoulombFrictionJoint, GameWorld, PrismaticSlider, RopeJointVisual, SphericalJointVisual,
    SpringJointVisual, VelocityFrictionJoint, COULOMB_FRICTION_JOINT, PRISMATIC_SLIDER,
    ROPE_JOINT_VISUAL, SPHERICAL_JOINT_VISUAL, SPRING_JOINT_VISUAL, VELOCITY_FRICTION_JOINT,
};
use crate::systems::ui::spawn_label;
use nightshade::ecs::physics::joints::{
    FixedJoint, JointAxisDirection, JointLimits, PrismaticJoint, RevoluteJoint, RopeJoint,
    SphericalJoint, SpringJoint, create_fixed_joint, create_prismatic_joint, create_revolute_joint,
    create_rope_joint, create_spherical_joint, create_spring_joint,
};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(super) fn spawn_fixed_joint_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    spawn_label(
        world,
        "Fixed Joint",
        nalgebra_glm::vec3(center.x + 1.0, 3.5, center.z),
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

    let num_vertebrae = 6;
    let block_size = 0.3;
    let block_spacing = 0.35;

    let beam_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
    let anchor_entity = spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, 2.5, center.z),
        nalgebra_glm::vec3(0.3, 0.3, 0.3),
        beam_material,
    );

    let colors = [
        nalgebra_glm::vec3(0.8, 0.3, 0.3),
        nalgebra_glm::vec3(0.8, 0.5, 0.3),
        nalgebra_glm::vec3(0.8, 0.8, 0.3),
        nalgebra_glm::vec3(0.3, 0.8, 0.3),
        nalgebra_glm::vec3(0.3, 0.5, 0.8),
        nalgebra_glm::vec3(0.6, 0.3, 0.8),
    ];

    let mut previous_entity = anchor_entity;
    for vertebra_index in 0..num_vertebrae {
        let color = colors[vertebra_index % colors.len()];
        let block_material = create_textured_material(color, 0.6, 0.2);
        let block_x = center.x + (vertebra_index as f32 + 1.0) * block_spacing;

        let block_entity = spawn_dynamic_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(block_x, 2.5, center.z),
            nalgebra_glm::vec3(block_size, block_size, block_size),
            1.5,
            block_material,
        );
        game_world.resources.physics_objects.push(block_entity);

        create_fixed_joint(
            world,
            previous_entity,
            block_entity,
            FixedJoint::new()
                .with_local_anchor1(nalgebra_glm::vec3(block_size / 2.0 + 0.025, 0.0, 0.0))
                .with_local_anchor2(nalgebra_glm::vec3(-block_size / 2.0 - 0.025, 0.0, 0.0)),
        );

        previous_entity = block_entity;
    }
}

pub(super) fn spawn_spherical_joint_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    spawn_label(
        world,
        "Spherical Joint",
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

    let anchor_position = nalgebra_glm::vec3(center.x, 3.0, center.z);
    let ball_position = nalgebra_glm::vec3(center.x, 1.8, center.z);
    let rod_length = 1.0;

    let beam_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
    let anchor_entity = spawn_static_physics_cube_with_material(
        world,
        anchor_position,
        nalgebra_glm::vec3(0.3, 0.3, 0.3),
        beam_material,
    );

    let pendulum_material =
        create_textured_material(nalgebra_glm::vec3(0.3, 0.8, 0.3), 0.5, 0.3);
    let pendulum_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        ball_position,
        0.2,
        3.0,
        pendulum_material,
    );
    game_world.resources.physics_objects.push(pendulum_entity);

    create_spherical_joint(
        world,
        anchor_entity,
        pendulum_entity,
        SphericalJoint::new()
            .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.15, 0.0))
            .with_local_anchor2(nalgebra_glm::vec3(0.0, rod_length, 0.0)),
    );

    let rod_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.55, 0.5), 0.7, 0.2);
    let rod_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(rod_entity) {
        name.0 = "Spherical Joint Rod".to_string();
    }

    let midpoint = (anchor_position + ball_position) * 0.5;
    let distance = nalgebra_glm::distance(&anchor_position, &ball_position);

    if let Some(transform) = world.core.get_local_transform_mut(rod_entity) {
        transform.translation = midpoint;
        transform.scale = nalgebra_glm::vec3(0.03, distance / 2.0, 0.03);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(rod_entity) {
        mesh.name = "Cylinder".to_string();
    }

    let material_name = format!("SphericalRod_{}", rod_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        rod_material,
    );
    if let Some(&mat_index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(mat_index);
    }
    world
        .core
        .set_material_ref(rod_entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(rod_entity) {
        *bv = BoundingVolume::from_mesh_type("Cylinder");
    }

    let game_entity = game_world.spawn_entities(SPHERICAL_JOINT_VISUAL, 1)[0];
    game_world.set_spherical_joint_visual(
        game_entity,
        SphericalJointVisual {
            anchor_entity,
            ball_entity: pendulum_entity,
            rod_entity,
        },
    );
}

pub(super) fn spawn_rope_joint_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    spawn_label(
        world,
        "Rope Joint",
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

    let anchor_height = 3.0;
    let anchor_position = nalgebra_glm::vec3(center.x, anchor_height, center.z);
    let ball_start_position = nalgebra_glm::vec3(center.x, anchor_height - 0.3, center.z);

    let beam_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
    let anchor_entity = spawn_static_physics_cube_with_material(
        world,
        anchor_position,
        nalgebra_glm::vec3(0.3, 0.3, 0.3),
        beam_material,
    );

    let ball_material = create_textured_material(nalgebra_glm::vec3(0.8, 0.4, 0.8), 0.4, 0.5);
    let ball_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        ball_start_position,
        0.25,
        2.0,
        ball_material,
    );
    game_world.resources.physics_objects.push(ball_entity);

    create_rope_joint(
        world,
        anchor_entity,
        ball_entity,
        RopeJoint::new(1.8)
            .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.15, 0.0))
            .with_local_anchor2(nalgebra_glm::vec3(0.0, 0.0, 0.0)),
    );

    let rope_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.5, 0.35), 0.9, 0.0);
    let rope_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(rope_entity) {
        name.0 = "Rope Joint Visual".to_string();
    }

    let anchor_attach = anchor_position - nalgebra_glm::vec3(0.0, 0.15, 0.0);
    let midpoint = (anchor_attach + ball_start_position) * 0.5;
    let distance = nalgebra_glm::distance(&anchor_attach, &ball_start_position);

    if let Some(transform) = world.core.get_local_transform_mut(rope_entity) {
        transform.translation = midpoint;
        transform.scale = nalgebra_glm::vec3(0.02, distance / 2.0, 0.02);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(rope_entity) {
        mesh.name = "Cylinder".to_string();
    }

    let material_name = format!("RopeVisual_{}", rope_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        rope_material,
    );
    if let Some(&mat_index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(mat_index);
    }
    world
        .core
        .set_material_ref(rope_entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(rope_entity) {
        *bv = BoundingVolume::from_mesh_type("Cylinder");
    }

    let game_entity = game_world.spawn_entities(ROPE_JOINT_VISUAL, 1)[0];
    game_world.set_rope_joint_visual(
        game_entity,
        RopeJointVisual {
            anchor_entity,
            ball_entity,
            rope_entity,
        },
    );
}

pub(super) fn spawn_spring_joint_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    spawn_label(
        world,
        "Spring Joint",
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

    let anchor_height = 3.0;
    let anchor_position = nalgebra_glm::vec3(center.x, anchor_height, center.z);
    let object_position = nalgebra_glm::vec3(center.x, anchor_height - 1.5, center.z);

    let beam_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
    let anchor_entity = spawn_static_physics_cube_with_material(
        world,
        anchor_position,
        nalgebra_glm::vec3(0.3, 0.3, 0.3),
        beam_material,
    );

    let spring_cube_material =
        create_textured_material(nalgebra_glm::vec3(0.3, 0.8, 0.8), 0.4, 0.5);
    let spring_entity = spawn_dynamic_physics_cube_with_material(
        world,
        object_position,
        nalgebra_glm::vec3(0.4, 0.4, 0.4),
        3.0,
        spring_cube_material,
    );
    game_world.resources.physics_objects.push(spring_entity);

    create_spring_joint(
        world,
        anchor_entity,
        spring_entity,
        SpringJoint::new(1.0, 50.0, 2.0)
            .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.15, 0.0))
            .with_local_anchor2(nalgebra_glm::vec3(0.0, 0.2, 0.0)),
    );

    let coil_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.7, 0.75), 0.3, 0.8);
    let num_coils = 8;
    let mut spring_visual_entities = Vec::new();

    for coil_index in 0..num_coils {
        let coil_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | VISIBILITY,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(coil_entity) {
            name.0 = format!("Spring Coil {}", coil_index);
        }

        if let Some(transform) = world.core.get_local_transform_mut(coil_entity) {
            transform.translation = anchor_position;
            transform.scale = nalgebra_glm::vec3(0.015, 0.1, 0.015);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(coil_entity) {
            mesh.name = "Cylinder".to_string();
        }

        let material_name = format!("SpringCoil_{}_{}", spring_entity.id, coil_index);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            coil_material.clone(),
        );
        if let Some(&mat_index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(mat_index);
        }
        world
            .core
            .set_material_ref(coil_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(coil_entity) {
            *bv = BoundingVolume::from_mesh_type("Cylinder");
        }

        spring_visual_entities.push(coil_entity);
    }

    let game_entity = game_world.spawn_entities(SPRING_JOINT_VISUAL, 1)[0];
    game_world.set_spring_joint_visual(
        game_entity,
        SpringJointVisual {
            anchor_entity,
            object_entity: spring_entity,
            spring_entities: spring_visual_entities,
        },
    );
}

pub(super) fn spawn_prismatic_joint_exhibit(
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
    game_world.resources.physics_objects.push(slider_entity);

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

pub(super) fn spawn_revolute_joint_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    spawn_label(
        world,
        "Revolute Joint",
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

    let bracket_material =
        create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.3, 0.7);
    let bracket_entity = spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, hinge_height, center.z),
        nalgebra_glm::vec3(0.2, 0.2, 0.2),
        bracket_material,
    );

    let arm_center_y = hinge_height - arm_length / 2.0;
    let arm_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.25, 0.25), 0.6, 0.3);
    let arm_entity = spawn_dynamic_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, arm_center_y, center.z),
        nalgebra_glm::vec3(arm_thickness, arm_length, arm_thickness),
        1.5,
        arm_material,
    );
    game_world.resources.physics_objects.push(arm_entity);

    let weight_y = hinge_height - arm_length;
    let weight_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.3, 0.7), 0.4, 0.5);
    let weight_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        nalgebra_glm::vec3(center.x, weight_y - weight_radius, center.z),
        weight_radius,
        4.0,
        weight_material,
    );
    game_world.resources.physics_objects.push(weight_entity);

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
}

pub(super) fn spawn_velocity_friction_joint_exhibit(
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

    let bracket_material =
        create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.3, 0.7);
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
    game_world.resources.physics_objects.push(arm_entity);

    let weight_y = hinge_height - arm_length;
    let weight_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.5, 0.3), 0.4, 0.5);
    let weight_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        nalgebra_glm::vec3(center.x, weight_y - weight_radius, center.z),
        weight_radius,
        4.0,
        weight_material,
    );
    game_world.resources.physics_objects.push(weight_entity);

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

pub(super) fn spawn_coulomb_friction_joint_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    spawn_label(
        world,
        "Coulomb Friction",
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
    let friction_torque = 0.5;

    let bracket_material =
        create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.3, 0.7);
    let bracket_entity = spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, hinge_height, center.z),
        nalgebra_glm::vec3(0.2, 0.2, 0.2),
        bracket_material,
    );

    let arm_center_y = hinge_height - arm_length / 2.0;
    let arm_material = create_textured_material(nalgebra_glm::vec3(0.8, 0.4, 0.2), 0.6, 0.3);
    let arm_entity = spawn_dynamic_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, arm_center_y, center.z),
        nalgebra_glm::vec3(arm_thickness, arm_length, arm_thickness),
        1.5,
        arm_material,
    );
    game_world.resources.physics_objects.push(arm_entity);

    let weight_y = hinge_height - arm_length;
    let weight_material = create_textured_material(nalgebra_glm::vec3(0.8, 0.4, 0.2), 0.4, 0.5);
    let weight_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        nalgebra_glm::vec3(center.x, weight_y - weight_radius, center.z),
        weight_radius,
        4.0,
        weight_material,
    );
    game_world.resources.physics_objects.push(weight_entity);

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

    let game_entity = game_world.spawn_entities(COULOMB_FRICTION_JOINT, 1)[0];
    game_world.set_coulomb_friction_joint(
        game_entity,
        CoulombFrictionJoint {
            arm_entity,
            friction_torque,
        },
    );
}

pub fn update_prismatic_sliders(game_world: &mut GameWorld, world: &mut World) {
    let dt = world.resources.window.timing.delta_time;

    let slider_entities: Vec<freecs::Entity> =
        game_world.query_entities(PRISMATIC_SLIDER).collect();

    let grabbed = game_world.resources.interaction.grabbed_entity;

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

pub fn update_joint_visuals(game_world: &GameWorld, world: &mut World) {
    let spherical_entities: Vec<freecs::Entity> =
        game_world.query_entities(SPHERICAL_JOINT_VISUAL).collect();

    for game_entity in spherical_entities {
        let Some(visual) = game_world.get_spherical_joint_visual(game_entity) else {
            continue;
        };
        let anchor_pos = world
            .core
            .get_global_transform(visual.anchor_entity)
            .map(|t| t.translation())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));
        let ball_pos = world
            .core
            .get_global_transform(visual.ball_entity)
            .map(|t| t.translation())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));

        let anchor_attach = anchor_pos - nalgebra_glm::vec3(0.0, 0.15, 0.0);
        let ball_attach = ball_pos + nalgebra_glm::vec3(0.0, 0.2, 0.0);
        let midpoint = (anchor_attach + ball_attach) * 0.5;
        let distance = nalgebra_glm::distance(&anchor_attach, &ball_attach);

        let rotation = rotation_from_to_direction(
            nalgebra_glm::vec3(0.0, 1.0, 0.0),
            ball_attach - anchor_attach,
        );

        if let Some(transform) = world.core.get_local_transform_mut(visual.rod_entity) {
            transform.translation = midpoint;
            transform.rotation = rotation;
            transform.scale = nalgebra_glm::vec3(0.03, distance.max(0.01), 0.03);
        }
        nightshade::ecs::transform::commands::mark_local_transform_dirty(
            world,
            visual.rod_entity,
        );
    }

    let rope_entities: Vec<freecs::Entity> =
        game_world.query_entities(ROPE_JOINT_VISUAL).collect();

    for game_entity in rope_entities {
        let Some(visual) = game_world.get_rope_joint_visual(game_entity) else {
            continue;
        };
        let anchor_pos = world
            .core
            .get_global_transform(visual.anchor_entity)
            .map(|t| t.translation())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));
        let ball_pos = world
            .core
            .get_global_transform(visual.ball_entity)
            .map(|t| t.translation())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));

        let anchor_attach = anchor_pos - nalgebra_glm::vec3(0.0, 0.15, 0.0);
        let midpoint = (anchor_attach + ball_pos) * 0.5;
        let distance = nalgebra_glm::distance(&anchor_attach, &ball_pos);

        let rotation = rotation_from_to_direction(
            nalgebra_glm::vec3(0.0, 1.0, 0.0),
            ball_pos - anchor_attach,
        );

        if let Some(transform) = world.core.get_local_transform_mut(visual.rope_entity) {
            transform.translation = midpoint;
            transform.rotation = rotation;
            transform.scale = nalgebra_glm::vec3(0.02, distance.max(0.01), 0.02);
        }
        nightshade::ecs::transform::commands::mark_local_transform_dirty(
            world,
            visual.rope_entity,
        );
    }

    let spring_entities: Vec<freecs::Entity> =
        game_world.query_entities(SPRING_JOINT_VISUAL).collect();

    for game_entity in spring_entities {
        let Some(visual) = game_world.get_spring_joint_visual(game_entity) else {
            continue;
        };
        let anchor_pos = world
            .core
            .get_global_transform(visual.anchor_entity)
            .map(|t| t.translation())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));
        let object_pos = world
            .core
            .get_global_transform(visual.object_entity)
            .map(|t| t.translation())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));

        let anchor_attach = anchor_pos - nalgebra_glm::vec3(0.0, 0.15, 0.0);
        let object_attach = object_pos + nalgebra_glm::vec3(0.0, 0.2, 0.0);
        let total_distance = nalgebra_glm::distance(&anchor_attach, &object_attach);

        let num_coils = visual.spring_entities.len();
        if num_coils == 0 {
            continue;
        }

        let direction = if total_distance > 0.001 {
            nalgebra_glm::normalize(&(object_attach - anchor_attach))
        } else {
            nalgebra_glm::vec3(0.0, -1.0, 0.0)
        };

        let up = nalgebra_glm::vec3(0.0, 1.0, 0.0);
        let coil_radius = 0.08;

        for (coil_index, &coil_entity) in visual.spring_entities.iter().enumerate() {
            let t = (coil_index as f32 + 0.5) / num_coils as f32;
            let base_pos = anchor_attach + direction * (t * total_distance);

            let angle = coil_index as f32 * std::f32::consts::PI;
            let perpendicular = if direction.y.abs() > 0.999_f32 {
                nalgebra_glm::vec3(1.0, 0.0, 0.0)
            } else {
                nalgebra_glm::normalize(&nalgebra_glm::cross(&direction, &up))
            };
            let perpendicular2 = nalgebra_glm::cross(&direction, &perpendicular);

            let offset = perpendicular * (angle.cos() * coil_radius)
                + perpendicular2 * (angle.sin() * coil_radius);
            let coil_pos = base_pos + offset;

            let next_t = ((coil_index + 1) as f32 + 0.5) / num_coils as f32;
            let next_base_pos = anchor_attach + direction * (next_t * total_distance);
            let next_angle = (coil_index + 1) as f32 * std::f32::consts::PI;
            let next_offset = perpendicular * (next_angle.cos() * coil_radius)
                + perpendicular2 * (next_angle.sin() * coil_radius);
            let next_coil_pos = next_base_pos + next_offset;

            let coil_direction_vec = next_coil_pos - coil_pos;
            let coil_length = nalgebra_glm::length(&coil_direction_vec);

            let coil_rotation = rotation_from_to_direction(
                nalgebra_glm::vec3(0.0, 1.0, 0.0),
                coil_direction_vec,
            );

            let midpoint = (coil_pos + next_coil_pos) * 0.5;

            if let Some(transform) = world.core.get_local_transform_mut(coil_entity) {
                transform.translation = midpoint;
                transform.rotation = coil_rotation;
                transform.scale = nalgebra_glm::vec3(0.015, coil_length.max(0.01), 0.015);
            }
            nightshade::ecs::transform::commands::mark_local_transform_dirty(
                world,
                coil_entity,
            );
        }
    }
}

fn rotation_from_to_direction(from: Vec3, to: Vec3) -> nalgebra_glm::Quat {
    let to_normalized = nalgebra_glm::normalize(&to);
    let from_normalized = nalgebra_glm::normalize(&from);

    let dot: f32 = from_normalized.dot(&to_normalized);

    if dot > 0.9999 {
        return nalgebra_glm::quat_identity();
    }

    if dot < -0.9999 {
        let mut axis =
            nalgebra_glm::cross(&nalgebra_glm::vec3(1.0, 0.0, 0.0), &from_normalized);
        if nalgebra_glm::length(&axis) < 0.0001 {
            axis = nalgebra_glm::cross(&nalgebra_glm::vec3(0.0, 1.0, 0.0), &from_normalized);
        }
        axis = nalgebra_glm::normalize(&axis);
        return nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &axis);
    }

    let axis = nalgebra_glm::cross(&from_normalized, &to_normalized);
    let s = ((1.0 + dot) * 2.0).sqrt();
    let inv_s = 1.0 / s;

    nalgebra_glm::quat(axis.x * inv_s, axis.y * inv_s, axis.z * inv_s, s * 0.5)
}

pub fn update_coulomb_friction_joints(game_world: &GameWorld, world: &mut World) {
    let joint_entities: Vec<freecs::Entity> =
        game_world.query_entities(COULOMB_FRICTION_JOINT).collect();

    for game_entity in joint_entities {
        let Some(joint_state) = game_world.get_coulomb_friction_joint(game_entity) else {
            continue;
        };
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

        let angular_velocity = rigid_body.angvel();
        let angular_speed_z = angular_velocity.z;

        if angular_speed_z.abs() > 0.001 {
            let friction_direction = -angular_speed_z.signum();
            let friction_torque_vector = rapier3d::math::Vector::new(
                0.0,
                0.0,
                friction_direction * joint_state.friction_torque,
            );
            rigid_body.apply_torque_impulse(friction_torque_vector, true);
        }
    }
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
