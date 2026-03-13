use crate::ecs::GameWorld;
use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::prelude::*;

pub fn spawn_camera(game_world: &mut GameWorld, world: &mut World) {
    let camera_entity = spawn_pan_orbit_camera(
        world,
        Vec3::new(0.0, 1.0, 0.0),
        5.0,
        0.0,
        0.4,
        "Third Person Camera".to_string(),
    );
    world.resources.active_camera = Some(camera_entity);

    game_world.resources.camera_entity = Some(freecs::Entity {
        id: camera_entity.id,
        generation: camera_entity.generation,
    });
}

pub fn camera_follow_system(game_world: &GameWorld, world: &mut World) {
    let Some(controller_entity) = game_world.resources.controller_entity else {
        return;
    };
    let Some(camera_entity) = game_world.resources.camera_entity else {
        return;
    };

    let engine_controller = nightshade::prelude::Entity {
        id: controller_entity.id,
        generation: controller_entity.generation,
    };
    let engine_camera = nightshade::prelude::Entity {
        id: camera_entity.id,
        generation: camera_entity.generation,
    };

    let controller_position = world
        .core.get_local_transform(engine_controller)
        .map(|t| t.translation)
        .unwrap_or(Vec3::zeros());

    let target_focus = Vec3::new(
        controller_position.x,
        controller_position.y,
        controller_position.z,
    );
    let delta_time = world.resources.window.timing.delta_time;

    if let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(engine_camera) {
        let lerp_speed = 5.0;
        let t = (lerp_speed * delta_time).min(1.0);
        pan_orbit.target_focus =
            pan_orbit.target_focus + (target_focus - pan_orbit.target_focus) * t;
    }
}
