use crate::ecs::{Door, GameWorld, Interactable, InteractableKind, DOOR, INTERACTABLE};
use super::super::environment::spawn_visual_cube;
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(crate) fn spawn_door_exhibit(
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
            | RIGID_BODY
            | COLLIDER,
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
        *bv = BoundingVolume::from_mesh_type("Cube");
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

    let interactable_entity = game_world.spawn_entities(INTERACTABLE, 1)[0];
    game_world.set_interactable(interactable_entity, Interactable {
        engine_entity: door_entity,
        game_entity,
        kind: InteractableKind::Door,
    });
}
