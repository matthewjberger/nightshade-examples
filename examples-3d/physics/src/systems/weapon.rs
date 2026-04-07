use nightshade::ecs::transform::components::Parent;
use nightshade::prelude::*;

fn spawn_weapon_part(
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
    crate::systems::spawn::assign_material(world, entity, material_name, material);

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
        *bounding_volume = BoundingVolume::from_mesh_type(mesh_name);
    }

    if let Some(parent_comp) = world.core.get_parent_mut(entity) {
        *parent_comp = Parent(Some(parent));
    }

    if let Some(render_layer) = world.core.get_render_layer_mut(entity) {
        render_layer.0 = RenderLayer::OVERLAY;
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

    let body_color = nightshade::ecs::material::components::Material {
        base_color: [0.15, 0.15, 0.17, 1.0],
        roughness: 0.4,
        metallic: 0.8,
        ..Default::default()
    };
    let barrel_color = nightshade::ecs::material::components::Material {
        base_color: [0.1, 0.1, 0.12, 1.0],
        roughness: 0.3,
        metallic: 0.9,
        ..Default::default()
    };
    let accent_color = nightshade::ecs::material::components::Material {
        base_color: [0.2, 0.18, 0.15, 1.0],
        roughness: 0.6,
        metallic: 0.3,
        ..Default::default()
    };

    spawn_weapon_part(world, root, nalgebra_glm::vec3(0.0, 0.0, 0.0), nalgebra_glm::vec3(0.025, 0.03, 0.08), "Cube", body_color.clone());
    spawn_weapon_part(world, root, nalgebra_glm::vec3(0.0, 0.005, -0.10), nalgebra_glm::vec3(0.012, 0.015, 0.12), "Cube", barrel_color.clone());
    spawn_weapon_part(world, root, nalgebra_glm::vec3(0.0, -0.02, 0.02), nalgebra_glm::vec3(0.015, 0.04, 0.025), "Cube", accent_color);
    spawn_weapon_part(world, root, nalgebra_glm::vec3(0.0, 0.018, -0.03), nalgebra_glm::vec3(0.018, 0.005, 0.04), "Cube", body_color);
    spawn_weapon_part(world, root, nalgebra_glm::vec3(0.0, 0.015, -0.06), nalgebra_glm::vec3(0.003, 0.01, 0.003), "Cube", barrel_color.clone());
    spawn_weapon_part(world, root, nalgebra_glm::vec3(0.0, 0.015, -0.08), nalgebra_glm::vec3(0.003, 0.008, 0.003), "Cube", barrel_color.clone());
    spawn_weapon_part(world, root, nalgebra_glm::vec3(0.0, 0.005, -0.20), nalgebra_glm::vec3(0.004, 0.004, 0.015), "Cube", barrel_color);

    root
}
