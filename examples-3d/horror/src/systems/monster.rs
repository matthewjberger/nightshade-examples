use crate::ecs::{
    CutscenePhase, ENGINE_ENTITY, EngineEntity, GameWorld, MONSTER_PART, MonsterPart,
    MonsterPartRole,
};
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

        let game_entity = game_world.spawn_entities(ENGINE_ENTITY, 1)[0];
        game_world.set_engine_entity(game_entity, EngineEntity(entity));
        game_world.add_physics_prop(game_entity);
    }

    spawn_dust_particles(game_world, world, break_pos);
}

fn spawn_dust_particles(game_world: &mut GameWorld, world: &mut World, position: Vec3) {
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

        let game_entity = game_world.spawn_entities(ENGINE_ENTITY, 1)[0];
        game_world.set_engine_entity(game_entity, EngineEntity(entity));
        game_world.add_physics_prop(game_entity);
    }
}

fn spawn_monster(game_world: &mut GameWorld, world: &mut World) {
    let break_pos = game_world.resources.cutscene.wall_break_position;
    let monster_pos = nalgebra_glm::vec3(break_pos.x - 0.5, 0.0, break_pos.z);

    let flesh_material = create_textured_material(nalgebra_glm::vec3(0.45, 0.08, 0.08), 0.85, 0.15);
    let dark_flesh = create_textured_material(nalgebra_glm::vec3(0.25, 0.03, 0.03), 0.9, 0.1);
    let bone_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.65, 0.55), 0.6, 0.2);
    let vein_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.0, 0.0), 0.5, 0.4);
    let eye_material = create_emissive_material(nalgebra_glm::vec3(1.0, 0.2, 0.0), 5.0);
    let inner_glow = create_emissive_material(nalgebra_glm::vec3(0.8, 0.1, 0.0), 2.0);

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 1.4, -0.1),
        nalgebra_glm::vec3(0.7, 0.9, 0.5),
        "Cube",
        flesh_material.clone(),
        MonsterPartRole::Torso,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 1.5, 0.2),
        nalgebra_glm::vec3(0.5, 0.6, 0.15),
        "Cube",
        dark_flesh.clone(),
        MonsterPartRole::Chest,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 1.35, 0.15),
        nalgebra_glm::vec3(0.3, 0.25, 0.1),
        "Sphere",
        inner_glow.clone(),
        MonsterPartRole::Chest,
    );

    for rib_index in 0..4 {
        let y_offset = 1.2 + rib_index as f32 * 0.15;
        spawn_monster_part(
            game_world,
            world,
            monster_pos + nalgebra_glm::vec3(-0.3, y_offset, 0.1),
            nalgebra_glm::vec3(0.12, 0.04, 0.2),
            "Cube",
            bone_material.clone(),
            MonsterPartRole::Ribcage,
        );
        spawn_monster_part(
            game_world,
            world,
            monster_pos + nalgebra_glm::vec3(0.3, y_offset, 0.1),
            nalgebra_glm::vec3(0.12, 0.04, 0.2),
            "Cube",
            bone_material.clone(),
            MonsterPartRole::Ribcage,
        );
    }

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 1.95, 0.0),
        nalgebra_glm::vec3(0.2, 0.2, 0.2),
        "Cylinder",
        dark_flesh.clone(),
        MonsterPartRole::Head,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 2.2, 0.15),
        nalgebra_glm::vec3(0.35, 0.4, 0.4),
        "Cube",
        flesh_material.clone(),
        MonsterPartRole::Head,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 2.45, 0.0),
        nalgebra_glm::vec3(0.25, 0.12, 0.35),
        "Cube",
        bone_material.clone(),
        MonsterPartRole::Head,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 2.0, 0.3),
        nalgebra_glm::vec3(0.28, 0.15, 0.2),
        "Cube",
        dark_flesh.clone(),
        MonsterPartRole::Head,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(-0.12, 2.25, 0.35),
        nalgebra_glm::vec3(0.1, 0.1, 0.1),
        "Sphere",
        eye_material.clone(),
        MonsterPartRole::Head,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.12, 2.25, 0.35),
        nalgebra_glm::vec3(0.1, 0.1, 0.1),
        "Sphere",
        eye_material.clone(),
        MonsterPartRole::Head,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 2.35, 0.38),
        nalgebra_glm::vec3(0.06, 0.06, 0.06),
        "Sphere",
        eye_material,
        MonsterPartRole::Head,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(-0.5, 1.7, 0.0),
        nalgebra_glm::vec3(0.25, 0.2, 0.25),
        "Sphere",
        flesh_material.clone(),
        MonsterPartRole::Arm,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(-0.65, 1.4, 0.1),
        nalgebra_glm::vec3(0.12, 0.5, 0.12),
        "Cube",
        flesh_material.clone(),
        MonsterPartRole::Arm,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(-0.7, 0.95, 0.2),
        nalgebra_glm::vec3(0.1, 0.45, 0.1),
        "Cube",
        dark_flesh.clone(),
        MonsterPartRole::Arm,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(-0.72, 0.6, 0.25),
        nalgebra_glm::vec3(0.15, 0.2, 0.08),
        "Cube",
        dark_flesh.clone(),
        MonsterPartRole::Arm,
    );

    for finger_index in 0..4 {
        spawn_monster_part(
            game_world,
            world,
            monster_pos
                + nalgebra_glm::vec3(
                    -0.68 - finger_index as f32 * 0.03,
                    0.42,
                    0.22 + finger_index as f32 * 0.04,
                ),
            nalgebra_glm::vec3(0.02, 0.18, 0.02),
            "Cube",
            bone_material.clone(),
            MonsterPartRole::Arm,
        );
    }

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.5, 1.7, 0.0),
        nalgebra_glm::vec3(0.25, 0.2, 0.25),
        "Sphere",
        flesh_material.clone(),
        MonsterPartRole::Arm,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.65, 1.4, 0.1),
        nalgebra_glm::vec3(0.12, 0.5, 0.12),
        "Cube",
        flesh_material.clone(),
        MonsterPartRole::Arm,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.7, 0.95, 0.2),
        nalgebra_glm::vec3(0.1, 0.45, 0.1),
        "Cube",
        dark_flesh.clone(),
        MonsterPartRole::Arm,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.72, 0.6, 0.25),
        nalgebra_glm::vec3(0.15, 0.2, 0.08),
        "Cube",
        dark_flesh.clone(),
        MonsterPartRole::Arm,
    );

    for finger_index in 0..4 {
        spawn_monster_part(
            game_world,
            world,
            monster_pos
                + nalgebra_glm::vec3(
                    0.68 + finger_index as f32 * 0.03,
                    0.42,
                    0.22 + finger_index as f32 * 0.04,
                ),
            nalgebra_glm::vec3(0.02, 0.18, 0.02),
            "Cube",
            bone_material.clone(),
            MonsterPartRole::Arm,
        );
    }

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(-0.35, 1.3, -0.2),
        nalgebra_glm::vec3(0.15, 0.12, 0.15),
        "Sphere",
        dark_flesh.clone(),
        MonsterPartRole::Arm,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(-0.45, 1.0, -0.15),
        nalgebra_glm::vec3(0.08, 0.4, 0.08),
        "Cube",
        dark_flesh.clone(),
        MonsterPartRole::Arm,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 0.85, -0.05),
        nalgebra_glm::vec3(0.55, 0.35, 0.4),
        "Cube",
        flesh_material.clone(),
        MonsterPartRole::Leg,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(-0.22, 0.5, 0.0),
        nalgebra_glm::vec3(0.18, 0.45, 0.18),
        "Cube",
        flesh_material.clone(),
        MonsterPartRole::Leg,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(-0.22, 0.15, 0.05),
        nalgebra_glm::vec3(0.12, 0.35, 0.12),
        "Cube",
        dark_flesh.clone(),
        MonsterPartRole::Leg,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.22, 0.5, 0.0),
        nalgebra_glm::vec3(0.18, 0.45, 0.18),
        "Cube",
        flesh_material.clone(),
        MonsterPartRole::Leg,
    );

    spawn_monster_part(
        game_world,
        world,
        monster_pos + nalgebra_glm::vec3(0.22, 0.15, 0.05),
        nalgebra_glm::vec3(0.12, 0.35, 0.12),
        "Cube",
        dark_flesh.clone(),
        MonsterPartRole::Leg,
    );

    for spine_index in 0..6 {
        let size = 0.08 + (spine_index as f32 * 0.015);
        spawn_monster_part(
            game_world,
            world,
            monster_pos
                + nalgebra_glm::vec3(
                    0.0,
                    1.0 + spine_index as f32 * 0.18,
                    -0.3 - spine_index as f32 * 0.02,
                ),
            nalgebra_glm::vec3(size, size * 1.5, size * 2.0),
            "Cube",
            bone_material.clone(),
            MonsterPartRole::Spine,
        );
    }

    for vein_index in 0..5 {
        let angle = vein_index as f32 * 1.2;
        spawn_monster_part(
            game_world,
            world,
            monster_pos
                + nalgebra_glm::vec3(
                    angle.sin() * 0.25,
                    1.2 + vein_index as f32 * 0.12,
                    angle.cos() * 0.2,
                ),
            nalgebra_glm::vec3(0.03, 0.15, 0.03),
            "Cylinder",
            vein_material.clone(),
            MonsterPartRole::Spine,
        );
    }

    for tendril_index in 0..3 {
        let x_offset = (tendril_index as f32 - 1.0) * 0.2;
        spawn_monster_part(
            game_world,
            world,
            monster_pos + nalgebra_glm::vec3(x_offset, 0.6, -0.35),
            nalgebra_glm::vec3(0.04, 0.5, 0.04),
            "Cylinder",
            dark_flesh.clone(),
            MonsterPartRole::Spine,
        );
    }

    game_world.resources.monster.speed = 2.0;
}

