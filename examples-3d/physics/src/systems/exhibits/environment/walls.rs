use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(in crate::systems::exhibits) struct RoomConfig {
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

pub(in crate::systems::exhibits) fn spawn_room_walls(world: &mut World, config: &RoomConfig) {
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
        nalgebra_glm::vec3(
            center.x,
            wall_center_y,
            center.z + half_depth - wall_thickness / 2.0,
        ),
        nalgebra_glm::vec3(room_width, room_height, wall_thickness),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(
            center.x - half_width + wall_thickness / 2.0,
            wall_center_y,
            center.z,
        ),
        nalgebra_glm::vec3(wall_thickness, room_height, room_depth),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(
            center.x + half_width - wall_thickness / 2.0,
            wall_center_y,
            center.z,
        ),
        nalgebra_glm::vec3(wall_thickness, room_height, room_depth),
        wall_material.clone(),
    );

    let front_z = center.z - half_depth + wall_thickness / 2.0;
    let segment_width = (room_width - doorway_width) / 2.0;

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(
            center.x - half_width + segment_width / 2.0,
            wall_center_y,
            front_z,
        ),
        nalgebra_glm::vec3(segment_width, room_height, wall_thickness),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(
            center.x + half_width - segment_width / 2.0,
            wall_center_y,
            front_z,
        ),
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

pub(in crate::systems::exhibits) fn spawn_room_light(
    world: &mut World,
    position: Vec3,
    color: Vec3,
    intensity: f32,
) {
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
