use nightshade::prelude::*;

pub fn assign_material(
    world: &mut World,
    entity: Entity,
    name: String,
    material: nightshade::ecs::material::components::Material,
) {
    material_registry_insert(
        &mut world.resources.material_registry,
        name.clone(),
        material,
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.core.set_material_ref(entity, MaterialRef::new(name));
}

pub fn spawn_visual_entity_with_shadow(
    world: &mut World,
    position: Vec3,
    scale: Vec3,
    mesh_name: &str,
    material: nightshade::ecs::material::components::Material,
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
            | VISIBILITY,
        1,
    )[0];

    if let Some(n) = world.core.get_name_mut(entity) {
        n.0 = name;
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = scale;
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = mesh_name.to_string();
    }

    let material_name = format!("VisualShadow_{}", entity.id);
    assign_material(world, entity, material_name, material);

    if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
        *bv = BoundingVolume::from_mesh_type(mesh_name);
    }

    entity
}
