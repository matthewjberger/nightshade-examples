use crate::ecs::GameWorld;
use nightshade::prelude::*;

pub fn update_day_night_cycle(game_world: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time;
    game_world.resources.current_hour += game_world.resources.time_speed * delta_time;

    if game_world.resources.current_hour >= 24.0 {
        game_world.resources.current_hour -= 24.0;
    }

    world.resources.graphics.day_night_hour = game_world.resources.current_hour;

    update_sun(game_world, world);
    update_ambient(game_world, world);
}

fn sun_direction(hour: f32) -> Vec3 {
    if !(6.0..=18.0).contains(&hour) {
        Vec3::new(0.0, -1.0, 0.0)
    } else {
        let sun_angle = (hour - 6.0) / 12.0 * std::f32::consts::PI;
        nalgebra_glm::normalize(&Vec3::new(-sun_angle.cos(), sun_angle.sin(), -0.3))
    }
}

fn update_sun(game_world: &GameWorld, world: &mut World) {
    let Some(sun) = game_world.resources.sun_entity else {
        return;
    };

    let hour = game_world.resources.current_hour;
    let direction = sun_direction(hour);
    let is_night = !(6.0..=18.0).contains(&hour);

    let intensity = if is_night {
        0.0
    } else {
        let elevation = direction.y.max(0.0);
        3.5 * elevation.sqrt()
    };

    let warm = Vec3::new(1.0, 0.7, 0.4);
    let white = Vec3::new(1.0, 0.95, 0.8);
    let color = if is_night {
        Vec3::new(0.0, 0.0, 0.0)
    } else if hour < 7.5 {
        let transition = ((hour - 6.0) / 1.5).clamp(0.0, 1.0);
        nalgebra_glm::lerp(&warm, &white, transition)
    } else if hour > 16.5 {
        let transition = ((18.0 - hour) / 1.5).clamp(0.0, 1.0);
        nalgebra_glm::lerp(&warm, &white, transition)
    } else {
        white
    };

    if let Some(light) = world.core.get_light_mut(sun) {
        light.intensity = intensity;
        light.color = color;
    }

    let sun_position = direction * 100.0;
    if let Some(transform) = world.core.get_local_transform_mut(sun) {
        transform.translation = sun_position;
        let look_direction = -direction;
        let up = Vec3::y();
        let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&look_direction, &up));
        if right.norm() > 0.001 {
            let corrected_up = nalgebra_glm::cross(&right, &look_direction);
            transform.rotation =
                nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[
                    right,
                    corrected_up,
                    -look_direction,
                ]));
        }
    }
    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, sun);
}

fn update_ambient(game_world: &GameWorld, world: &mut World) {
    let hour = game_world.resources.current_hour;
    let is_night = !(6.0..=18.0).contains(&hour);

    let ambient_color = if is_night {
        [0.05, 0.05, 0.1, 1.0]
    } else if !(7.5..=16.5).contains(&hour) {
        let transition = if hour < 7.5 {
            (hour - 6.0) / 1.5
        } else {
            (18.0 - hour) / 1.5
        };
        [
            0.05 + 0.20 * transition,
            0.05 + 0.17 * transition,
            0.10 + 0.10 * transition,
            1.0,
        ]
    } else {
        [0.25, 0.22, 0.20, 1.0]
    };

    world.resources.graphics.ambient_light = ambient_color;
}
