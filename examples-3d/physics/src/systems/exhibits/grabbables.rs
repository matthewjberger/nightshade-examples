use crate::ecs::{
    BaubleSpawn, Button, ButtonAction, Door, Drawer, Lever, Note, Wheel, BAUBLE_SPAWN, BUTTON,
    DOOR, DRAWER, GameWorld, LEVER, NOTE, WHEEL,
};
use super::environment::spawn_visual_cube;
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::physics::*;
use nightshade::ecs::transform::components::Parent;
use nightshade::ecs::world::{
    BOUNDING_VOLUME, CASTS_SHADOW, GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM,
    LOCAL_TRANSFORM_DIRTY, MATERIAL_REF, NAME, PARENT, RENDER_MESH, VISIBILITY,
};
use nightshade::prelude::*;

pub(super) fn spawn_grabbables_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    let pedestal_material =
        create_textured_material(nalgebra_glm::vec3(0.25, 0.25, 0.28), 0.85, 0.0);

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, 0.4, center.z),
        nalgebra_glm::vec3(2.5, 0.8, 2.5),
        pedestal_material,
    );

    let table_top_y = 0.8;
    let box_size = 0.25;
    let box_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.5, 0.35), 0.7, 0.0);

    let positions = [
        nalgebra_glm::vec3(center.x - 0.5, table_top_y + box_size / 2.0, center.z - 0.5),
        nalgebra_glm::vec3(center.x + 0.5, table_top_y + box_size / 2.0, center.z - 0.5),
        nalgebra_glm::vec3(center.x, table_top_y + box_size / 2.0, center.z + 0.5),
    ];

    for position in positions {
        let entity = spawn_dynamic_physics_cube_with_material(
            world,
            position,
            nalgebra_glm::vec3(box_size, box_size, box_size),
            2.0,
            box_material.clone(),
        );
        game_world.resources.physics_objects.push(entity);
    }

    let sphere_radius = 0.2;
    let sphere_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.2, 0.2), 0.5, 0.3);
    let sphere_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        nalgebra_glm::vec3(center.x, table_top_y + sphere_radius, center.z),
        sphere_radius,
        1.5,
        sphere_material,
    );
    game_world.resources.physics_objects.push(sphere_entity);

    let cylinder_half_height = 0.2;
    let cylinder_radius = 0.12;
    let metal_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.3, 0.8);
    let cylinder_entity = spawn_dynamic_physics_cylinder_with_material(
        world,
        nalgebra_glm::vec3(center.x - 0.7, table_top_y + cylinder_half_height, center.z),
        cylinder_half_height,
        cylinder_radius,
        3.0,
        metal_material,
    );
    game_world.resources.physics_objects.push(cylinder_entity);
}

