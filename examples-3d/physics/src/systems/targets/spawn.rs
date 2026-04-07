use crate::ecs::{GameWorld, Health, Target, TARGET};
use nightshade::ecs::transform::components::Parent;
use nightshade::prelude::*;

pub(crate) fn spawn_targets(game_world: &mut GameWorld, world: &mut World) {
    let targets: &[(Vec3, Vec3, f32, f32, &str)] = &[
        (nalgebra_glm::vec3(-6.0, 3.0, -6.0), nalgebra_glm::vec3(0.9, 0.3, 0.3), 0.25, 3.0, "Sphere"),
        (nalgebra_glm::vec3(6.0, 4.5, -6.0), nalgebra_glm::vec3(0.3, 0.9, 0.3), 0.3, 5.0, "Sphere"),
        (nalgebra_glm::vec3(0.0, 5.0, -10.0), nalgebra_glm::vec3(0.3, 0.5, 0.9), 0.4, 8.0, "Cube"),
        (nalgebra_glm::vec3(-8.0, 2.5, 0.0), nalgebra_glm::vec3(0.9, 0.7, 0.2), 0.2, 2.0, "Sphere"),
        (nalgebra_glm::vec3(8.0, 3.5, 0.0), nalgebra_glm::vec3(0.7, 0.3, 0.9), 0.35, 6.0, "Cube"),
        (nalgebra_glm::vec3(-3.0, 6.0, -3.0), nalgebra_glm::vec3(0.2, 0.8, 0.8), 0.2, 2.0, "Sphere"),
        (nalgebra_glm::vec3(3.0, 4.0, 3.0), nalgebra_glm::vec3(0.9, 0.4, 0.6), 0.45, 10.0, "Cube"),
        (nalgebra_glm::vec3(0.0, 3.0, 6.0), nalgebra_glm::vec3(0.8, 0.8, 0.3), 0.3, 4.0, "Sphere"),
        (nalgebra_glm::vec3(-5.0, 5.5, 5.0), nalgebra_glm::vec3(0.4, 0.6, 0.9), 0.25, 3.0, "Sphere"),
        (nalgebra_glm::vec3(5.0, 2.0, -3.0), nalgebra_glm::vec3(0.9, 0.5, 0.3), 0.5, 12.0, "Cylinder"),
    ];

    for &(position, color, scale, max_health, mesh_name) in targets {
        let entity = spawn_target_mesh(world, position, scale, color, mesh_name);
        let (bar_entity, fill_entity) = spawn_healthbar(world, position);

        let game_entity = game_world.spawn_entities(TARGET, 1)[0];
        game_world.set_target(
            game_entity,
            Target {
                entity,
                position,
                base_scale: scale,
                color,
                health: Health::new(max_health, bar_entity, fill_entity),
                popped: false,
                pop_time_ms: 0,
                respawn_delay_ms: 3000,
                pulse_phase: position.x * 2.0 + position.z,
                pop_emitter_entity: None,
            },
        );
    }
}

fn spawn_target_mesh(
    world: &mut World,
    position: Vec3,
    scale: f32,
    color: Vec3,
    mesh_name: &str,
) -> Entity {
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
        name.0 = "Target".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = nalgebra_glm::vec3(scale, scale, scale);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = mesh_name.to_string();
    }

    let material_name = format!("Target_{}", entity.id);
    crate::systems::spawn::assign_material(
        world,
        entity,
        material_name,
        nightshade::ecs::material::components::Material {
            base_color: [color.x, color.y, color.z, 1.0],
            emissive_factor: [color.x * 2.0, color.y * 2.0, color.z * 2.0],
            roughness: 0.4,
            metallic: 0.6,
            ..Default::default()
        },
    );

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
        *bounding_volume = BoundingVolume::from_mesh_type(mesh_name);
    }

    world.resources.mesh_render_state.mark_entity_added(entity);

    entity
}

pub(super) fn spawn_healthbar(world: &mut World, position: Vec3) -> (Entity, Entity) {
    let bar_width = 0.8;
    let bar_height = 0.08;
    let bar_y_offset = 0.6;

    let background = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | RENDER_MESH
            | MATERIAL_REF | BOUNDING_VOLUME | VISIBILITY,
        1,
    )[0];

    if let Some(transform) = world.core.get_local_transform_mut(background) {
        transform.translation = nalgebra_glm::vec3(position.x, position.y + bar_y_offset, position.z);
        transform.scale = nalgebra_glm::vec3(bar_width, bar_height, 0.02);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(background) {
        mesh.name = "Cube".to_string();
    }

    let bg_material_name = format!("HealthBarBg_{}", background.id);
    crate::systems::spawn::assign_material(
        world,
        background,
        bg_material_name,
        create_textured_material(nalgebra_glm::vec3(0.1, 0.1, 0.1), 0.9, 0.0),
    );

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(background) {
        *bounding_volume = BoundingVolume::from_mesh_type("Cube");
    }

    world.resources.mesh_render_state.mark_entity_added(background);

    let fill = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | RENDER_MESH
            | MATERIAL_REF | BOUNDING_VOLUME | VISIBILITY | PARENT,
        1,
    )[0];

    if let Some(transform) = world.core.get_local_transform_mut(fill) {
        transform.translation = nalgebra_glm::vec3(0.0, 0.0, 0.01);
        transform.scale = nalgebra_glm::vec3(1.0, 1.0, 1.1);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(fill) {
        mesh.name = "Cube".to_string();
    }

    let fill_material_name = format!("HealthBarFill_{}", fill.id);
    crate::systems::spawn::assign_material(
        world,
        fill,
        fill_material_name,
        create_textured_material(nalgebra_glm::vec3(0.2, 0.9, 0.2), 0.8, 0.0),
    );

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(fill) {
        *bounding_volume = BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(parent) = world.core.get_parent_mut(fill) {
        *parent = Parent(Some(background));
    }

    world.resources.mesh_render_state.mark_entity_added(fill);

    (background, fill)
}
