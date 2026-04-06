use crate::ecs::{GameWorld, SpringJointVisual, SPRING_JOINT_VISUAL};
use crate::systems::ui::spawn_label;
use nightshade::ecs::physics::joints::{SpringJoint, create_spring_joint};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(crate) fn spawn_spring_joint_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    spawn_label(
        world,
        "Spring Joint",
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
    let object_position = nalgebra_glm::vec3(center.x, anchor_height - 1.5, center.z);

    let beam_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
    let anchor_entity = spawn_static_physics_cube_with_material(
        world,
        anchor_position,
        nalgebra_glm::vec3(0.3, 0.3, 0.3),
        beam_material,
    );

    let spring_cube_material =
        create_textured_material(nalgebra_glm::vec3(0.3, 0.8, 0.8), 0.4, 0.5);
    let spring_entity = spawn_dynamic_physics_cube_with_material(
        world,
        object_position,
        nalgebra_glm::vec3(0.4, 0.4, 0.4),
        3.0,
        spring_cube_material,
    );
    game_world.add_grabbable(spring_entity);

    create_spring_joint(
        world,
        anchor_entity,
        spring_entity,
        SpringJoint::new(1.0, 50.0, 2.0)
            .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.15, 0.0))
            .with_local_anchor2(nalgebra_glm::vec3(0.0, 0.2, 0.0)),
    );

    let coil_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.7, 0.75), 0.3, 0.8);
    let num_coils = 8;
    let mut spring_visual_entities = Vec::new();

    for coil_index in 0..num_coils {
        let coil_entity = world.spawn_entities(
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

        if let Some(name) = world.core.get_name_mut(coil_entity) {
            name.0 = format!("Spring Coil {}", coil_index);
        }

        if let Some(transform) = world.core.get_local_transform_mut(coil_entity) {
            transform.translation = anchor_position;
            transform.scale = nalgebra_glm::vec3(0.015, 0.1, 0.015);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(coil_entity) {
            mesh.name = "Cylinder".to_string();
        }

        let material_name = format!("SpringCoil_{}_{}", spring_entity.id, coil_index);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            coil_material.clone(),
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
            .set_material_ref(coil_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(coil_entity) {
            *bv = BoundingVolume::from_mesh_type("Cylinder");
        }

        spring_visual_entities.push(coil_entity);
    }

    let game_entity = game_world.spawn_entities(SPRING_JOINT_VISUAL, 1)[0];
    game_world.set_spring_joint_visual(
        game_entity,
        SpringJointVisual {
            anchor_entity,
            object_entity: spring_entity,
            spring_entities: spring_visual_entities,
        },
    );
}
