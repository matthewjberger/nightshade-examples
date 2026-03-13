use crate::state::PropState;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::physics::{ColliderComponent, RigidBodyComponent};
use nightshade::prelude::*;

pub fn spawn_grabbable_prop(
    world: &mut World,
    physics_objects: &mut Vec<Entity>,
    props: &mut Vec<PropState>,
    position: Vec3,
    shape: PropShape,
    material: Material,
    mass: f32,
) -> Entity {
    let (mesh_name, collider, scale) = match shape {
        PropShape::Cube(size) => (
            "Cube",
            ColliderComponent::new_cuboid(size / 2.0, size / 2.0, size / 2.0),
            Vec3::new(size, size, size),
        ),
        PropShape::Sphere(radius) => (
            "Sphere",
            ColliderComponent::new_ball(radius),
            Vec3::new(radius * 2.0, radius * 2.0, radius * 2.0),
        ),
        PropShape::Cylinder { radius, height } => (
            "Cylinder",
            ColliderComponent::new_cylinder(height / 2.0, radius),
            Vec3::new(radius * 2.0, height, radius * 2.0),
        ),
    };

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY
            | nightshade::ecs::world::RIGID_BODY
            | nightshade::ecs::world::COLLIDER
            | nightshade::ecs::world::PHYSICS_INTERPOLATION,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(entity) {
        name.0 = format!("Prop_{}", entity.id);
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = scale;
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = mesh_name.to_string();
    }

    let material_name = format!("Prop_{}", entity.id);
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
    world.core.set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type(mesh_name);
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(entity) {
        *rigid_body = RigidBodyComponent::new_dynamic()
            .with_translation(position.x, position.y, position.z)
            .with_mass(mass);
    }

    if let Some(coll) = world.core.get_collider_mut(entity) {
        *coll = collider.with_friction(0.5);
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
        handle
    };

    physics_objects.push(entity);
    props.push(PropState {
        _rigid_body_handle: handle,
    });

    entity
}

pub enum PropShape {
    Cube(f32),
    Sphere(f32),
    Cylinder { radius: f32, height: f32 },
}
