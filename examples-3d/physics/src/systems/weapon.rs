use crate::ecs::{GameWorld, InputMode, TARGET};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::ecs::transform::components::Parent;
use nightshade::prelude::*;

const HIP_POSITION: Vec3 = Vec3::new(0.15, -0.10, -0.25);
const ADS_POSITION: Vec3 = Vec3::new(0.0, -0.055, -0.18);
const ADS_LERP_SPEED: f32 = 12.0;
const AUTO_AIM_RADIUS: f32 = 8.0;
const AUTO_AIM_CONE: f32 = 0.15;
const AUTO_AIM_STRENGTH: f32 = 0.03;

pub fn spawn_weapon_part(
    world: &mut World,
    parent: Entity,
    position: Vec3,
    scale: Vec3,
    mesh_name: &str,
    material: nightshade::ecs::material::components::Material,
) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT
            | VISIBILITY
            | nightshade::ecs::world::RENDER_LAYER,
        1,
    )[0];

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = scale;
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = mesh_name.to_string();
    }

    let material_name = format!("WeaponPart_{}", entity.id);
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
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
        *bounding_volume = BoundingVolume::from_mesh_type(mesh_name);
    }

    if let Some(parent_component) = world.core.get_parent_mut(entity) {
        *parent_component = Parent(Some(parent));
    }

    if let Some(render_layer) = world.core.get_render_layer_mut(entity) {
        render_layer.0 = nightshade::ecs::render_layer::components::RenderLayer::OVERLAY;
    }

    world.resources.mesh_render_state.mark_entity_added(entity);

    entity
}

pub fn spawn_weapon(world: &mut World, camera_entity: Entity) -> Entity {
    let root = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | PARENT,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(root) {
        name.0 = "Weapon".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(root) {
        transform.translation = HIP_POSITION;
    }

    if let Some(parent) = world.core.get_parent_mut(root) {
        *parent = Parent(Some(camera_entity));
    }

    let body_color = create_textured_material(nalgebra_glm::vec3(0.18, 0.18, 0.20), 0.45, 0.85);
    let barrel_color = create_textured_material(nalgebra_glm::vec3(0.12, 0.12, 0.14), 0.3, 0.9);
    let grip_color = create_textured_material(nalgebra_glm::vec3(0.08, 0.08, 0.06), 0.8, 0.1);
    let accent_color = create_textured_material(nalgebra_glm::vec3(0.25, 0.25, 0.28), 0.35, 0.8);

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, 0.0, 0.0),
        nalgebra_glm::vec3(0.025, 0.035, 0.10),
        "Cube",
        body_color.clone(),
    );

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, 0.005, -0.09),
        nalgebra_glm::vec3(0.012, 0.012, 0.08),
        "Cube",
        barrel_color.clone(),
    );

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, 0.005, -0.135),
        nalgebra_glm::vec3(0.016, 0.016, 0.01),
        "Cube",
        accent_color.clone(),
    );

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, -0.045, 0.02),
        nalgebra_glm::vec3(0.02, 0.055, 0.025),
        "Cube",
        grip_color,
    );

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, 0.023, -0.01),
        nalgebra_glm::vec3(0.012, 0.006, 0.06),
        "Cube",
        accent_color,
    );

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, -0.018, 0.0),
        nalgebra_glm::vec3(0.018, 0.008, 0.025),
        "Cube",
        body_color,
    );

    spawn_weapon_part(
        world,
        root,
        nalgebra_glm::vec3(0.0, -0.006, -0.055),
        nalgebra_glm::vec3(0.004, 0.004, 0.015),
        "Cube",
        barrel_color,
    );

    root
}

