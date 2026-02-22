use crate::state::{CutscenePhase, HorrorDemo};
use crate::systems::doors::slam_door_closed;
use nightshade::ecs::material::resources::material_registry_insert;
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

pub fn start_cutscene(demo: &mut HorrorDemo, world: &mut World) {
    demo.cutscene.active = true;
    demo.cutscene.phase = CutscenePhase::LookAtWall;
    demo.cutscene.timer = 0.0;

    demo.cutscene.saved_base_rotation = demo.lean_state.base_rotation;

    world.resources.graphics.letterbox_target = 1.0;

    if let Some(ambient_entity) = demo.ambient_audio_entity
        && let Some(source) = world.get_audio_source_mut(ambient_entity)
    {
        source.playing = false;
    }

    let wall_break_pos = nalgebra_glm::vec3(-4.5, 1.5, -16.0);
    demo.cutscene.wall_break_position = wall_break_pos;

    if let Some(camera_entity) = demo.camera_entity
        && let Some(camera_transform) = world.get_global_transform(camera_entity)
    {
        let camera_pos = camera_transform.translation();
        let direction = wall_break_pos - camera_pos;
        let direction_normalized = nalgebra_glm::normalize(&direction);

        let yaw = (-direction_normalized.x).atan2(-direction_normalized.z);
        let pitch = direction_normalized.y.asin();

        let yaw_quat = nalgebra_glm::quat_angle_axis(yaw, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
        let pitch_quat = nalgebra_glm::quat_angle_axis(-pitch, &nalgebra_glm::vec3(1.0, 0.0, 0.0));
        demo.cutscene.target_rotation = yaw_quat * pitch_quat;
    }
}

pub fn cutscene_system(demo: &mut HorrorDemo, world: &mut World) {
    if !demo.cutscene.active {
        return;
    }

    let dt = world.resources.window.timing.delta_time;
    demo.cutscene.timer += dt;

    match demo.cutscene.phase {
        CutscenePhase::None => {}
        CutscenePhase::LookAtWall => {
            let progress = (demo.cutscene.timer / CUTSCENE_LOOK_DURATION).min(1.0);
            let smoothed = smooth_step(progress);

            demo.lean_state.base_rotation = nalgebra_glm::quat_slerp(
                &demo.cutscene.saved_base_rotation,
                &demo.cutscene.target_rotation,
                smoothed,
            );

            if demo.cutscene.timer >= CUTSCENE_LOOK_DURATION {
                demo.cutscene.phase = CutscenePhase::WallBreaks;
                demo.cutscene.timer = 0.0;
                spawn_wall_destruction(demo, world);

                if let Some(rubble_entity) = demo.rubble_audio_entity
                    && let Some(source) = world.get_audio_source_mut(rubble_entity)
                {
                    source.playing = true;
                }
            }
        }
        CutscenePhase::WallBreaks => {
            if demo.cutscene.timer >= CUTSCENE_WALL_BREAK_DURATION {
                demo.cutscene.phase = CutscenePhase::MonsterEmerges;
                demo.cutscene.timer = 0.0;
                spawn_monster(demo, world);
            }
        }
        CutscenePhase::MonsterEmerges => {
            if demo.cutscene.timer >= CUTSCENE_EMERGE_DURATION {
                demo.cutscene.phase = CutscenePhase::ReturnControl;
                demo.cutscene.timer = 0.0;
                demo.monster.active = true;

                if let Some(monster_audio_entity) = demo.monster_audio_entity
                    && let Some(source) = world.get_audio_source_mut(monster_audio_entity)
                {
                    source.playing = true;
                }
            }
        }
        CutscenePhase::ReturnControl => {
            world.resources.graphics.letterbox_target = 0.0;
            if demo.cutscene.timer >= CUTSCENE_RETURN_DURATION {
                demo.cutscene.active = false;
                demo.cutscene.phase = CutscenePhase::None;
            }
        }
        CutscenePhase::DoorSlam => {
            if demo.cutscene.timer >= CUTSCENE_DOOR_SLAM_DURATION {
                demo.cutscene.phase = CutscenePhase::LookAtDoor;
                demo.cutscene.timer = 0.0;

                if let Some(camera_entity) = demo.camera_entity
                    && let Some(exit_door) = demo.doors.get(demo.exit_door_index)
                    && let Some(camera_transform) = world.get_global_transform(camera_entity)
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
                    demo.cutscene.target_rotation = yaw_quat * pitch_quat;
                }
            }
        }
        CutscenePhase::LookAtDoor => {
            let progress = (demo.cutscene.timer / CUTSCENE_LOOK_AT_DOOR_DURATION).min(1.0);
            let smoothed = smooth_step(progress);

            demo.lean_state.base_rotation = nalgebra_glm::quat_slerp(
                &demo.cutscene.saved_base_rotation,
                &demo.cutscene.target_rotation,
                smoothed,
            );

            if demo.cutscene.timer >= CUTSCENE_LOOK_AT_DOOR_DURATION {
                demo.fade_target = 1.0;
                demo.cutscene.active = false;
                demo.cutscene.phase = CutscenePhase::None;
                demo.game_won = true;
            }
        }
    }
}

fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn spawn_wall_destruction(demo: &mut HorrorDemo, world: &mut World) {
    let break_pos = demo.cutscene.wall_break_position;

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

        if let Some(rb_comp) = world.get_rigid_body(entity)
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

        demo.physics_objects.push(entity);
    }

    spawn_dust_particles(demo, world, break_pos);
}

fn spawn_dust_particles(demo: &mut HorrorDemo, world: &mut World, position: Vec3) {
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

        if let Some(rb_comp) = world.get_rigid_body(entity)
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

        demo.physics_objects.push(entity);
    }
}

fn spawn_monster(demo: &mut HorrorDemo, world: &mut World) {
    let break_pos = demo.cutscene.wall_break_position;
    let monster_pos = nalgebra_glm::vec3(break_pos.x - 0.5, 0.0, break_pos.z);

    let flesh_material = create_textured_material(nalgebra_glm::vec3(0.45, 0.08, 0.08), 0.85, 0.15);
    let dark_flesh = create_textured_material(nalgebra_glm::vec3(0.25, 0.03, 0.03), 0.9, 0.1);
    let bone_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.65, 0.55), 0.6, 0.2);
    let vein_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.0, 0.0), 0.5, 0.4);
    let eye_material = create_emissive_material(nalgebra_glm::vec3(1.0, 0.2, 0.0), 5.0);
    let inner_glow = create_emissive_material(nalgebra_glm::vec3(0.8, 0.1, 0.0), 2.0);

    let torso = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 1.4, -0.1),
        nalgebra_glm::vec3(0.7, 0.9, 0.5),
        "Cube",
        flesh_material.clone(),
    );
    demo.monster.body_parts.push(torso);

    let chest_detail = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 1.5, 0.2),
        nalgebra_glm::vec3(0.5, 0.6, 0.15),
        "Cube",
        dark_flesh.clone(),
    );
    demo.monster.body_parts.push(chest_detail);

    let ribcage_glow = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 1.35, 0.15),
        nalgebra_glm::vec3(0.3, 0.25, 0.1),
        "Sphere",
        inner_glow.clone(),
    );
    demo.monster.body_parts.push(ribcage_glow);

    for rib_index in 0..4 {
        let y_offset = 1.2 + rib_index as f32 * 0.15;
        let rib_left = spawn_monster_part(
            world,
            monster_pos + nalgebra_glm::vec3(-0.3, y_offset, 0.1),
            nalgebra_glm::vec3(0.12, 0.04, 0.2),
            "Cube",
            bone_material.clone(),
        );
        demo.monster.body_parts.push(rib_left);
        let rib_right = spawn_monster_part(
            world,
            monster_pos + nalgebra_glm::vec3(0.3, y_offset, 0.1),
            nalgebra_glm::vec3(0.12, 0.04, 0.2),
            "Cube",
            bone_material.clone(),
        );
        demo.monster.body_parts.push(rib_right);
    }

    let neck = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 1.95, 0.0),
        nalgebra_glm::vec3(0.2, 0.2, 0.2),
        "Cylinder",
        dark_flesh.clone(),
    );
    demo.monster.body_parts.push(neck);

    let head = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 2.2, 0.15),
        nalgebra_glm::vec3(0.35, 0.4, 0.4),
        "Cube",
        flesh_material.clone(),
    );
    demo.monster.body_parts.push(head);

    let skull_ridge = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 2.45, 0.0),
        nalgebra_glm::vec3(0.25, 0.12, 0.35),
        "Cube",
        bone_material.clone(),
    );
    demo.monster.body_parts.push(skull_ridge);

    let jaw = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 2.0, 0.3),
        nalgebra_glm::vec3(0.28, 0.15, 0.2),
        "Cube",
        dark_flesh.clone(),
    );
    demo.monster.body_parts.push(jaw);

    let left_eye = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(-0.12, 2.25, 0.35),
        nalgebra_glm::vec3(0.1, 0.1, 0.1),
        "Sphere",
        eye_material.clone(),
    );
    demo.monster.body_parts.push(left_eye);

    let right_eye = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.12, 2.25, 0.35),
        nalgebra_glm::vec3(0.1, 0.1, 0.1),
        "Sphere",
        eye_material.clone(),
    );
    demo.monster.body_parts.push(right_eye);

    let third_eye = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 2.35, 0.38),
        nalgebra_glm::vec3(0.06, 0.06, 0.06),
        "Sphere",
        eye_material,
    );
    demo.monster.body_parts.push(third_eye);

    let left_shoulder = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(-0.5, 1.7, 0.0),
        nalgebra_glm::vec3(0.25, 0.2, 0.25),
        "Sphere",
        flesh_material.clone(),
    );
    demo.monster.body_parts.push(left_shoulder);

    let left_upper_arm = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(-0.65, 1.4, 0.1),
        nalgebra_glm::vec3(0.12, 0.5, 0.12),
        "Cube",
        flesh_material.clone(),
    );
    demo.monster.body_parts.push(left_upper_arm);

    let left_lower_arm = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(-0.7, 0.95, 0.2),
        nalgebra_glm::vec3(0.1, 0.45, 0.1),
        "Cube",
        dark_flesh.clone(),
    );
    demo.monster.body_parts.push(left_lower_arm);

    let left_hand = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(-0.72, 0.6, 0.25),
        nalgebra_glm::vec3(0.15, 0.2, 0.08),
        "Cube",
        dark_flesh.clone(),
    );
    demo.monster.body_parts.push(left_hand);

    for finger_index in 0..4 {
        let finger = spawn_monster_part(
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
        );
        demo.monster.body_parts.push(finger);
    }

    let right_shoulder = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.5, 1.7, 0.0),
        nalgebra_glm::vec3(0.25, 0.2, 0.25),
        "Sphere",
        flesh_material.clone(),
    );
    demo.monster.body_parts.push(right_shoulder);

    let right_upper_arm = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.65, 1.4, 0.1),
        nalgebra_glm::vec3(0.12, 0.5, 0.12),
        "Cube",
        flesh_material.clone(),
    );
    demo.monster.body_parts.push(right_upper_arm);

    let right_lower_arm = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.7, 0.95, 0.2),
        nalgebra_glm::vec3(0.1, 0.45, 0.1),
        "Cube",
        dark_flesh.clone(),
    );
    demo.monster.body_parts.push(right_lower_arm);

    let right_hand = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.72, 0.6, 0.25),
        nalgebra_glm::vec3(0.15, 0.2, 0.08),
        "Cube",
        dark_flesh.clone(),
    );
    demo.monster.body_parts.push(right_hand);

    for finger_index in 0..4 {
        let finger = spawn_monster_part(
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
        );
        demo.monster.body_parts.push(finger);
    }

    let extra_arm_shoulder = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(-0.35, 1.3, -0.2),
        nalgebra_glm::vec3(0.15, 0.12, 0.15),
        "Sphere",
        dark_flesh.clone(),
    );
    demo.monster.body_parts.push(extra_arm_shoulder);

    let extra_arm = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(-0.45, 1.0, -0.15),
        nalgebra_glm::vec3(0.08, 0.4, 0.08),
        "Cube",
        dark_flesh.clone(),
    );
    demo.monster.body_parts.push(extra_arm);

    let pelvis = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.0, 0.85, -0.05),
        nalgebra_glm::vec3(0.55, 0.35, 0.4),
        "Cube",
        flesh_material.clone(),
    );
    demo.monster.body_parts.push(pelvis);

    let left_thigh = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(-0.22, 0.5, 0.0),
        nalgebra_glm::vec3(0.18, 0.45, 0.18),
        "Cube",
        flesh_material.clone(),
    );
    demo.monster.body_parts.push(left_thigh);

    let left_shin = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(-0.22, 0.15, 0.05),
        nalgebra_glm::vec3(0.12, 0.35, 0.12),
        "Cube",
        dark_flesh.clone(),
    );
    demo.monster.body_parts.push(left_shin);

    let right_thigh = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.22, 0.5, 0.0),
        nalgebra_glm::vec3(0.18, 0.45, 0.18),
        "Cube",
        flesh_material.clone(),
    );
    demo.monster.body_parts.push(right_thigh);

    let right_shin = spawn_monster_part(
        world,
        monster_pos + nalgebra_glm::vec3(0.22, 0.15, 0.05),
        nalgebra_glm::vec3(0.12, 0.35, 0.12),
        "Cube",
        dark_flesh.clone(),
    );
    demo.monster.body_parts.push(right_shin);

    for spine_index in 0..6 {
        let size = 0.08 + (spine_index as f32 * 0.015);
        let spine = spawn_monster_part(
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
        );
        demo.monster.body_parts.push(spine);
    }

    for vein_index in 0..5 {
        let angle = vein_index as f32 * 1.2;
        let vein = spawn_monster_part(
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
        );
        demo.monster.body_parts.push(vein);
    }

    for tendril_index in 0..3 {
        let x_offset = (tendril_index as f32 - 1.0) * 0.2;
        let tendril = spawn_monster_part(
            world,
            monster_pos + nalgebra_glm::vec3(x_offset, 0.6, -0.35),
            nalgebra_glm::vec3(0.04, 0.5, 0.04),
            "Cylinder",
            dark_flesh.clone(),
        );
        demo.monster.body_parts.push(tendril);
    }

    demo.monster.entity = Some(torso);
    demo.monster.speed = 2.0;
}

