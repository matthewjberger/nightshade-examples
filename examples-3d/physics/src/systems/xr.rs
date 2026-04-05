use crate::ecs::GameWorld;
use nightshade::ecs::material::resources::registry_entry_by_name_mut;
use nightshade::ecs::transform::components::Parent;
use nightshade::prelude::*;

pub fn spawn_hand_cube(world: &mut World, color: Vec3) -> Entity {
    let cube_size = 0.08;
    let material = create_textured_material(color, 0.3, 0.7);

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(entity) {
        name.0 = "HandCube".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = nalgebra_glm::vec3(0.0, -100.0, 0.0);
        transform.scale = nalgebra_glm::vec3(cube_size, cube_size, cube_size);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = "Cube".to_string();
    }

    let material_name = format!("HandCube_{}", entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        material,
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
        .set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
        *bv = BoundingVolume::from_mesh_type("Cube");
    }

    entity
}

pub fn spawn_bauble_gun(game_world: &mut GameWorld, world: &mut World, hand_entity: Entity) {
    let gun_body_material =
        create_textured_material(nalgebra_glm::vec3(0.15, 0.15, 0.18), 0.6, 0.8);
    let gun_barrel_material =
        create_textured_material(nalgebra_glm::vec3(0.25, 0.25, 0.28), 0.4, 0.9);
    let gun_grip_material =
        create_textured_material(nalgebra_glm::vec3(0.12, 0.08, 0.06), 0.9, 0.0);
    let gun_accent_material =
        create_textured_material(nalgebra_glm::vec3(0.9, 0.4, 0.1), 0.3, 0.7);

    let parts: Vec<(
        &str,
        &str,
        Vec3,
        Vec3,
        nightshade::ecs::material::components::Material,
    )> = vec![
        (
            "GunBody",
            "Cube",
            nalgebra_glm::vec3(0.0, 0.06, 0.0),
            nalgebra_glm::vec3(0.025, 0.015, 0.04),
            gun_body_material,
        ),
        (
            "GunBarrel",
            "Cylinder",
            nalgebra_glm::vec3(0.0, 0.12, 0.0),
            nalgebra_glm::vec3(0.008, 0.04, 0.008),
            gun_barrel_material,
        ),
        (
            "GunGrip",
            "Cube",
            nalgebra_glm::vec3(0.0, 0.0, 0.01),
            nalgebra_glm::vec3(0.015, 0.03, 0.012),
            gun_grip_material,
        ),
        (
            "GunMuzzle",
            "Sphere",
            nalgebra_glm::vec3(0.0, 0.165, 0.0),
            nalgebra_glm::vec3(0.012, 0.012, 0.012),
            gun_accent_material,
        ),
    ];

    for (name, mesh_name, offset, scale, material) in parts {
        let entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | VISIBILITY
                | PARENT,
            1,
        )[0];

        if let Some(n) = world.core.get_name_mut(entity) {
            n.0 = name.to_string();
        }
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = offset;
            transform.scale = scale;
        }
        if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
            mesh.name = mesh_name.to_string();
        }
        if let Some(parent) = world.core.get_parent_mut(entity) {
            *parent = Parent(Some(hand_entity));
        }
        if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
            *bv = BoundingVolume::from_mesh_type(mesh_name);
        }

        let material_name = format!("BaubleGun_{}_{}", name, entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            material,
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
            .set_material_ref(entity, MaterialRef::new(material_name));

        game_world.resources.bauble_gun_entities.push(entity);
    }
}

pub fn xr_hand_tracking_system(game_world: &mut GameWorld, world: &mut World) {
    let Some(xr_input) = world.resources.xr.input.clone() else {
        return;
    };

    if let Some(left_hand_entity) = game_world.resources.left_hand_cube {
        if let Some(left_pos) = xr_input.left_hand_position() {
            if let Some(transform) = world.core.get_local_transform_mut(left_hand_entity) {
                transform.translation = left_pos;
            }
            if let Some(rotation) = xr_input.left_hand_rotation() {
                if let Some(transform) = world.core.get_local_transform_mut(left_hand_entity) {
                    transform.rotation = rotation;
                }
            }
            world.mark_local_transform_dirty(left_hand_entity);
        }

        let left_trigger_pressed = xr_input.left_trigger_pressed();
        let hand_color = if left_trigger_pressed {
            [0.2, 0.9, 0.3, 1.0]
        } else {
            [0.2, 0.6, 0.9, 1.0]
        };
        let left_mat_name = world
            .core
            .get_material_ref(left_hand_entity)
            .map(|r| r.name.clone());
        if let Some(name) = left_mat_name
            && let Some(mat) = registry_entry_by_name_mut(
                &mut world.resources.material_registry.registry,
                &name,
            )
        {
            mat.base_color = hand_color;
        }
    }

    if let Some(right_hand_entity) = game_world.resources.right_hand_cube {
        if let Some(right_pos) = xr_input.right_hand_position() {
            if let Some(transform) = world.core.get_local_transform_mut(right_hand_entity) {
                transform.translation = right_pos;
            }
            if let Some(rotation) = xr_input.right_hand_rotation() {
                if let Some(transform) = world.core.get_local_transform_mut(right_hand_entity) {
                    transform.rotation = rotation;
                }
            }
            world.mark_local_transform_dirty(right_hand_entity);
        }

        let right_trigger_pressed = xr_input.right_trigger_pressed();
        let hand_color = if right_trigger_pressed {
            [0.2, 0.9, 0.3, 1.0]
        } else {
            [0.9, 0.6, 0.2, 1.0]
        };
        let right_mat_name = world
            .core
            .get_material_ref(right_hand_entity)
            .map(|r| r.name.clone());
        if let Some(name) = right_mat_name
            && let Some(mat) = registry_entry_by_name_mut(
                &mut world.resources.material_registry.registry,
                &name,
            )
        {
            mat.base_color = hand_color;
        }
    }
}