pub(super) fn spawn_door_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    let door_width = 0.9;
    let door_height = 2.5;
    let door_thickness = 0.15;
    let frame_thickness = 0.1;
    let frame_depth = 0.15;

    let frame_left_x = center.x;
    let frame_right_x = frame_left_x + door_width + frame_thickness * 2.0;
    let frame_center_x = (frame_left_x + frame_right_x) / 2.0;
    let frame_z = center.z;
    let frame_y = door_height / 2.0;

    let hinge_x = frame_left_x + frame_thickness;
    let door_center_x = hinge_x + door_width / 2.0;
    let hinge_position = nalgebra_glm::vec3(hinge_x, frame_y, frame_z);

    let door_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.35, 0.2), 0.8, 0.0);

    let door_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY
            | nightshade::ecs::world::RIGID_BODY
            | nightshade::ecs::world::COLLIDER,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(door_entity) {
        name.0 = "Door".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(door_entity) {
        transform.translation = nalgebra_glm::vec3(door_center_x, frame_y, frame_z);
        transform.scale = nalgebra_glm::vec3(door_width, door_height, door_thickness);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(door_entity) {
        mesh.name = "Cube".to_string();
    }

    let material_name = format!("Door_{}", door_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        door_material,
    );
    if let Some(&index) = world
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
            .add_reference(index);
    }
    world
        .core
        .set_material_ref(door_entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(door_entity) {
        *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(door_entity) {
        *rigid_body = RigidBodyComponent::new_kinematic().with_translation(
            door_center_x,
            frame_y,
            frame_z,
        );
    }

    if let Some(collider) = world.core.get_collider_mut(door_entity) {
        *collider = ColliderComponent::new_cuboid(
            door_width / 2.0,
            door_height / 2.0,
            door_thickness / 2.0,
        )
        .with_friction(0.5);
    }

    let door_rb_handle = {
        let rigid_body_comp = world.core.get_rigid_body(door_entity).cloned().unwrap();
        let collider_comp = world.core.get_collider(door_entity).cloned();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(collider_comp) = collider_comp {
            let collider = collider_comp.to_rapier_collider();
            world.resources.physics.add_collider(collider, handle);
        }
        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(door_entity) {
            rigid_body_mut.handle = Some(handle.into());
        }
        handle
    };

    let door_frame_material =
        create_textured_material(nalgebra_glm::vec3(0.35, 0.25, 0.15), 0.85, 0.0);

    spawn_visual_cube(
        world,
        nalgebra_glm::vec3(frame_center_x, door_height + frame_thickness / 2.0, frame_z),
        nalgebra_glm::vec3(
            door_width + frame_thickness * 2.0,
            frame_thickness,
            frame_depth,
        ),
        door_frame_material.clone(),
        "Door Frame Top".to_string(),
    );

    spawn_visual_cube(
        world,
        nalgebra_glm::vec3(frame_left_x + frame_thickness / 2.0, frame_y, frame_z),
        nalgebra_glm::vec3(frame_thickness, door_height, frame_depth),
        door_frame_material.clone(),
        "Door Frame Left".to_string(),
    );

    spawn_visual_cube(
        world,
        nalgebra_glm::vec3(frame_right_x - frame_thickness / 2.0, frame_y, frame_z),
        nalgebra_glm::vec3(frame_thickness, door_height, frame_depth),
        door_frame_material,
        "Door Frame Right".to_string(),
    );

    let game_entity = game_world.spawn_entities(DOOR, 1)[0];
    game_world.set_door(
        game_entity,
        Door {
            entity: door_entity,
            rigid_body_handle: door_rb_handle,
            hinge_position,
            door_half_width: door_width / 2.0,
            current_angle: 0.0,
            angular_velocity: 0.0,
            min_angle: -std::f32::consts::FRAC_PI_2 * 0.9,
            max_angle: std::f32::consts::FRAC_PI_2 * 0.9,
        },
    );
}

pub(super) fn spawn_drawer_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    let cabinet_width = 1.0;
    let cabinet_height = 1.2;
    let cabinet_depth = 0.6;
    let cabinet_x = center.x;
    let cabinet_z = center.z;
    let cabinet_bottom_y = 0.0;

    let cabinet_material =
        create_textured_material(nalgebra_glm::vec3(0.4, 0.3, 0.2), 0.85, 0.0);

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(
            cabinet_x,
            cabinet_bottom_y + cabinet_height / 2.0,
            cabinet_z - cabinet_depth / 2.0 + 0.05,
        ),
        nalgebra_glm::vec3(cabinet_width, cabinet_height, cabinet_depth - 0.1),
        cabinet_material,
    );

    let drawer_front_material =
        create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.75, 0.0);
    let drawer_interior_material =
        create_textured_material(nalgebra_glm::vec3(0.6, 0.55, 0.45), 0.9, 0.0);

    let drawer_count = 3;
    let drawer_height = 0.3;
    let drawer_gap = 0.05;
    let drawer_inner_width = cabinet_width - 0.1;
    let drawer_inner_depth = cabinet_depth - 0.1;
    let drawer_inner_height = drawer_height - 0.05;
    let panel_thickness = 0.02;
    let max_slide = cabinet_depth * 0.6;

    for index in 0..drawer_count {
        let drawer_y = cabinet_bottom_y
            + drawer_gap
            + drawer_height / 2.0
            + index as f32 * (drawer_height + drawer_gap);
        let drawer_closed_z = cabinet_z - drawer_inner_depth / 2.0;
        let closed_position = nalgebra_glm::vec3(cabinet_x, drawer_y, drawer_closed_z);

        let drawer_parent = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(drawer_parent) {
            name.0 = format!("Drawer {}", index + 1);
        }

        if let Some(transform) = world.core.get_local_transform_mut(drawer_parent) {
            transform.translation = closed_position;
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(drawer_parent) {
            *rigid_body = RigidBodyComponent::new_kinematic().with_translation(
                closed_position.x,
                closed_position.y,
                closed_position.z,
            );
        }

        if let Some(collider) = world.core.get_collider_mut(drawer_parent) {
            *collider = ColliderComponent::new_cuboid(
                drawer_inner_width / 2.0,
                drawer_inner_height / 2.0,
                drawer_inner_depth / 2.0,
            )
            .with_friction(0.3);
        }

        let front_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | PARENT
                | VISIBILITY,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(front_entity) {
            name.0 = format!("Drawer {} Front", index + 1);
        }

        if let Some(transform) = world.core.get_local_transform_mut(front_entity) {
            transform.translation = nalgebra_glm::vec3(0.0, 0.0, drawer_inner_depth / 2.0);
            transform.scale = nalgebra_glm::vec3(cabinet_width, drawer_height, panel_thickness);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(front_entity) {
            mesh.name = "Cube".to_string();
        }

        let material_name = format!("DrawerFront_{}", front_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            drawer_front_material.clone(),
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
            .set_material_ref(front_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(front_entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
        }

        if let Some(parent) = world.core.get_parent_mut(front_entity) {
            *parent = Parent(Some(drawer_parent));
        }

        spawn_drawer_panel(
            world,
            drawer_parent,
            nalgebra_glm::vec3(0.0, -drawer_inner_height / 2.0, 0.0),
            nalgebra_glm::vec3(drawer_inner_width, panel_thickness, drawer_inner_depth),
            drawer_interior_material.clone(),
            format!("Drawer {} Bottom", index + 1),
        );

        spawn_drawer_panel(
            world,
            drawer_parent,
            nalgebra_glm::vec3(-drawer_inner_width / 2.0, 0.0, 0.0),
            nalgebra_glm::vec3(panel_thickness, drawer_inner_height, drawer_inner_depth),
            drawer_interior_material.clone(),
            format!("Drawer {} Left", index + 1),
        );

        spawn_drawer_panel(
            world,
            drawer_parent,
            nalgebra_glm::vec3(drawer_inner_width / 2.0, 0.0, 0.0),
            nalgebra_glm::vec3(panel_thickness, drawer_inner_height, drawer_inner_depth),
            drawer_interior_material.clone(),
            format!("Drawer {} Right", index + 1),
        );

        spawn_drawer_panel(
            world,
            drawer_parent,
            nalgebra_glm::vec3(0.0, 0.0, -drawer_inner_depth / 2.0),
            nalgebra_glm::vec3(drawer_inner_width, drawer_inner_height, panel_thickness),
            drawer_interior_material.clone(),
            format!("Drawer {} Back", index + 1),
        );

        let drawer_rb_handle = {
            let rigid_body_comp = world.core.get_rigid_body(drawer_parent).cloned().unwrap();
            let collider_comp = world.core.get_collider(drawer_parent).cloned();
            let rigid_body = rigid_body_comp.to_rapier_rigid_body();
            let handle = world.resources.physics.add_rigid_body(rigid_body);
            if let Some(collider_comp) = collider_comp {
                let collider = collider_comp.to_rapier_collider();
                world.resources.physics.add_collider(collider, handle);
            }
            if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(drawer_parent) {
                rigid_body_mut.handle = Some(handle.into());
            }
            handle
        };

        let game_entity = game_world.spawn_entities(DRAWER, 1)[0];
        game_world.set_drawer(
            game_entity,
            Drawer {
                entity: drawer_parent,
                front_entity,
                rigid_body_handle: drawer_rb_handle,
                closed_position,
                current_offset: 0.0,
                velocity: 0.0,
                max_offset: max_slide,
            },
        );
    }
}

pub(super) fn spawn_lever_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    let base_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.3, 0.32), 0.8, 0.1);
    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, 0.75, center.z),
        nalgebra_glm::vec3(0.8, 1.5, 0.4),
        base_material,
    );

    let pivot_position = nalgebra_glm::vec3(center.x, 1.2, center.z + 0.21);
    let arm_half_length = 0.2;
    let arm_half_thickness = 0.03;
    let handle_radius = 0.05;

    let pivot_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(pivot_entity) {
        name.0 = "Lever Pivot".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(pivot_entity) {
        transform.translation = pivot_position;
    }

    let lever_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.35, 0.2), 0.7, 0.2);

    let arm_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | PARENT
            | VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(arm_entity) {
        name.0 = "Lever Arm".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(arm_entity) {
        transform.translation = nalgebra_glm::vec3(0.0, 0.0, arm_half_length);
        transform.scale = nalgebra_glm::vec3(
            arm_half_thickness * 2.0,
            arm_half_thickness * 2.0,
            arm_half_length * 2.0,
        );
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(arm_entity) {
        mesh.name = "Cube".to_string();
    }

    let material_name = format!("LeverArm_{}", arm_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        lever_material.clone(),
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
        .set_material_ref(arm_entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(arm_entity) {
        *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(parent) = world.core.get_parent_mut(arm_entity) {
        *parent = Parent(Some(pivot_entity));
    }

    let handle_material =
        create_textured_material(nalgebra_glm::vec3(0.15, 0.15, 0.17), 0.3, 0.7);
    let handle_offset = arm_half_length * 2.0 + handle_radius;

    let handle_visual_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | PARENT
            | VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(handle_visual_entity) {
        name.0 = "Lever Handle Visual".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(handle_visual_entity) {
        transform.translation = nalgebra_glm::vec3(0.0, 0.0, handle_offset);
        transform.scale = nalgebra_glm::vec3(
            handle_radius * 2.0,
            handle_radius * 2.0,
            handle_radius * 2.0,
        );
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(handle_visual_entity) {
        mesh.name = "Sphere".to_string();
    }

    let material_name = format!("LeverHandle_{}", handle_visual_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        handle_material,
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
        .set_material_ref(handle_visual_entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(handle_visual_entity) {
        *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Sphere");
    }

    if let Some(parent) = world.core.get_parent_mut(handle_visual_entity) {
        *parent = Parent(Some(pivot_entity));
    }

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
            | nightshade::ecs::world::RIGID_BODY
            | nightshade::ecs::world::COLLIDER
            | nightshade::ecs::world::BOUNDING_VOLUME
            | nightshade::ecs::world::VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(collider_entity) {
        name.0 = "Lever Collider".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(collider_entity) {
        transform.translation = collider_world_position;
        transform.scale = nalgebra_glm::vec3(
            arm_half_thickness * 2.0,
            arm_half_thickness * 2.0,
            collider_half_length * 2.0,
        );
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(collider_entity) {
        *rigid_body = RigidBodyComponent::new_kinematic().with_translation(
            collider_world_position.x,
            collider_world_position.y,
            collider_world_position.z,
        );
    }

    let hitbox_padding = 0.08;
    if let Some(collider) = world.core.get_collider_mut(collider_entity) {
        *collider = ColliderComponent::new_cuboid(
            arm_half_thickness + hitbox_padding,
            arm_half_thickness + hitbox_padding,
            collider_half_length,
        )
        .with_friction(0.5);
    }

    if let Some(bv) = world.core.get_bounding_volume_mut(collider_entity) {
        *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
    }

    let collider_rb_handle = {
        let rigid_body_comp = world.core.get_rigid_body(collider_entity).cloned().unwrap();
        let collider_comp = world.core.get_collider(collider_entity).cloned();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let rb_handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(collider_comp) = collider_comp {
            let collider = collider_comp.to_rapier_collider();
            world.resources.physics.add_collider(collider, rb_handle);
        }
        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(collider_entity) {
            rigid_body_mut.handle = Some(rb_handle.into());
        }
        rb_handle
    };

    let game_entity = game_world.spawn_entities(LEVER, 1)[0];
    game_world.set_lever(
        game_entity,
        Lever {
            pivot_entity,
            collider_entity,
            collider_rb_handle,
            pivot_position,
            arm_half_length: collider_half_length,
            current_angle: -std::f32::consts::FRAC_PI_4,
            angular_velocity: 0.0,
            min_angle: -std::f32::consts::FRAC_PI_4,
            max_angle: std::f32::consts::FRAC_PI_3,
        },
    );

    let lever_entities: Vec<freecs::Entity> = game_world
        .query_entities(crate::ecs::LEVER)
        .collect();
    if let Some(&last_lever_entity) = lever_entities.last() {
        crate::systems::interaction::apply_lever_transform(game_world, world, last_lever_entity);
    }
}

pub(super) fn spawn_wheel_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    let mount_material =
        create_textured_material(nalgebra_glm::vec3(0.35, 0.35, 0.38), 0.85, 0.1);
    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, 0.75, center.z - 0.2),
        nalgebra_glm::vec3(0.3, 1.5, 0.4),
        mount_material,
    );

    let wheel_center = nalgebra_glm::vec3(center.x, 1.2, center.z + 0.15);
    let wheel_radius = 0.4;
    let wheel_thickness = 0.08;

    let wheel_material =
        create_textured_material(nalgebra_glm::vec3(0.5, 0.35, 0.2), 0.75, 0.1);

    let wheel_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY
            | nightshade::ecs::world::RIGID_BODY
            | nightshade::ecs::world::COLLIDER,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(wheel_entity) {
        name.0 = "Wheel".to_string();
    }

    let base_rotation = nalgebra_glm::quat_angle_axis(
        std::f32::consts::FRAC_PI_2,
        &nalgebra_glm::vec3(1.0, 0.0, 0.0),
    );

    if let Some(transform) = world.core.get_local_transform_mut(wheel_entity) {
        transform.translation = wheel_center;
        transform.scale = nalgebra_glm::vec3(wheel_radius, wheel_thickness / 2.0, wheel_radius);
        transform.rotation = base_rotation;
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(wheel_entity) {
        mesh.name = "Cylinder".to_string();
    }

    let material_name = format!("Wheel_{}", wheel_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        wheel_material,
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
        .set_material_ref(wheel_entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(wheel_entity) {
        *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(wheel_entity) {
        *rigid_body = RigidBodyComponent::new_kinematic()
            .with_translation(wheel_center.x, wheel_center.y, wheel_center.z)
            .with_rotation(
                base_rotation.i,
                base_rotation.j,
                base_rotation.k,
                base_rotation.w,
            );
    }

    if let Some(collider) = world.core.get_collider_mut(wheel_entity) {
        *collider = ColliderComponent::new_cylinder(wheel_thickness / 2.0, wheel_radius)
            .with_friction(0.5);
    }

    let spoke_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.2, 0.15), 0.8, 0.0);
    let mut spoke_entities = Vec::new();
    for spoke_index in 0..4 {
        let angle = spoke_index as f32 * std::f32::consts::FRAC_PI_2;
        let spoke_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | PARENT
                | VISIBILITY,
            1,
        )[0];

        spoke_entities.push(spoke_entity);

        if let Some(name) = world.core.get_name_mut(spoke_entity) {
            name.0 = format!("Wheel Spoke {}", spoke_index + 1);
        }

        if let Some(transform) = world.core.get_local_transform_mut(spoke_entity) {
            transform.translation = nalgebra_glm::vec3(0.0, 0.0, 0.0);
            transform.scale = nalgebra_glm::vec3(0.04 / wheel_radius, 1.8, 0.04 / wheel_radius);
            transform.rotation =
                nalgebra_glm::quat_angle_axis(angle, &nalgebra_glm::vec3(0.0, 0.0, 1.0));
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(spoke_entity) {
            mesh.name = "Cube".to_string();
        }

        let material_name = format!("WheelSpoke_{}", spoke_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            spoke_material.clone(),
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
            .set_material_ref(spoke_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(spoke_entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
        }

        if let Some(parent) = world.core.get_parent_mut(spoke_entity) {
            *parent = Parent(Some(wheel_entity));
        }
    }

    let wheel_rb_handle = {
        let rigid_body_comp = world.core.get_rigid_body(wheel_entity).cloned().unwrap();
        let collider_comp = world.core.get_collider(wheel_entity).cloned();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(collider_comp) = collider_comp {
            let collider = collider_comp.to_rapier_collider();
            world.resources.physics.add_collider(collider, handle);
        }
        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(wheel_entity) {
            rigid_body_mut.handle = Some(handle.into());
        }
        handle
    };

    let game_entity = game_world.spawn_entities(WHEEL, 1)[0];
    game_world.set_wheel(
        game_entity,
        Wheel {
            entity: wheel_entity,
            spoke_entities,
            rigid_body_handle: wheel_rb_handle,
            center_position: wheel_center,
            current_angle: 0.0,
            angular_velocity: 0.0,
        },
    );
}

pub(super) fn spawn_chain_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    use rapier3d::prelude::*;

    let anchor_height = 2.5;
    let anchor_position = nalgebra_glm::vec3(center.x, anchor_height, center.z);

    let beam_material =
        create_textured_material(nalgebra_glm::vec3(0.35, 0.25, 0.15), 0.9, 0.0);
    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, anchor_height + 0.1, center.z),
        nalgebra_glm::vec3(0.4, 0.2, 0.4),
        beam_material,
    );

    let anchor_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::RIGID_BODY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(anchor_entity) {
        name.0 = "Chain Anchor".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(anchor_entity) {
        transform.translation = anchor_position;
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(anchor_entity) {
        *rigid_body = RigidBodyComponent::new_static().with_translation(
            anchor_position.x,
            anchor_position.y,
            anchor_position.z,
        );
    }

    let anchor_handle = {
        let rigid_body_comp = world.core.get_rigid_body(anchor_entity).cloned().unwrap();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(anchor_entity) {
            rigid_body_mut.handle = Some(handle.into());
        }
        handle
    };

    let chain_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.3, 0.32), 0.4, 0.8);

    let num_links = 8;
    let link_length = 0.15;
    let link_radius = 0.02;

    let mut _link_entities = Vec::new();
    let mut _link_handles = Vec::new();
    let mut prev_handle: Option<RigidBodyHandle> = Some(anchor_handle);

    for link_index in 0..num_links {
        let link_y = anchor_height - (link_index as f32 + 0.5) * link_length;
        let link_position = nalgebra_glm::vec3(center.x, link_y, center.z);

        let entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | VISIBILITY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER
                | nightshade::ecs::world::PHYSICS_INTERPOLATION,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(entity) {
            name.0 = format!("Chain Link {}", link_index + 1);
        }

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = link_position;
            transform.scale =
                nalgebra_glm::vec3(link_radius * 2.0, link_length, link_radius * 2.0);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
            mesh.name = "Cylinder".to_string();
        }

        let material_name = format!("ChainLink_{}", entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            chain_material.clone(),
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
            .set_material_ref(entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(entity) {
            *rigid_body = RigidBodyComponent::new_dynamic()
                .with_translation(link_position.x, link_position.y, link_position.z)
                .with_mass(0.1);
        }

        if let Some(collider) = world.core.get_collider_mut(entity) {
            *collider =
                ColliderComponent::new_capsule(link_length / 2.0 - link_radius, link_radius)
                    .with_friction(0.3);
        }

        let handle = {
            let rigid_body_comp = world.core.get_rigid_body(entity).cloned().unwrap();
            let collider_comp = world.core.get_collider(entity).cloned();
            let rigid_body = rigid_body_comp.to_rapier_rigid_body();
            let handle = world.resources.physics.add_rigid_body(rigid_body);
            if let Some(collider_comp) = collider_comp {
                let collider = collider_comp.to_rapier_collider();
                world.resources.physics.add_collider(collider, handle);
            }
            if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(entity) {
                rigid_body_mut.handle = Some(handle.into());
            }
            world
                .resources
                .physics
                .handle_to_entity
                .insert(handle, entity);
            world
                .resources
                .physics
                .entity_to_handle
                .insert(entity, handle);
            if let Some(interpolation) = world.core.get_physics_interpolation_mut(entity) {
                interpolation.previous_translation = link_position;
                interpolation.previous_rotation = nalgebra_glm::quat_identity();
                interpolation.current_translation = link_position;
                interpolation.current_rotation = nalgebra_glm::quat_identity();
                interpolation.enabled = true;
            }
            if let Some(rb) = world.resources.physics.rigid_body_set.get_mut(handle) {
                rb.set_linear_damping(0.5);
                rb.set_angular_damping(0.5);
            }
            handle
        };

        if let Some(prev) = prev_handle {
            let local_anchor1 = if link_index == 0 {
                point![0.0, 0.0, 0.0]
            } else {
                point![0.0, -link_length / 2.0, 0.0]
            };
            let joint = SphericalJointBuilder::new()
                .local_anchor1(local_anchor1)
                .local_anchor2(point![0.0, link_length / 2.0, 0.0]);
            world.resources.physics.add_joint(prev, handle, joint);
        }

        _link_entities.push(entity);
        _link_handles.push(handle);
        prev_handle = Some(handle);
    }

    let lantern_material = create_emissive_material(nalgebra_glm::vec3(1.0, 0.8, 0.4), 2.0);

    let lantern_y = anchor_height - (num_links as f32 * link_length) - 0.15;
    let lantern_position = nalgebra_glm::vec3(center.x, lantern_y, center.z);

    let lantern_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY
            | nightshade::ecs::world::RIGID_BODY
            | nightshade::ecs::world::COLLIDER
            | nightshade::ecs::world::PHYSICS_INTERPOLATION,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(lantern_entity) {
        name.0 = "Lantern".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(lantern_entity) {
        transform.translation = lantern_position;
        transform.scale = nalgebra_glm::vec3(0.25, 0.35, 0.25);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(lantern_entity) {
        mesh.name = "Cube".to_string();
    }

    let material_name = format!("Lantern_{}", lantern_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        lantern_material,
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
        .set_material_ref(lantern_entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(lantern_entity) {
        *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(lantern_entity) {
        *rigid_body = RigidBodyComponent::new_dynamic()
            .with_translation(lantern_position.x, lantern_position.y, lantern_position.z)
            .with_mass(0.5);
    }

    if let Some(collider) = world.core.get_collider_mut(lantern_entity) {
        *collider = ColliderComponent::new_cuboid(0.125, 0.175, 0.125).with_friction(0.5);
    }

    let lantern_handle = {
        let rigid_body_comp = world.core.get_rigid_body(lantern_entity).cloned().unwrap();
        let collider_comp = world.core.get_collider(lantern_entity).cloned();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(collider_comp) = collider_comp {
            let collider = collider_comp.to_rapier_collider();
            world.resources.physics.add_collider(collider, handle);
        }
        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(lantern_entity) {
            rigid_body_mut.handle = Some(handle.into());
        }
        if let Some(rb) = world.resources.physics.rigid_body_set.get_mut(handle) {
            rb.set_linear_damping(0.5);
            rb.set_angular_damping(0.5);
        }
        handle
    };

    world
        .resources
        .physics
        .handle_to_entity
        .insert(lantern_handle, lantern_entity);
    world
        .resources
        .physics
        .entity_to_handle
        .insert(lantern_entity, lantern_handle);

    if let Some(interpolation) = world.core.get_physics_interpolation_mut(lantern_entity) {
        interpolation.previous_translation = lantern_position;
        interpolation.previous_rotation = nalgebra_glm::quat_identity();
        interpolation.current_translation = lantern_position;
        interpolation.current_rotation = nalgebra_glm::quat_identity();
        interpolation.enabled = true;
    }

    if let Some(last_link_handle) = prev_handle {
        let joint = SphericalJointBuilder::new()
            .local_anchor1(point![0.0, -link_length / 2.0, 0.0])
            .local_anchor2(point![0.0, 0.175, 0.0]);
        world
            .resources
            .physics
            .add_joint(last_link_handle, lantern_handle, joint);
    }

    let light_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | LIGHT,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(light_entity) {
        name.0 = "Lantern Light".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(light_entity) {
        transform.translation = lantern_position;
    }

    if let Some(light) = world.core.get_light_mut(light_entity) {
        *light = Light {
            light_type: LightType::Point,
            color: nalgebra_glm::vec3(1.0, 0.85, 0.6),
            intensity: 12.0,
            range: 15.0,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: true,
            shadow_bias: 0.005,
        };
    }

    game_world.resources.lantern_entity = Some(lantern_entity);
    game_world.resources.lantern_light_entity = Some(light_entity);
    game_world.resources.physics_objects.push(lantern_entity);
}

fn spawn_drawer_panel(
    world: &mut World,
    parent: Entity,
    local_position: Vec3,
    scale: Vec3,
    material: nightshade::ecs::material::components::Material,
    name: String,
) {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | PARENT
            | VISIBILITY,
        1,
    )[0];

    if let Some(n) = world.core.get_name_mut(entity) {
        n.0 = name;
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = local_position;
        transform.scale = scale;
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = "Cube".to_string();
    }

    let material_name = format!("DrawerPanel_{}", entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        material,
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
        .set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
        *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(p) = world.core.get_parent_mut(entity) {
        *p = Parent(Some(parent));
    }
}

fn spawn_bauble(
    game_world: &mut GameWorld,
    world: &mut World,
    world_position: Vec3,
    radius: f32,
    color: Vec3,
    name: String,
) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY
            | nightshade::ecs::world::RIGID_BODY
            | nightshade::ecs::world::COLLIDER,
        1,
    )[0];

    if let Some(n) = world.core.get_name_mut(entity) {
        n.0 = name;
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = world_position;
        transform.scale = nalgebra_glm::vec3(radius, radius, radius);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = "Sphere".to_string();
    }

    let material_name = format!("Bauble_{}", entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        create_textured_material(color, 0.2, 0.8),
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
        .set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
        *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Sphere");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(entity) {
        *rigid_body = RigidBodyComponent::new_dynamic()
            .with_translation(world_position.x, world_position.y, world_position.z)
            .with_mass(0.1);
    }

    if let Some(collider) = world.core.get_collider_mut(entity) {
        *collider = ColliderComponent::new_ball(radius)
            .with_friction(0.5)
            .with_restitution(0.3);
    }

    let rigid_body_comp = world.core.get_rigid_body(entity).cloned().unwrap();
    let collider_comp = world.core.get_collider(entity).cloned();
    let rigid_body = rigid_body_comp.to_rapier_rigid_body();
    let handle = world.resources.physics.add_rigid_body(rigid_body);
    if let Some(collider_comp) = collider_comp {
        let collider = collider_comp.to_rapier_collider();
        world.resources.physics.add_collider(collider, handle);
    }
    if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(entity) {
        rigid_body_mut.handle = Some(handle.into());
    }
    world
        .resources
        .physics
        .handle_to_entity
        .insert(handle, entity);
    world
        .resources
        .physics
        .entity_to_handle
        .insert(entity, handle);

    game_world.resources.physics_objects.push(entity);
    entity
}

pub(super) fn spawn_bauble_table(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    let table_top_material =
        create_textured_material(nalgebra_glm::vec3(0.45, 0.32, 0.2), 0.7, 0.1);
    let table_leg_material =
        create_textured_material(nalgebra_glm::vec3(0.35, 0.25, 0.15), 0.85, 0.0);

    let table_top_y = 0.75;
    let table_top_thickness = 0.05;
    let table_width = 1.4;
    let table_depth = 1.4;
    let leg_thickness = 0.08;
    let leg_height = table_top_y - table_top_thickness / 2.0;

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, table_top_y, center.z),
        nalgebra_glm::vec3(table_width, table_top_thickness, table_depth),
        table_top_material,
    );

    let leg_offset_x = table_width / 2.0 - leg_thickness / 2.0 - 0.05;
    let leg_offset_z = table_depth / 2.0 - leg_thickness / 2.0 - 0.05;
    let leg_positions = [
        (leg_offset_x, leg_offset_z),
        (-leg_offset_x, leg_offset_z),
        (leg_offset_x, -leg_offset_z),
        (-leg_offset_x, -leg_offset_z),
    ];

    for (offset_x, offset_z) in leg_positions {
        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x + offset_x, leg_height / 2.0, center.z + offset_z),
            nalgebra_glm::vec3(leg_thickness, leg_height, leg_thickness),
            table_leg_material.clone(),
        );
    }

    let table_top_y = table_top_y + table_top_thickness / 2.0;
    game_world.resources.bauble_table_center = center;
    game_world.resources.bauble_table_top_y = table_top_y;

    spawn_recall_pedestal(game_world, world, nalgebra_glm::vec3(center.x + 2.0, 0.0, center.z));
    let bauble_colors = [
        nalgebra_glm::vec3(0.9, 0.2, 0.2),
        nalgebra_glm::vec3(0.2, 0.8, 0.3),
        nalgebra_glm::vec3(0.2, 0.4, 0.9),
        nalgebra_glm::vec3(0.9, 0.8, 0.1),
        nalgebra_glm::vec3(0.8, 0.3, 0.8),
        nalgebra_glm::vec3(0.1, 0.8, 0.8),
        nalgebra_glm::vec3(0.9, 0.5, 0.2),
        nalgebra_glm::vec3(0.6, 0.2, 0.6),
    ];

    let mut bauble_positions = Vec::new();
    let mut rng_seed = 12345u32;
    for _ in 0..80 {
        rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let offset_x = ((rng_seed % 1000) as f32 / 1000.0 - 0.5) * 1.1;
        rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let offset_z = ((rng_seed % 1000) as f32 / 1000.0 - 0.5) * 1.1;
        rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let radius = 0.035 + ((rng_seed % 1000) as f32 / 1000.0) * 0.035;
        bauble_positions.push((offset_x, offset_z, radius));
    }

    for (index, (offset_x, offset_z, radius)) in bauble_positions.iter().enumerate() {
        let color = bauble_colors[index % bauble_colors.len()];
        let pos = nalgebra_glm::vec3(
            center.x + offset_x,
            table_top_y + radius + 0.01,
            center.z + offset_z,
        );
        let entity =
            spawn_bauble(game_world, world, pos, *radius, color, format!("Bauble {}", index + 1));
        let game_entity = game_world.spawn_entities(BAUBLE_SPAWN, 1)[0];
        game_world.set_bauble_spawn(
            game_entity,
            BaubleSpawn {
                entity,
                spawn_position: pos,
            },
        );
    }
}

pub(super) fn spawn_note_table(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    let table_top_material =
        create_textured_material(nalgebra_glm::vec3(0.35, 0.25, 0.15), 0.8, 0.0);
    let table_leg_material =
        create_textured_material(nalgebra_glm::vec3(0.3, 0.2, 0.1), 0.85, 0.0);

    let table_top_y = 0.75;
    let table_top_thickness = 0.04;
    let table_width = 0.8;
    let table_depth = 0.5;
    let leg_thickness = 0.06;
    let leg_height = table_top_y - table_top_thickness / 2.0;

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, table_top_y, center.z),
        nalgebra_glm::vec3(table_width, table_top_thickness, table_depth),
        table_top_material,
    );

    let leg_offset_x = table_width / 2.0 - leg_thickness / 2.0 - 0.02;
    let leg_offset_z = table_depth / 2.0 - leg_thickness / 2.0 - 0.02;
    let leg_positions = [
        (leg_offset_x, leg_offset_z),
        (-leg_offset_x, leg_offset_z),
        (leg_offset_x, -leg_offset_z),
        (-leg_offset_x, -leg_offset_z),
    ];

    for (offset_x, offset_z) in leg_positions {
        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x + offset_x, leg_height / 2.0, center.z + offset_z),
            nalgebra_glm::vec3(leg_thickness, leg_height, leg_thickness),
            table_leg_material.clone(),
        );
    }

    let note_y = table_top_y + table_top_thickness / 2.0 + 0.005;
    let note_material = create_textured_material(nalgebra_glm::vec3(0.9, 0.85, 0.7), 0.95, 0.0);

    let note_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY
            | nightshade::ecs::world::RIGID_BODY
            | nightshade::ecs::world::COLLIDER,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(note_entity) {
        name.0 = "Note".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(note_entity) {
        transform.translation = nalgebra_glm::vec3(center.x, note_y, center.z);
        transform.scale = nalgebra_glm::vec3(0.15, 0.002, 0.2);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(note_entity) {
        mesh.name = "Cube".to_string();
    }

    let material_name = format!("Note_{}", note_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        note_material,
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
        .set_material_ref(note_entity, MaterialRef::new(material_name));

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(note_entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(note_entity) {
        *rigid_body =
            RigidBodyComponent::new_static().with_translation(center.x, note_y, center.z);
    }

    if let Some(collider) = world.core.get_collider_mut(note_entity) {
        *collider = ColliderComponent::new_cuboid(0.075, 0.001, 0.1).with_friction(0.5);
    }

    let rigid_body_comp = world.core.get_rigid_body(note_entity).cloned().unwrap();
    let collider_comp = world.core.get_collider(note_entity).cloned();
    let rigid_body = rigid_body_comp.to_rapier_rigid_body();
    let handle = world.resources.physics.add_rigid_body(rigid_body);
    if let Some(collider_comp) = collider_comp {
        let collider = collider_comp.to_rapier_collider();
        world.resources.physics.add_collider(collider, handle);
    }
    if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(note_entity) {
        rigid_body_mut.handle = Some(handle.into());
    }

    let game_entity = game_world.spawn_entities(NOTE, 1)[0];
    game_world.set_note(
        game_entity,
        Note {
            entity: note_entity,
            title: "Engineer's Log - Day 37".to_string(),
            content: "The generator keeps failing. I've replaced the fuel lines twice now, \
but something else is draining the power. The lights flicker at night, \
and I hear... things... in the walls.\n\n\
Whatever is down here, it doesn't want us to leave.\n\n\
If you find this note, get out while you still can. \
Don't go to the lower levels. Don't follow the sounds.\n\n\
                - M. Richter"
                .to_string(),
        },
    );
}

fn spawn_recall_pedestal(game_world: &mut GameWorld, world: &mut World, center: Vec3) {
    let pedestal_material =
        create_textured_material(nalgebra_glm::vec3(0.3, 0.3, 0.35), 0.85, 0.0);

    let pedestal_height = 1.0;
    let pedestal_width = 0.4;

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, pedestal_height / 2.0, center.z),
        nalgebra_glm::vec3(pedestal_width, pedestal_height, pedestal_width),
        pedestal_material,
    );

    let button_radius = 0.12;
    let button_height = 0.06;
    let button_base_y = pedestal_height + button_height / 2.0;

    let mut button_material =
        create_textured_material(nalgebra_glm::vec3(0.8, 0.15, 0.15), 0.3, 0.6);
    button_material.emissive_factor = [0.4, 0.05, 0.05];

    let button_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY
            | nightshade::ecs::world::RIGID_BODY
            | nightshade::ecs::world::COLLIDER,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(button_entity) {
        name.0 = "Recall Button".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(button_entity) {
        transform.translation = nalgebra_glm::vec3(center.x, button_base_y, center.z);
        transform.scale = nalgebra_glm::vec3(button_radius, button_height / 2.0, button_radius);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(button_entity) {
        mesh.name = "Cylinder".to_string();
    }

    let material_name = format!("Button_{}", button_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        button_material,
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
        .set_material_ref(button_entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(button_entity) {
        *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(button_entity) {
        *rigid_body = RigidBodyComponent::new_kinematic().with_translation(
            center.x,
            button_base_y,
            center.z,
        );
    }

    if let Some(collider) = world.core.get_collider_mut(button_entity) {
        *collider = ColliderComponent::new_cylinder(button_height / 2.0, button_radius);
    }

    let rigid_body_comp = world.core.get_rigid_body(button_entity).cloned().unwrap();
    let collider_comp = world.core.get_collider(button_entity).cloned();
    let rigid_body = rigid_body_comp.to_rapier_rigid_body();
    let handle = world.resources.physics.add_rigid_body(rigid_body);
    if let Some(collider_comp) = collider_comp {
        let collider = collider_comp.to_rapier_collider();
        world.resources.physics.add_collider(collider, handle);
    }
    if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(button_entity) {
        rigid_body_mut.handle = Some(handle.into());
    }

    let game_entity = game_world.spawn_entities(BUTTON, 1)[0];
    game_world.set_button(
        game_entity,
        Button {
            entity: button_entity,
            base_position: nalgebra_glm::vec3(center.x, button_base_y, center.z),
            current_press: 0.0,
            is_pressed: false,
            action: ButtonAction::RecallBaubles,
        },
    );
}
