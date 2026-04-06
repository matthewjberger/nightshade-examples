use crate::ecs::GameWorld;
use crate::systems::ui::spawn_label;
use nightshade::ecs::physics::joints::{FixedJoint, create_fixed_joint};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(crate) fn spawn_fixed_joint_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    spawn_label(
        world,
        "Fixed Joint",
        nalgebra_glm::vec3(center.x + 1.0, 3.5, center.z),
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

    let num_vertebrae = 6;
    let block_size = 0.3;
    let block_spacing = 0.35;

    let beam_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
    let anchor_entity = spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, 2.5, center.z),
        nalgebra_glm::vec3(0.3, 0.3, 0.3),
        beam_material,
    );

    let colors = [
        nalgebra_glm::vec3(0.8, 0.3, 0.3),
        nalgebra_glm::vec3(0.8, 0.5, 0.3),
        nalgebra_glm::vec3(0.8, 0.8, 0.3),
        nalgebra_glm::vec3(0.3, 0.8, 0.3),
        nalgebra_glm::vec3(0.3, 0.5, 0.8),
        nalgebra_glm::vec3(0.6, 0.3, 0.8),
    ];

    let mut previous_entity = anchor_entity;
    for vertebra_index in 0..num_vertebrae {
        let color = colors[vertebra_index % colors.len()];
        let block_material = create_textured_material(color, 0.6, 0.2);
        let block_x = center.x + (vertebra_index as f32 + 1.0) * block_spacing;

        let block_entity = spawn_dynamic_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(block_x, 2.5, center.z),
            nalgebra_glm::vec3(block_size, block_size, block_size),
            1.5,
            block_material,
        );
        game_world.add_grabbable(block_entity);

        create_fixed_joint(
            world,
            previous_entity,
            block_entity,
            FixedJoint::new()
                .with_local_anchor1(nalgebra_glm::vec3(block_size / 2.0 + 0.025, 0.0, 0.0))
                .with_local_anchor2(nalgebra_glm::vec3(-block_size / 2.0 - 0.025, 0.0, 0.0)),
        );

        previous_entity = block_entity;
    }
}
