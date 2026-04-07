use crate::ecs::{GameWorld, RopeJointVisual, ROPE_JOINT_VISUAL};
use crate::systems::ui::spawn_label;
use nightshade::ecs::physics::joints::{RopeJoint, create_rope_joint};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(crate) fn spawn_rope_joint_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    spawn_label(
        world,
        "Rope Joint",
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

    let anchor_height = 3.0;
    let anchor_position = nalgebra_glm::vec3(center.x, anchor_height, center.z);
    let ball_start_position = nalgebra_glm::vec3(center.x, anchor_height - 0.3, center.z);

    let beam_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
    let anchor_entity = spawn_static_physics_cube_with_material(
        world,
        anchor_position,
        nalgebra_glm::vec3(0.3, 0.3, 0.3),
        beam_material,
    );

    let ball_material = create_textured_material(nalgebra_glm::vec3(0.8, 0.4, 0.8), 0.4, 0.5);
    let ball_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        ball_start_position,
        0.25,
        2.0,
        ball_material,
    );
    crate::systems::spawn::register_grabbable(game_world, ball_entity);

    create_rope_joint(
        world,
        anchor_entity,
        ball_entity,
        RopeJoint::new(1.8)
            .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.15, 0.0))
            .with_local_anchor2(nalgebra_glm::vec3(0.0, 0.0, 0.0)),
    );

    let rope_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.5, 0.35), 0.9, 0.0);
    let rope_entity = world.spawn_entities(
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

    if let Some(name) = world.core.get_name_mut(rope_entity) {
        name.0 = "Rope Joint Visual".to_string();
    }

    let anchor_attach = anchor_position - nalgebra_glm::vec3(0.0, 0.15, 0.0);
    let midpoint = (anchor_attach + ball_start_position) * 0.5;
    let distance = nalgebra_glm::distance(&anchor_attach, &ball_start_position);

    if let Some(transform) = world.core.get_local_transform_mut(rope_entity) {
        transform.translation = midpoint;
        transform.scale = nalgebra_glm::vec3(0.02, distance / 2.0, 0.02);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(rope_entity) {
        mesh.name = "Cylinder".to_string();
    }

    let material_name = format!("RopeVisual_{}", rope_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        rope_material,
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
        .set_material_ref(rope_entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(rope_entity) {
        *bv = BoundingVolume::from_mesh_type("Cylinder");
    }

    let game_entity = game_world.spawn_entities(ROPE_JOINT_VISUAL, 1)[0];
    game_world.set_rope_joint_visual(
        game_entity,
        RopeJointVisual {
            anchor_entity,
            ball_entity,
            rope_entity,
        },
    );
}
