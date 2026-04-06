use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(super) struct RoomConfig {
    pub center: Vec3,
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub wall_thickness: f32,
    pub doorway_width: f32,
    pub doorway_height: f32,
    pub wall_material: nightshade::ecs::material::components::Material,
    pub ceiling_material: nightshade::ecs::material::components::Material,
}

pub fn spawn_sun_overhead(world: &mut World) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | LIGHT,
        1,
    )[0];

    world
        .core
        .set_name(entity, Name("Sun".to_string()));
    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: nalgebra_glm::Vec3::new(5.0, 10.0, 5.0),
            rotation: nalgebra_glm::quat_angle_axis(
                std::f32::consts::FRAC_PI_4,
                &nalgebra_glm::Vec3::new(0.0, 1.0, 0.0),
            ) * nalgebra_glm::quat_angle_axis(
                -std::f32::consts::FRAC_PI_4,
                &nalgebra_glm::Vec3::new(1.0, 0.0, 0.0),
            ),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world.core.set_light(
        entity,
        Light {
            light_type: LightType::Directional,
            color: nalgebra_glm::vec3(1.0, 0.95, 0.8),
            intensity: 5.0,
            range: 100.0,
            inner_cone_angle: std::f32::consts::PI / 6.0,
            outer_cone_angle: std::f32::consts::PI / 4.0,
            cast_shadows: true,
            shadow_bias: 0.0005,
        },
    );

    entity
}

pub fn spawn_environment(world: &mut World) {
    let floor_material =
        create_textured_material(nalgebra_glm::vec3(0.15, 0.15, 0.18), 0.9, 0.0);
    let wall_material =
        create_textured_material(nalgebra_glm::vec3(0.2, 0.18, 0.16), 0.95, 0.0);
    let platform_material =
        create_textured_material(nalgebra_glm::vec3(0.22, 0.22, 0.25), 0.85, 0.1);
    let accent_material =
        create_textured_material(nalgebra_glm::vec3(0.3, 0.25, 0.2), 0.8, 0.0);

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(0.0, -0.25, 11.0),
        nalgebra_glm::vec3(60.0, 0.5, 52.0),
        floor_material,
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(0.0, 5.0, -15.0),
        nalgebra_glm::vec3(60.0, 10.0, 0.5),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(0.0, 5.0, 37.0),
        nalgebra_glm::vec3(60.0, 10.0, 0.5),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-30.0, 5.0, 11.0),
        nalgebra_glm::vec3(0.5, 10.0, 52.0),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(30.0, 5.0, 11.0),
        nalgebra_glm::vec3(0.5, 10.0, 52.0),
        wall_material,
    );

    spawn_parkour_course(world, &platform_material, &accent_material);
}

fn spawn_parkour_course(
    world: &mut World,
    platform_material: &nightshade::ecs::material::components::Material,
    accent_material: &nightshade::ecs::material::components::Material,
) {
    use crate::systems::ui::spawn_label;
    use nightshade::ecs::text::components::{TextAlignment, TextProperties, VerticalAlignment};

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
    spawn_label(world, "0.5m", nalgebra_glm::vec3(-8.0, 1.0, stair_start_z - 1.0), label_properties.clone());

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-8.0, 0.75, stair_start_z + 2.5),
        nalgebra_glm::vec3(3.0, 0.5, 2.0),
        platform_material.clone(),
    );
    spawn_label(world, "1m", nalgebra_glm::vec3(-8.0, 1.5, stair_start_z + 1.5), label_properties.clone());

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-8.0, 1.5, stair_start_z + 5.0),
        nalgebra_glm::vec3(3.0, 0.5, 2.0),
        platform_material.clone(),
    );
    spawn_label(world, "2m", nalgebra_glm::vec3(-8.0, 2.3, stair_start_z + 4.0), label_properties.clone());

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-8.0, 2.5, stair_start_z + 7.5),
        nalgebra_glm::vec3(3.0, 0.5, 2.0),
        platform_material.clone(),
    );
    spawn_label(world, "3m", nalgebra_glm::vec3(-8.0, 3.3, stair_start_z + 6.5), label_properties.clone());

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-8.0, 4.0, stair_start_z + 10.0),
        nalgebra_glm::vec3(4.0, 0.5, 3.0),
        accent_material.clone(),
    );
    spawn_label(world, "4m ledge", nalgebra_glm::vec3(-8.0, 4.8, stair_start_z + 8.5), label_properties.clone());

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-8.0, 6.0, stair_start_z + 14.0),
        nalgebra_glm::vec3(3.0, 0.5, 2.0),
        platform_material.clone(),
    );
    spawn_label(world, "6m", nalgebra_glm::vec3(-8.0, 6.8, stair_start_z + 13.0), label_properties.clone());

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

pub(super) fn spawn_visual_cube(
    world: &mut World,
    position: Vec3,
    scale: Vec3,
    material: nightshade::ecs::material::components::Material,
    name: String,
) {
    crate::systems::spawn::spawn_visual_entity_with_shadow(world, position, scale, "Cube", material, name);
}

pub(super) fn spawn_room_walls(world: &mut World, config: &RoomConfig) {
    let center = config.center;
    let room_width = config.width;
    let room_depth = config.depth;
    let room_height = config.height;
    let wall_thickness = config.wall_thickness;
    let doorway_width = config.doorway_width;
    let doorway_height = config.doorway_height;
    let wall_material = config.wall_material.clone();
    let ceiling_material = config.ceiling_material.clone();

    let half_width = room_width / 2.0;
    let half_depth = room_depth / 2.0;
    let wall_center_y = room_height / 2.0;

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, wall_center_y, center.z + half_depth - wall_thickness / 2.0),
        nalgebra_glm::vec3(room_width, room_height, wall_thickness),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x - half_width + wall_thickness / 2.0, wall_center_y, center.z),
        nalgebra_glm::vec3(wall_thickness, room_height, room_depth),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x + half_width - wall_thickness / 2.0, wall_center_y, center.z),
        nalgebra_glm::vec3(wall_thickness, room_height, room_depth),
        wall_material.clone(),
    );

    let front_z = center.z - half_depth + wall_thickness / 2.0;
    let segment_width = (room_width - doorway_width) / 2.0;

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x - half_width + segment_width / 2.0, wall_center_y, front_z),
        nalgebra_glm::vec3(segment_width, room_height, wall_thickness),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x + half_width - segment_width / 2.0, wall_center_y, front_z),
        nalgebra_glm::vec3(segment_width, room_height, wall_thickness),
        wall_material.clone(),
    );

    let header_height = room_height - doorway_height;
    if header_height > 0.01 {
        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, doorway_height + header_height / 2.0, front_z),
            nalgebra_glm::vec3(doorway_width, header_height, wall_thickness),
            wall_material,
        );
    }

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, room_height + wall_thickness / 2.0, center.z),
        nalgebra_glm::vec3(room_width, wall_thickness, room_depth),
        ceiling_material,
    );
}

pub(super) fn spawn_room_light(world: &mut World, position: Vec3, color: Vec3, intensity: f32) {
    let light_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | LIGHT,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(light_entity) {
        name.0 = "Room Light".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(light_entity) {
        transform.translation = position;
    }

    if let Some(light) = world.core.get_light_mut(light_entity) {
        *light = Light {
            light_type: LightType::Point,
            color,
            intensity,
            range: 8.0,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: true,
            shadow_bias: 0.005,
        };
    }
}
