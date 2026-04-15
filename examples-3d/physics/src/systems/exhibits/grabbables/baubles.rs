use crate::ecs::{
    BAUBLE_SPAWN, BUTTON, BaubleSpawn, Button, ButtonAction, GameWorld, INTERACTABLE, Interactable,
    InteractableKind,
};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(crate) fn spawn_bauble_table(game_world: &mut GameWorld, world: &mut World, center: Vec3) {
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

    spawn_recall_pedestal(
        game_world,
        world,
        nalgebra_glm::vec3(center.x + 2.0, 0.0, center.z),
    );
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
        let entity = spawn_bauble(
            game_world,
            world,
            pos,
            *radius,
            color,
            format!("Bauble {}", index + 1),
        );
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
            | RIGID_BODY
            | COLLIDER,
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
        *bv = BoundingVolume::from_mesh_type("Sphere");
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

    crate::systems::spawn::register_grabbable(game_world, entity);
    entity
}

fn spawn_recall_pedestal(game_world: &mut GameWorld, world: &mut World, center: Vec3) {
    let pedestal_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.3, 0.35), 0.85, 0.0);

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
            | RIGID_BODY
            | COLLIDER,
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
        *bv = BoundingVolume::from_mesh_type("Cylinder");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(button_entity) {
        *rigid_body =
            RigidBodyComponent::new_kinematic().with_translation(center.x, button_base_y, center.z);
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

    let interactable_entity = game_world.spawn_entities(INTERACTABLE, 1)[0];
    game_world.set_interactable(
        interactable_entity,
        Interactable {
            engine_entity: button_entity,
            game_entity,
            kind: InteractableKind::Button,
        },
    );
}