pub fn update_weapon_sway(game_world: &mut GameWorld, world: &mut World) {
    let Some(weapon_entity) = game_world.resources.weapon_entity else {
        return;
    };
    let Some(camera_entity) = game_world.resources.camera_entity else {
        return;
    };

    let ads_held = world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::MIDDLE_CLICKED)
        || query_active_gamepad(world)
            .is_some_and(|gamepad| gamepad.is_pressed(gilrs::Button::West));

    game_world.resources.aiming_down_sights = ads_held;

    let delta_time = world.resources.window.timing.delta_time;
    let target_blend = if ads_held { 1.0 } else { 0.0 };
    let blend_diff = target_blend - game_world.resources.aim_blend;
    game_world.resources.aim_blend += blend_diff * (ADS_LERP_SPEED * delta_time).min(1.0);

    let forward = world
        .core
        .get_local_transform(camera_entity)
        .map(|transform| transform.forward_vector())
        .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, -1.0));

    let current_yaw = forward.x.atan2(-forward.z);
    let current_pitch = forward.y.asin();

    let yaw_delta = current_yaw - game_world.resources.weapon_previous_yaw;
    let pitch_delta = current_pitch - game_world.resources.weapon_previous_pitch;
    game_world.resources.weapon_previous_yaw = current_yaw;
    game_world.resources.weapon_previous_pitch = current_pitch;

    let sway_strength = 0.6 * (1.0 - game_world.resources.aim_blend * 0.8);
    game_world.resources.weapon_sway.x -= yaw_delta * sway_strength;
    game_world.resources.weapon_sway.y -= pitch_delta * sway_strength;

    let max_sway = 0.08 * (1.0 - game_world.resources.aim_blend * 0.7);
    game_world.resources.weapon_sway.x = game_world.resources.weapon_sway.x.clamp(-max_sway, max_sway);
    game_world.resources.weapon_sway.y = game_world.resources.weapon_sway.y.clamp(-max_sway, max_sway);

    let recovery_speed = 8.0 + game_world.resources.aim_blend * 8.0;
    let decay = (-recovery_speed * delta_time).exp();
    game_world.resources.weapon_sway.x *= decay;
    game_world.resources.weapon_sway.y *= decay;

    let blend = game_world.resources.aim_blend;
    let base_position = nalgebra_glm::lerp(&HIP_POSITION, &ADS_POSITION, blend);

    if let Some(transform) = world.core.get_local_transform_mut(weapon_entity) {
        transform.translation = nalgebra_glm::vec3(
            base_position.x + game_world.resources.weapon_sway.x,
            base_position.y + game_world.resources.weapon_sway.y,
            base_position.z,
        );
    }
    mark_local_transform_dirty(world, weapon_entity);

    if game_world.resources.input_mode == InputMode::Gamepad && ads_held {
        apply_auto_aim(game_world, world);
    }
}

fn apply_auto_aim(game_world: &mut GameWorld, world: &mut World) {
    let Some(camera_entity) = game_world.resources.camera_entity else {
        return;
    };
    let Some(camera_transform) = world.core.get_global_transform(camera_entity).cloned() else {
        return;
    };

    let camera_position = camera_transform.translation();
    let camera_forward = camera_transform.forward_vector();

    let target_entities: Vec<freecs::Entity> = game_world.query_entities(TARGET).collect();

    let mut closest_dot = -1.0_f32;
    let mut closest_direction = None;

    for game_entity in &target_entities {
        let Some(target) = game_world.get_target(*game_entity) else {
            continue;
        };
        if target.popped {
            continue;
        }

        let target_position = world
            .core
            .get_global_transform(target.entity)
            .map(|transform| transform.translation())
            .unwrap_or(target.position);

        let to_target = target_position - camera_position;
        let distance = nalgebra_glm::length(&to_target);

        if !(0.5..=AUTO_AIM_RADIUS).contains(&distance) {
            continue;
        }

        let direction = nalgebra_glm::normalize(&to_target);
        let dot = nalgebra_glm::dot(&camera_forward, &direction);

        if dot > (1.0 - AUTO_AIM_CONE) && dot > closest_dot {
            closest_dot = dot;
            closest_direction = Some(direction);
        }
    }

    if let Some(direction) = closest_direction {
        let current_rotation = world
            .core
            .get_local_transform(camera_entity)
            .map(|transform| transform.rotation)
            .unwrap_or(nalgebra_glm::quat_identity());

        let target_yaw = direction.x.atan2(-direction.z);
        let target_pitch = direction.y.asin();

        let current_forward = nalgebra_glm::quat_rotate_vec3(
            &current_rotation,
            &nalgebra_glm::vec3(0.0, 0.0, -1.0),
        );
        let current_yaw = current_forward.x.atan2(-current_forward.z);
        let current_pitch = current_forward.y.asin();

        let yaw_diff = target_yaw - current_yaw;
        let pitch_diff = target_pitch - current_pitch;

        let nudge_yaw = yaw_diff * AUTO_AIM_STRENGTH;
        let nudge_pitch = pitch_diff * AUTO_AIM_STRENGTH;

        let yaw_rotation =
            nalgebra_glm::quat_angle_axis(-nudge_yaw, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
        let new_rotation = yaw_rotation * current_rotation;

        let pitch_rotation =
            nalgebra_glm::quat_angle_axis(-nudge_pitch, &nalgebra_glm::vec3(1.0, 0.0, 0.0));
        let new_rotation = new_rotation * pitch_rotation;

        if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
            transform.rotation = new_rotation;
        }
        mark_local_transform_dirty(world, camera_entity);

        #[cfg(not(feature = "openxr"))]
        {
            game_world.resources.lean.base_rotation = new_rotation;
        }
    }
}
