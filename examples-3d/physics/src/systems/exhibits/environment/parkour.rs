use nightshade::ecs::physics::*;
use nightshade::ecs::text::components::{TextAlignment, TextProperties, VerticalAlignment};
use nightshade::prelude::*;

pub(super) fn spawn_parkour_course(
    world: &mut World,
    platform_material: &nightshade::ecs::material::components::Material,
    accent_material: &nightshade::ecs::material::components::Material,
) {
    use crate::systems::ui::spawn_label;

    let label_properties = TextProperties {
        font_size: 16.0,
        color: nalgebra_glm::Vec4::new(0.9, 0.9, 0.9, 1.0),
        alignment: TextAlignment::Center,
        vertical_alignment: VerticalAlignment::Middle,
        outline_width: 0.03,
        outline_color: nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 1.0),
        ..Default::default()
    };

    let stair_start_z = 16.0;

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-8.0, 0.25, stair_start_z),
        nalgebra_glm::vec3(3.0, 0.5, 2.0),
        platform_material.clone(),
    );
    spawn_label(
        world,
        "0.5m",
        nalgebra_glm::vec3(-8.0, 1.0, stair_start_z - 1.0),
        label_properties.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-8.0, 0.75, stair_start_z + 2.5),
        nalgebra_glm::vec3(3.0, 0.5, 2.0),
        platform_material.clone(),
    );
    spawn_label(
        world,
        "1m",
        nalgebra_glm::vec3(-8.0, 1.5, stair_start_z + 1.5),
        label_properties.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-8.0, 1.5, stair_start_z + 5.0),
        nalgebra_glm::vec3(3.0, 0.5, 2.0),
        platform_material.clone(),
    );
    spawn_label(
        world,
        "2m",
        nalgebra_glm::vec3(-8.0, 2.3, stair_start_z + 4.0),
        label_properties.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-8.0, 2.5, stair_start_z + 7.5),
        nalgebra_glm::vec3(3.0, 0.5, 2.0),
        platform_material.clone(),
    );
    spawn_label(
        world,
        "3m",
        nalgebra_glm::vec3(-8.0, 3.3, stair_start_z + 6.5),
        label_properties.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-8.0, 4.0, stair_start_z + 10.0),
        nalgebra_glm::vec3(4.0, 0.5, 3.0),
        accent_material.clone(),
    );
    spawn_label(
        world,
        "4m ledge",
        nalgebra_glm::vec3(-8.0, 4.8, stair_start_z + 8.5),
        label_properties.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-8.0, 6.0, stair_start_z + 14.0),
        nalgebra_glm::vec3(3.0, 0.5, 2.0),
        platform_material.clone(),
    );
    spawn_label(
        world,
        "6m",
        nalgebra_glm::vec3(-8.0, 6.8, stair_start_z + 13.0),
        label_properties.clone(),
    );

    let gap_z = stair_start_z + 2.0;
    for gap_index in 0..5 {
        let gap_distance = 2.0 + gap_index as f32 * 1.5;
        let platform_x = 5.0 + gap_index as f32 * gap_distance;
        let platform_z = gap_z + gap_index as f32 * 3.0;

        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(platform_x.min(25.0), 0.25, platform_z),
            nalgebra_glm::vec3(2.0, 0.5, 2.0),
            platform_material.clone(),
        );

        let label_text = format!("{}m gap", gap_distance as i32);
        spawn_label(
            world,
            &label_text,
            nalgebra_glm::vec3(platform_x.min(25.0), 1.2, platform_z),
            label_properties.clone(),
        );
    }

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(0.0, 1.5, stair_start_z + 18.0),
        nalgebra_glm::vec3(8.0, 3.0, 0.5),
        accent_material.clone(),
    );
    spawn_label(
        world,
        "3m wall",
        nalgebra_glm::vec3(0.0, 3.5, stair_start_z + 17.5),
        label_properties.clone(),
    );

    let ramp_base_z = stair_start_z + 6.0;
    for ramp_index in 0..4 {
        let ramp_x = 20.0;
        let ramp_z = ramp_base_z + ramp_index as f32 * 2.0;
        let ramp_height = 0.15 + ramp_index as f32 * 0.15;

        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(ramp_x, ramp_height, ramp_z),
            nalgebra_glm::vec3(4.0, ramp_height * 2.0, 1.5),
            platform_material.clone(),
        );
    }
    spawn_label(
        world,
        "ramp",
        nalgebra_glm::vec3(20.0, 1.5, ramp_base_z - 0.5),
        label_properties.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-20.0, 3.0, stair_start_z + 10.0),
        nalgebra_glm::vec3(6.0, 0.3, 6.0),
        accent_material.clone(),
    );
    spawn_label(
        world,
        "elevated arena 3m",
        nalgebra_glm::vec3(-20.0, 3.8, stair_start_z + 7.0),
        label_properties.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-20.0, 1.5, stair_start_z + 6.5),
        nalgebra_glm::vec3(2.0, 0.3, 1.0),
        platform_material.clone(),
    );
    spawn_label(
        world,
        "step up",
        nalgebra_glm::vec3(-20.0, 2.2, stair_start_z + 5.5),
        label_properties.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-25.0, 5.0, stair_start_z + 10.0),
        nalgebra_glm::vec3(3.0, 0.3, 3.0),
        platform_material.clone(),
    );
    spawn_label(
        world,
        "5m platform",
        nalgebra_glm::vec3(-25.0, 5.8, stair_start_z + 8.5),
        label_properties.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-20.0, 7.0, stair_start_z + 14.0),
        nalgebra_glm::vec3(3.0, 0.3, 3.0),
        platform_material.clone(),
    );
    spawn_label(
        world,
        "7m platform",
        nalgebra_glm::vec3(-20.0, 7.8, stair_start_z + 12.5),
        label_properties.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-15.0, 9.0, stair_start_z + 10.0),
        nalgebra_glm::vec3(3.0, 0.3, 3.0),
        accent_material.clone(),
    );
    spawn_label(
        world,
        "9m peak",
        nalgebra_glm::vec3(-15.0, 9.8, stair_start_z + 8.5),
        label_properties,
    );
}
