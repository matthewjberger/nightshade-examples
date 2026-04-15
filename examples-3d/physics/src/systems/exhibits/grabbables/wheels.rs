use crate::ecs::{GameWorld, INTERACTABLE, Interactable, InteractableKind, WHEEL, Wheel};
use nightshade::ecs::physics::*;
use nightshade::ecs::transform::components::Parent;
use nightshade::prelude::*;

pub(crate) fn spawn_wheel_exhibit(game_world: &mut GameWorld, world: &mut World, center: Vec3) {
    let mount_material = create_textured_material(nalgebra_glm::vec3(0.35, 0.35, 0.38), 0.85, 0.1);
    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, 0.75, center.z - 0.2),
        nalgebra_glm::vec3(0.3, 1.5, 0.4),
        mount_material,
    );

    let wheel_center = nalgebra_glm::vec3(center.x, 1.2, center.z + 0.15);
    let wheel_radius = 0.4;
    let wheel_thickness = 0.08;

    let wheel_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.35, 0.2), 0.75, 0.1);

    let wheel_entity = world.spawn_entities(
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
        *bv = BoundingVolume::from_mesh_type("Cylinder");
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
        *collider =
            ColliderComponent::new_cylinder(wheel_thickness / 2.0, wheel_radius).with_friction(0.5);
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
            *bv = BoundingVolume::from_mesh_type("Cube");
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

    let interactable_entity = game_world.spawn_entities(INTERACTABLE, 1)[0];
    game_world.set_interactable(
        interactable_entity,
        Interactable {
            engine_entity: wheel_entity,
            game_entity,
            kind: InteractableKind::Wheel,
        },
    );
}
