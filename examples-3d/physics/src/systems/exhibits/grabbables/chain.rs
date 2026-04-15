use crate::ecs::GameWorld;
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(crate) fn spawn_chain_exhibit(game_world: &mut GameWorld, world: &mut World, center: Vec3) {
    use nightshade::ecs::bounding_volume::components::BoundingVolume;
    use rapier3d::prelude::*;

    let anchor_height = 2.5;
    let anchor_position = nalgebra_glm::vec3(center.x, anchor_height, center.z);

    let beam_material = create_textured_material(nalgebra_glm::vec3(0.35, 0.25, 0.15), 0.9, 0.0);
    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, anchor_height + 0.1, center.z),
        nalgebra_glm::vec3(0.4, 0.2, 0.4),
        beam_material,
    );

    let anchor_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | RIGID_BODY,
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
                | RIGID_BODY
                | COLLIDER
                | PHYSICS_INTERPOLATION,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(entity) {
            name.0 = format!("Chain Link {}", link_index + 1);
        }

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = link_position;
            transform.scale = nalgebra_glm::vec3(link_radius * 2.0, link_length, link_radius * 2.0);
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
            *bv = BoundingVolume::from_mesh_type("Cylinder");
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
            | RIGID_BODY
            | COLLIDER
            | PHYSICS_INTERPOLATION,
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
        *bv = BoundingVolume::from_mesh_type("Cube");
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
    crate::systems::spawn::register_grabbable(game_world, lantern_entity);
}