fn spawn_monster_part(
    world: &mut World,
    position: Vec3,
    scale: Vec3,
    mesh_name: &str,
    material: Material,
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

    if let Some(name) = world.get_name_mut(entity) {
        name.0 = "Monster Part".to_string();
    }

    if let Some(transform) = world.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = scale;
    }

    if let Some(mesh) = world.get_render_mesh_mut(entity) {
        mesh.name = mesh_name.to_string();
    }

    let material_name = format!("MonsterPart_{}", entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        material,
    );
    if let Some(&index) = world
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
            .add_reference(index);
    }
    world.set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(bv) = world.get_bounding_volume_mut(entity) {
        *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type(mesh_name);
    }

    world.resources.mesh_render_state.mark_entity_added(entity);

    entity
}

pub fn monster_chase_system(demo: &mut HorrorDemo, world: &mut World) {
    if !demo.monster.active {
        return;
    }

    let Some(player_entity) = demo.player_entity else {
        return;
    };

    let Some(monster_entity) = demo.monster.entity else {
        return;
    };

    let dt = world.resources.window.timing.delta_time;

    let player_pos = world
        .get_global_transform(player_entity)
        .map(|t| t.translation())
        .unwrap_or(Vec3::zeros());

    if player_pos.z < EXIT_AREA_Z_THRESHOLD && !demo.game_won {
        start_exit_cutscene(demo, world);
        despawn_monster(demo, world);
        return;
    }

    if !demo.monster.chasing {
        demo.monster.pause_timer += dt;
        if demo.monster.pause_timer >= MONSTER_PAUSE_DURATION {
            demo.monster.chasing = true;
        }
        return;
    }

    let monster_pos = world
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
    let movement = normalized_dir * demo.monster.speed * dt;

    let angle = (-normalized_dir.x).atan2(-normalized_dir.z);
    let target_rotation = nalgebra_glm::quat_angle_axis(angle, &nalgebra_glm::vec3(0.0, 1.0, 0.0));

    let total_time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

    let walk_cycle = total_time * 3.5;
    let body_bob = (walk_cycle * 2.0).sin() * 0.015;
    let body_sway = (walk_cycle).sin() * 0.008;
    let arm_swing = (walk_cycle).sin() * 0.04;
    let head_bob = (walk_cycle * 2.0).sin() * 0.008;
    let breathing = (total_time * 1.5).sin() * 0.006;

    for (part_index, &part_entity) in demo.monster.body_parts.iter().enumerate() {
        if let Some(transform) = world.get_local_transform_mut(part_entity) {
            transform.translation += movement;

            let current_rotation = transform.rotation;
            transform.rotation =
                nalgebra_glm::quat_slerp(&current_rotation, &target_rotation, dt * 8.0);

            match part_index {
                0 => {
                    transform.translation.y += body_bob + breathing;
                    transform.translation.x += body_sway;
                }
                1..=2 => {
                    transform.translation.y += body_bob + breathing * 1.2;
                }
                3..=10 => {
                    transform.translation.y += body_bob * 0.8;
                }
                11..=13 => {
                    transform.translation.y += body_bob * 0.5 + head_bob;
                }
                14..=18 => {
                    transform.translation.y += head_bob * 1.2;
                }
                19..=24 => {
                    transform.translation.z += arm_swing;
                    transform.translation.y += body_bob * 0.3;
                }
                25..=30 => {
                    transform.translation.z -= arm_swing;
                    transform.translation.y += body_bob * 0.3;
                }
                31..=32 => {
                    transform.translation.z += arm_swing * 0.5;
                }
                33..=38 => {
                    transform.translation.y += body_bob * 0.6;
                }
                _ => {
                    transform.translation.y += body_bob * 0.4;
                    transform.translation.x += body_sway * 0.5;
                }
            }
        }
        world.mark_local_transform_dirty(part_entity);
    }

    if distance < 1.2 && !demo.game_won {
        demo.game_won = true;
    }
}

fn start_exit_cutscene(demo: &mut HorrorDemo, world: &mut World) {
    demo.cutscene.active = true;
    demo.cutscene.phase = CutscenePhase::DoorSlam;
    demo.cutscene.timer = 0.0;
    demo.cutscene.saved_base_rotation = demo.lean_state.base_rotation;

    world.resources.graphics.letterbox_target = 1.0;

    slam_door_closed(demo, world, demo.exit_door_index);
}

fn despawn_monster(demo: &mut HorrorDemo, world: &mut World) {
    for &part_entity in &demo.monster.body_parts {
        world.queue_despawn_entity(part_entity);
    }
    demo.monster.body_parts.clear();
    demo.monster.entity = None;
    demo.monster.active = false;
    demo.monster.chasing = false;

    if let Some(monster_audio_entity) = demo.monster_audio_entity
        && let Some(source) = world.get_audio_source_mut(monster_audio_entity)
    {
        source.playing = false;
    }
}
