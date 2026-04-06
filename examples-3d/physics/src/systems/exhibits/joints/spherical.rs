use crate::ecs::{GameWorld, SphericalJointVisual, SPHERICAL_JOINT_VISUAL};
use crate::systems::ui::spawn_label;
use nightshade::ecs::physics::joints::{SphericalJoint, create_spherical_joint};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(crate) fn spawn_spherical_joint_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    spawn_label(
        world,
        "Spherical Joint",
        nalgebra_glm::vec3(center.x, 4.0, center.z),
        TextProperties {
            font_size: 24.0,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 0.03,
            outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        },
    );

    let anchor_position = nalgebra_glm::vec3(center.x, 3.0, center.z);
    let ball_position = nalgebra_glm::vec3(center.x, 1.8, center.z);
    let rod_length = 1.0;

    let beam_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
    let anchor_entity = spawn_static_physics_cube_with_material(
        world,
        anchor_position,
        nalgebra_glm::vec3(0.3, 0.3, 0.3),
        beam_material,
    );

    let pendulum_material =
        create_textured_material(nalgebra_glm::vec3(0.3, 0.8, 0.3), 0.5, 0.3);
    let pendulum_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        ball_position,
        0.2,
        3.0,
        pendulum_material,
    );
    game_world.add_grabbable(pendulum_entity);

    create_spherical_joint(
        world,
        anchor_entity,
        pendulum_entity,
        SphericalJoint::new()
            .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.15, 0.0))
            .with_local_anchor2(nalgebra_glm::vec3(0.0, rod_length, 0.0)),
    );

    let rod_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.55, 0.5), 0.7, 0.2);
    let rod_entity = world.spawn_entities(
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

    if let Some(name) = world.core.get_name_mut(rod_entity) {
        name.0 = "Spherical Joint Rod".to_string();
    }

    let midpoint = (anchor_position + ball_position) * 0.5;
    let distance = nalgebra_glm::distance(&anchor_position, &ball_position);

    if let Some(transform) = world.core.get_local_transform_mut(rod_entity) {
        transform.translation = midpoint;
        transform.scale = nalgebra_glm::vec3(0.03, distance / 2.0, 0.03);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(rod_entity) {
        mesh.name = "Cylinder".to_string();
    }

    let material_name = format!("SphericalRod_{}", rod_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        rod_material,
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
        .set_material_ref(rod_entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(rod_entity) {
        *bv = BoundingVolume::from_mesh_type("Cylinder");
    }

    let game_entity = game_world.spawn_entities(SPHERICAL_JOINT_VISUAL, 1)[0];
    game_world.set_spherical_joint_visual(
        game_entity,
        SphericalJointVisual {
            anchor_entity,
            ball_entity: pendulum_entity,
            rod_entity,
        },
    );
}
