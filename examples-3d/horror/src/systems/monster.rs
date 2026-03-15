use crate::ecs::{CutscenePhase, GameWorld};
use crate::systems::doors::slam_door_closed;
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

const CUTSCENE_LOOK_DURATION: f32 = 1.5;
const CUTSCENE_WALL_BREAK_DURATION: f32 = 1.0;
const CUTSCENE_EMERGE_DURATION: f32 = 1.5;
const CUTSCENE_RETURN_DURATION: f32 = 0.5;
const MONSTER_PAUSE_DURATION: f32 = 3.0;
const CUTSCENE_DOOR_SLAM_DURATION: f32 = 0.3;
const CUTSCENE_LOOK_AT_DOOR_DURATION: f32 = 2.0;
const EXIT_AREA_Z_THRESHOLD: f32 = -27.0;

pub fn start_cutscene(game_world: &mut GameWorld, world: &mut World) {
    game_world.resources.cutscene.active = true;
    game_world.resources.cutscene.phase = CutscenePhase::LookAtWall;
    game_world.resources.cutscene.timer = 0.0;

    game_world.resources.cutscene.saved_base_rotation =
        game_world.resources.lean_state.base_rotation;

    world.resources.graphics.letterbox_target = 1.0;

    if let Some(player_entity) = game_world.resources.player_entity
        && let Some(rigid_body) = world.core.get_rigid_body(player_entity)
        && let Some(handle) = rigid_body.handle
        && let Some(rb) = world
            .resources
            .physics
            .rigid_body_set
            .get_mut(handle.into())
    {
        rb.set_linvel(rapier3d::math::Vector::zeros(), true);
    }

    if let Some(ambient_entity) = game_world.resources.ambient_audio_entity
        && let Some(source) = world.core.get_audio_source_mut(ambient_entity)
    {
        source.playing = false;
    }

    let wall_break_pos = nalgebra_glm::vec3(-4.5, 1.5, -16.0);
    game_world.resources.cutscene.wall_break_position = wall_break_pos;

    if let Some(camera_entity) = game_world.resources.camera_entity
        && let Some(camera_transform) = world.core.get_global_transform(camera_entity)
    {
        let camera_pos = camera_transform.translation();
        let direction = wall_break_pos - camera_pos;
        let direction_normalized = nalgebra_glm::normalize(&direction);

        let yaw = (-direction_normalized.x).atan2(-direction_normalized.z);
        let pitch = direction_normalized.y.asin();

        let yaw_quat = nalgebra_glm::quat_angle_axis(yaw, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
        let pitch_quat = nalgebra_glm::quat_angle_axis(-pitch, &nalgebra_glm::vec3(1.0, 0.0, 0.0));
        game_world.resources.cutscene.target_rotation = yaw_quat * pitch_quat;
    }
}

pub fn cutscene_system(game_world: &mut GameWorld, world: &mut World) {
    if !game_world.resources.cutscene.active {
        return;
    }

    let dt = world.resources.window.timing.delta_time;
    game_world.resources.cutscene.timer += dt;

    match game_world.resources.cutscene.phase {
        CutscenePhase::None => {}
        CutscenePhase::LookAtWall => {
            let progress = (game_world.resources.cutscene.timer / CUTSCENE_LOOK_DURATION).min(1.0);
            let smoothed = smooth_step(progress);

            game_world.resources.lean_state.base_rotation = nalgebra_glm::quat_slerp(
                &game_world.resources.cutscene.saved_base_rotation,
                &game_world.resources.cutscene.target_rotation,
                smoothed,
            );

            if game_world.resources.cutscene.timer >= CUTSCENE_LOOK_DURATION {
                game_world.resources.cutscene.phase = CutscenePhase::WallBreaks;
                game_world.resources.cutscene.timer = 0.0;
                spawn_wall_destruction(game_world, world);

                if let Some(rubble_entity) = game_world.resources.rubble_audio_entity
                    && let Some(source) = world.core.get_audio_source_mut(rubble_entity)
                {
                    source.playing = true;
                }
            }
        }
        CutscenePhase::WallBreaks => {
            if game_world.resources.cutscene.timer >= CUTSCENE_WALL_BREAK_DURATION {
                game_world.resources.cutscene.phase = CutscenePhase::MonsterEmerges;
                game_world.resources.cutscene.timer = 0.0;
                spawn_monster(game_world, world);
            }
        }
        CutscenePhase::MonsterEmerges => {
            if game_world.resources.cutscene.timer >= CUTSCENE_EMERGE_DURATION {
                game_world.resources.cutscene.phase = CutscenePhase::ReturnControl;
                game_world.resources.cutscene.timer = 0.0;
                game_world.resources.monster.active = true;

                if let Some(monster_audio_entity) = game_world.resources.monster_audio_entity
                    && let Some(source) = world.core.get_audio_source_mut(monster_audio_entity)
                {
                    source.playing = true;
                }
            }
        }
        CutscenePhase::ReturnControl => {
            world.resources.graphics.letterbox_target = 0.0;
            if game_world.resources.cutscene.timer >= CUTSCENE_RETURN_DURATION {
                game_world.resources.cutscene.active = false;
                game_world.resources.cutscene.phase = CutscenePhase::None;
            }
        }
        CutscenePhase::DoorSlam => {
            if game_world.resources.cutscene.timer >= CUTSCENE_DOOR_SLAM_DURATION {
                game_world.resources.cutscene.phase = CutscenePhase::LookAtDoor;
                game_world.resources.cutscene.timer = 0.0;

                if let Some(camera_entity) = game_world.resources.camera_entity
                    && let Some(exit_door_game_entity) = game_world.resources.exit_door
                    && let Some(exit_door) = game_world.get_door(exit_door_game_entity)
                    && let Some(camera_transform) = world.core.get_global_transform(camera_entity)
                {
                    let door_pos = exit_door.hinge_position;
                    let camera_pos = camera_transform.translation();
                    let direction = door_pos - camera_pos;
                    let direction_normalized = nalgebra_glm::normalize(&direction);

                    let yaw = (-direction_normalized.x).atan2(-direction_normalized.z);
                    let pitch = direction_normalized.y.asin();

                    let yaw_quat =
                        nalgebra_glm::quat_angle_axis(yaw, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
                    let pitch_quat =
                        nalgebra_glm::quat_angle_axis(-pitch, &nalgebra_glm::vec3(1.0, 0.0, 0.0));
                    game_world.resources.cutscene.target_rotation = yaw_quat * pitch_quat;
                }
            }
        }
        CutscenePhase::LookAtDoor => {
            let progress =
                (game_world.resources.cutscene.timer / CUTSCENE_LOOK_AT_DOOR_DURATION).min(1.0);
            let smoothed = smooth_step(progress);

            game_world.resources.lean_state.base_rotation = nalgebra_glm::quat_slerp(
                &game_world.resources.cutscene.saved_base_rotation,
                &game_world.resources.cutscene.target_rotation,
                smoothed,
            );

            if game_world.resources.cutscene.timer >= CUTSCENE_LOOK_AT_DOOR_DURATION {
                game_world.resources.fade_target = 1.0;
                game_world.resources.cutscene.active = false;
                game_world.resources.cutscene.phase = CutscenePhase::None;
                game_world.resources.game_won = true;
            }
        }
    }
}

fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn spawn_wall_destruction(game_world: &mut GameWorld, world: &mut World) {
    let break_pos = game_world.resources.cutscene.wall_break_position;

    let rubble_material = create_textured_material(nalgebra_glm::vec3(0.35, 0.35, 0.38), 0.9, 0.1);

    for index in 0..20 {
        let offset_x = (index % 5) as f32 * 0.3 - 0.6;
        let offset_y = (index / 5) as f32 * 0.3;
        let offset_z = ((index * 7) % 3) as f32 * 0.2 - 0.2;

        let size = 0.15 + (index % 4) as f32 * 0.08;

        let entity = spawn_dynamic_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(
                break_pos.x + offset_x,
                break_pos.y + offset_y,
                break_pos.z + offset_z,
            ),
            nalgebra_glm::vec3(size, size, size),
            2.0 + (index % 3) as f32,
            rubble_material.clone(),
        );

        if let Some(rb_comp) = world.core.get_rigid_body(entity)
            && let Some(handle) = rb_comp.handle
            && let Some(rb) = world
                .resources
                .physics
                .rigid_body_set
                .get_mut(handle.into())
        {
            let impulse_x = ((index * 17) % 10) as f32 * 0.3 - 1.5;
            let impulse_y = 2.0 + (index % 5) as f32 * 0.5;
            let impulse_z = 3.0 + ((index * 13) % 7) as f32 * 0.4;
            rb.apply_impulse(nalgebra_glm::vec3(impulse_x, impulse_y, impulse_z), true);
            rb.apply_torque_impulse(
                nalgebra_glm::vec3(
                    (index % 3) as f32 - 1.0,
                    (index % 5) as f32 - 2.0,
                    (index % 4) as f32 - 1.5,
                ),
                true,
            );
        }
    }

    spawn_dust_particles(world, break_pos);
}

fn spawn_dust_particles(world: &mut World, position: Vec3) {
    let dust_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.55, 0.5), 0.95, 0.0);

    for index in 0..15 {
        let angle = (index as f32 / 15.0) * std::f32::consts::TAU;
        let radius = 0.3 + (index % 3) as f32 * 0.2;
        let offset_x = angle.cos() * radius;
        let offset_z = angle.sin() * radius + 0.5;
        let offset_y = (index % 4) as f32 * 0.3;

        let size = 0.08 + (index % 3) as f32 * 0.04;

        let entity = spawn_dynamic_physics_sphere_with_material(
            world,
            nalgebra_glm::vec3(
                position.x + offset_x,
                position.y + offset_y,
                position.z + offset_z,
            ),
            size,
            0.1,
            dust_material.clone(),
        );

        if let Some(rb_comp) = world.core.get_rigid_body(entity)
            && let Some(handle) = rb_comp.handle
            && let Some(rb) = world
                .resources
                .physics
                .rigid_body_set
                .get_mut(handle.into())
        {
            rb.set_linear_damping(3.0);
            rb.set_gravity_scale(0.1, true);
            let impulse_x = offset_x * 2.0;
            let impulse_y = 1.0 + (index % 3) as f32 * 0.3;
            let impulse_z = offset_z * 2.0 + 1.0;
            rb.apply_impulse(nalgebra_glm::vec3(impulse_x, impulse_y, impulse_z), true);
        }
    }
}

