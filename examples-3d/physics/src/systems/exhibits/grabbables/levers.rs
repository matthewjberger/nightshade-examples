use crate::ecs::{GameWorld, Interactable, InteractableKind, Lever, INTERACTABLE, LEVER};
use nightshade::ecs::physics::*;
use nightshade::ecs::transform::components::Parent;
use nightshade::prelude::*;

pub(crate) fn spawn_lever_exhibit(
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
        *bv = BoundingVolume::from_mesh_type("Cube");
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
        *bv = BoundingVolume::from_mesh_type("Sphere");
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
            | RIGID_BODY
            | COLLIDER
            | BOUNDING_VOLUME
            | VISIBILITY,
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
        *bv = BoundingVolume::from_mesh_type("Cube");
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

    let interactable_entity = game_world.spawn_entities(INTERACTABLE, 1)[0];
    game_world.set_interactable(interactable_entity, Interactable {
        engine_entity: collider_entity,
        game_entity,
        kind: InteractableKind::Lever,
    });

    let lever_entities: Vec<freecs::Entity> = game_world
        .query_entities(crate::ecs::LEVER)
        .collect();
    if let Some(&last_lever_entity) = lever_entities.last() {
        crate::systems::interaction::apply_lever_transform(game_world, world, last_lever_entity);
    }
}
