use crate::ecs::GameWorld;
use nightshade::ecs::generational_registry::registry_entry_by_name_mut;
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

pub fn spawn_bauble_gun(
    game_world: &mut GameWorld,
    world: &mut World,
    hand_entity: Entity,
) -> Entity {
    let hand_cube_scale = 0.08_f32;
    let inverse_scale = 1.0 / hand_cube_scale;

    let root = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | PARENT,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(root) {
        name.0 = "BaubleGun".to_string();
    }
    if let Some(parent) = world.core.get_parent_mut(root) {
        *parent = Parent(Some(hand_entity));
    }
    if let Some(transform) = world.core.get_local_transform_mut(root) {
        let rot_y = nalgebra_glm::quat_angle_axis(
            std::f32::consts::PI,
            &nalgebra_glm::vec3(0.0, 1.0, 0.0),
        );
        let rot_z = nalgebra_glm::quat_angle_axis(
            std::f32::consts::PI,
            &nalgebra_glm::vec3(0.0, 0.0, 1.0),
        );
        transform.rotation = rot_z * rot_y;
        transform.scale = nalgebra_glm::vec3(inverse_scale, inverse_scale, inverse_scale);
    }

    let body_color =
        create_textured_material(nalgebra_glm::vec3(0.18, 0.18, 0.20), 0.45, 0.85);
    let barrel_color =
        create_textured_material(nalgebra_glm::vec3(0.12, 0.12, 0.14), 0.3, 0.9);
    let grip_color =
        create_textured_material(nalgebra_glm::vec3(0.08, 0.08, 0.06), 0.8, 0.1);
    let accent_color =
        create_textured_material(nalgebra_glm::vec3(0.25, 0.25, 0.28), 0.35, 0.8);

    let parts: Vec<(
        &str,
        &str,
        Vec3,
        Vec3,
        nightshade::ecs::material::components::Material,
    )> = vec![
        (
            "WeaponBody",
            "Cube",
            nalgebra_glm::vec3(0.0, 0.0, 0.0),
            nalgebra_glm::vec3(0.025, 0.035, 0.10),
            body_color.clone(),
        ),
        (
            "WeaponBarrel",
            "Cube",
            nalgebra_glm::vec3(0.0, 0.005, -0.09),
            nalgebra_glm::vec3(0.012, 0.012, 0.08),
            barrel_color.clone(),
        ),
        (
            "WeaponMuzzle",
            "Cube",
            nalgebra_glm::vec3(0.0, 0.005, -0.135),
            nalgebra_glm::vec3(0.016, 0.016, 0.01),
            accent_color.clone(),
        ),
        (
            "WeaponGrip",
            "Cube",
            nalgebra_glm::vec3(0.0, -0.045, 0.02),
            nalgebra_glm::vec3(0.02, 0.055, 0.025),
            grip_color,
        ),
        (
            "WeaponRail",
            "Cube",
            nalgebra_glm::vec3(0.0, 0.023, -0.01),
            nalgebra_glm::vec3(0.012, 0.006, 0.06),
            accent_color,
        ),
        (
            "WeaponTriggerGuard",
            "Cube",
            nalgebra_glm::vec3(0.0, -0.018, 0.0),
            nalgebra_glm::vec3(0.018, 0.008, 0.025),
            body_color,
        ),
        (
            "WeaponSight",
            "Cube",
            nalgebra_glm::vec3(0.0, -0.006, -0.055),
            nalgebra_glm::vec3(0.004, 0.004, 0.015),
            barrel_color,
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
            *parent = Parent(Some(root));
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

        world.resources.mesh_render_state.mark_entity_added(entity);
        game_world.resources.bauble_gun_entities.push(entity);
    }

    root
}

pub fn xr_hand_tracking_system(game_world: &mut GameWorld, world: &mut World) {
    let Some(xr_input) = world.resources.xr.input.clone() else {
        return;
    };

    if let Some(left_hand_entity) = game_world.resources.left_hand_cube
        && let Some(left_pos) = xr_input.left_hand_position()
    {
        if let Some(transform) = world.core.get_local_transform_mut(left_hand_entity) {
            transform.translation = left_pos;
        }
        if let Some(rotation) = xr_input.left_hand_rotation()
            && let Some(transform) = world.core.get_local_transform_mut(left_hand_entity)
        {
            transform.rotation = rotation;
        }
        world.mark_local_transform_dirty(left_hand_entity);
    }

    if let Some(right_hand_entity) = game_world.resources.right_hand_cube
        && let Some(right_pos) = xr_input.right_hand_position()
    {
        if let Some(transform) = world.core.get_local_transform_mut(right_hand_entity) {
            transform.translation = right_pos;
        }
        if let Some(rotation) = xr_input.right_hand_rotation()
            && let Some(transform) = world.core.get_local_transform_mut(right_hand_entity)
        {
            transform.rotation = rotation;
        }
        world.mark_local_transform_dirty(right_hand_entity);
    }

    let a_button = xr_input.a_button_pressed();
    let a_just_pressed = a_button && !game_world.resources.xr_a_was_pressed;
    game_world.resources.xr_a_was_pressed = a_button;

    if let Some(gun_root) = game_world.resources.gun_root_entity {
        let new_hand = if a_just_pressed {
            Some(match game_world.resources.gun_hand {
                crate::ecs::GunHand::Left => crate::ecs::GunHand::Right,
                crate::ecs::GunHand::Right => crate::ecs::GunHand::Left,
            })
        } else {
            None
        };

        if let Some(hand) = new_hand {
            let new_parent = match hand {
                crate::ecs::GunHand::Left => game_world.resources.left_hand_cube,
                crate::ecs::GunHand::Right => game_world.resources.right_hand_cube,
            };
            if let Some(parent_entity) = new_parent
                && let Some(parent) = world.core.get_parent_mut(gun_root)
            {
                *parent = Parent(Some(parent_entity));
                world.mark_local_transform_dirty(gun_root);
                game_world.resources.gun_hand = hand;
            }
        }
    }

    update_hand_color(game_world, world, true);
    update_hand_color(game_world, world, false);
}

fn update_hand_color(game_world: &GameWorld, world: &mut World, is_left: bool) {
    let hand_entity = if is_left {
        game_world.resources.left_hand_cube
    } else {
        game_world.resources.right_hand_cube
    };

    let Some(entity) = hand_entity else {
        return;
    };

    let has_gun = if is_left {
        game_world.resources.gun_hand == crate::ecs::GunHand::Left
    } else {
        game_world.resources.gun_hand == crate::ecs::GunHand::Right
    };

    let base_color = if is_left {
        [0.2, 0.6, 0.9, 1.0]
    } else {
        [0.9, 0.6, 0.2, 1.0]
    };

    let hand_color = if has_gun {
        [base_color[0] * 0.3, base_color[1] * 0.3, base_color[2] * 0.3, 1.0]
    } else {
        base_color
    };

    let mat_name = world
        .core
        .get_material_ref(entity)
        .map(|r| r.name.clone());
    if let Some(name) = mat_name
        && let Some(mat) = registry_entry_by_name_mut(
            &mut world.resources.material_registry.registry,
            &name,
        )
    {
        mat.base_color = hand_color;
    }
}
