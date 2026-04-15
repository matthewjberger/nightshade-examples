use crate::ecs::{DRAWER, Drawer, GameWorld, INTERACTABLE, Interactable, InteractableKind};
use nightshade::ecs::physics::*;
use nightshade::ecs::transform::components::Parent;
use nightshade::prelude::*;

pub(crate) fn spawn_drawer_exhibit(game_world: &mut GameWorld, world: &mut World, center: Vec3) {
    let cabinet_width = 1.0;
    let cabinet_height = 1.2;
    let cabinet_depth = 0.6;
    let cabinet_x = center.x;
    let cabinet_z = center.z;
    let cabinet_bottom_y = 0.0;

    let cabinet_material = create_textured_material(nalgebra_glm::vec3(0.4, 0.3, 0.2), 0.85, 0.0);

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
                | RIGID_BODY
                | COLLIDER,
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
            *bv = BoundingVolume::from_mesh_type("Cube");
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
                rigid_body_handle: drawer_rb_handle,
                closed_position,
                current_offset: 0.0,
                velocity: 0.0,
                max_offset: max_slide,
            },
        );

        let interactable_entity = game_world.spawn_entities(INTERACTABLE, 1)[0];
        game_world.set_interactable(
            interactable_entity,
            Interactable {
                engine_entity: front_entity,
                game_entity,
                kind: InteractableKind::Drawer,
            },
        );
    }
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
        *bv = BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(p) = world.core.get_parent_mut(entity) {
        *p = Parent(Some(parent));
    }
}
