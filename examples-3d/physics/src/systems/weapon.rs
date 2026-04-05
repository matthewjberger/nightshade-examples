use crate::ecs::GameWorld;
use nightshade::ecs::transform::components::Parent;
use nightshade::prelude::*;

pub fn spawn_weapon_part(
    world: &mut World,
    parent: Entity,
    position: Vec3,
    scale: Vec3,
    mesh_name: &str,
    material: nightshade::ecs::material::components::Material,
) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT
            | VISIBILITY
            | RENDER_LAYER,
        1,
    )[0];

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = scale;
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = mesh_name.to_string();
    }

    let material_name = format!("WeaponPart_{}", entity.id);
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

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
        *bounding_volume =
            BoundingVolume::from_mesh_type(mesh_name);
    }

    if let Some(p) = world.core.get_parent_mut(entity) {
        *p = Parent(Some(parent));
    }

    if let Some(render_layer) = world.core.get_render_layer_mut(entity) {
        render_layer.0 = nightshade::ecs::render_layer::components::RenderLayer::OVERLAY;
    }

    world.resources.mesh_render_state.mark_entity_added(entity);

    entity
}

pub fn spawn_weapon(world: &mut World, camera_entity: Entity) -> Entity {
    let root = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | PARENT,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(root) {
        name.0 = "Weapon".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(root) {
        transform.translation = nalgebra_glm::vec3(0.15, -0.10, -0.25);
    }

    if let Some(parent) = world.core.get_parent_mut(root) {
        *parent = Parent(Some(camera_entity));
    }

    let body_color = create_textured_material(nalgebra_glm::vec3(0.18, 0.18, 0.20), 0.45, 0.85);
    let barrel_color = create_textured_material(nalgebra_glm::vec3(0.12, 0.12, 0.14), 0.3, 0.9);
    let grip_color = create_textured_material(nalgebra_glm::vec3(0.08, 0.08, 0.06), 0.8, 0.1);
    let accent_color = create_textured_material(nalgebra_glm::vec3(0.25, 0.25, 0.28), 0.35, 0.8);

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, 0.0, 0.0),
        nalgebra_glm::vec3(0.025, 0.035, 0.10),
        "Cube",
        body_color.clone(),
    );

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, 0.005, -0.09),
        nalgebra_glm::vec3(0.012, 0.012, 0.08),
        "Cube",
        barrel_color.clone(),
    );

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, 0.005, -0.135),
        nalgebra_glm::vec3(0.016, 0.016, 0.01),
        "Cube",
        accent_color.clone(),
    );

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, -0.045, 0.02),
        nalgebra_glm::vec3(0.02, 0.055, 0.025),
        "Cube",
        grip_color,
    );

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, 0.023, -0.01),
        nalgebra_glm::vec3(0.012, 0.006, 0.06),
        "Cube",
        accent_color,
    );

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, -0.018, 0.0),
        nalgebra_glm::vec3(0.018, 0.008, 0.025),
        "Cube",
        body_color,
    );

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, -0.006, -0.055),
        nalgebra_glm::vec3(0.004, 0.004, 0.015),
        "Cube",
        barrel_color,
    );

    root
}

pub fn update_weapon_sway(game_world: &mut GameWorld, world: &mut World) {
    let Some(weapon_entity) = game_world.resources.weapon_entity else {
        return;
    };
    let Some(camera_entity) = game_world.resources.camera_entity else {
        return;
    };

    let forward = world
        .core
        .get_local_transform(camera_entity)
        .map(|transform| transform.forward_vector())
        .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, -1.0));

    let current_yaw = forward.x.atan2(-forward.z);
    let current_pitch = forward.y.asin();

    let yaw_delta = current_yaw - game_world.resources.weapon_previous_yaw;
    let pitch_delta = current_pitch - game_world.resources.weapon_previous_pitch;
    game_world.resources.weapon_previous_yaw = current_yaw;
    game_world.resources.weapon_previous_pitch = current_pitch;

    let sway_strength = 0.6;
    game_world.resources.weapon_sway.x -= yaw_delta * sway_strength;
    game_world.resources.weapon_sway.y -= pitch_delta * sway_strength;

    let max_sway = 0.08;
    game_world.resources.weapon_sway.x = game_world.resources.weapon_sway.x.clamp(-max_sway, max_sway);
    game_world.resources.weapon_sway.y = game_world.resources.weapon_sway.y.clamp(-max_sway, max_sway);

    let delta_time = world.resources.window.timing.delta_time;
    let recovery_speed = 8.0;
    let decay = (-recovery_speed * delta_time).exp();
    game_world.resources.weapon_sway.x *= decay;
    game_world.resources.weapon_sway.y *= decay;

    if let Some(transform) = world.core.get_local_transform_mut(weapon_entity) {
        transform.translation = nalgebra_glm::vec3(
            0.15 + game_world.resources.weapon_sway.x,
            -0.10 + game_world.resources.weapon_sway.y,
            -0.25,
        );
    }
    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, weapon_entity);
}
