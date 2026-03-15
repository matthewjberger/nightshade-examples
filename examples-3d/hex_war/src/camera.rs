use crate::hex::hex_to_world_position;
use nightshade::prelude::*;

pub struct CameraBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
}

pub fn calculate_camera_bounds(
    hex_width: f32,
    hex_depth: f32,
    map_width: i32,
    map_height: i32,
) -> CameraBounds {
    let min_pos = hex_to_world_position(0, 0, hex_width, hex_depth);
    let max_pos = hex_to_world_position(map_width - 1, map_height - 1, hex_width, hex_depth);

    let padding_x = hex_width * 2.0;
    let padding_z = hex_depth * 2.0;

    CameraBounds {
        min_x: min_pos.x - padding_x,
        max_x: max_pos.x + padding_x,
        min_z: min_pos.z - padding_z,
        max_z: max_pos.z + padding_z,
    }
}

pub fn clamp_camera_to_bounds(world: &mut World, bounds: &CameraBounds) {
    let Some(camera_entity) = world.resources.active_camera else {
        return;
    };

    let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(camera_entity) else {
        return;
    };

    pan_orbit.target_focus.x = pan_orbit.target_focus.x.clamp(bounds.min_x, bounds.max_x);
    pan_orbit.target_focus.z = pan_orbit.target_focus.z.clamp(bounds.min_z, bounds.max_z);
}

pub fn reset_camera_to_map(
    world: &mut World,
    hex_width: f32,
    hex_depth: f32,
    map_width: i32,
    map_height: i32,
) {
    let Some(camera_entity) = world.resources.active_camera else {
        return;
    };

    let world_width = map_width as f32 * hex_width * 0.75;
    let world_height = map_height as f32 * hex_depth;

    let center_column = (map_width - 1) / 2;
    let center_row = (map_height - 1) / 2;
    let center_pos = hex_to_world_position(center_column, center_row, hex_width, hex_depth);

    let y_fov_rad = if let Some(camera) = world.core.get_camera(camera_entity) {
        match &camera.projection {
            Projection::Perspective(persp) => persp.y_fov_rad,
            Projection::Orthographic(_) => std::f32::consts::FRAC_PI_4,
        }
    } else {
        std::f32::consts::FRAC_PI_4
    };

    let (viewport_width, viewport_height) = world
        .resources
        .window
        .cached_viewport_size
        .unwrap_or((1920, 1080));
    let aspect_ratio = viewport_width as f32 / viewport_height as f32;

    let half_fov_tan = (y_fov_rad / 2.0).tan();
    let radius_for_height = (world_height / 2.0) / half_fov_tan;
    let radius_for_width = (world_width / 2.0) / (half_fov_tan * aspect_ratio);
    let radius = radius_for_height.max(radius_for_width) * 1.1;

    let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(camera_entity) else {
        return;
    };

    pan_orbit.target_focus = nalgebra_glm::vec3(center_pos.x, 0.0, center_pos.z);
    pan_orbit.target_radius = radius;
    pan_orbit.target_yaw = 0.0;
    pan_orbit.target_pitch = std::f32::consts::FRAC_PI_2 - 0.01;
}