fn spawn_monster(game_world: &mut GameWorld, world: &mut World) {
    let break_pos = game_world.resources.cutscene.wall_break_position;
    let monster_pos = nalgebra_glm::vec3(break_pos.x - 0.5, 1.0, break_pos.z);

    let entity = world.spawn_entities(
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

    if let Some(name) = world.core.get_name_mut(entity) {
        name.0 = "Monster".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = monster_pos;
        transform.scale = nalgebra_glm::vec3(0.5, 2.0, 0.5);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = "Cylinder".to_string();
    }

    let monster_material = Material {
        base_color: [0.8, 0.1, 0.1, 1.0],
        emissive_factor: [0.3, 0.0, 0.0],
        roughness: 0.6,
        metallic: 0.2,
        ..Default::default()
    };
    world.register_material(entity, "monster".to_string(), monster_material);

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
    }

    game_world.resources.monster.root_entity = Some(entity);
    game_world.resources.monster.speed = 2.0;
}

pub fn monster_chase_system(game_world: &mut GameWorld, world: &mut World) {
    if !game_world.resources.monster.active {
        return;
    }

    let Some(player_entity) = game_world.resources.player_entity else {
        return;
    };

    let Some(monster_entity) = game_world.resources.monster.root_entity else {
        return;
    };

    let dt = world.resources.window.timing.delta_time;

    let player_pos = world
        .core
        .get_global_transform(player_entity)
        .map(|t| t.translation())
        .unwrap_or(Vec3::zeros());

    if player_pos.z < EXIT_AREA_Z_THRESHOLD && !game_world.resources.game_won {
        start_exit_cutscene(game_world, world);
        despawn_monster(game_world, world);
        return;
    }

    if !game_world.resources.monster.chasing {
        game_world.resources.monster.pause_timer += dt;
        if game_world.resources.monster.pause_timer >= MONSTER_PAUSE_DURATION {
            game_world.resources.monster.chasing = true;
        }
        return;
    }

    let monster_pos = world
        .core
        .get_local_transform(monster_entity)
        .map(|t| t.translation)
        .unwrap_or(Vec3::zeros());

    let direction = player_pos - monster_pos;
    let horizontal_dir = nalgebra_glm::vec3(direction.x, 0.0, direction.z);
    let distance = nalgebra_glm::length(&horizontal_dir);

    if distance < 0.1 {
        return;
    }

    let normalized_dir = nalgebra_glm::normalize(&horizontal_dir);
    let movement = normalized_dir * game_world.resources.monster.speed * dt;

    let angle = (-normalized_dir.x).atan2(-normalized_dir.z);
    let target_rotation = nalgebra_glm::quat_angle_axis(angle, &nalgebra_glm::vec3(0.0, 1.0, 0.0));

    if let Some(transform) = world.core.get_local_transform_mut(monster_entity) {
        transform.translation += movement;
        let current_rotation = transform.rotation;
        transform.rotation =
            nalgebra_glm::quat_slerp(&current_rotation, &target_rotation, dt * 8.0);
    }
    world.mark_local_transform_dirty(monster_entity);

    if distance < 1.2 && !game_world.resources.game_won {
        game_world.resources.game_won = true;
    }
}

fn start_exit_cutscene(game_world: &mut GameWorld, world: &mut World) {
    game_world.resources.cutscene.active = true;
    game_world.resources.cutscene.phase = CutscenePhase::DoorSlam;
    game_world.resources.cutscene.timer = 0.0;
    game_world.resources.cutscene.saved_base_rotation =
        game_world.resources.lean_state.base_rotation;

    world.resources.graphics.letterbox_target = 1.0;

    if let Some(exit_door_game_entity) = game_world.resources.exit_door {
        slam_door_closed(game_world, world, exit_door_game_entity);
    }
}

fn despawn_monster(game_world: &mut GameWorld, world: &mut World) {
    if let Some(entity) = game_world.resources.monster.root_entity {
        world.queue_despawn_entity(entity);
    }

    game_world.resources.monster.root_entity = None;
    game_world.resources.monster.active = false;
    game_world.resources.monster.chasing = false;

    if let Some(monster_audio_entity) = game_world.resources.monster_audio_entity
        && let Some(source) = world.core.get_audio_source_mut(monster_audio_entity)
    {
        source.playing = false;
    }
}