fn spawn_monster_part(
    game_world: &mut GameWorld,
    world: &mut World,
    position: Vec3,
    scale: Vec3,
    mesh_name: &str,
    material: Material,
    role: MonsterPartRole,
) -> Entity {
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
        name.0 = "Monster Part".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = scale;
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = mesh_name.to_string();
    }

    let material_name = format!("MonsterPart_{}", entity.id);
    world.register_material(entity, material_name, material);

    if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
        *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type(mesh_name);
    }

    let game_entity = game_world.spawn_entities(ENGINE_ENTITY | MONSTER_PART, 1)[0];
    game_world.set_engine_entity(game_entity, EngineEntity(entity));
    game_world.set_monster_part(game_entity, MonsterPart { role });

    entity
}

pub fn monster_chase_system(game_world: &mut GameWorld, world: &mut World) {
    if !game_world.resources.monster.active {
        return;
    }

    let Some(player_entity) = game_world.resources.player_entity else {
        return;
    };

    let Some(monster_entity) = game_world
        .query_entities(MONSTER_PART | ENGINE_ENTITY)
        .find(|&game_entity| {
            game_world
                .get_monster_part(game_entity)
                .is_some_and(|part| part.role == MonsterPartRole::Torso)
        })
        .and_then(|game_entity| game_world.get_engine_entity(game_entity).map(|e| e.0))
    else {
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

    let total_time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

    let walk_cycle = total_time * 3.5;
    let body_bob = (walk_cycle * 2.0).sin() * 0.015;
    let body_sway = walk_cycle.sin() * 0.008;
    let arm_swing = walk_cycle.sin() * 0.04;
    let head_bob = (walk_cycle * 2.0).sin() * 0.008;
    let breathing = (total_time * 1.5).sin() * 0.006;

    let monster_parts: Vec<(Entity, MonsterPartRole)> = game_world
        .query_entities(MONSTER_PART | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            let engine_entity = game_world.get_engine_entity(game_entity)?.0;
            let role = game_world.get_monster_part(game_entity)?.role;
            Some((engine_entity, role))
        })
        .collect();

    for &(part_entity, role) in &monster_parts {
        if let Some(transform) = world.core.get_local_transform_mut(part_entity) {
            transform.translation += movement;
            let current_rotation = transform.rotation;
            transform.rotation =
                nalgebra_glm::quat_slerp(&current_rotation, &target_rotation, dt * 8.0);

            match role {
                MonsterPartRole::Torso => {
                    transform.translation.y += body_bob + breathing;
                    transform.translation.x += body_sway;
                }
                MonsterPartRole::Chest => {
                    transform.translation.y += body_bob + breathing * 1.2;
                }
                MonsterPartRole::Ribcage => {
                    transform.translation.y += body_bob * 0.8;
                }
                MonsterPartRole::Head => {
                    transform.translation.y += body_bob * 0.5 + head_bob;
                }
                MonsterPartRole::Arm => {
                    transform.translation.z += arm_swing;
                    transform.translation.y += body_bob * 0.3;
                }
                MonsterPartRole::Leg => {
                    transform.translation.y += body_bob * 0.6;
                }
                MonsterPartRole::Spine => {
                    transform.translation.y += body_bob * 0.4;
                    transform.translation.x += body_sway * 0.5;
                }
            }
        }
        world.mark_local_transform_dirty(part_entity);
    }

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
    let monster_parts: Vec<freecs::Entity> = game_world.query_entities(MONSTER_PART).collect();

    for game_entity in &monster_parts {
        if let Some(engine_entity) = game_world.get_engine_entity(*game_entity) {
            world.queue_despawn_entity(engine_entity.0);
        }
    }
    game_world.despawn_entities(&monster_parts);

    game_world.resources.monster.active = false;
    game_world.resources.monster.chasing = false;

    if let Some(monster_audio_entity) = game_world.resources.monster_audio_entity
        && let Some(source) = world.core.get_audio_source_mut(monster_audio_entity)
    {
        source.playing = false;
    }
}
