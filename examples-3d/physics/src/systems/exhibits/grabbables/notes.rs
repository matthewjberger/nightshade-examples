use crate::ecs::{GameWorld, INTERACTABLE, Interactable, InteractableKind, NOTE, Note};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(crate) fn spawn_note_table(game_world: &mut GameWorld, world: &mut World, center: Vec3) {
    let table_top_material =
        create_textured_material(nalgebra_glm::vec3(0.35, 0.25, 0.15), 0.8, 0.0);
    let table_leg_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.2, 0.1), 0.85, 0.0);

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
            | RIGID_BODY
            | COLLIDER,
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
        *bounding_volume = BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(note_entity) {
        *rigid_body = RigidBodyComponent::new_static().with_translation(center.x, note_y, center.z);
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

    let interactable_entity = game_world.spawn_entities(INTERACTABLE, 1)[0];
    game_world.set_interactable(
        interactable_entity,
        Interactable {
            engine_entity: note_entity,
            game_entity,
            kind: InteractableKind::Note,
        },
    );
}
