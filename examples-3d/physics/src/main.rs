use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::physics::joints::{
    FixedJoint, JointAxisDirection, JointLimits, PrismaticJoint, RevoluteJoint, RopeJoint,
    SphericalJoint, SpringJoint, create_fixed_joint, create_prismatic_joint, create_revolute_joint,
    create_rope_joint, create_spherical_joint, create_spring_joint,
};
use nightshade::ecs::physics::*;
use nightshade::ecs::picking::{PickingOptions, PickingResult, pick_entities};
use nightshade::ecs::text::commands::spawn_ui_text;
use nightshade::ecs::text::components::{TextAlignment, TextProperties, VerticalAlignment};
use nightshade::ecs::transform::components::Parent;
use nightshade::ecs::world::commands::spawn_3d_text_with_properties;
use nightshade::ecs::world::resources::MouseState;
use nightshade::ecs::world::{
    BOUNDING_VOLUME, CASTS_SHADOW, GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY,
    MATERIAL_REF, NAME, PARENT, RENDER_MESH, VISIBILITY,
};
use nightshade::prelude::*;

stateless::statemachine! {
    name: Movement,
    transitions: {
        *Grounded + Jump = Airborne,
        Grounded + Dash = GroundDash,
        GroundDash + Land = Grounded,
        GroundDash + BecomeAirborne = Airborne,
        Airborne + DoubleJump = DoubleJumped,
        Airborne + Dash = AirDash,
        DoubleJumped + Dash = AirDash,
        AirDash + DashEnd = Falling,
        Falling + Land = Grounded,
        Airborne + Land = Grounded,
        DoubleJumped + Land = Grounded,
    }
}

const DASH_INITIAL_SPEED: f32 = 40.0;
const DASH_DURATION: f32 = 0.4;
const DASH_DECAY_RATE: f32 = 4.0;
const DOUBLE_JUMP_IMPULSE: f32 = 4.5;
const MAX_DASH_CHARGES: u32 = 2;
const DASH_COOLDOWN: f32 = 1.5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(PhysicsDemo::default())
}

const GRAB_RANGE: f32 = 3.0;
const INTERACT_RANGE: f32 = 2.5;
const INTERACT_CONE_RADIUS: f32 = 40.0;
const MIN_GRAB_DISTANCE: f32 = 0.8;
const MAX_GRAB_DISTANCE: f32 = 3.0;
const SCROLL_DISTANCE_SPEED: f32 = 0.3;
const THROW_STRENGTH: f32 = 12.0;
const GRAB_STIFFNESS: f32 = 150.0;
const GRAB_DAMPING_RATIO: f32 = 1.0;
const MAX_GRAB_FORCE: f32 = 80.0;
const ANGULAR_DAMPING: f32 = 0.95;
const STANDING_CAMERA_HEIGHT: f32 = 0.8;
const CROUCHING_CAMERA_HEIGHT: f32 = 0.3;
const LEAN_AMOUNT: f32 = 0.4;
const LEAN_ANGLE: f32 = 0.15;
const LEAN_SPEED: f32 = 8.0;
const MAX_SHOT_BAUBLES: usize = 200;
const BAUBLE_LIFETIME_MS: u64 = 30000;
const BAUBLE_SHRINK_DURATION_MS: u64 = 2000;

#[derive(Default)]
struct PhysicsDemo {
    player_entity: Option<Entity>,
    camera_entity: Option<Entity>,
    physics_objects: Vec<Entity>,
    doors: Vec<DoorState>,
    drawers: Vec<DrawerState>,
    levers: Vec<LeverState>,
    wheels: Vec<WheelState>,
    buttons: Vec<ButtonState>,
    baubles: Vec<BaubleSpawnData>,
    bauble_table_center: Vec3,
    bauble_table_top_y: f32,
    lantern_entity: Option<Entity>,
    lantern_light_entity: Option<Entity>,
    shot_baubles: Vec<ShotBauble>,
    interaction: InteractionState,
    interaction_prompt_entity: Option<Entity>,
    interaction_prompt_text_index: Option<usize>,
    lean_state: LeanState,
    input_mode: InputMode,
    input_mode_text_entity: Option<Entity>,
    input_mode_text_index: Option<usize>,
    show_physics_debug: bool,
    key4_was_pressed: bool,
    notes: Vec<NoteState>,
    reading_note: Option<usize>,
    note_close_key_released: bool,
    prismatic_sliders: Vec<PrismaticSliderState>,
    spherical_joint_visuals: Vec<SphericalJointVisual>,
    rope_joint_visuals: Vec<RopeJointVisual>,
    spring_joint_visuals: Vec<SpringJointVisual>,
    coulomb_friction_joints: Vec<CoulombFrictionJointState>,
    velocity_friction_joints: Vec<VelocityFrictionJointState>,
    movement_state: MovementState,
    dash_timer: f32,
    dash_direction: Vec3,
    dash_charges: u32,
    dash_cooldown_timer: f32,
    dash_button_was_pressed: bool,
    jump_button_was_pressed: bool,
    dash_hud_entity: Option<Entity>,
    dash_hud_state_text_entity: Option<Entity>,
    dash_hud_charge_entities: Vec<Entity>,
    weapon_entity: Option<Entity>,
    weapon_sway: nalgebra_glm::Vec2,
    weapon_previous_yaw: f32,
    weapon_previous_pitch: f32,
    flashlight_entity: Option<Entity>,
    flashlight_on: bool,
    flashlight_key_was_pressed: bool,
    crosshair_entity: Option<Entity>,
    crosshair_arms: Vec<Entity>,
    note_overlay_entity: Option<Entity>,
    note_title_entity: Option<Entity>,
    note_content_entity: Option<Entity>,
    last_shown_note: Option<usize>,
    #[cfg(feature = "openxr")]
    left_hand_cube: Option<Entity>,
    #[cfg(feature = "openxr")]
    right_hand_cube: Option<Entity>,
    #[cfg(feature = "openxr")]
    bauble_gun_entities: Vec<Entity>,
    #[cfg(feature = "openxr")]
    xr_rt_was_pressed: bool,
    #[cfg(feature = "openxr")]
    xr_lt_was_pressed: bool,
}

struct RoomConfig {
    center: Vec3,
    width: f32,
    depth: f32,
    height: f32,
    wall_thickness: f32,
    doorway_width: f32,
    doorway_height: f32,
    wall_material: nightshade::ecs::material::components::Material,
    ceiling_material: nightshade::ecs::material::components::Material,
}

struct PrismaticSliderState {
    entity: Entity,
    time_accumulator: f32,
}

struct SphericalJointVisual {
    anchor_entity: Entity,
    ball_entity: Entity,
    rod_entity: Entity,
}

struct RopeJointVisual {
    anchor_entity: Entity,
    ball_entity: Entity,
    rope_entity: Entity,
}

struct SpringJointVisual {
    anchor_entity: Entity,
    object_entity: Entity,
    spring_entities: Vec<Entity>,
}

struct CoulombFrictionJointState {
    arm_entity: Entity,
    friction_torque: f32,
}

struct VelocityFrictionJointState {
    arm_entity: Entity,
    damping_factor: f32,
    initialized: bool,
}

struct LeanState {
    current_lean: f32,
    target_lean: f32,
    base_rotation: nalgebra_glm::Quat,
}

impl Default for LeanState {
    fn default() -> Self {
        Self {
            current_lean: 0.0,
            target_lean: 0.0,
            base_rotation: nalgebra_glm::quat_identity(),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    MouseKeyboard,
    Gamepad,
    #[cfg(feature = "openxr")]
    Xr,
}

#[derive(Default)]
struct InteractionState {
    grabbed_entity: Option<Entity>,
    grab_distance: f32,
    manipulated_door_index: Option<usize>,
    manipulated_drawer_index: Option<usize>,
    manipulated_lever_index: Option<usize>,
    manipulated_wheel_index: Option<usize>,
    manipulated_button_index: Option<usize>,
    gamepad_rt_was_pressed: bool,
    shoot_was_pressed: bool,
    shoot_hold_start_ms: Option<u64>,
    last_rapid_fire_ms: u64,
    require_interact_release: bool,
}

struct DoorState {
    entity: Entity,
    rigid_body_handle: rapier3d::prelude::RigidBodyHandle,
    hinge_position: Vec3,
    door_half_width: f32,
    current_angle: f32,
    angular_velocity: f32,
    min_angle: f32,
    max_angle: f32,
}

struct DrawerState {
    entity: Entity,
    front_entity: Entity,
    rigid_body_handle: rapier3d::prelude::RigidBodyHandle,
    closed_position: Vec3,
    current_offset: f32,
    velocity: f32,
    max_offset: f32,
}

struct LeverState {
    pivot_entity: Entity,
    collider_entity: Entity,
    collider_rb_handle: rapier3d::prelude::RigidBodyHandle,
    pivot_position: Vec3,
    arm_half_length: f32,
    current_angle: f32,
    angular_velocity: f32,
    min_angle: f32,
    max_angle: f32,
}

struct WheelState {
    entity: Entity,
    spoke_entities: Vec<Entity>,
    rigid_body_handle: rapier3d::prelude::RigidBodyHandle,
    center_position: Vec3,
    current_angle: f32,
    angular_velocity: f32,
}

struct ButtonState {
    entity: Entity,
    base_position: Vec3,
    current_press: f32,
    is_pressed: bool,
    action: ButtonAction,
}

#[derive(Clone)]
enum ButtonAction {
    RecallBaubles,
}

struct BaubleSpawnData {
    entity: Entity,
    spawn_position: Vec3,
}

struct ShotBauble {
    entity: Entity,
    spawn_time_ms: u64,
    original_scale: f32,
    landed: bool,
}

struct NoteState {
    entity: Entity,
    title: String,
    content: String,
}

impl State for PhysicsDemo {
    fn title(&self) -> &str {
        "Physics Interaction Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.use_fullscreen = true;

        self.show_physics_debug = false;
        self.dash_charges = MAX_DASH_CHARGES;
        world.resources.physics.debug_draw = self.show_physics_debug;

        #[cfg(feature = "openxr")]
        {
            world.resources.xr.locomotion_enabled = false;
            self.input_mode = InputMode::Xr;
        }

        spawn_sun_overhead(world);

        let player_position = nalgebra_glm::vec3(0.0, 1.2, 8.0);
        let (player_entity, camera_entity) = spawn_first_person_player(world, player_position);

        if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
            transform.translation.y = 0.8;
        }

        if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
            controller.max_speed = 2.5;
            controller.sprint_speed_multiplier = 2.0;
        }

        self.player_entity = Some(player_entity);
        self.camera_entity = Some(camera_entity);

        world.resources.graphics.render_layer_world_enabled = true;
        world.resources.graphics.render_layer_overlay_enabled = true;

        let weapon = spawn_weapon(world, camera_entity);
        self.weapon_entity = Some(weapon);

        let flashlight = spawn_flashlight(world);
        self.flashlight_entity = Some(flashlight);
        self.flashlight_on = false;
        if let Some(light) = world.core.get_light_mut(flashlight) {
            light.intensity = 0.0;
        }

        self.spawn_environment(world);
        self.spawn_exhibits(world);

        let prompt_entity = spawn_ui_text(world, "", nalgebra_glm::Vec2::zeros());
        if let Some(hud_text) = world.core.get_text(prompt_entity) {
            self.interaction_prompt_text_index = Some(hud_text.text_index);
        }
        self.interaction_prompt_entity = Some(prompt_entity);

        let input_mode_entity = spawn_ui_text(world, "Mouse/Keyboard", nalgebra_glm::Vec2::zeros());
        if let Some(hud_text) = world.core.get_text(input_mode_entity) {
            self.input_mode_text_index = Some(hud_text.text_index);
        }
        self.input_mode_text_entity = Some(input_mode_entity);

        #[cfg(feature = "openxr")]
        {
            self.left_hand_cube =
                Some(self.spawn_hand_cube(world, nalgebra_glm::vec3(0.2, 0.6, 0.9)));
            let right_hand = self.spawn_hand_cube(world, nalgebra_glm::vec3(0.9, 0.6, 0.2));
            self.right_hand_cube = Some(right_hand);
            self.spawn_bauble_gun(world, right_hand);
        }

        world.resources.retained_ui.enabled = true;
        let (crosshair, crosshair_arms) = build_crosshair(world);
        self.crosshair_entity = Some(crosshair);
        self.crosshair_arms = crosshair_arms;
        let (note_overlay, note_title, note_content) = build_note_overlay(world);
        self.note_overlay_entity = Some(note_overlay);
        self.note_title_entity = Some(note_title);
        self.note_content_entity = Some(note_content);

        let (dash_hud, dash_state_text, dash_charges) = build_dash_hud(world);
        self.dash_hud_entity = Some(dash_hud);
        self.dash_hud_state_text_entity = Some(dash_state_text);
        self.dash_hud_charge_entities = dash_charges;
    }

    fn run_systems(&mut self, world: &mut World) {
        if let Some(crosshair) = self.crosshair_entity {
            world.ui_set_visible(crosshair, self.reading_note.is_none());
        }

        if let Some(overlay) = self.note_overlay_entity {
            if let Some(note_index) = self.reading_note {
                world.ui_set_visible(overlay, true);
                if self.last_shown_note != Some(note_index) {
                    let title = self.notes[note_index].title.clone();
                    let content = self.notes[note_index].content.clone();
                    if let Some(entity) = self.note_title_entity {
                        world.ui_set_text(entity, &title);
                    }
                    if let Some(entity) = self.note_content_entity {
                        world.ui_set_text(entity, &content);
                    }
                    self.last_shown_note = Some(note_index);
                }
            } else {
                world.ui_set_visible(overlay, false);
                if self.last_shown_note.is_some() {
                    self.last_shown_note = None;
                }
            }
        }

        if self.reading_note.is_some() {
            self.note_reading_system(world);
        }

        escape_key_exit_system(world);
        if let Some(gamepad) = query_active_gamepad(world)
            && gamepad.is_pressed(gilrs::Button::Select)
        {
            world.resources.window.should_exit = true;
        }
        self.debug_toggle_system(world);
        #[cfg(not(feature = "openxr"))]
        self.detect_input_mode(world);
        self.check_fall_reset(world);
        #[cfg(not(feature = "openxr"))]
        self.camera_look_system(world);
        #[cfg(not(feature = "openxr"))]
        self.lean_system(world);
        #[cfg(not(feature = "openxr"))]
        self.crouch_camera_system(world);
        #[cfg(feature = "openxr")]
        self.xr_hand_tracking_system(world);
        self.dash_system(world);
        self.update_weapon_sway(world);
        nightshade::ecs::transform::systems::update_global_transforms_system(world);
        self.interaction_system(world);
        self.update_shot_baubles(world);
        self.update_doors_momentum(world);
        self.update_drawers_momentum(world);
        self.update_levers_momentum(world);
        self.update_wheels_momentum(world);
        self.update_lantern_light(world);
        self.update_flashlight(world);
        self.update_interaction_prompt(world);
        self.update_prismatic_sliders(world);
        self.update_joint_visuals(world);
        self.update_coulomb_friction_joints(world);
        self.setup_velocity_friction_joints(world);
    }

}

impl PhysicsDemo {
    fn spawn_environment(&self, world: &mut World) {
        let floor_material =
            create_textured_material(nalgebra_glm::vec3(0.15, 0.15, 0.18), 0.9, 0.0);
        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(0.0, -0.25, 0.0),
            nalgebra_glm::vec3(30.0, 0.5, 30.0),
            floor_material,
        );

        let wall_material =
            create_textured_material(nalgebra_glm::vec3(0.2, 0.18, 0.16), 0.95, 0.0);

        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(0.0, 2.0, -15.0),
            nalgebra_glm::vec3(30.0, 4.0, 0.5),
            wall_material.clone(),
        );

        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(-15.0, 2.0, 0.0),
            nalgebra_glm::vec3(0.5, 4.0, 30.0),
            wall_material.clone(),
        );

        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(15.0, 2.0, 0.0),
            nalgebra_glm::vec3(0.5, 4.0, 30.0),
            wall_material,
        );
    }

    fn spawn_exhibits(&mut self, world: &mut World) {
        self.spawn_grabbables_exhibit(world, nalgebra_glm::vec3(-10.0, 0.0, -10.0));
        self.spawn_door_exhibit(world, nalgebra_glm::vec3(-4.0, 0.0, -12.0));
        self.spawn_drawer_exhibit(world, nalgebra_glm::vec3(4.0, 0.0, -12.0));
        self.spawn_lever_exhibit(world, nalgebra_glm::vec3(10.0, 0.0, -10.0));
        self.spawn_wheel_exhibit(world, nalgebra_glm::vec3(-10.0, 0.0, -4.0));
        self.spawn_chain_exhibit(world, nalgebra_glm::vec3(0.0, 0.0, -12.0));
        self.spawn_bauble_table(world, nalgebra_glm::vec3(10.0, 0.0, -4.0));
        self.spawn_note_table(world, nalgebra_glm::vec3(0.0, 0.0, 4.0));

        self.spawn_fixed_joint_exhibit(world, nalgebra_glm::vec3(-10.0, 0.0, 4.0));
        self.spawn_spherical_joint_exhibit(world, nalgebra_glm::vec3(-4.0, 0.0, 4.0));
        self.spawn_rope_joint_exhibit(world, nalgebra_glm::vec3(4.0, 0.0, 4.0));
        self.spawn_spring_joint_exhibit(world, nalgebra_glm::vec3(10.0, 0.0, 4.0));
        self.spawn_prismatic_joint_exhibit(world, nalgebra_glm::vec3(-6.0, 0.0, 8.0));
        self.spawn_revolute_joint_exhibit(world, nalgebra_glm::vec3(6.0, 0.0, 8.0));
        self.spawn_velocity_friction_joint_exhibit(world, nalgebra_glm::vec3(0.0, 0.0, 10.0));
        self.spawn_coulomb_friction_joint_exhibit(world, nalgebra_glm::vec3(-6.0, 0.0, 12.0));

        self.spawn_curiosity_room(world, nalgebra_glm::vec3(12.0, 0.0, 11.0));
        self.spawn_workshop_room(world, nalgebra_glm::vec3(-12.0, 0.0, 11.0));
    }

    fn spawn_grabbables_exhibit(&mut self, world: &mut World, center: Vec3) {
        let pedestal_material =
            create_textured_material(nalgebra_glm::vec3(0.25, 0.25, 0.28), 0.85, 0.0);

        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, 0.4, center.z),
            nalgebra_glm::vec3(2.5, 0.8, 2.5),
            pedestal_material,
        );

        let table_top_y = 0.8;
        let box_size = 0.25;
        let box_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.5, 0.35), 0.7, 0.0);

        let positions = [
            nalgebra_glm::vec3(center.x - 0.5, table_top_y + box_size / 2.0, center.z - 0.5),
            nalgebra_glm::vec3(center.x + 0.5, table_top_y + box_size / 2.0, center.z - 0.5),
            nalgebra_glm::vec3(center.x, table_top_y + box_size / 2.0, center.z + 0.5),
        ];

        for position in positions {
            let entity = spawn_dynamic_physics_cube_with_material(
                world,
                position,
                nalgebra_glm::vec3(box_size, box_size, box_size),
                2.0,
                box_material.clone(),
            );
            self.physics_objects.push(entity);
        }

        let sphere_radius = 0.2;
        let sphere_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.2, 0.2), 0.5, 0.3);
        let sphere_entity = spawn_dynamic_physics_sphere_with_material(
            world,
            nalgebra_glm::vec3(center.x, table_top_y + sphere_radius, center.z),
            sphere_radius,
            1.5,
            sphere_material,
        );
        self.physics_objects.push(sphere_entity);

        let cylinder_half_height = 0.2;
        let cylinder_radius = 0.12;
        let metal_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.3, 0.8);
        let cylinder_entity = spawn_dynamic_physics_cylinder_with_material(
            world,
            nalgebra_glm::vec3(center.x - 0.7, table_top_y + cylinder_half_height, center.z),
            cylinder_half_height,
            cylinder_radius,
            3.0,
            metal_material,
        );
        self.physics_objects.push(cylinder_entity);
    }

    fn spawn_door_exhibit(&mut self, world: &mut World, center: Vec3) {
        let door_width = 0.9;
        let door_height = 2.5;
        let door_thickness = 0.15;
        let frame_thickness = 0.1;
        let frame_depth = 0.15;

        let frame_left_x = center.x;
        let frame_right_x = frame_left_x + door_width + frame_thickness * 2.0;
        let frame_center_x = (frame_left_x + frame_right_x) / 2.0;
        let frame_z = center.z;
        let frame_y = door_height / 2.0;

        let hinge_x = frame_left_x + frame_thickness;
        let door_center_x = hinge_x + door_width / 2.0;
        let hinge_position = nalgebra_glm::vec3(hinge_x, frame_y, frame_z);

        let door_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.35, 0.2), 0.8, 0.0);

        let door_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | VISIBILITY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(door_entity) {
            name.0 = "Door".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(door_entity) {
            transform.translation = nalgebra_glm::vec3(door_center_x, frame_y, frame_z);
            transform.scale = nalgebra_glm::vec3(door_width, door_height, door_thickness);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(door_entity) {
            mesh.name = "Cube".to_string();
        }

        let material_name = format!("Door_{}", door_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            door_material,
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
            .set_material_ref(door_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(door_entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(door_entity) {
            *rigid_body = RigidBodyComponent::new_kinematic().with_translation(
                door_center_x,
                frame_y,
                frame_z,
            );
        }

        if let Some(collider) = world.core.get_collider_mut(door_entity) {
            *collider = ColliderComponent::new_cuboid(
                door_width / 2.0,
                door_height / 2.0,
                door_thickness / 2.0,
            )
            .with_friction(0.5);
        }

        let door_rb_handle = {
            let rigid_body_comp = world.core.get_rigid_body(door_entity).cloned().unwrap();
            let collider_comp = world.core.get_collider(door_entity).cloned();
            let rigid_body = rigid_body_comp.to_rapier_rigid_body();
            let handle = world.resources.physics.add_rigid_body(rigid_body);
            if let Some(collider_comp) = collider_comp {
                let collider = collider_comp.to_rapier_collider();
                world.resources.physics.add_collider(collider, handle);
            }
            if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(door_entity) {
                rigid_body_mut.handle = Some(handle.into());
            }
            handle
        };

        let door_frame_material =
            create_textured_material(nalgebra_glm::vec3(0.35, 0.25, 0.15), 0.85, 0.0);

        self.spawn_visual_cube(
            world,
            nalgebra_glm::vec3(frame_center_x, door_height + frame_thickness / 2.0, frame_z),
            nalgebra_glm::vec3(
                door_width + frame_thickness * 2.0,
                frame_thickness,
                frame_depth,
            ),
            door_frame_material.clone(),
            "Door Frame Top".to_string(),
        );

        self.spawn_visual_cube(
            world,
            nalgebra_glm::vec3(frame_left_x + frame_thickness / 2.0, frame_y, frame_z),
            nalgebra_glm::vec3(frame_thickness, door_height, frame_depth),
            door_frame_material.clone(),
            "Door Frame Left".to_string(),
        );

        self.spawn_visual_cube(
            world,
            nalgebra_glm::vec3(frame_right_x - frame_thickness / 2.0, frame_y, frame_z),
            nalgebra_glm::vec3(frame_thickness, door_height, frame_depth),
            door_frame_material,
            "Door Frame Right".to_string(),
        );

        self.doors.push(DoorState {
            entity: door_entity,
            rigid_body_handle: door_rb_handle,
            hinge_position,
            door_half_width: door_width / 2.0,
            current_angle: 0.0,
            angular_velocity: 0.0,
            min_angle: -std::f32::consts::FRAC_PI_2 * 0.9,
            max_angle: std::f32::consts::FRAC_PI_2 * 0.9,
        });
    }

    fn spawn_drawer_exhibit(&mut self, world: &mut World, center: Vec3) {
        let cabinet_width = 1.0;
        let cabinet_height = 1.2;
        let cabinet_depth = 0.6;
        let cabinet_x = center.x;
        let cabinet_z = center.z;
        let cabinet_bottom_y = 0.0;

        let cabinet_material =
            create_textured_material(nalgebra_glm::vec3(0.4, 0.3, 0.2), 0.85, 0.0);

        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(
                cabinet_x,
                cabinet_bottom_y + cabinet_height / 2.0,
                cabinet_z - cabinet_depth / 2.0 + 0.05,
            ),
            nalgebra_glm::vec3(cabinet_width, cabinet_height, cabinet_depth - 0.1),
            cabinet_material,
        );

        let drawer_front_material =
            create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.75, 0.0);
        let drawer_interior_material =
            create_textured_material(nalgebra_glm::vec3(0.6, 0.55, 0.45), 0.9, 0.0);

        let drawer_count = 3;
        let drawer_height = 0.3;
        let drawer_gap = 0.05;
        let drawer_inner_width = cabinet_width - 0.1;
        let drawer_inner_depth = cabinet_depth - 0.1;
        let drawer_inner_height = drawer_height - 0.05;
        let panel_thickness = 0.02;
        let max_slide = cabinet_depth * 0.6;

        for index in 0..drawer_count {
            let drawer_y = cabinet_bottom_y
                + drawer_gap
                + drawer_height / 2.0
                + index as f32 * (drawer_height + drawer_gap);
            let drawer_closed_z = cabinet_z - drawer_inner_depth / 2.0;
            let closed_position = nalgebra_glm::vec3(cabinet_x, drawer_y, drawer_closed_z);

            let drawer_parent = world.spawn_entities(
                NAME | LOCAL_TRANSFORM
                    | GLOBAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | nightshade::ecs::world::RIGID_BODY
                    | nightshade::ecs::world::COLLIDER,
                1,
            )[0];

            if let Some(name) = world.core.get_name_mut(drawer_parent) {
                name.0 = format!("Drawer {}", index + 1);
            }

            if let Some(transform) = world.core.get_local_transform_mut(drawer_parent) {
                transform.translation = closed_position;
            }

            if let Some(rigid_body) = world.core.get_rigid_body_mut(drawer_parent) {
                *rigid_body = RigidBodyComponent::new_kinematic().with_translation(
                    closed_position.x,
                    closed_position.y,
                    closed_position.z,
                );
            }

            if let Some(collider) = world.core.get_collider_mut(drawer_parent) {
                *collider = ColliderComponent::new_cuboid(
                    drawer_inner_width / 2.0,
                    drawer_inner_height / 2.0,
                    drawer_inner_depth / 2.0,
                )
                .with_friction(0.3);
            }

            let front_entity = world.spawn_entities(
                NAME | LOCAL_TRANSFORM
                    | GLOBAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | RENDER_MESH
                    | MATERIAL_REF
                    | BOUNDING_VOLUME
                    | CASTS_SHADOW
                    | PARENT
                    | VISIBILITY,
                1,
            )[0];

            if let Some(name) = world.core.get_name_mut(front_entity) {
                name.0 = format!("Drawer {} Front", index + 1);
            }

            if let Some(transform) = world.core.get_local_transform_mut(front_entity) {
                transform.translation = nalgebra_glm::vec3(0.0, 0.0, drawer_inner_depth / 2.0);
                transform.scale = nalgebra_glm::vec3(cabinet_width, drawer_height, panel_thickness);
            }

            if let Some(mesh) = world.core.get_render_mesh_mut(front_entity) {
                mesh.name = "Cube".to_string();
            }

            let material_name = format!("DrawerFront_{}", front_entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                drawer_front_material.clone(),
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
                .set_material_ref(front_entity, MaterialRef::new(material_name));

            if let Some(bv) = world.core.get_bounding_volume_mut(front_entity) {
                *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
            }

            if let Some(parent) = world.core.get_parent_mut(front_entity) {
                *parent = Parent(Some(drawer_parent));
            }

            self.spawn_drawer_panel(
                world,
                drawer_parent,
                nalgebra_glm::vec3(0.0, -drawer_inner_height / 2.0, 0.0),
                nalgebra_glm::vec3(drawer_inner_width, panel_thickness, drawer_inner_depth),
                drawer_interior_material.clone(),
                format!("Drawer {} Bottom", index + 1),
            );

            self.spawn_drawer_panel(
                world,
                drawer_parent,
                nalgebra_glm::vec3(-drawer_inner_width / 2.0, 0.0, 0.0),
                nalgebra_glm::vec3(panel_thickness, drawer_inner_height, drawer_inner_depth),
                drawer_interior_material.clone(),
                format!("Drawer {} Left", index + 1),
            );

            self.spawn_drawer_panel(
                world,
                drawer_parent,
                nalgebra_glm::vec3(drawer_inner_width / 2.0, 0.0, 0.0),
                nalgebra_glm::vec3(panel_thickness, drawer_inner_height, drawer_inner_depth),
                drawer_interior_material.clone(),
                format!("Drawer {} Right", index + 1),
            );

            self.spawn_drawer_panel(
                world,
                drawer_parent,
                nalgebra_glm::vec3(0.0, 0.0, -drawer_inner_depth / 2.0),
                nalgebra_glm::vec3(drawer_inner_width, drawer_inner_height, panel_thickness),
                drawer_interior_material.clone(),
                format!("Drawer {} Back", index + 1),
            );

            let drawer_rb_handle = {
                let rigid_body_comp = world.core.get_rigid_body(drawer_parent).cloned().unwrap();
                let collider_comp = world.core.get_collider(drawer_parent).cloned();
                let rigid_body = rigid_body_comp.to_rapier_rigid_body();
                let handle = world.resources.physics.add_rigid_body(rigid_body);
                if let Some(collider_comp) = collider_comp {
                    let collider = collider_comp.to_rapier_collider();
                    world.resources.physics.add_collider(collider, handle);
                }
                if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(drawer_parent) {
                    rigid_body_mut.handle = Some(handle.into());
                }
                handle
            };

            self.drawers.push(DrawerState {
                entity: drawer_parent,
                front_entity,
                rigid_body_handle: drawer_rb_handle,
                closed_position,
                current_offset: 0.0,
                velocity: 0.0,
                max_offset: max_slide,
            });
        }
    }

    fn spawn_lever_exhibit(&mut self, world: &mut World, center: Vec3) {
        let base_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.3, 0.32), 0.8, 0.1);
        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, 0.75, center.z),
            nalgebra_glm::vec3(0.8, 1.5, 0.4),
            base_material,
        );

        let pivot_position = nalgebra_glm::vec3(center.x, 1.2, center.z + 0.21);
        let arm_half_length = 0.2;
        let arm_half_thickness = 0.03;
        let handle_radius = 0.05;

        let pivot_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(pivot_entity) {
            name.0 = "Lever Pivot".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(pivot_entity) {
            transform.translation = pivot_position;
        }

        let lever_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.35, 0.2), 0.7, 0.2);

        let arm_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | PARENT
                | VISIBILITY,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(arm_entity) {
            name.0 = "Lever Arm".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(arm_entity) {
            transform.translation = nalgebra_glm::vec3(0.0, 0.0, arm_half_length);
            transform.scale = nalgebra_glm::vec3(
                arm_half_thickness * 2.0,
                arm_half_thickness * 2.0,
                arm_half_length * 2.0,
            );
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(arm_entity) {
            mesh.name = "Cube".to_string();
        }

        let material_name = format!("LeverArm_{}", arm_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            lever_material.clone(),
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
            .set_material_ref(arm_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(arm_entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
        }

        if let Some(parent) = world.core.get_parent_mut(arm_entity) {
            *parent = Parent(Some(pivot_entity));
        }

        let handle_material =
            create_textured_material(nalgebra_glm::vec3(0.15, 0.15, 0.17), 0.3, 0.7);
        let handle_offset = arm_half_length * 2.0 + handle_radius;

        let handle_visual_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | PARENT
                | VISIBILITY,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(handle_visual_entity) {
            name.0 = "Lever Handle Visual".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(handle_visual_entity) {
            transform.translation = nalgebra_glm::vec3(0.0, 0.0, handle_offset);
            transform.scale = nalgebra_glm::vec3(
                handle_radius * 2.0,
                handle_radius * 2.0,
                handle_radius * 2.0,
            );
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(handle_visual_entity) {
            mesh.name = "Sphere".to_string();
        }

        let material_name = format!("LeverHandle_{}", handle_visual_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            handle_material,
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
            .set_material_ref(handle_visual_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(handle_visual_entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Sphere");
        }

        if let Some(parent) = world.core.get_parent_mut(handle_visual_entity) {
            *parent = Parent(Some(pivot_entity));
        }

        let collider_half_length = arm_half_length + handle_radius;
        let collider_center_offset = collider_half_length;
        let collider_world_position = nalgebra_glm::vec3(
            pivot_position.x,
            pivot_position.y,
            pivot_position.z + collider_center_offset,
        );

        let collider_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER
                | nightshade::ecs::world::BOUNDING_VOLUME
                | nightshade::ecs::world::VISIBILITY,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(collider_entity) {
            name.0 = "Lever Collider".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(collider_entity) {
            transform.translation = collider_world_position;
            transform.scale = nalgebra_glm::vec3(
                arm_half_thickness * 2.0,
                arm_half_thickness * 2.0,
                collider_half_length * 2.0,
            );
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(collider_entity) {
            *rigid_body = RigidBodyComponent::new_kinematic().with_translation(
                collider_world_position.x,
                collider_world_position.y,
                collider_world_position.z,
            );
        }

        let hitbox_padding = 0.08;
        if let Some(collider) = world.core.get_collider_mut(collider_entity) {
            *collider = ColliderComponent::new_cuboid(
                arm_half_thickness + hitbox_padding,
                arm_half_thickness + hitbox_padding,
                collider_half_length,
            )
            .with_friction(0.5);
        }

        if let Some(bv) = world.core.get_bounding_volume_mut(collider_entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
        }

        let collider_rb_handle = {
            let rigid_body_comp = world.core.get_rigid_body(collider_entity).cloned().unwrap();
            let collider_comp = world.core.get_collider(collider_entity).cloned();
            let rigid_body = rigid_body_comp.to_rapier_rigid_body();
            let rb_handle = world.resources.physics.add_rigid_body(rigid_body);
            if let Some(collider_comp) = collider_comp {
                let collider = collider_comp.to_rapier_collider();
                world.resources.physics.add_collider(collider, rb_handle);
            }
            if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(collider_entity) {
                rigid_body_mut.handle = Some(rb_handle.into());
            }
            rb_handle
        };

        self.levers.push(LeverState {
            pivot_entity,
            collider_entity,
            collider_rb_handle,
            pivot_position,
            arm_half_length: collider_half_length,
            current_angle: -std::f32::consts::FRAC_PI_4,
            angular_velocity: 0.0,
            min_angle: -std::f32::consts::FRAC_PI_4,
            max_angle: std::f32::consts::FRAC_PI_3,
        });

        self.apply_lever_transform(world, self.levers.len() - 1);
    }

    fn spawn_wheel_exhibit(&mut self, world: &mut World, center: Vec3) {
        let mount_material =
            create_textured_material(nalgebra_glm::vec3(0.35, 0.35, 0.38), 0.85, 0.1);
        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, 0.75, center.z - 0.2),
            nalgebra_glm::vec3(0.3, 1.5, 0.4),
            mount_material,
        );

        let wheel_center = nalgebra_glm::vec3(center.x, 1.2, center.z + 0.15);
        let wheel_radius = 0.4;
        let wheel_thickness = 0.08;

        let wheel_material =
            create_textured_material(nalgebra_glm::vec3(0.5, 0.35, 0.2), 0.75, 0.1);

        let wheel_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | VISIBILITY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(wheel_entity) {
            name.0 = "Wheel".to_string();
        }

        let base_rotation = nalgebra_glm::quat_angle_axis(
            std::f32::consts::FRAC_PI_2,
            &nalgebra_glm::vec3(1.0, 0.0, 0.0),
        );

        if let Some(transform) = world.core.get_local_transform_mut(wheel_entity) {
            transform.translation = wheel_center;
            transform.scale = nalgebra_glm::vec3(wheel_radius, wheel_thickness / 2.0, wheel_radius);
            transform.rotation = base_rotation;
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(wheel_entity) {
            mesh.name = "Cylinder".to_string();
        }

        let material_name = format!("Wheel_{}", wheel_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            wheel_material,
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
            .set_material_ref(wheel_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(wheel_entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(wheel_entity) {
            *rigid_body = RigidBodyComponent::new_kinematic()
                .with_translation(wheel_center.x, wheel_center.y, wheel_center.z)
                .with_rotation(
                    base_rotation.i,
                    base_rotation.j,
                    base_rotation.k,
                    base_rotation.w,
                );
        }

        if let Some(collider) = world.core.get_collider_mut(wheel_entity) {
            *collider = ColliderComponent::new_cylinder(wheel_thickness / 2.0, wheel_radius)
                .with_friction(0.5);
        }

        let spoke_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.2, 0.15), 0.8, 0.0);
        let mut spoke_entities = Vec::new();
        for spoke_index in 0..4 {
            let angle = spoke_index as f32 * std::f32::consts::FRAC_PI_2;
            let spoke_entity = world.spawn_entities(
                NAME | LOCAL_TRANSFORM
                    | GLOBAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | RENDER_MESH
                    | MATERIAL_REF
                    | BOUNDING_VOLUME
                    | CASTS_SHADOW
                    | PARENT
                    | VISIBILITY,
                1,
            )[0];

            spoke_entities.push(spoke_entity);

            if let Some(name) = world.core.get_name_mut(spoke_entity) {
                name.0 = format!("Wheel Spoke {}", spoke_index + 1);
            }

            if let Some(transform) = world.core.get_local_transform_mut(spoke_entity) {
                transform.translation = nalgebra_glm::vec3(0.0, 0.0, 0.0);
                transform.scale = nalgebra_glm::vec3(0.04 / wheel_radius, 1.8, 0.04 / wheel_radius);
                transform.rotation =
                    nalgebra_glm::quat_angle_axis(angle, &nalgebra_glm::vec3(0.0, 0.0, 1.0));
            }

            if let Some(mesh) = world.core.get_render_mesh_mut(spoke_entity) {
                mesh.name = "Cube".to_string();
            }

            let material_name = format!("WheelSpoke_{}", spoke_entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                spoke_material.clone(),
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
                .set_material_ref(spoke_entity, MaterialRef::new(material_name));

            if let Some(bv) = world.core.get_bounding_volume_mut(spoke_entity) {
                *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
            }

            if let Some(parent) = world.core.get_parent_mut(spoke_entity) {
                *parent = Parent(Some(wheel_entity));
            }
        }

        let wheel_rb_handle = {
            let rigid_body_comp = world.core.get_rigid_body(wheel_entity).cloned().unwrap();
            let collider_comp = world.core.get_collider(wheel_entity).cloned();
            let rigid_body = rigid_body_comp.to_rapier_rigid_body();
            let handle = world.resources.physics.add_rigid_body(rigid_body);
            if let Some(collider_comp) = collider_comp {
                let collider = collider_comp.to_rapier_collider();
                world.resources.physics.add_collider(collider, handle);
            }
            if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(wheel_entity) {
                rigid_body_mut.handle = Some(handle.into());
            }
            handle
        };

        self.wheels.push(WheelState {
            entity: wheel_entity,
            spoke_entities,
            rigid_body_handle: wheel_rb_handle,
            center_position: wheel_center,
            current_angle: 0.0,
            angular_velocity: 0.0,
        });
    }

    fn spawn_chain_exhibit(&mut self, world: &mut World, center: Vec3) {
        use rapier3d::prelude::*;

        let anchor_height = 2.5;
        let anchor_position = nalgebra_glm::vec3(center.x, anchor_height, center.z);

        let beam_material =
            create_textured_material(nalgebra_glm::vec3(0.35, 0.25, 0.15), 0.9, 0.0);
        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, anchor_height + 0.1, center.z),
            nalgebra_glm::vec3(0.4, 0.2, 0.4),
            beam_material,
        );

        let anchor_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | nightshade::ecs::world::RIGID_BODY,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(anchor_entity) {
            name.0 = "Chain Anchor".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(anchor_entity) {
            transform.translation = anchor_position;
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(anchor_entity) {
            *rigid_body = RigidBodyComponent::new_static().with_translation(
                anchor_position.x,
                anchor_position.y,
                anchor_position.z,
            );
        }

        let anchor_handle = {
            let rigid_body_comp = world.core.get_rigid_body(anchor_entity).cloned().unwrap();
            let rigid_body = rigid_body_comp.to_rapier_rigid_body();
            let handle = world.resources.physics.add_rigid_body(rigid_body);
            if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(anchor_entity) {
                rigid_body_mut.handle = Some(handle.into());
            }
            handle
        };

        let chain_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.3, 0.32), 0.4, 0.8);

        let num_links = 8;
        let link_length = 0.15;
        let link_radius = 0.02;

        let mut _link_entities = Vec::new();
        let mut _link_handles = Vec::new();
        let mut prev_handle: Option<RigidBodyHandle> = Some(anchor_handle);

        for link_index in 0..num_links {
            let link_y = anchor_height - (link_index as f32 + 0.5) * link_length;
            let link_position = nalgebra_glm::vec3(center.x, link_y, center.z);

            let entity = world.spawn_entities(
                NAME | LOCAL_TRANSFORM
                    | GLOBAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | RENDER_MESH
                    | MATERIAL_REF
                    | BOUNDING_VOLUME
                    | CASTS_SHADOW
                    | VISIBILITY
                    | nightshade::ecs::world::RIGID_BODY
                    | nightshade::ecs::world::COLLIDER
                    | nightshade::ecs::world::PHYSICS_INTERPOLATION,
                1,
            )[0];

            if let Some(name) = world.core.get_name_mut(entity) {
                name.0 = format!("Chain Link {}", link_index + 1);
            }

            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                transform.translation = link_position;
                transform.scale =
                    nalgebra_glm::vec3(link_radius * 2.0, link_length, link_radius * 2.0);
            }

            if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
                mesh.name = "Cylinder".to_string();
            }

            let material_name = format!("ChainLink_{}", entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                chain_material.clone(),
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

            if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
                *bv =
                    nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
            }

            if let Some(rigid_body) = world.core.get_rigid_body_mut(entity) {
                *rigid_body = RigidBodyComponent::new_dynamic()
                    .with_translation(link_position.x, link_position.y, link_position.z)
                    .with_mass(0.1);
            }

            if let Some(collider) = world.core.get_collider_mut(entity) {
                *collider =
                    ColliderComponent::new_capsule(link_length / 2.0 - link_radius, link_radius)
                        .with_friction(0.3);
            }

            let handle = {
                let rigid_body_comp = world.core.get_rigid_body(entity).cloned().unwrap();
                let collider_comp = world.core.get_collider(entity).cloned();
                let rigid_body = rigid_body_comp.to_rapier_rigid_body();
                let handle = world.resources.physics.add_rigid_body(rigid_body);
                if let Some(collider_comp) = collider_comp {
                    let collider = collider_comp.to_rapier_collider();
                    world.resources.physics.add_collider(collider, handle);
                }
                if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(entity) {
                    rigid_body_mut.handle = Some(handle.into());
                }
                world
                    .resources
                    .physics
                    .handle_to_entity
                    .insert(handle, entity);
                world
                    .resources
                    .physics
                    .entity_to_handle
                    .insert(entity, handle);
                if let Some(interpolation) = world.core.get_physics_interpolation_mut(entity) {
                    interpolation.previous_translation = link_position;
                    interpolation.previous_rotation = nalgebra_glm::quat_identity();
                    interpolation.current_translation = link_position;
                    interpolation.current_rotation = nalgebra_glm::quat_identity();
                    interpolation.enabled = true;
                }
                if let Some(rb) = world.resources.physics.rigid_body_set.get_mut(handle) {
                    rb.set_linear_damping(0.5);
                    rb.set_angular_damping(0.5);
                }
                handle
            };

            if let Some(prev) = prev_handle {
                let local_anchor1 = if link_index == 0 {
                    point![0.0, 0.0, 0.0]
                } else {
                    point![0.0, -link_length / 2.0, 0.0]
                };
                let joint = SphericalJointBuilder::new()
                    .local_anchor1(local_anchor1)
                    .local_anchor2(point![0.0, link_length / 2.0, 0.0]);
                world.resources.physics.add_joint(prev, handle, joint);
            }

            _link_entities.push(entity);
            _link_handles.push(handle);
            prev_handle = Some(handle);
        }

        let lantern_material = create_emissive_material(nalgebra_glm::vec3(1.0, 0.8, 0.4), 2.0);

        let lantern_y = anchor_height - (num_links as f32 * link_length) - 0.15;
        let lantern_position = nalgebra_glm::vec3(center.x, lantern_y, center.z);

        let lantern_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | VISIBILITY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER
                | nightshade::ecs::world::PHYSICS_INTERPOLATION,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(lantern_entity) {
            name.0 = "Lantern".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(lantern_entity) {
            transform.translation = lantern_position;
            transform.scale = nalgebra_glm::vec3(0.25, 0.35, 0.25);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(lantern_entity) {
            mesh.name = "Cube".to_string();
        }

        let material_name = format!("Lantern_{}", lantern_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            lantern_material,
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
            .set_material_ref(lantern_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(lantern_entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(lantern_entity) {
            *rigid_body = RigidBodyComponent::new_dynamic()
                .with_translation(lantern_position.x, lantern_position.y, lantern_position.z)
                .with_mass(0.5);
        }

        if let Some(collider) = world.core.get_collider_mut(lantern_entity) {
            *collider = ColliderComponent::new_cuboid(0.125, 0.175, 0.125).with_friction(0.5);
        }

        let lantern_handle = {
            let rigid_body_comp = world.core.get_rigid_body(lantern_entity).cloned().unwrap();
            let collider_comp = world.core.get_collider(lantern_entity).cloned();
            let rigid_body = rigid_body_comp.to_rapier_rigid_body();
            let handle = world.resources.physics.add_rigid_body(rigid_body);
            if let Some(collider_comp) = collider_comp {
                let collider = collider_comp.to_rapier_collider();
                world.resources.physics.add_collider(collider, handle);
            }
            if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(lantern_entity) {
                rigid_body_mut.handle = Some(handle.into());
            }
            if let Some(rb) = world.resources.physics.rigid_body_set.get_mut(handle) {
                rb.set_linear_damping(0.5);
                rb.set_angular_damping(0.5);
            }
            handle
        };

        world
            .resources
            .physics
            .handle_to_entity
            .insert(lantern_handle, lantern_entity);
        world
            .resources
            .physics
            .entity_to_handle
            .insert(lantern_entity, lantern_handle);

        if let Some(interpolation) = world.core.get_physics_interpolation_mut(lantern_entity) {
            interpolation.previous_translation = lantern_position;
            interpolation.previous_rotation = nalgebra_glm::quat_identity();
            interpolation.current_translation = lantern_position;
            interpolation.current_rotation = nalgebra_glm::quat_identity();
            interpolation.enabled = true;
        }

        if let Some(last_link_handle) = prev_handle {
            let joint = SphericalJointBuilder::new()
                .local_anchor1(point![0.0, -link_length / 2.0, 0.0])
                .local_anchor2(point![0.0, 0.175, 0.0]);
            world
                .resources
                .physics
                .add_joint(last_link_handle, lantern_handle, joint);
        }

        let light_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | LIGHT,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(light_entity) {
            name.0 = "Lantern Light".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(light_entity) {
            transform.translation = lantern_position;
        }

        if let Some(light) = world.core.get_light_mut(light_entity) {
            *light = Light {
                light_type: LightType::Point,
                color: nalgebra_glm::vec3(1.0, 0.85, 0.6),
                intensity: 12.0,
                range: 15.0,
                inner_cone_angle: 0.0,
                outer_cone_angle: 0.0,
                cast_shadows: true,
                shadow_bias: 0.005,
            };
        }

        self.lantern_entity = Some(lantern_entity);
        self.lantern_light_entity = Some(light_entity);
        self.physics_objects.push(lantern_entity);
    }

    fn spawn_drawer_panel(
        &self,
        world: &mut World,
        parent: Entity,
        local_position: Vec3,
        scale: Vec3,
        material: nightshade::ecs::material::components::Material,
        name: String,
    ) {
        let entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | PARENT
                | VISIBILITY,
            1,
        )[0];

        if let Some(n) = world.core.get_name_mut(entity) {
            n.0 = name;
        }

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = local_position;
            transform.scale = scale;
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
            mesh.name = "Cube".to_string();
        }

        let material_name = format!("DrawerPanel_{}", entity.id);
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

        if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
        }

        if let Some(p) = world.core.get_parent_mut(entity) {
            *p = Parent(Some(parent));
        }
    }

    fn spawn_bauble(
        &mut self,
        world: &mut World,
        world_position: Vec3,
        radius: f32,
        color: Vec3,
        name: String,
    ) -> Entity {
        let entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | VISIBILITY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER,
            1,
        )[0];

        if let Some(n) = world.core.get_name_mut(entity) {
            n.0 = name;
        }

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = world_position;
            transform.scale = nalgebra_glm::vec3(radius, radius, radius);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
            mesh.name = "Sphere".to_string();
        }

        let material_name = format!("Bauble_{}", entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            create_textured_material(color, 0.2, 0.8),
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

        if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Sphere");
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(entity) {
            *rigid_body = RigidBodyComponent::new_dynamic()
                .with_translation(world_position.x, world_position.y, world_position.z)
                .with_mass(0.1);
        }

        if let Some(collider) = world.core.get_collider_mut(entity) {
            *collider = ColliderComponent::new_ball(radius)
                .with_friction(0.5)
                .with_restitution(0.3);
        }

        let rigid_body_comp = world.core.get_rigid_body(entity).cloned().unwrap();
        let collider_comp = world.core.get_collider(entity).cloned();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(collider_comp) = collider_comp {
            let collider = collider_comp.to_rapier_collider();
            world.resources.physics.add_collider(collider, handle);
        }
        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(entity) {
            rigid_body_mut.handle = Some(handle.into());
        }
        world
            .resources
            .physics
            .handle_to_entity
            .insert(handle, entity);
        world
            .resources
            .physics
            .entity_to_handle
            .insert(entity, handle);

        self.physics_objects.push(entity);
        entity
    }

    fn spawn_bauble_table(&mut self, world: &mut World, center: Vec3) {
        let table_top_material =
            create_textured_material(nalgebra_glm::vec3(0.45, 0.32, 0.2), 0.7, 0.1);
        let table_leg_material =
            create_textured_material(nalgebra_glm::vec3(0.35, 0.25, 0.15), 0.85, 0.0);

        let table_top_y = 0.75;
        let table_top_thickness = 0.05;
        let table_width = 1.4;
        let table_depth = 1.4;
        let leg_thickness = 0.08;
        let leg_height = table_top_y - table_top_thickness / 2.0;

        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, table_top_y, center.z),
            nalgebra_glm::vec3(table_width, table_top_thickness, table_depth),
            table_top_material,
        );

        let leg_offset_x = table_width / 2.0 - leg_thickness / 2.0 - 0.05;
        let leg_offset_z = table_depth / 2.0 - leg_thickness / 2.0 - 0.05;
        let leg_positions = [
            (leg_offset_x, leg_offset_z),
            (-leg_offset_x, leg_offset_z),
            (leg_offset_x, -leg_offset_z),
            (-leg_offset_x, -leg_offset_z),
        ];

        for (offset_x, offset_z) in leg_positions {
            spawn_static_physics_cube_with_material(
                world,
                nalgebra_glm::vec3(center.x + offset_x, leg_height / 2.0, center.z + offset_z),
                nalgebra_glm::vec3(leg_thickness, leg_height, leg_thickness),
                table_leg_material.clone(),
            );
        }

        let table_top_y = table_top_y + table_top_thickness / 2.0;
        self.bauble_table_center = center;
        self.bauble_table_top_y = table_top_y;

        self.spawn_recall_pedestal(world, nalgebra_glm::vec3(center.x + 2.0, 0.0, center.z));
        let bauble_colors = [
            nalgebra_glm::vec3(0.9, 0.2, 0.2),
            nalgebra_glm::vec3(0.2, 0.8, 0.3),
            nalgebra_glm::vec3(0.2, 0.4, 0.9),
            nalgebra_glm::vec3(0.9, 0.8, 0.1),
            nalgebra_glm::vec3(0.8, 0.3, 0.8),
            nalgebra_glm::vec3(0.1, 0.8, 0.8),
            nalgebra_glm::vec3(0.9, 0.5, 0.2),
            nalgebra_glm::vec3(0.6, 0.2, 0.6),
        ];

        let mut bauble_positions = Vec::new();
        let mut rng_seed = 12345u32;
        for _ in 0..80 {
            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let offset_x = ((rng_seed % 1000) as f32 / 1000.0 - 0.5) * 1.1;
            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let offset_z = ((rng_seed % 1000) as f32 / 1000.0 - 0.5) * 1.1;
            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let radius = 0.035 + ((rng_seed % 1000) as f32 / 1000.0) * 0.035;
            bauble_positions.push((offset_x, offset_z, radius));
        }

        for (index, (offset_x, offset_z, radius)) in bauble_positions.iter().enumerate() {
            let color = bauble_colors[index % bauble_colors.len()];
            let pos = nalgebra_glm::vec3(
                center.x + offset_x,
                table_top_y + radius + 0.01,
                center.z + offset_z,
            );
            let entity =
                self.spawn_bauble(world, pos, *radius, color, format!("Bauble {}", index + 1));
            self.baubles.push(BaubleSpawnData {
                entity,
                spawn_position: pos,
            });
        }
    }

    fn spawn_note_table(&mut self, world: &mut World, center: Vec3) {
        let table_top_material =
            create_textured_material(nalgebra_glm::vec3(0.35, 0.25, 0.15), 0.8, 0.0);
        let table_leg_material =
            create_textured_material(nalgebra_glm::vec3(0.3, 0.2, 0.1), 0.85, 0.0);

        let table_top_y = 0.75;
        let table_top_thickness = 0.04;
        let table_width = 0.8;
        let table_depth = 0.5;
        let leg_thickness = 0.06;
        let leg_height = table_top_y - table_top_thickness / 2.0;

        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, table_top_y, center.z),
            nalgebra_glm::vec3(table_width, table_top_thickness, table_depth),
            table_top_material,
        );

        let leg_offset_x = table_width / 2.0 - leg_thickness / 2.0 - 0.02;
        let leg_offset_z = table_depth / 2.0 - leg_thickness / 2.0 - 0.02;
        let leg_positions = [
            (leg_offset_x, leg_offset_z),
            (-leg_offset_x, leg_offset_z),
            (leg_offset_x, -leg_offset_z),
            (-leg_offset_x, -leg_offset_z),
        ];

        for (offset_x, offset_z) in leg_positions {
            spawn_static_physics_cube_with_material(
                world,
                nalgebra_glm::vec3(center.x + offset_x, leg_height / 2.0, center.z + offset_z),
                nalgebra_glm::vec3(leg_thickness, leg_height, leg_thickness),
                table_leg_material.clone(),
            );
        }

        let note_y = table_top_y + table_top_thickness / 2.0 + 0.005;
        let note_material = create_textured_material(nalgebra_glm::vec3(0.9, 0.85, 0.7), 0.95, 0.0);

        let note_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | VISIBILITY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(note_entity) {
            name.0 = "Note".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(note_entity) {
            transform.translation = nalgebra_glm::vec3(center.x, note_y, center.z);
            transform.scale = nalgebra_glm::vec3(0.15, 0.002, 0.2);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(note_entity) {
            mesh.name = "Cube".to_string();
        }

        let material_name = format!("Note_{}", note_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            note_material,
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
            .set_material_ref(note_entity, MaterialRef::new(material_name));

        if let Some(bounding_volume) = world.core.get_bounding_volume_mut(note_entity) {
            *bounding_volume =
                nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(note_entity) {
            *rigid_body =
                RigidBodyComponent::new_static().with_translation(center.x, note_y, center.z);
        }

        if let Some(collider) = world.core.get_collider_mut(note_entity) {
            *collider = ColliderComponent::new_cuboid(0.075, 0.001, 0.1).with_friction(0.5);
        }

        let rigid_body_comp = world.core.get_rigid_body(note_entity).cloned().unwrap();
        let collider_comp = world.core.get_collider(note_entity).cloned();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(collider_comp) = collider_comp {
            let collider = collider_comp.to_rapier_collider();
            world.resources.physics.add_collider(collider, handle);
        }
        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(note_entity) {
            rigid_body_mut.handle = Some(handle.into());
        }

        self.notes.push(NoteState {
            entity: note_entity,
            title: "Engineer's Log - Day 37".to_string(),
            content: "The generator keeps failing. I've replaced the fuel lines twice now, \
but something else is draining the power. The lights flicker at night, \
and I hear... things... in the walls.\n\n\
Whatever is down here, it doesn't want us to leave.\n\n\
If you find this note, get out while you still can. \
Don't go to the lower levels. Don't follow the sounds.\n\n\
                - M. Richter"
                .to_string(),
        });
    }

    fn spawn_fixed_joint_exhibit(&mut self, world: &mut World, center: Vec3) {
        spawn_label(
            world,
            "Fixed Joint",
            nalgebra_glm::vec3(center.x + 1.0, 3.5, center.z),
            TextProperties {
                font_size: 24.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        let num_vertebrae = 6;
        let block_size = 0.3;
        let block_spacing = 0.35;

        let beam_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
        let anchor_entity = spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, 2.5, center.z),
            nalgebra_glm::vec3(0.3, 0.3, 0.3),
            beam_material,
        );

        let colors = [
            nalgebra_glm::vec3(0.8, 0.3, 0.3),
            nalgebra_glm::vec3(0.8, 0.5, 0.3),
            nalgebra_glm::vec3(0.8, 0.8, 0.3),
            nalgebra_glm::vec3(0.3, 0.8, 0.3),
            nalgebra_glm::vec3(0.3, 0.5, 0.8),
            nalgebra_glm::vec3(0.6, 0.3, 0.8),
        ];

        let mut previous_entity = anchor_entity;
        for vertebra_index in 0..num_vertebrae {
            let color = colors[vertebra_index % colors.len()];
            let block_material = create_textured_material(color, 0.6, 0.2);
            let block_x = center.x + (vertebra_index as f32 + 1.0) * block_spacing;

            let block_entity = spawn_dynamic_physics_cube_with_material(
                world,
                nalgebra_glm::vec3(block_x, 2.5, center.z),
                nalgebra_glm::vec3(block_size, block_size, block_size),
                1.5,
                block_material,
            );
            self.physics_objects.push(block_entity);

            create_fixed_joint(
                world,
                previous_entity,
                block_entity,
                FixedJoint::new()
                    .with_local_anchor1(nalgebra_glm::vec3(block_size / 2.0 + 0.025, 0.0, 0.0))
                    .with_local_anchor2(nalgebra_glm::vec3(-block_size / 2.0 - 0.025, 0.0, 0.0)),
            );

            previous_entity = block_entity;
        }
    }

    fn spawn_spherical_joint_exhibit(&mut self, world: &mut World, center: Vec3) {
        spawn_label(
            world,
            "Spherical Joint",
            nalgebra_glm::vec3(center.x, 4.0, center.z),
            TextProperties {
                font_size: 24.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        let anchor_position = nalgebra_glm::vec3(center.x, 3.0, center.z);
        let ball_position = nalgebra_glm::vec3(center.x, 1.8, center.z);
        let rod_length = 1.0;

        let beam_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
        let anchor_entity = spawn_static_physics_cube_with_material(
            world,
            anchor_position,
            nalgebra_glm::vec3(0.3, 0.3, 0.3),
            beam_material,
        );

        let pendulum_material =
            create_textured_material(nalgebra_glm::vec3(0.3, 0.8, 0.3), 0.5, 0.3);
        let pendulum_entity = spawn_dynamic_physics_sphere_with_material(
            world,
            ball_position,
            0.2,
            3.0,
            pendulum_material,
        );
        self.physics_objects.push(pendulum_entity);

        create_spherical_joint(
            world,
            anchor_entity,
            pendulum_entity,
            SphericalJoint::new()
                .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.15, 0.0))
                .with_local_anchor2(nalgebra_glm::vec3(0.0, rod_length, 0.0)),
        );

        let rod_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.55, 0.5), 0.7, 0.2);
        let rod_entity = world.spawn_entities(
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

        if let Some(name) = world.core.get_name_mut(rod_entity) {
            name.0 = "Spherical Joint Rod".to_string();
        }

        let midpoint = (anchor_position + ball_position) * 0.5;
        let distance = nalgebra_glm::distance(&anchor_position, &ball_position);

        if let Some(transform) = world.core.get_local_transform_mut(rod_entity) {
            transform.translation = midpoint;
            transform.scale = nalgebra_glm::vec3(0.03, distance / 2.0, 0.03);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(rod_entity) {
            mesh.name = "Cylinder".to_string();
        }

        let material_name = format!("SphericalRod_{}", rod_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            rod_material,
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
            .set_material_ref(rod_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(rod_entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
        }

        self.spherical_joint_visuals.push(SphericalJointVisual {
            anchor_entity,
            ball_entity: pendulum_entity,
            rod_entity,
        });
    }

    fn spawn_rope_joint_exhibit(&mut self, world: &mut World, center: Vec3) {
        spawn_label(
            world,
            "Rope Joint",
            nalgebra_glm::vec3(center.x, 4.0, center.z),
            TextProperties {
                font_size: 24.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        let anchor_height = 3.0;
        let anchor_position = nalgebra_glm::vec3(center.x, anchor_height, center.z);
        let ball_start_position = nalgebra_glm::vec3(center.x, anchor_height - 0.3, center.z);

        let beam_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
        let anchor_entity = spawn_static_physics_cube_with_material(
            world,
            anchor_position,
            nalgebra_glm::vec3(0.3, 0.3, 0.3),
            beam_material,
        );

        let ball_material = create_textured_material(nalgebra_glm::vec3(0.8, 0.4, 0.8), 0.4, 0.5);
        let ball_entity = spawn_dynamic_physics_sphere_with_material(
            world,
            ball_start_position,
            0.25,
            2.0,
            ball_material,
        );
        self.physics_objects.push(ball_entity);

        create_rope_joint(
            world,
            anchor_entity,
            ball_entity,
            RopeJoint::new(1.8)
                .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.15, 0.0))
                .with_local_anchor2(nalgebra_glm::vec3(0.0, 0.0, 0.0)),
        );

        let rope_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.5, 0.35), 0.9, 0.0);
        let rope_entity = world.spawn_entities(
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

        if let Some(name) = world.core.get_name_mut(rope_entity) {
            name.0 = "Rope Joint Visual".to_string();
        }

        let anchor_attach = anchor_position - nalgebra_glm::vec3(0.0, 0.15, 0.0);
        let midpoint = (anchor_attach + ball_start_position) * 0.5;
        let distance = nalgebra_glm::distance(&anchor_attach, &ball_start_position);

        if let Some(transform) = world.core.get_local_transform_mut(rope_entity) {
            transform.translation = midpoint;
            transform.scale = nalgebra_glm::vec3(0.02, distance / 2.0, 0.02);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(rope_entity) {
            mesh.name = "Cylinder".to_string();
        }

        let material_name = format!("RopeVisual_{}", rope_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            rope_material,
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
            .set_material_ref(rope_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(rope_entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
        }

        self.rope_joint_visuals.push(RopeJointVisual {
            anchor_entity,
            ball_entity,
            rope_entity,
        });
    }

    fn spawn_spring_joint_exhibit(&mut self, world: &mut World, center: Vec3) {
        spawn_label(
            world,
            "Spring Joint",
            nalgebra_glm::vec3(center.x, 4.0, center.z),
            TextProperties {
                font_size: 24.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        let anchor_height = 3.0;
        let anchor_position = nalgebra_glm::vec3(center.x, anchor_height, center.z);
        let object_position = nalgebra_glm::vec3(center.x, anchor_height - 1.5, center.z);

        let beam_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
        let anchor_entity = spawn_static_physics_cube_with_material(
            world,
            anchor_position,
            nalgebra_glm::vec3(0.3, 0.3, 0.3),
            beam_material,
        );

        let spring_cube_material =
            create_textured_material(nalgebra_glm::vec3(0.3, 0.8, 0.8), 0.4, 0.5);
        let spring_entity = spawn_dynamic_physics_cube_with_material(
            world,
            object_position,
            nalgebra_glm::vec3(0.4, 0.4, 0.4),
            3.0,
            spring_cube_material,
        );
        self.physics_objects.push(spring_entity);

        create_spring_joint(
            world,
            anchor_entity,
            spring_entity,
            SpringJoint::new(1.0, 50.0, 2.0)
                .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.15, 0.0))
                .with_local_anchor2(nalgebra_glm::vec3(0.0, 0.2, 0.0)),
        );

        let coil_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.7, 0.75), 0.3, 0.8);
        let num_coils = 8;
        let mut spring_entities = Vec::new();

        for coil_index in 0..num_coils {
            let coil_entity = world.spawn_entities(
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

            if let Some(name) = world.core.get_name_mut(coil_entity) {
                name.0 = format!("Spring Coil {}", coil_index);
            }

            if let Some(transform) = world.core.get_local_transform_mut(coil_entity) {
                transform.translation = anchor_position;
                transform.scale = nalgebra_glm::vec3(0.015, 0.1, 0.015);
            }

            if let Some(mesh) = world.core.get_render_mesh_mut(coil_entity) {
                mesh.name = "Cylinder".to_string();
            }

            let material_name = format!("SpringCoil_{}_{}", spring_entity.id, coil_index);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                coil_material.clone(),
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
                .set_material_ref(coil_entity, MaterialRef::new(material_name));

            if let Some(bv) = world.core.get_bounding_volume_mut(coil_entity) {
                *bv =
                    nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
            }

            spring_entities.push(coil_entity);
        }

        self.spring_joint_visuals.push(SpringJointVisual {
            anchor_entity,
            object_entity: spring_entity,
            spring_entities,
        });
    }

    fn spawn_prismatic_joint_exhibit(&mut self, world: &mut World, center: Vec3) {
        spawn_label(
            world,
            "Prismatic Joint",
            nalgebra_glm::vec3(center.x, 2.5, center.z),
            TextProperties {
                font_size: 24.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        let rail_y = 1.5;
        let rail_half_height = 0.075;
        let slider_half_height = 0.15;
        let slider_y = rail_y + rail_half_height + slider_half_height;

        let rail_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.4, 0.3), 0.9, 0.0);
        let rail_entity = spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, rail_y, center.z),
            nalgebra_glm::vec3(3.0, 0.15, 0.15),
            rail_material,
        );

        let slider_material = create_textured_material(nalgebra_glm::vec3(0.8, 0.8, 0.3), 0.5, 0.4);
        let slider_entity = spawn_dynamic_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x - 1.0, slider_y, center.z),
            nalgebra_glm::vec3(0.3, 0.3, 0.3),
            1.0,
            slider_material,
        );
        self.physics_objects.push(slider_entity);

        create_prismatic_joint(
            world,
            rail_entity,
            slider_entity,
            PrismaticJoint::new(JointAxisDirection::X)
                .with_local_anchor1(nalgebra_glm::vec3(
                    0.0,
                    rail_half_height + slider_half_height,
                    0.0,
                ))
                .with_local_anchor2(nalgebra_glm::vec3(0.0, 0.0, 0.0))
                .with_limits(JointLimits::new(-1.3, 1.3)),
        );

        self.prismatic_sliders.push(PrismaticSliderState {
            entity: slider_entity,
            time_accumulator: 0.0,
        });
    }

    fn update_prismatic_sliders(&mut self, world: &mut World) {
        let dt = world.resources.window.timing.delta_time;

        for slider in &mut self.prismatic_sliders {
            if self.interaction.grabbed_entity == Some(slider.entity) {
                continue;
            }

            slider.time_accumulator += dt;

            let target_velocity = (slider.time_accumulator * 1.5).sin() * 2.0;

            let Some(rigid_body_component) = world.core.get_rigid_body(slider.entity) else {
                continue;
            };
            let Some(handle) = rigid_body_component.handle else {
                continue;
            };
            let Some(rigid_body) = world
                .resources
                .physics
                .rigid_body_set
                .get_mut(handle.into())
            else {
                continue;
            };

            let current_vel = rigid_body.linvel();
            rigid_body.set_linvel(
                rapier3d::math::Vector::new(target_velocity, current_vel.y, current_vel.z),
                true,
            );
        }
    }

    fn update_joint_visuals(&self, world: &mut World) {
        for visual in &self.spherical_joint_visuals {
            let anchor_pos = world
                .core
                .get_global_transform(visual.anchor_entity)
                .map(|t| t.translation())
                .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));
            let ball_pos = world
                .core
                .get_global_transform(visual.ball_entity)
                .map(|t| t.translation())
                .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));

            let anchor_attach = anchor_pos - nalgebra_glm::vec3(0.0, 0.15, 0.0);
            let ball_attach = ball_pos + nalgebra_glm::vec3(0.0, 0.2, 0.0);
            let midpoint = (anchor_attach + ball_attach) * 0.5;
            let distance = nalgebra_glm::distance(&anchor_attach, &ball_attach);

            let rotation = Self::rotation_from_to_direction(
                nalgebra_glm::vec3(0.0, 1.0, 0.0),
                ball_attach - anchor_attach,
            );

            if let Some(transform) = world.core.get_local_transform_mut(visual.rod_entity) {
                transform.translation = midpoint;
                transform.rotation = rotation;
                transform.scale = nalgebra_glm::vec3(0.03, distance.max(0.01), 0.03);
            }
            nightshade::ecs::transform::commands::mark_local_transform_dirty(
                world,
                visual.rod_entity,
            );
        }

        for visual in &self.rope_joint_visuals {
            let anchor_pos = world
                .core
                .get_global_transform(visual.anchor_entity)
                .map(|t| t.translation())
                .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));
            let ball_pos = world
                .core
                .get_global_transform(visual.ball_entity)
                .map(|t| t.translation())
                .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));

            let anchor_attach = anchor_pos - nalgebra_glm::vec3(0.0, 0.15, 0.0);
            let midpoint = (anchor_attach + ball_pos) * 0.5;
            let distance = nalgebra_glm::distance(&anchor_attach, &ball_pos);

            let rotation = Self::rotation_from_to_direction(
                nalgebra_glm::vec3(0.0, 1.0, 0.0),
                ball_pos - anchor_attach,
            );

            if let Some(transform) = world.core.get_local_transform_mut(visual.rope_entity) {
                transform.translation = midpoint;
                transform.rotation = rotation;
                transform.scale = nalgebra_glm::vec3(0.02, distance.max(0.01), 0.02);
            }
            nightshade::ecs::transform::commands::mark_local_transform_dirty(
                world,
                visual.rope_entity,
            );
        }

        for visual in &self.spring_joint_visuals {
            let anchor_pos = world
                .core
                .get_global_transform(visual.anchor_entity)
                .map(|t| t.translation())
                .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));
            let object_pos = world
                .core
                .get_global_transform(visual.object_entity)
                .map(|t| t.translation())
                .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));

            let anchor_attach = anchor_pos - nalgebra_glm::vec3(0.0, 0.15, 0.0);
            let object_attach = object_pos + nalgebra_glm::vec3(0.0, 0.2, 0.0);
            let total_distance = nalgebra_glm::distance(&anchor_attach, &object_attach);

            let num_coils = visual.spring_entities.len();
            if num_coils == 0 {
                continue;
            }

            let direction = if total_distance > 0.001 {
                nalgebra_glm::normalize(&(object_attach - anchor_attach))
            } else {
                nalgebra_glm::vec3(0.0, -1.0, 0.0)
            };

            let up = nalgebra_glm::vec3(0.0, 1.0, 0.0);
            let coil_radius = 0.08;

            for (coil_index, &coil_entity) in visual.spring_entities.iter().enumerate() {
                let t = (coil_index as f32 + 0.5) / num_coils as f32;
                let base_pos = anchor_attach + direction * (t * total_distance);

                let angle = coil_index as f32 * std::f32::consts::PI;
                let perpendicular = if direction.y.abs() > 0.999_f32 {
                    nalgebra_glm::vec3(1.0, 0.0, 0.0)
                } else {
                    nalgebra_glm::normalize(&nalgebra_glm::cross(&direction, &up))
                };
                let perpendicular2 = nalgebra_glm::cross(&direction, &perpendicular);

                let offset = perpendicular * (angle.cos() * coil_radius)
                    + perpendicular2 * (angle.sin() * coil_radius);
                let coil_pos = base_pos + offset;

                let next_t = ((coil_index + 1) as f32 + 0.5) / num_coils as f32;
                let next_base_pos = anchor_attach + direction * (next_t * total_distance);
                let next_angle = (coil_index + 1) as f32 * std::f32::consts::PI;
                let next_offset = perpendicular * (next_angle.cos() * coil_radius)
                    + perpendicular2 * (next_angle.sin() * coil_radius);
                let next_coil_pos = next_base_pos + next_offset;

                let coil_direction_vec = next_coil_pos - coil_pos;
                let coil_length = nalgebra_glm::length(&coil_direction_vec);

                let coil_rotation = Self::rotation_from_to_direction(
                    nalgebra_glm::vec3(0.0, 1.0, 0.0),
                    coil_direction_vec,
                );

                let midpoint = (coil_pos + next_coil_pos) * 0.5;

                if let Some(transform) = world.core.get_local_transform_mut(coil_entity) {
                    transform.translation = midpoint;
                    transform.rotation = coil_rotation;
                    transform.scale = nalgebra_glm::vec3(0.015, coil_length.max(0.01), 0.015);
                }
                nightshade::ecs::transform::commands::mark_local_transform_dirty(
                    world,
                    coil_entity,
                );
            }
        }
    }

    fn rotation_from_to_direction(from: Vec3, to: Vec3) -> nalgebra_glm::Quat {
        let to_normalized = nalgebra_glm::normalize(&to);
        let from_normalized = nalgebra_glm::normalize(&from);

        let dot: f32 = from_normalized.dot(&to_normalized);

        if dot > 0.9999 {
            return nalgebra_glm::quat_identity();
        }

        if dot < -0.9999 {
            let mut axis =
                nalgebra_glm::cross(&nalgebra_glm::vec3(1.0, 0.0, 0.0), &from_normalized);
            if nalgebra_glm::length(&axis) < 0.0001 {
                axis = nalgebra_glm::cross(&nalgebra_glm::vec3(0.0, 1.0, 0.0), &from_normalized);
            }
            axis = nalgebra_glm::normalize(&axis);
            return nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &axis);
        }

        let axis = nalgebra_glm::cross(&from_normalized, &to_normalized);
        let s = ((1.0 + dot) * 2.0).sqrt();
        let inv_s = 1.0 / s;

        nalgebra_glm::quat(axis.x * inv_s, axis.y * inv_s, axis.z * inv_s, s * 0.5)
    }

    fn spawn_revolute_joint_exhibit(&mut self, world: &mut World, center: Vec3) {
        spawn_label(
            world,
            "Revolute Joint",
            nalgebra_glm::vec3(center.x, 4.0, center.z),
            TextProperties {
                font_size: 24.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        let hinge_height = 3.0;
        let arm_length = 1.2;
        let arm_thickness = 0.1;
        let weight_radius = 0.15;

        let bracket_material =
            create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.3, 0.7);
        let bracket_entity = spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, hinge_height, center.z),
            nalgebra_glm::vec3(0.2, 0.2, 0.2),
            bracket_material,
        );

        let arm_center_y = hinge_height - arm_length / 2.0;
        let arm_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.25, 0.25), 0.6, 0.3);
        let arm_entity = spawn_dynamic_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, arm_center_y, center.z),
            nalgebra_glm::vec3(arm_thickness, arm_length, arm_thickness),
            1.5,
            arm_material,
        );
        self.physics_objects.push(arm_entity);

        let weight_y = hinge_height - arm_length;
        let weight_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.3, 0.7), 0.4, 0.5);
        let weight_entity = spawn_dynamic_physics_sphere_with_material(
            world,
            nalgebra_glm::vec3(center.x, weight_y - weight_radius, center.z),
            weight_radius,
            4.0,
            weight_material,
        );
        self.physics_objects.push(weight_entity);

        create_revolute_joint(
            world,
            bracket_entity,
            arm_entity,
            RevoluteJoint::new(JointAxisDirection::Z)
                .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.1, 0.0))
                .with_local_anchor2(nalgebra_glm::vec3(0.0, arm_length / 2.0, 0.0)),
        );

        create_fixed_joint(
            world,
            arm_entity,
            weight_entity,
            FixedJoint::new()
                .with_local_anchor1(nalgebra_glm::vec3(0.0, -arm_length / 2.0, 0.0))
                .with_local_anchor2(nalgebra_glm::vec3(0.0, weight_radius, 0.0)),
        );
    }

    fn spawn_velocity_friction_joint_exhibit(&mut self, world: &mut World, center: Vec3) {
        spawn_label(
            world,
            "Velocity Friction",
            nalgebra_glm::vec3(center.x, 4.0, center.z),
            TextProperties {
                font_size: 24.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        let hinge_height = 3.0;
        let arm_length = 1.2;
        let arm_thickness = 0.1;
        let weight_radius = 0.15;
        let damping_factor = 2.0;

        let bracket_material =
            create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.3, 0.7);
        let bracket_entity = spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, hinge_height, center.z),
            nalgebra_glm::vec3(0.2, 0.2, 0.2),
            bracket_material,
        );

        let arm_center_y = hinge_height - arm_length / 2.0;
        let arm_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.5, 0.25), 0.6, 0.3);
        let arm_entity = spawn_dynamic_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, arm_center_y, center.z),
            nalgebra_glm::vec3(arm_thickness, arm_length, arm_thickness),
            1.5,
            arm_material,
        );
        self.physics_objects.push(arm_entity);

        let weight_y = hinge_height - arm_length;
        let weight_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.5, 0.3), 0.4, 0.5);
        let weight_entity = spawn_dynamic_physics_sphere_with_material(
            world,
            nalgebra_glm::vec3(center.x, weight_y - weight_radius, center.z),
            weight_radius,
            4.0,
            weight_material,
        );
        self.physics_objects.push(weight_entity);

        create_revolute_joint(
            world,
            bracket_entity,
            arm_entity,
            RevoluteJoint::new(JointAxisDirection::Z)
                .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.1, 0.0))
                .with_local_anchor2(nalgebra_glm::vec3(0.0, arm_length / 2.0, 0.0)),
        );

        create_fixed_joint(
            world,
            arm_entity,
            weight_entity,
            FixedJoint::new()
                .with_local_anchor1(nalgebra_glm::vec3(0.0, -arm_length / 2.0, 0.0))
                .with_local_anchor2(nalgebra_glm::vec3(0.0, weight_radius, 0.0)),
        );

        self.velocity_friction_joints
            .push(VelocityFrictionJointState {
                arm_entity,
                damping_factor,
                initialized: false,
            });
    }

    fn spawn_coulomb_friction_joint_exhibit(&mut self, world: &mut World, center: Vec3) {
        spawn_label(
            world,
            "Coulomb Friction",
            nalgebra_glm::vec3(center.x, 4.0, center.z),
            TextProperties {
                font_size: 24.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        let hinge_height = 3.0;
        let arm_length = 1.2;
        let arm_thickness = 0.1;
        let weight_radius = 0.15;
        let friction_torque = 0.5;

        let bracket_material =
            create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.3, 0.7);
        let bracket_entity = spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, hinge_height, center.z),
            nalgebra_glm::vec3(0.2, 0.2, 0.2),
            bracket_material,
        );

        let arm_center_y = hinge_height - arm_length / 2.0;
        let arm_material = create_textured_material(nalgebra_glm::vec3(0.8, 0.4, 0.2), 0.6, 0.3);
        let arm_entity = spawn_dynamic_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, arm_center_y, center.z),
            nalgebra_glm::vec3(arm_thickness, arm_length, arm_thickness),
            1.5,
            arm_material,
        );
        self.physics_objects.push(arm_entity);

        let weight_y = hinge_height - arm_length;
        let weight_material = create_textured_material(nalgebra_glm::vec3(0.8, 0.4, 0.2), 0.4, 0.5);
        let weight_entity = spawn_dynamic_physics_sphere_with_material(
            world,
            nalgebra_glm::vec3(center.x, weight_y - weight_radius, center.z),
            weight_radius,
            4.0,
            weight_material,
        );
        self.physics_objects.push(weight_entity);

        create_revolute_joint(
            world,
            bracket_entity,
            arm_entity,
            RevoluteJoint::new(JointAxisDirection::Z)
                .with_local_anchor1(nalgebra_glm::vec3(0.0, -0.1, 0.0))
                .with_local_anchor2(nalgebra_glm::vec3(0.0, arm_length / 2.0, 0.0)),
        );

        create_fixed_joint(
            world,
            arm_entity,
            weight_entity,
            FixedJoint::new()
                .with_local_anchor1(nalgebra_glm::vec3(0.0, -arm_length / 2.0, 0.0))
                .with_local_anchor2(nalgebra_glm::vec3(0.0, weight_radius, 0.0)),
        );

        self.coulomb_friction_joints
            .push(CoulombFrictionJointState {
                arm_entity,
                friction_torque,
            });
    }

    fn update_coulomb_friction_joints(&mut self, world: &mut World) {
        for joint_state in &self.coulomb_friction_joints {
            let Some(rigid_body_component) = world.core.get_rigid_body(joint_state.arm_entity)
            else {
                continue;
            };
            let Some(handle) = rigid_body_component.handle else {
                continue;
            };
            let Some(rigid_body) = world
                .resources
                .physics
                .rigid_body_set
                .get_mut(handle.into())
            else {
                continue;
            };

            let angular_velocity = rigid_body.angvel();
            let angular_speed_z = angular_velocity.z;

            if angular_speed_z.abs() > 0.001 {
                let friction_direction = -angular_speed_z.signum();
                let friction_torque_vector = rapier3d::math::Vector::new(
                    0.0,
                    0.0,
                    friction_direction * joint_state.friction_torque,
                );
                rigid_body.apply_torque_impulse(friction_torque_vector, true);
            }
        }
    }

    fn setup_velocity_friction_joints(&mut self, world: &mut World) {
        for joint_state in &self.velocity_friction_joints {
            if joint_state.initialized {
                continue;
            }
            let Some(rigid_body_component) = world.core.get_rigid_body(joint_state.arm_entity)
            else {
                continue;
            };
            let Some(handle) = rigid_body_component.handle else {
                continue;
            };
            let Some(rigid_body) = world
                .resources
                .physics
                .rigid_body_set
                .get_mut(handle.into())
            else {
                continue;
            };

            rigid_body.set_angular_damping(joint_state.damping_factor);
        }
        for joint_state in &mut self.velocity_friction_joints {
            joint_state.initialized = true;
        }
    }

    fn spawn_room_walls(&self, world: &mut World, config: &RoomConfig) {
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

    fn spawn_room_light(&self, world: &mut World, position: Vec3, color: Vec3, intensity: f32) {
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

    fn spawn_curiosity_room(&mut self, world: &mut World, center: Vec3) {
        let room_height = 3.0;

        let config = RoomConfig {
            center,
            width: 4.0,
            depth: 4.0,
            height: room_height,
            wall_thickness: 0.15,
            doorway_width: 1.2,
            doorway_height: 2.3,
            wall_material: create_textured_material(
                nalgebra_glm::vec3(0.28, 0.22, 0.18),
                0.92,
                0.0,
            ),
            ceiling_material: create_textured_material(
                nalgebra_glm::vec3(0.3, 0.28, 0.25),
                0.95,
                0.0,
            ),
        };

        self.spawn_room_walls(world, &config);

        let front_z = center.z - config.depth / 2.0;
        spawn_label(
            world,
            "Curiosity Cabinet",
            nalgebra_glm::vec3(center.x, config.doorway_height + 0.25, front_z - 0.3),
            TextProperties {
                font_size: 20.0,
                color: Vec4::new(1.0, 0.9, 0.7, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.04,
                outline_color: Vec4::new(0.15, 0.1, 0.05, 1.0),
                ..Default::default()
            },
        );

        let back_wall_z = center.z + config.depth / 2.0 - config.wall_thickness - 0.05;
        spawn_label(
            world,
            "Take only what you need",
            nalgebra_glm::vec3(center.x, 2.0, back_wall_z),
            TextProperties {
                font_size: 12.0,
                color: Vec4::new(0.8, 0.75, 0.6, 0.9),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(0.1, 0.08, 0.05, 0.8),
                ..Default::default()
            },
        );

        self.spawn_room_light(
            world,
            nalgebra_glm::vec3(center.x, room_height - 0.3, center.z),
            nalgebra_glm::vec3(1.0, 0.9, 0.7),
            8.0,
        );

        let shelf_material = create_textured_material(nalgebra_glm::vec3(0.4, 0.3, 0.2), 0.75, 0.1);
        let shelf_y = 0.9;
        let shelf_z = center.z + 1.2;
        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, shelf_y, shelf_z),
            nalgebra_glm::vec3(2.0, 0.06, 0.5),
            shelf_material,
        );

        let gold_material = create_textured_material(nalgebra_glm::vec3(0.85, 0.7, 0.2), 0.3, 0.9);
        let gem_positions = [
            nalgebra_glm::vec3(center.x - 0.4, shelf_y + 0.03 + 0.1, shelf_z),
            nalgebra_glm::vec3(center.x + 0.5, shelf_y + 0.03 + 0.1, shelf_z - 0.1),
        ];
        for (index, position) in gem_positions.iter().enumerate() {
            let entity = spawn_dynamic_physics_sphere_with_material(
                world,
                *position,
                0.1,
                0.5,
                gold_material.clone(),
            );
            world
                .core
                .set_name(entity, Name(format!("Gold Gem {}", index + 1)));
            self.physics_objects.push(entity);
        }

        let crystal_material =
            create_textured_material(nalgebra_glm::vec3(0.2, 0.4, 0.9), 0.15, 0.7);
        let crystal_entity = spawn_dynamic_physics_sphere_with_material(
            world,
            nalgebra_glm::vec3(center.x + 0.1, shelf_y + 0.03 + 0.12, shelf_z + 0.1),
            0.12,
            0.8,
            crystal_material,
        );
        world
            .core
            .set_name(crystal_entity, Name("Crystal Orb".to_string()));
        self.physics_objects.push(crystal_entity);

        let trinket_material =
            create_textured_material(nalgebra_glm::vec3(0.6, 0.5, 0.35), 0.7, 0.0);
        let trinket_size = 0.12;
        let trinket_positions = [
            nalgebra_glm::vec3(center.x - 0.7, trinket_size / 2.0, center.z - 0.5),
            nalgebra_glm::vec3(center.x + 0.8, trinket_size / 2.0, center.z + 0.3),
        ];
        for (index, position) in trinket_positions.iter().enumerate() {
            let entity = spawn_dynamic_physics_cube_with_material(
                world,
                *position,
                nalgebra_glm::vec3(trinket_size, trinket_size, trinket_size),
                0.8,
                trinket_material.clone(),
            );
            world
                .core
                .set_name(entity, Name(format!("Trinket Box {}", index + 1)));
            self.physics_objects.push(entity);
        }

        let vase_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.3, 0.25), 0.5, 0.2);
        let vase_entity = spawn_dynamic_physics_cylinder_with_material(
            world,
            nalgebra_glm::vec3(center.x - 0.6, 0.2, center.z + 0.8),
            0.2,
            0.08,
            1.0,
            vase_material,
        );
        world
            .core
            .set_name(vase_entity, Name("Ceramic Vase".to_string()));
        self.physics_objects.push(vase_entity);

        let emerald_material =
            create_textured_material(nalgebra_glm::vec3(0.1, 0.7, 0.3), 0.2, 0.6);
        let emerald_entity = spawn_dynamic_physics_sphere_with_material(
            world,
            nalgebra_glm::vec3(center.x + 0.3, 0.08, center.z - 0.8),
            0.08,
            0.3,
            emerald_material,
        );
        world
            .core
            .set_name(emerald_entity, Name("Emerald".to_string()));
        self.physics_objects.push(emerald_entity);
    }

    fn spawn_workshop_room(&mut self, world: &mut World, center: Vec3) {
        let room_height = 3.0;

        let config = RoomConfig {
            center,
            width: 4.0,
            depth: 4.0,
            height: room_height,
            wall_thickness: 0.15,
            doorway_width: 1.2,
            doorway_height: 2.3,
            wall_material: create_textured_material(
                nalgebra_glm::vec3(0.22, 0.22, 0.24),
                0.9,
                0.05,
            ),
            ceiling_material: create_textured_material(
                nalgebra_glm::vec3(0.25, 0.25, 0.27),
                0.95,
                0.0,
            ),
        };

        self.spawn_room_walls(world, &config);

        let front_z = center.z - config.depth / 2.0;
        spawn_label(
            world,
            "Workshop",
            nalgebra_glm::vec3(center.x, config.doorway_height + 0.25, front_z - 0.3),
            TextProperties {
                font_size: 20.0,
                color: Vec4::new(0.9, 0.95, 1.0, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.04,
                outline_color: Vec4::new(0.08, 0.08, 0.12, 1.0),
                ..Default::default()
            },
        );

        let back_wall_z = center.z + config.depth / 2.0 - config.wall_thickness - 0.05;
        spawn_label(
            world,
            "Mind the sharp edges",
            nalgebra_glm::vec3(center.x, 2.0, back_wall_z),
            TextProperties {
                font_size: 12.0,
                color: Vec4::new(0.85, 0.85, 0.9, 0.9),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(0.08, 0.08, 0.1, 0.8),
                ..Default::default()
            },
        );

        self.spawn_room_light(
            world,
            nalgebra_glm::vec3(center.x, room_height - 0.3, center.z),
            nalgebra_glm::vec3(0.9, 0.95, 1.0),
            10.0,
        );

        let bench_material =
            create_textured_material(nalgebra_glm::vec3(0.35, 0.28, 0.18), 0.8, 0.05);
        let bench_y = 0.8;
        let bench_z = center.z + 1.0;
        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, bench_y / 2.0, bench_z),
            nalgebra_glm::vec3(2.4, bench_y, 0.7),
            bench_material,
        );

        let metal_material =
            create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.3, 0.85);
        let tool_configs = [
            (
                nalgebra_glm::vec3(center.x - 0.5, bench_y + 0.1, bench_z),
                0.1,
                0.05,
            ),
            (
                nalgebra_glm::vec3(center.x + 0.3, bench_y + 0.08, bench_z + 0.1),
                0.08,
                0.04,
            ),
            (
                nalgebra_glm::vec3(center.x + 0.7, bench_y + 0.12, bench_z - 0.1),
                0.12,
                0.035,
            ),
        ];
        for (index, (position, half_height, radius)) in tool_configs.iter().enumerate() {
            let entity = spawn_dynamic_physics_cylinder_with_material(
                world,
                *position,
                *half_height,
                *radius,
                2.0,
                metal_material.clone(),
            );
            world
                .core
                .set_name(entity, Name(format!("Metal Part {}", index + 1)));
            self.physics_objects.push(entity);
        }

        let brick_material =
            create_textured_material(nalgebra_glm::vec3(0.65, 0.25, 0.2), 0.85, 0.0);
        let brick_size = 0.15;
        let brick_positions = [
            nalgebra_glm::vec3(center.x - 0.7, brick_size / 2.0, center.z - 0.5),
            nalgebra_glm::vec3(center.x - 0.5, brick_size / 2.0, center.z - 0.7),
        ];
        for (index, position) in brick_positions.iter().enumerate() {
            let entity = spawn_dynamic_physics_cube_with_material(
                world,
                *position,
                nalgebra_glm::vec3(brick_size, brick_size, brick_size),
                3.0,
                brick_material.clone(),
            );
            world
                .core
                .set_name(entity, Name(format!("Brick {}", index + 1)));
            self.physics_objects.push(entity);
        }

        let orb_material = create_textured_material(nalgebra_glm::vec3(0.2, 0.7, 0.3), 0.4, 0.5);
        let orb_entity = spawn_dynamic_physics_sphere_with_material(
            world,
            nalgebra_glm::vec3(center.x + 0.6, 0.12, center.z - 0.6),
            0.12,
            1.0,
            orb_material,
        );
        world
            .core
            .set_name(orb_entity, Name("Green Orb".to_string()));
        self.physics_objects.push(orb_entity);

        let gear_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.55, 0.5), 0.25, 0.9);
        let gear_entity = spawn_dynamic_physics_cylinder_with_material(
            world,
            nalgebra_glm::vec3(center.x + 0.2, bench_y + 0.06, bench_z - 0.2),
            0.03,
            0.1,
            1.5,
            gear_material,
        );
        world
            .core
            .set_name(gear_entity, Name("Brass Gear".to_string()));
        self.physics_objects.push(gear_entity);

        let bolt_material = create_textured_material(nalgebra_glm::vec3(0.4, 0.4, 0.45), 0.2, 0.9);
        let bolt_positions = [
            nalgebra_glm::vec3(center.x - 0.2, bench_y + 0.05, bench_z + 0.2),
            nalgebra_glm::vec3(center.x + 0.6, bench_y + 0.05, bench_z + 0.15),
        ];
        for (index, position) in bolt_positions.iter().enumerate() {
            let entity = spawn_dynamic_physics_cube_with_material(
                world,
                *position,
                nalgebra_glm::vec3(0.06, 0.06, 0.06),
                0.5,
                bolt_material.clone(),
            );
            world
                .core
                .set_name(entity, Name(format!("Bolt {}", index + 1)));
            self.physics_objects.push(entity);
        }
    }

    fn spawn_recall_pedestal(&mut self, world: &mut World, center: Vec3) {
        let pedestal_material =
            create_textured_material(nalgebra_glm::vec3(0.3, 0.3, 0.35), 0.85, 0.0);

        let pedestal_height = 1.0;
        let pedestal_width = 0.4;

        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, pedestal_height / 2.0, center.z),
            nalgebra_glm::vec3(pedestal_width, pedestal_height, pedestal_width),
            pedestal_material,
        );

        let button_radius = 0.12;
        let button_height = 0.06;
        let button_base_y = pedestal_height + button_height / 2.0;

        let mut button_material =
            create_textured_material(nalgebra_glm::vec3(0.8, 0.15, 0.15), 0.3, 0.6);
        button_material.emissive_factor = [0.4, 0.05, 0.05];

        let button_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | VISIBILITY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(button_entity) {
            name.0 = "Recall Button".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(button_entity) {
            transform.translation = nalgebra_glm::vec3(center.x, button_base_y, center.z);
            transform.scale = nalgebra_glm::vec3(button_radius, button_height / 2.0, button_radius);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(button_entity) {
            mesh.name = "Cylinder".to_string();
        }

        let material_name = format!("Button_{}", button_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            button_material,
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
            .set_material_ref(button_entity, MaterialRef::new(material_name));

        if let Some(bv) = world.core.get_bounding_volume_mut(button_entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(button_entity) {
            *rigid_body = RigidBodyComponent::new_kinematic().with_translation(
                center.x,
                button_base_y,
                center.z,
            );
        }

        if let Some(collider) = world.core.get_collider_mut(button_entity) {
            *collider = ColliderComponent::new_cylinder(button_height / 2.0, button_radius);
        }

        let rigid_body_comp = world.core.get_rigid_body(button_entity).cloned().unwrap();
        let collider_comp = world.core.get_collider(button_entity).cloned();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(collider_comp) = collider_comp {
            let collider = collider_comp.to_rapier_collider();
            world.resources.physics.add_collider(collider, handle);
        }
        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(button_entity) {
            rigid_body_mut.handle = Some(handle.into());
        }

        self.buttons.push(ButtonState {
            entity: button_entity,
            base_position: nalgebra_glm::vec3(center.x, button_base_y, center.z),
            current_press: 0.0,
            is_pressed: false,
            action: ButtonAction::RecallBaubles,
        });
    }

    fn shoot_bauble(&mut self, world: &mut World, position: Vec3, direction: Vec3) {
        let bauble_radius = 0.05;
        let bauble_colors = [
            nalgebra_glm::vec3(0.9, 0.2, 0.2),
            nalgebra_glm::vec3(0.2, 0.8, 0.3),
            nalgebra_glm::vec3(0.2, 0.4, 0.9),
            nalgebra_glm::vec3(0.9, 0.8, 0.1),
        ];

        let color_index = (world.resources.window.timing.uptime_milliseconds / 100) as usize
            % bauble_colors.len();
        let color = bauble_colors[color_index];

        let entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | VISIBILITY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER
                | nightshade::ecs::world::COLLISION_LISTENER
                | nightshade::ecs::world::PHYSICS_INTERPOLATION,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(entity) {
            name.0 = "Shot Bauble".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
            transform.scale = nalgebra_glm::vec3(bauble_radius, bauble_radius, bauble_radius);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
            mesh.name = "Sphere".to_string();
        }

        let material_name = format!("ShotBauble_{}", entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            create_textured_material(color, 0.2, 0.8),
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

        if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Sphere");
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(entity) {
            *rigid_body = RigidBodyComponent::new_dynamic()
                .with_translation(position.x, position.y, position.z)
                .with_mass(0.05);
        }

        if let Some(collider) = world.core.get_collider_mut(entity) {
            *collider = ColliderComponent::new_ball(bauble_radius)
                .with_friction(0.5)
                .with_restitution(0.5);
        }

        let rigid_body_comp = world.core.get_rigid_body(entity).cloned().unwrap();
        let collider_comp = world.core.get_collider(entity).cloned();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(collider_comp) = collider_comp {
            let collider = collider_comp.to_rapier_collider();
            world.resources.physics.add_collider(collider, handle);
        }
        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(entity) {
            rigid_body_mut.handle = Some(handle.into());
        }
        world
            .resources
            .physics
            .handle_to_entity
            .insert(handle, entity);
        world
            .resources
            .physics
            .entity_to_handle
            .insert(entity, handle);

        world.resources.mesh_render_state.mark_entity_added(entity);
        self.physics_objects.push(entity);

        if let Some(interpolation) = world.core.get_physics_interpolation_mut(entity) {
            interpolation.previous_translation = position;
            interpolation.previous_rotation = nalgebra_glm::quat_identity();
            interpolation.current_translation = position;
            interpolation.current_rotation = nalgebra_glm::quat_identity();
            interpolation.enabled = true;
        }

        let shoot_speed = 15.0;
        let velocity = direction * shoot_speed;
        if let Some(rb) = world.resources.physics.rigid_body_set.get_mut(handle) {
            rb.set_linvel(
                rapier3d::math::Vector::new(velocity.x, velocity.y, velocity.z),
                true,
            );
        }

        self.shot_baubles.push(ShotBauble {
            entity,
            spawn_time_ms: world.resources.window.timing.uptime_milliseconds,
            original_scale: bauble_radius,
            landed: false,
        });
    }

    fn update_shot_baubles(&mut self, world: &mut World) {
        let current_time = world.resources.window.timing.uptime_milliseconds;

        while self.shot_baubles.len() > MAX_SHOT_BAUBLES {
            let bauble = self.shot_baubles.remove(0);
            self.despawn_bauble(world, bauble.entity);
        }

        let mut collided_entities = std::collections::HashSet::new();
        for event in world.resources.physics.collision_events() {
            if event.kind == CollisionEventKind::Started {
                collided_entities.insert(event.entity_a);
                collided_entities.insert(event.entity_b);
            }
        }

        let mut baubles_to_remove = Vec::new();
        let mut baubles_just_landed = Vec::new();

        for (index, bauble) in self.shot_baubles.iter().enumerate() {
            let age_ms = current_time.saturating_sub(bauble.spawn_time_ms);

            if !bauble.landed && collided_entities.contains(&bauble.entity) {
                baubles_just_landed.push((index, bauble.entity));
            }

            if age_ms >= BAUBLE_LIFETIME_MS {
                let shrink_progress_ms = age_ms - BAUBLE_LIFETIME_MS;
                let shrink_factor =
                    1.0 - (shrink_progress_ms as f32 / BAUBLE_SHRINK_DURATION_MS as f32);

                if shrink_factor <= 0.0 {
                    baubles_to_remove.push(index);
                } else {
                    let new_scale = bauble.original_scale * shrink_factor;
                    if let Some(transform) = world.core.get_local_transform_mut(bauble.entity) {
                        transform.scale = nalgebra_glm::vec3(new_scale, new_scale, new_scale);
                    }
                }
            }
        }

        for (index, entity) in baubles_just_landed {
            self.shot_baubles[index].landed = true;
            self.physics_objects.push(entity);
        }

        for index in baubles_to_remove.into_iter().rev() {
            let bauble = self.shot_baubles.remove(index);
            self.despawn_bauble(world, bauble.entity);
        }
    }

    fn despawn_bauble(&mut self, world: &mut World, entity: Entity) {
        if let Some(rigid_body) = world.core.get_rigid_body(entity)
            && let Some(handle) = rigid_body.handle
        {
            world.resources.physics.remove_rigid_body(handle.into());
        }

        self.physics_objects.retain(|e| *e != entity);
        world.despawn_entities(&[entity]);
    }

    fn spawn_visual_cube(
        &self,
        world: &mut World,
        position: Vec3,
        scale: Vec3,
        material: nightshade::ecs::material::components::Material,
        name: String,
    ) {
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

        if let Some(n) = world.core.get_name_mut(entity) {
            n.0 = name;
        }

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
            transform.scale = scale;
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
            mesh.name = "Cube".to_string();
        }

        let material_name = format!("VisualCube_{}", entity.id);
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

        if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
        }
    }

    fn detect_input_mode(&mut self, world: &mut World) {
        let keyboard = &world.resources.input.keyboard;
        let mouse = &world.resources.input.mouse;

        let has_keyboard_input = keyboard
            .keystates
            .values()
            .any(|state| *state == ElementState::Pressed);
        let has_mouse_input = mouse.raw_mouse_delta.x.abs() > 0.1
            || mouse.raw_mouse_delta.y.abs() > 0.1
            || mouse.state.contains(MouseState::LEFT_CLICKED)
            || mouse.state.contains(MouseState::RIGHT_CLICKED)
            || mouse.wheel_delta.y.abs() > 0.01;

        let has_gamepad_input = if let Some(gamepad) = query_active_gamepad(world) {
            let left_stick_x = gamepad.value(gilrs::Axis::LeftStickX);
            let left_stick_y = gamepad.value(gilrs::Axis::LeftStickY);
            let right_stick_x = gamepad.value(gilrs::Axis::RightStickX);
            let right_stick_y = gamepad.value(gilrs::Axis::RightStickY);
            let rt_value = gamepad.value(gilrs::Axis::RightZ);
            let lt_value = gamepad.value(gilrs::Axis::LeftZ);

            let deadzone = 0.15;
            left_stick_x.abs() > deadzone
                || left_stick_y.abs() > deadzone
                || right_stick_x.abs() > deadzone
                || right_stick_y.abs() > deadzone
                || rt_value > 0.3
                || lt_value > 0.3
                || gamepad.is_pressed(gilrs::Button::South)
                || gamepad.is_pressed(gilrs::Button::East)
                || gamepad.is_pressed(gilrs::Button::West)
                || gamepad.is_pressed(gilrs::Button::North)
                || gamepad.is_pressed(gilrs::Button::LeftTrigger)
                || gamepad.is_pressed(gilrs::Button::RightTrigger)
                || gamepad.is_pressed(gilrs::Button::LeftTrigger2)
                || gamepad.is_pressed(gilrs::Button::RightTrigger2)
                || gamepad.is_pressed(gilrs::Button::LeftThumb)
                || gamepad.is_pressed(gilrs::Button::RightThumb)
        } else {
            false
        };

        let previous_mode = self.input_mode;

        if has_gamepad_input && self.input_mode != InputMode::Gamepad {
            self.input_mode = InputMode::Gamepad;
        } else if (has_keyboard_input || has_mouse_input)
            && self.input_mode != InputMode::MouseKeyboard
        {
            self.input_mode = InputMode::MouseKeyboard;
        }

        if self.input_mode != previous_mode {
            world.resources.graphics.show_cursor = false;
            world.set_cursor_visible(false);

            if let Some(text_index) = self.input_mode_text_index {
                let text = match self.input_mode {
                    InputMode::MouseKeyboard => "Mouse/Keyboard",
                    InputMode::Gamepad => "Gamepad",
                    #[cfg(feature = "openxr")]
                    InputMode::Xr => "VR",
                };
                world.resources.text_cache.set_text(text_index, text);
                if let Some(entity) = self.input_mode_text_entity
                    && let Some(hud_text) = world.core.get_text_mut(entity)
                {
                    hud_text.dirty = true;
                }
            }
        }
    }

    fn camera_look_system(&mut self, world: &mut World) {
        let Some(camera_entity) = self.camera_entity else {
            return;
        };

        let is_manipulating = self.interaction.manipulated_door_index.is_some()
            || self.interaction.manipulated_drawer_index.is_some()
            || self.interaction.manipulated_lever_index.is_some()
            || self.interaction.manipulated_wheel_index.is_some();

        let (gamepad_right_stick_x, gamepad_right_stick_y) =
            if self.input_mode == InputMode::Gamepad && !is_manipulating {
                if let Some(gamepad) = query_active_gamepad(world) {
                    let deadzone = 0.15;
                    let raw_x = gamepad.value(gilrs::Axis::RightStickX);
                    let raw_y = gamepad.value(gilrs::Axis::RightStickY);
                    let magnitude = (raw_x * raw_x + raw_y * raw_y).sqrt();
                    if magnitude > deadzone {
                        let normalized = (magnitude - deadzone) / (1.0 - deadzone);
                        (
                            raw_x * normalized / magnitude,
                            raw_y * normalized / magnitude,
                        )
                    } else {
                        (0.0, 0.0)
                    }
                } else {
                    (0.0, 0.0)
                }
            } else {
                (0.0, 0.0)
            };

        let has_gamepad_input =
            gamepad_right_stick_x.abs() > 0.0 || gamepad_right_stick_y.abs() > 0.0;

        if self.input_mode == InputMode::MouseKeyboard {
            world.set_cursor_locked(true);
            world.set_cursor_visible(false);
        }

        let can_look_mouse =
            self.input_mode == InputMode::MouseKeyboard && !is_manipulating;

        if !can_look_mouse && !has_gamepad_input {
            return;
        }

        let dt = world.resources.window.timing.delta_time;

        let delta = if self.input_mode == InputMode::Gamepad && has_gamepad_input {
            let gamepad_sensitivity = 1.2;
            nalgebra_glm::vec2(
                gamepad_right_stick_x * gamepad_sensitivity * dt,
                -gamepad_right_stick_y * gamepad_sensitivity * dt,
            )
        } else if self.input_mode == InputMode::MouseKeyboard {
            let raw_delta = world.resources.input.mouse.raw_mouse_delta;
            let mouse_sensitivity = 0.002;
            raw_delta * mouse_sensitivity
        } else {
            return;
        };

        let yaw = nalgebra_glm::quat_angle_axis(-delta.x, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
        self.lean_state.base_rotation = yaw * self.lean_state.base_rotation;

        let forward = nalgebra_glm::quat_rotate_vec3(
            &self.lean_state.base_rotation,
            &nalgebra_glm::vec3(0.0, 0.0, -1.0),
        );
        let current_pitch = forward.y.asin();
        let new_pitch = current_pitch - delta.y;

        if new_pitch.abs() <= 85_f32.to_radians() {
            let pitch = nalgebra_glm::quat_angle_axis(-delta.y, &nalgebra_glm::vec3(1.0, 0.0, 0.0));
            self.lean_state.base_rotation *= pitch;
        }

        nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
    }

    fn crouch_camera_system(&mut self, world: &mut World) {
        let Some(player_entity) = self.player_entity else {
            return;
        };
        let Some(camera_entity) = self.camera_entity else {
            return;
        };

        let is_crouching = world
            .core
            .get_character_controller(player_entity)
            .map(|cc| cc.is_crouching)
            .unwrap_or(false);

        let target_height = if is_crouching {
            CROUCHING_CAMERA_HEIGHT
        } else {
            STANDING_CAMERA_HEIGHT
        };

        let dt = world.resources.window.timing.delta_time;

        let Some(camera_transform) = world.core.get_local_transform_mut(camera_entity) else {
            return;
        };

        let current_height = camera_transform.translation.y;
        let crouch_lerp_speed = 10.0;
        let new_height =
            current_height + (target_height - current_height) * (crouch_lerp_speed * dt).min(1.0);

        camera_transform.translation.y = new_height;
        nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
    }

    fn lean_system(&mut self, world: &mut World) {
        let Some(camera_entity) = self.camera_entity else {
            return;
        };

        let (lean_left_key, lean_right_key) = if self.input_mode == InputMode::MouseKeyboard {
            let keyboard = &world.resources.input.keyboard;
            (
                keyboard.is_key_pressed(KeyCode::KeyQ),
                keyboard.is_key_pressed(KeyCode::KeyE),
            )
        } else {
            (false, false)
        };

        let (gamepad_lean_left, gamepad_lean_right) = if self.input_mode == InputMode::Gamepad {
            if let Some(gamepad) = query_active_gamepad(world) {
                (
                    gamepad.is_pressed(gilrs::Button::LeftTrigger),
                    gamepad.is_pressed(gilrs::Button::RightTrigger),
                )
            } else {
                (false, false)
            }
        } else {
            (false, false)
        };

        let lean_left = lean_left_key || gamepad_lean_left;
        let lean_right = lean_right_key || gamepad_lean_right;

        self.lean_state.target_lean = if lean_left && !lean_right {
            -1.0
        } else if lean_right && !lean_left {
            1.0
        } else {
            0.0
        };

        let dt = world.resources.window.timing.delta_time;
        let lean_diff = self.lean_state.target_lean - self.lean_state.current_lean;
        self.lean_state.current_lean += lean_diff * (LEAN_SPEED * dt).min(1.0);

        let right_vector = nalgebra_glm::quat_rotate_vec3(
            &self.lean_state.base_rotation,
            &nalgebra_glm::vec3(1.0, 0.0, 0.0),
        );
        let horizontal_right =
            nalgebra_glm::normalize(&nalgebra_glm::vec3(right_vector.x, 0.0, right_vector.z));

        let lean_offset = horizontal_right * (self.lean_state.current_lean * LEAN_AMOUNT);

        let lean_roll = -self.lean_state.current_lean * LEAN_ANGLE;
        let roll_quat =
            nalgebra_glm::quat_angle_axis(lean_roll, &nalgebra_glm::vec3(0.0, 0.0, 1.0));

        let final_rotation = self.lean_state.base_rotation * roll_quat;

        let Some(camera_transform) = world.core.get_local_transform_mut(camera_entity) else {
            return;
        };

        camera_transform.translation.x = lean_offset.x;
        camera_transform.translation.z = lean_offset.z;
        camera_transform.rotation = final_rotation;

        nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
    }

    fn interaction_system(&mut self, world: &mut World) {
        let (left_clicked, _left_just_pressed, right_clicked, scroll_delta) =
            if self.input_mode == InputMode::MouseKeyboard {
                let mouse = &world.resources.input.mouse;
                (
                    mouse.state.contains(MouseState::LEFT_CLICKED),
                    mouse.state.contains(MouseState::LEFT_JUST_PRESSED),
                    mouse.state.contains(MouseState::RIGHT_CLICKED),
                    mouse.wheel_delta.y,
                )
            } else {
                (false, false, false, 0.0)
            };

        let (gamepad_rt_held, gamepad_lt_held, _gamepad_rt_just_pressed, gamepad_dpad_distance) =
            if self.input_mode == InputMode::Gamepad {
                if let Some(gamepad) = query_active_gamepad(world) {
                    let rt_axis_value = gamepad.value(gilrs::Axis::RightZ);
                    let lt_axis_value = gamepad.value(gilrs::Axis::LeftZ);
                    let rt_button = gamepad.is_pressed(gilrs::Button::RightTrigger2);
                    let lt_button = gamepad.is_pressed(gilrs::Button::LeftTrigger2);
                    let rt_held = rt_axis_value > 0.5 || rt_button;
                    let lt_held = lt_axis_value > 0.5 || lt_button;
                    let rt_just_pressed = rt_held && !self.interaction.gamepad_rt_was_pressed;
                    let dpad_up = gamepad.is_pressed(gilrs::Button::DPadUp);
                    let dpad_down = gamepad.is_pressed(gilrs::Button::DPadDown);
                    let dpad_distance: f32 = if dpad_up {
                        1.0
                    } else if dpad_down {
                        -1.0
                    } else {
                        0.0
                    };
                    (rt_held, lt_held, rt_just_pressed, dpad_distance)
                } else {
                    (false, false, false, 0.0)
                }
            } else {
                (false, false, false, 0.0)
            };

        self.interaction.gamepad_rt_was_pressed = if self.input_mode == InputMode::Gamepad {
            if let Some(gamepad) = query_active_gamepad(world) {
                gamepad.value(gilrs::Axis::RightZ) > 0.5
                    || gamepad.is_pressed(gilrs::Button::RightTrigger2)
            } else {
                false
            }
        } else {
            false
        };

        #[cfg(feature = "openxr")]
        let (xr_grip_held, xr_rt_held, xr_lt_held, xr_thumbstick_y) = {
            if let Some(xr_input) = &world.resources.xr.input {
                let grip_held = xr_input.right_grip_pressed();
                let rt_held = xr_input.right_trigger_pressed();
                let lt_held = xr_input.left_trigger_pressed();
                let thumbstick_y = xr_input.right_thumbstick.y;
                (grip_held, rt_held, lt_held, thumbstick_y)
            } else {
                (false, false, false, 0.0)
            }
        };

        #[cfg(not(feature = "openxr"))]
        let (xr_grip_held, xr_rt_held, xr_lt_held, xr_thumbstick_y) =
            (false, false, false, 0.0_f32);
        let _ = xr_lt_held;

        let interact_held = left_clicked || gamepad_lt_held || xr_grip_held;
        let throw_pressed = right_clicked || gamepad_rt_held || xr_lt_held;

        let keyboard_shoot_pressed = if self.input_mode == InputMode::MouseKeyboard {
            let keyboard = &world.resources.input.keyboard;
            keyboard.is_key_pressed(KeyCode::Enter)
        } else {
            false
        };
        let shoot_pressed = keyboard_shoot_pressed || gamepad_rt_held || xr_rt_held;

        let delta_time = world.resources.window.timing.delta_time;
        #[cfg(feature = "openxr")]
        let xr_distance_delta = if xr_thumbstick_y.abs() > 0.1 {
            xr_thumbstick_y * delta_time * 3.0
        } else {
            0.0
        };
        #[cfg(not(feature = "openxr"))]
        let xr_distance_delta = 0.0_f32;
        let _ = xr_thumbstick_y;

        let effective_scroll_delta =
            if self.input_mode == InputMode::Gamepad && gamepad_dpad_distance.abs() > 0.0 {
                gamepad_dpad_distance * delta_time * 3.0
            } else if xr_distance_delta.abs() > 0.0 {
                xr_distance_delta
            } else {
                scroll_delta
            };

        let Some(camera_entity) = self.camera_entity else {
            return;
        };
        let Some(camera_transform) = world.core.get_global_transform(camera_entity) else {
            return;
        };

        let camera_position = camera_transform.translation();
        let camera_forward = camera_transform.forward_vector();

        #[cfg(feature = "openxr")]
        let (shoot_origin, shoot_direction) = {
            if let Some(xr_input) = &world.resources.xr.input {
                if let (Some(hand_pos), Some(hand_rot)) = (
                    xr_input.right_hand_position(),
                    xr_input.right_hand_rotation(),
                ) {
                    let forward = nalgebra_glm::quat_rotate_vec3(
                        &hand_rot,
                        &nalgebra_glm::vec3(0.0, 1.0, 0.0),
                    );
                    (hand_pos, forward)
                } else {
                    (camera_position, camera_forward)
                }
            } else {
                (camera_position, camera_forward)
            }
        };
        #[cfg(not(feature = "openxr"))]
        let (shoot_origin, shoot_direction) = if let Some(weapon) = self.weapon_entity
            && let Some(weapon_transform) = world.core.get_global_transform(weapon)
        {
            let muzzle_local = nalgebra_glm::vec4(0.0, 0.005, -0.20, 1.0);
            let muzzle_world = weapon_transform.0 * muzzle_local;
            (muzzle_world.xyz(), camera_forward)
        } else {
            (camera_position, camera_forward)
        };

        let current_time_ms = world.resources.window.timing.uptime_milliseconds;
        let shoot_just_pressed = shoot_pressed && !self.interaction.shoot_was_pressed;
        self.interaction.shoot_was_pressed = shoot_pressed;

        if self.interaction.grabbed_entity.is_none() {
            if shoot_just_pressed {
                self.interaction.shoot_hold_start_ms = Some(current_time_ms);
                self.interaction.last_rapid_fire_ms = current_time_ms;
                self.shoot_bauble(world, shoot_origin, shoot_direction);
            } else if shoot_pressed {
                if let Some(hold_start) = self.interaction.shoot_hold_start_ms {
                    let hold_duration = current_time_ms.saturating_sub(hold_start);
                    if hold_duration > 200 {
                        let time_since_last_shot =
                            current_time_ms.saturating_sub(self.interaction.last_rapid_fire_ms);
                        if time_since_last_shot >= 80 {
                            self.interaction.last_rapid_fire_ms = current_time_ms;
                            self.shoot_bauble(world, shoot_origin, shoot_direction);
                        }
                    }
                }
            } else {
                self.interaction.shoot_hold_start_ms = None;
            }
        }

        if !interact_held {
            if let Some(button_index) = self.interaction.manipulated_button_index {
                self.release_button(world, button_index);
            }
            self.interaction.grabbed_entity = None;
            self.interaction.manipulated_door_index = None;
            self.interaction.manipulated_drawer_index = None;
            self.interaction.manipulated_lever_index = None;
            self.interaction.manipulated_wheel_index = None;
            self.interaction.manipulated_button_index = None;
            self.interaction.require_interact_release = false;
            return;
        }

        if self.interaction.require_interact_release {
            return;
        }

        if self.interaction.grabbed_entity.is_some() {
            self.update_grabbed_object(
                world,
                camera_position,
                camera_forward,
                effective_scroll_delta,
            );

            if throw_pressed {
                self.throw_grabbed_object(world, camera_forward);
                self.interaction.require_interact_release = true;
            }
            return;
        }

        if self.interaction.manipulated_door_index.is_some() {
            self.update_manipulated_door(world, camera_position);
            return;
        }

        if self.interaction.manipulated_drawer_index.is_some() {
            self.update_manipulated_drawer(world, camera_position);
            return;
        }

        if self.interaction.manipulated_lever_index.is_some() {
            self.update_manipulated_lever(world, camera_position);
            return;
        }

        if self.interaction.manipulated_wheel_index.is_some() {
            self.update_manipulated_wheel(world, camera_position);
            return;
        }

        if let Some(button_index) = self.interaction.manipulated_button_index {
            self.update_pressed_button(world, button_index);
            return;
        }

        let viewport_size = world
            .resources
            .window
            .cached_viewport_size
            .unwrap_or((800, 600));
        let screen_pos =
            nalgebra_glm::vec2(viewport_size.0 as f32 / 2.0, viewport_size.1 as f32 / 2.0);

        let options = PickingOptions {
            max_distance: GRAB_RANGE,
            ignore_invisible: true,
        };

        let pick_results = if self.input_mode == InputMode::Gamepad {
            self.pick_entities_cone(world, screen_pos, INTERACT_CONE_RADIUS, options)
        } else {
            pick_entities(world, screen_pos, options)
        };

        self.try_start_interaction(&pick_results);
    }

    fn try_start_interaction(&mut self, pick_results: &[PickingResult]) {
        for result in pick_results {
            if self.physics_objects.contains(&result.entity) {
                self.interaction.grabbed_entity = Some(result.entity);
                self.interaction.grab_distance = result.distance.min(MAX_GRAB_DISTANCE);
                return;
            }

            for (index, door) in self.doors.iter().enumerate() {
                if result.entity == door.entity && result.distance <= INTERACT_RANGE {
                    self.interaction.manipulated_door_index = Some(index);
                    return;
                }
            }

            for (index, drawer) in self.drawers.iter().enumerate() {
                if result.entity == drawer.front_entity && result.distance <= INTERACT_RANGE {
                    self.interaction.manipulated_drawer_index = Some(index);
                    return;
                }
            }

            for (index, lever) in self.levers.iter().enumerate() {
                if result.entity == lever.collider_entity && result.distance <= INTERACT_RANGE {
                    self.interaction.manipulated_lever_index = Some(index);
                    return;
                }
            }

            for (index, wheel) in self.wheels.iter().enumerate() {
                if result.entity == wheel.entity && result.distance <= INTERACT_RANGE {
                    self.interaction.manipulated_wheel_index = Some(index);
                    return;
                }
            }

            for (index, button) in self.buttons.iter().enumerate() {
                if result.entity == button.entity && result.distance <= INTERACT_RANGE {
                    self.interaction.manipulated_button_index = Some(index);
                    return;
                }
            }

            for (index, note) in self.notes.iter().enumerate() {
                if result.entity == note.entity && result.distance <= INTERACT_RANGE {
                    self.reading_note = Some(index);
                    self.note_close_key_released = false;
                    self.interaction.require_interact_release = true;
                    return;
                }
            }
        }
    }

    fn update_grabbed_object(
        &mut self,
        world: &mut World,
        camera_position: Vec3,
        camera_forward: Vec3,
        scroll_delta: f32,
    ) {
        self.interaction.grab_distance = (self.interaction.grab_distance
            + scroll_delta * SCROLL_DISTANCE_SPEED)
            .clamp(MIN_GRAB_DISTANCE, MAX_GRAB_DISTANCE);

        let target_position = camera_position + camera_forward * self.interaction.grab_distance;

        let Some(grabbed_entity) = self.interaction.grabbed_entity else {
            return;
        };

        let Some(rigid_body_component) = world.core.get_rigid_body(grabbed_entity) else {
            return;
        };
        let Some(handle) = rigid_body_component.handle else {
            return;
        };
        let Some(rigid_body) = world
            .resources
            .physics
            .rigid_body_set
            .get_mut(handle.into())
        else {
            return;
        };

        let current_pos = rigid_body.translation();
        let current_position = nalgebra_glm::vec3(current_pos.x, current_pos.y, current_pos.z);

        let displacement = target_position - current_position;

        let current_vel = rigid_body.linvel();
        let current_velocity = nalgebra_glm::vec3(current_vel.x, current_vel.y, current_vel.z);

        let mass = rigid_body.mass();
        let critical_damping = 2.0 * (GRAB_STIFFNESS * mass).sqrt();
        let damping = critical_damping * GRAB_DAMPING_RATIO;

        let spring_force = displacement * GRAB_STIFFNESS;
        let damping_force = -current_velocity * damping;
        let mut total_force = spring_force + damping_force;

        let force_magnitude = nalgebra_glm::length(&total_force);
        let max_force_for_mass = MAX_GRAB_FORCE * mass.max(0.5);
        if force_magnitude > max_force_for_mass {
            total_force *= max_force_for_mass / force_magnitude;
        }

        let acceleration = total_force / mass;
        let dt = world.resources.physics.fixed_timestep;
        let new_velocity = current_velocity + acceleration * dt;

        rigid_body.set_linvel(
            rapier3d::math::Vector::new(new_velocity.x, new_velocity.y, new_velocity.z),
            true,
        );

        let current_angvel = rigid_body.angvel();
        let angular_decay = (-ANGULAR_DAMPING * dt * 60.0).exp();
        rigid_body.set_angvel(current_angvel * angular_decay, true);
    }

    fn throw_grabbed_object(&mut self, world: &mut World, camera_forward: Vec3) {
        let Some(grabbed_entity) = self.interaction.grabbed_entity else {
            return;
        };

        let Some(rigid_body_component) = world.core.get_rigid_body(grabbed_entity) else {
            return;
        };
        let Some(handle) = rigid_body_component.handle else {
            return;
        };
        let Some(rigid_body) = world
            .resources
            .physics
            .rigid_body_set
            .get_mut(handle.into())
        else {
            return;
        };

        let throw_velocity = camera_forward * THROW_STRENGTH;
        rigid_body.set_linvel(
            rapier3d::math::Vector::new(throw_velocity.x, throw_velocity.y, throw_velocity.z),
            true,
        );

        self.interaction.grabbed_entity = None;
    }

    fn update_manipulated_door(&mut self, world: &mut World, camera_position: Vec3) {
        let Some(door_index) = self.interaction.manipulated_door_index else {
            return;
        };
        let Some(door) = self.doors.get_mut(door_index) else {
            return;
        };

        let distance_to_hinge = nalgebra_glm::distance(&camera_position, &door.hinge_position);

        if distance_to_hinge > INTERACT_RANGE * 3.0 {
            self.interaction.manipulated_door_index = None;
            return;
        }

        let dt = world.resources.physics.fixed_timestep;

        let mouse_input = if self.input_mode == InputMode::MouseKeyboard {
            -world.resources.input.mouse.raw_mouse_delta.x * 0.8
        } else {
            0.0
        };

        let gamepad_input = if self.input_mode == InputMode::Gamepad {
            if let Some(gamepad) = query_active_gamepad(world) {
                let right_stick_y = gamepad.value(gilrs::Axis::RightStickY);
                let deadzone = 0.15;
                if right_stick_y.abs() > deadzone {
                    right_stick_y * 3.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        let torque = mouse_input + gamepad_input;
        let friction = 6.0;

        door.angular_velocity += torque * dt;
        door.angular_velocity -= door.angular_velocity * friction * dt;

        let angle_delta = door.angular_velocity * dt;
        let new_angle = (door.current_angle + angle_delta).clamp(door.min_angle, door.max_angle);

        if (new_angle - door.min_angle).abs() < 0.001 && door.angular_velocity < 0.0 {
            door.angular_velocity = -door.angular_velocity * 0.2;
        }
        if (new_angle - door.max_angle).abs() < 0.001 && door.angular_velocity > 0.0 {
            door.angular_velocity = -door.angular_velocity * 0.2;
        }

        door.current_angle = new_angle;

        self.apply_door_transform(world, door_index);
    }

    fn apply_door_transform(&self, world: &mut World, door_index: usize) {
        let Some(door) = self.doors.get(door_index) else {
            return;
        };

        let cos_angle = door.current_angle.cos();
        let sin_angle = door.current_angle.sin();
        let new_center_x = door.hinge_position.x + door.door_half_width * cos_angle;
        let new_center_z = door.hinge_position.z - door.door_half_width * sin_angle;

        if let Some(transform) = world.core.get_local_transform_mut(door.entity) {
            transform.translation.x = new_center_x;
            transform.translation.z = new_center_z;
            transform.rotation = nalgebra_glm::quat_angle_axis(
                door.current_angle,
                &nalgebra_glm::vec3(0.0, 1.0, 0.0),
            );
        }
        nightshade::ecs::transform::commands::mark_local_transform_dirty(world, door.entity);

        if let Some(rb) = world
            .resources
            .physics
            .rigid_body_set
            .get_mut(door.rigid_body_handle)
        {
            let rotation = rapier3d::na::UnitQuaternion::from_axis_angle(
                &rapier3d::na::Vector3::y_axis(),
                door.current_angle,
            );
            rb.set_translation(
                rapier3d::math::Vector::new(new_center_x, door.hinge_position.y, new_center_z),
                true,
            );
            rb.set_rotation(rotation, true);
        }
    }

    fn update_doors_momentum(&mut self, world: &mut World) {
        let dt = world.resources.physics.fixed_timestep;
        let friction = 2.0;

        for door_index in 0..self.doors.len() {
            if self.interaction.manipulated_door_index == Some(door_index) {
                continue;
            }

            let door = &mut self.doors[door_index];

            if door.angular_velocity.abs() < 0.01 {
                door.angular_velocity = 0.0;
                continue;
            }

            door.angular_velocity *= (-friction * dt).exp();

            let angle_delta = door.angular_velocity * dt;
            let new_angle =
                (door.current_angle + angle_delta).clamp(door.min_angle, door.max_angle);

            if (new_angle - door.min_angle).abs() < 0.001
                || (new_angle - door.max_angle).abs() < 0.001
            {
                door.angular_velocity = -door.angular_velocity * 0.3;
            }

            door.current_angle = new_angle;

            self.apply_door_transform(world, door_index);
        }
    }

    fn update_manipulated_drawer(&mut self, world: &mut World, camera_position: Vec3) {
        let Some(drawer_index) = self.interaction.manipulated_drawer_index else {
            return;
        };
        let Some(drawer) = self.drawers.get_mut(drawer_index) else {
            return;
        };

        let current_pos = nalgebra_glm::vec3(
            drawer.closed_position.x,
            drawer.closed_position.y,
            drawer.closed_position.z + drawer.current_offset,
        );
        let distance_to_drawer = nalgebra_glm::distance(&camera_position, &current_pos);

        if distance_to_drawer > INTERACT_RANGE * 3.0 {
            self.interaction.manipulated_drawer_index = None;
            return;
        }

        let dt = world.resources.physics.fixed_timestep;

        let mouse_input = if self.input_mode == InputMode::MouseKeyboard {
            world.resources.input.mouse.raw_mouse_delta.y * 1.2
        } else {
            0.0
        };

        let gamepad_input = if self.input_mode == InputMode::Gamepad {
            if let Some(gamepad) = query_active_gamepad(world) {
                let right_stick_y = gamepad.value(gilrs::Axis::RightStickY);
                let deadzone = 0.15;
                if right_stick_y.abs() > deadzone {
                    -right_stick_y * 3.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        let pull_force = mouse_input + gamepad_input;
        let friction = 8.0;

        drawer.velocity += pull_force * dt;
        drawer.velocity -= drawer.velocity * friction * dt;

        let offset_delta = drawer.velocity * dt;
        let new_offset = (drawer.current_offset + offset_delta).clamp(0.0, drawer.max_offset);

        if new_offset <= 0.001 && drawer.velocity < 0.0 {
            drawer.velocity = -drawer.velocity * 0.3;
        }
        if (new_offset - drawer.max_offset).abs() < 0.001 && drawer.velocity > 0.0 {
            drawer.velocity = -drawer.velocity * 0.3;
        }

        drawer.current_offset = new_offset;

        self.apply_drawer_transform(world, drawer_index);
    }

    fn apply_drawer_transform(&self, world: &mut World, drawer_index: usize) {
        let Some(drawer) = self.drawers.get(drawer_index) else {
            return;
        };

        let new_z = drawer.closed_position.z + drawer.current_offset;

        if let Some(transform) = world.core.get_local_transform_mut(drawer.entity) {
            transform.translation.z = new_z;
        }
        nightshade::ecs::transform::commands::mark_local_transform_dirty(world, drawer.entity);

        if let Some(rb) = world
            .resources
            .physics
            .rigid_body_set
            .get_mut(drawer.rigid_body_handle)
        {
            rb.set_translation(
                rapier3d::math::Vector::new(
                    drawer.closed_position.x,
                    drawer.closed_position.y,
                    new_z,
                ),
                true,
            );
        }
    }

    fn update_drawers_momentum(&mut self, world: &mut World) {
        let dt = world.resources.physics.fixed_timestep;
        let friction = 3.0;

        for drawer_index in 0..self.drawers.len() {
            if self.interaction.manipulated_drawer_index == Some(drawer_index) {
                continue;
            }

            let drawer = &mut self.drawers[drawer_index];

            if drawer.velocity.abs() < 0.01 {
                drawer.velocity = 0.0;
                continue;
            }

            drawer.velocity *= (-friction * dt).exp();

            let offset_delta = drawer.velocity * dt;
            let new_offset = (drawer.current_offset + offset_delta).clamp(0.0, drawer.max_offset);

            if new_offset <= 0.001 || (new_offset - drawer.max_offset).abs() < 0.001 {
                drawer.velocity = -drawer.velocity * 0.2;
            }

            drawer.current_offset = new_offset;

            self.apply_drawer_transform(world, drawer_index);
        }
    }

    fn update_manipulated_lever(&mut self, world: &mut World, camera_position: Vec3) {
        let Some(lever_index) = self.interaction.manipulated_lever_index else {
            return;
        };
        let Some(lever) = self.levers.get_mut(lever_index) else {
            return;
        };

        let distance = nalgebra_glm::distance(&camera_position, &lever.pivot_position);

        if distance > INTERACT_RANGE * 3.0 {
            self.interaction.manipulated_lever_index = None;
            return;
        }

        let dt = world.resources.physics.fixed_timestep;

        let mouse_input = if self.input_mode == InputMode::MouseKeyboard {
            world.resources.input.mouse.raw_mouse_delta.y * 1.5
        } else {
            0.0
        };

        let gamepad_input = if self.input_mode == InputMode::Gamepad {
            if let Some(gamepad) = query_active_gamepad(world) {
                let right_stick_y = gamepad.value(gilrs::Axis::RightStickY);
                let deadzone = 0.15;
                if right_stick_y.abs() > deadzone {
                    -right_stick_y * 3.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        let torque = mouse_input + gamepad_input;
        let friction = 5.0;

        lever.angular_velocity += torque * dt;
        lever.angular_velocity -= lever.angular_velocity * friction * dt;

        let angle_delta = lever.angular_velocity * dt;
        let new_angle = (lever.current_angle + angle_delta).clamp(lever.min_angle, lever.max_angle);

        if (new_angle - lever.min_angle).abs() < 0.001 && lever.angular_velocity < 0.0 {
            lever.angular_velocity = -lever.angular_velocity * 0.2;
        }
        if (new_angle - lever.max_angle).abs() < 0.001 && lever.angular_velocity > 0.0 {
            lever.angular_velocity = -lever.angular_velocity * 0.2;
        }

        lever.current_angle = new_angle;

        self.apply_lever_transform(world, lever_index);
    }

    fn apply_lever_transform(&self, world: &mut World, lever_index: usize) {
        let Some(lever) = self.levers.get(lever_index) else {
            return;
        };

        let rotation =
            nalgebra_glm::quat_angle_axis(lever.current_angle, &nalgebra_glm::vec3(1.0, 0.0, 0.0));

        if let Some(transform) = world.core.get_local_transform_mut(lever.pivot_entity) {
            transform.rotation = rotation;
        }
        nightshade::ecs::transform::commands::mark_local_transform_dirty(world, lever.pivot_entity);

        let local_offset = nalgebra_glm::vec3(0.0, 0.0, lever.arm_half_length);
        let rotated_offset = nalgebra_glm::quat_rotate_vec3(&rotation, &local_offset);
        let center_pos = lever.pivot_position + rotated_offset;

        if let Some(transform) = world.core.get_local_transform_mut(lever.collider_entity) {
            transform.translation = center_pos;
            transform.rotation = rotation;
        }
        nightshade::ecs::transform::commands::mark_local_transform_dirty(
            world,
            lever.collider_entity,
        );

        if let Some(rb) = world
            .resources
            .physics
            .rigid_body_set
            .get_mut(lever.collider_rb_handle)
        {
            use rapier3d::prelude::*;
            let rapier_rotation =
                rapier3d::na::UnitQuaternion::from_quaternion(rapier3d::na::Quaternion::new(
                    rotation.w,
                    rotation.coords.x,
                    rotation.coords.y,
                    rotation.coords.z,
                ));
            rb.set_position(
                Isometry::from_parts(
                    Translation::new(center_pos.x, center_pos.y, center_pos.z),
                    rapier_rotation,
                ),
                true,
            );
        }
    }

    fn update_levers_momentum(&mut self, world: &mut World) {
        let dt = world.resources.physics.fixed_timestep;
        let friction = 2.5;

        for lever_index in 0..self.levers.len() {
            if self.interaction.manipulated_lever_index == Some(lever_index) {
                continue;
            }

            let lever = &mut self.levers[lever_index];

            if lever.angular_velocity.abs() < 0.01 {
                lever.angular_velocity = 0.0;
                continue;
            }

            lever.angular_velocity *= (-friction * dt).exp();

            let angle_delta = lever.angular_velocity * dt;
            let new_angle =
                (lever.current_angle + angle_delta).clamp(lever.min_angle, lever.max_angle);

            if (new_angle - lever.min_angle).abs() < 0.001
                || (new_angle - lever.max_angle).abs() < 0.001
            {
                lever.angular_velocity = -lever.angular_velocity * 0.3;
            }

            lever.current_angle = new_angle;

            self.apply_lever_transform(world, lever_index);
        }
    }

    fn update_manipulated_wheel(&mut self, world: &mut World, camera_position: Vec3) {
        let Some(wheel_index) = self.interaction.manipulated_wheel_index else {
            return;
        };
        let Some(wheel) = self.wheels.get_mut(wheel_index) else {
            return;
        };

        let distance = nalgebra_glm::distance(&camera_position, &wheel.center_position);

        if distance > INTERACT_RANGE * 3.0 {
            self.interaction.manipulated_wheel_index = None;
            return;
        }

        let dt = world.resources.physics.fixed_timestep;

        let mouse_input = if self.input_mode == InputMode::MouseKeyboard {
            -world.resources.input.mouse.raw_mouse_delta.x * 2.0
        } else {
            0.0
        };

        let gamepad_input = if self.input_mode == InputMode::Gamepad {
            if let Some(gamepad) = query_active_gamepad(world) {
                let right_stick_x = gamepad.value(gilrs::Axis::RightStickX);
                let deadzone = 0.15;
                if right_stick_x.abs() > deadzone {
                    -right_stick_x * 3.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        let torque = mouse_input + gamepad_input;
        let friction = 3.0;

        wheel.angular_velocity += torque * dt;
        wheel.angular_velocity -= wheel.angular_velocity * friction * dt;

        wheel.current_angle += wheel.angular_velocity * dt;

        self.apply_wheel_transform(world, wheel_index);
    }

    fn apply_wheel_transform(&self, world: &mut World, wheel_index: usize) {
        let Some(wheel) = self.wheels.get(wheel_index) else {
            return;
        };

        let base_rotation = nalgebra_glm::quat_angle_axis(
            std::f32::consts::FRAC_PI_2,
            &nalgebra_glm::vec3(1.0, 0.0, 0.0),
        );
        let spin_rotation =
            nalgebra_glm::quat_angle_axis(wheel.current_angle, &nalgebra_glm::vec3(0.0, 0.0, 1.0));

        if let Some(transform) = world.core.get_local_transform_mut(wheel.entity) {
            transform.rotation = spin_rotation * base_rotation;
        }
        nightshade::ecs::transform::commands::mark_local_transform_dirty(world, wheel.entity);

        for spoke_entity in &wheel.spoke_entities {
            nightshade::ecs::transform::commands::mark_local_transform_dirty(world, *spoke_entity);
        }

        if let Some(rb) = world
            .resources
            .physics
            .rigid_body_set
            .get_mut(wheel.rigid_body_handle)
        {
            let base_rot = rapier3d::na::UnitQuaternion::from_axis_angle(
                &rapier3d::na::Vector3::x_axis(),
                std::f32::consts::FRAC_PI_2,
            );
            let spin_rot = rapier3d::na::UnitQuaternion::from_axis_angle(
                &rapier3d::na::Vector3::z_axis(),
                wheel.current_angle,
            );
            rb.set_rotation(spin_rot * base_rot, true);
        }
    }

    fn update_wheels_momentum(&mut self, world: &mut World) {
        let dt = world.resources.physics.fixed_timestep;
        let friction = 1.5;

        for wheel_index in 0..self.wheels.len() {
            if self.interaction.manipulated_wheel_index == Some(wheel_index) {
                continue;
            }

            let wheel = &mut self.wheels[wheel_index];

            if wheel.angular_velocity.abs() < 0.01 {
                wheel.angular_velocity = 0.0;
                continue;
            }

            wheel.angular_velocity *= (-friction * dt).exp();
            wheel.current_angle += wheel.angular_velocity * dt;

            self.apply_wheel_transform(world, wheel_index);
        }
    }

    fn update_lantern_light(&self, world: &mut World) {
        let Some(lantern_entity) = self.lantern_entity else {
            return;
        };
        let Some(light_entity) = self.lantern_light_entity else {
            return;
        };

        let lantern_position =
            if let Some(global_transform) = world.core.get_global_transform(lantern_entity) {
                global_transform.translation()
            } else {
                return;
            };

        if let Some(transform) = world.core.get_local_transform_mut(light_entity) {
            transform.translation = lantern_position;
        }
        world.mark_local_transform_dirty(light_entity);
    }

    fn update_flashlight(&mut self, world: &mut World) {
        let Some(flashlight_entity) = self.flashlight_entity else {
            return;
        };
        let Some(camera) = self.camera_entity else {
            return;
        };

        let f_pressed = world.resources.input.keyboard.is_key_pressed(KeyCode::KeyF);
        let gamepad_flashlight_pressed =
            if let Some(gamepad) = query_active_gamepad(world) {
                gamepad.is_pressed(gilrs::Button::DPadDown)
            } else {
                false
            };
        let flashlight_input = f_pressed || gamepad_flashlight_pressed;

        if flashlight_input && !self.flashlight_key_was_pressed {
            self.flashlight_on = !self.flashlight_on;
            if let Some(light) = world.core.get_light_mut(flashlight_entity) {
                light.intensity = if self.flashlight_on { 60.0 } else { 0.0 };
            }
        }
        self.flashlight_key_was_pressed = flashlight_input;

        let (light_position, light_rotation) =
            if let Some(weapon) = self.weapon_entity
                && let Some(weapon_transform) = world.core.get_global_transform(weapon)
            {
                let muzzle_local = nalgebra_glm::vec4(0.0, 0.005, -0.20, 1.0);
                let muzzle_world = weapon_transform.0 * muzzle_local;
                let rotation = world
                    .core
                    .get_local_transform(camera)
                    .map(|t| t.rotation)
                    .unwrap_or(Quat::identity());
                (muzzle_world.xyz(), rotation)
            } else if let Some(camera_transform) =
                world.core.get_global_transform(camera).cloned()
            {
                let position =
                    camera_transform.translation() + camera_transform.forward_vector() * 0.3;
                let rotation = world
                    .core
                    .get_local_transform(camera)
                    .map(|t| t.rotation)
                    .unwrap_or(Quat::identity());
                (position, rotation)
            } else {
                return;
            };

        {
            let flashlight_transform = LocalTransform {
                translation: light_position,
                rotation: light_rotation,
                scale: Vec3::new(1.0, 1.0, 1.0),
            };

            world
                .core
                .set_local_transform(flashlight_entity, flashlight_transform);
            world
                .core
                .set_local_transform_dirty(flashlight_entity, LocalTransformDirty);
        }
    }

    fn update_pressed_button(&mut self, world: &mut World, button_index: usize) {
        let delta_time = world.resources.window.timing.delta_time;
        let press_speed = 8.0;
        let max_press = 0.03;

        let button = &mut self.buttons[button_index];
        button.current_press = (button.current_press + press_speed * delta_time).min(max_press);

        let pressed_y = button.base_position.y - button.current_press;
        if let Some(transform) = world.core.get_local_transform_mut(button.entity) {
            transform.translation.y = pressed_y;
        }
        world.mark_local_transform_dirty(button.entity);

        if let Some(rb) = world.core.get_rigid_body_mut(button.entity)
            && let Some(handle) = rb.handle
        {
            let physics = &mut world.resources.physics;
            if let Some(rigid_body) = physics.rigid_body_set.get_mut(handle.into()) {
                rigid_body.set_next_kinematic_translation(rapier3d::prelude::Vector::new(
                    button.base_position.x,
                    pressed_y,
                    button.base_position.z,
                ));
            }
        }

        if button.current_press >= max_press && !button.is_pressed {
            button.is_pressed = true;
            let action = button.action.clone();
            match action {
                ButtonAction::RecallBaubles => self.recall_baubles(world),
            }
        }
    }

    fn release_button(&mut self, world: &mut World, button_index: usize) {
        let button = &mut self.buttons[button_index];
        button.current_press = 0.0;
        button.is_pressed = false;

        if let Some(transform) = world.core.get_local_transform_mut(button.entity) {
            transform.translation.y = button.base_position.y;
        }
        world.mark_local_transform_dirty(button.entity);

        if let Some(rb) = world.core.get_rigid_body_mut(button.entity)
            && let Some(handle) = rb.handle
        {
            let physics = &mut world.resources.physics;
            if let Some(rigid_body) = physics.rigid_body_set.get_mut(handle.into()) {
                rigid_body.set_next_kinematic_translation(rapier3d::prelude::Vector::new(
                    button.base_position.x,
                    button.base_position.y,
                    button.base_position.z,
                ));
            }
        }
    }

    fn dash_system(&mut self, world: &mut World) {
        let Some(player_entity) = self.player_entity else {
            return;
        };
        let Some(camera_entity) = self.camera_entity else {
            return;
        };

        let grounded = world
            .core
            .get_character_controller(player_entity)
            .is_some_and(|controller| controller.grounded);

        let was_grounded_state = matches!(
            self.movement_state,
            MovementState::Grounded | MovementState::GroundDash
        );

        if grounded && !was_grounded_state {
            if let Some(new_state) = self.movement_state.process_event(MovementEvent::Land) {
                self.movement_state = new_state;
            }
        } else if !grounded && self.movement_state == MovementState::Grounded {
            if let Some(new_state) = self.movement_state.process_event(MovementEvent::Jump) {
                self.movement_state = new_state;
            }
        } else if !grounded && self.movement_state == MovementState::GroundDash
            && let Some(new_state) =
                self.movement_state.process_event(MovementEvent::BecomeAirborne)
        {
            self.movement_state = new_state;
        }

        let dash_pressed = if let Some(gamepad) = query_active_gamepad(world) {
            gamepad.is_pressed(gilrs::Button::East)
        } else {
            false
        };
        let dash_just_pressed = dash_pressed && !self.dash_button_was_pressed;
        self.dash_button_was_pressed = dash_pressed;

        let jump_pressed = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Space)
            || query_active_gamepad(world)
                .is_some_and(|gamepad| gamepad.is_pressed(gilrs::Button::South));
        let jump_just_pressed = jump_pressed && !self.jump_button_was_pressed;
        self.jump_button_was_pressed = jump_pressed;

        if jump_just_pressed && self.movement_state == MovementState::Airborne
            && let Some(new_state) =
                self.movement_state.process_event(MovementEvent::DoubleJump)
        {
            self.movement_state = new_state;
            if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
                controller.velocity.y = DOUBLE_JUMP_IMPULSE;
            }
        }

        if dash_just_pressed
            && self.dash_charges > 0
            && let Some(new_state) = self.movement_state.process_event(MovementEvent::Dash)
        {
            self.dash_charges -= 1;
            self.dash_cooldown_timer = DASH_COOLDOWN;
            self.movement_state = new_state;
            self.dash_timer = DASH_DURATION;

            let velocity = world
                .core
                .get_character_controller(player_entity)
                .map(|controller| controller.velocity)
                .unwrap_or(Vec3::zeros());

            let horizontal_velocity = nalgebra_glm::vec3(velocity.x, 0.0, velocity.z);
            let horizontal_speed = nalgebra_glm::length(&horizontal_velocity);

            self.dash_direction = if horizontal_speed > 0.1 {
                nalgebra_glm::normalize(&horizontal_velocity)
            } else {
                let forward = world
                    .core
                    .get_local_transform(camera_entity)
                    .map(|transform| transform.forward_vector())
                    .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, -1.0));
                nalgebra_glm::normalize(&nalgebra_glm::vec3(forward.x, 0.0, forward.z))
            };
        }

        let is_dashing = matches!(
            self.movement_state,
            MovementState::GroundDash | MovementState::AirDash
        );

        if is_dashing {
            let delta_time = world.resources.window.timing.delta_time;
            self.dash_timer -= delta_time;

            let elapsed = DASH_DURATION - self.dash_timer;
            let speed = DASH_INITIAL_SPEED * (-DASH_DECAY_RATE * elapsed).exp();

            if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
                controller.velocity.x = self.dash_direction.x * speed;
                controller.velocity.z = self.dash_direction.z * speed;
                if self.movement_state == MovementState::AirDash && elapsed < 0.08 {
                    controller.velocity.y = controller.velocity.y.max(0.0);
                }
            }

            if self.dash_timer <= 0.0 {
                if self.movement_state == MovementState::AirDash {
                    if let Some(new_state) =
                        self.movement_state.process_event(MovementEvent::DashEnd)
                    {
                        self.movement_state = new_state;
                    }
                } else if grounded
                    && let Some(new_state) =
                        self.movement_state.process_event(MovementEvent::Land)
                {
                    self.movement_state = new_state;
                }
            }
        }

        let delta_time = world.resources.window.timing.delta_time;
        if self.dash_charges < MAX_DASH_CHARGES {
            self.dash_cooldown_timer -= delta_time;
            if self.dash_cooldown_timer <= 0.0 {
                self.dash_charges += 1;
                if self.dash_charges < MAX_DASH_CHARGES {
                    self.dash_cooldown_timer = DASH_COOLDOWN;
                }
            }
        }

        self.update_dash_hud(world);
    }

    fn update_dash_hud(&mut self, world: &mut World) {
        if let Some(state_text) = self.dash_hud_state_text_entity {
            let label = match self.movement_state {
                MovementState::Grounded => "GROUNDED",
                MovementState::GroundDash => "DASH",
                MovementState::Airborne => "AIRBORNE",
                MovementState::DoubleJumped => "DOUBLE JUMP",
                MovementState::AirDash => "AIR DASH",
                MovementState::Falling => "FALLING",
            };
            world.ui_set_text(state_text, label);

            let text_color = match self.movement_state {
                MovementState::Grounded => nalgebra_glm::Vec4::new(0.6, 0.8, 0.6, 0.8),
                MovementState::GroundDash | MovementState::AirDash => {
                    nalgebra_glm::Vec4::new(0.3, 0.9, 1.0, 1.0)
                }
                MovementState::Airborne => nalgebra_glm::Vec4::new(0.8, 0.8, 0.5, 0.8),
                MovementState::DoubleJumped => nalgebra_glm::Vec4::new(1.0, 0.7, 0.3, 0.9),
                MovementState::Falling => nalgebra_glm::Vec4::new(0.7, 0.5, 0.5, 0.7),
            };
            if let Some(node_color) = world.ui.get_ui_node_color_mut(state_text) {
                node_color.colors[0] = Some(text_color);
                node_color.computed_color = text_color;
            }
        }

        let cooldown_fraction = if self.dash_charges < MAX_DASH_CHARGES {
            1.0 - (self.dash_cooldown_timer / DASH_COOLDOWN).clamp(0.0, 1.0)
        } else {
            1.0
        };

        for (index, &charge_entity) in self.dash_hud_charge_entities.iter().enumerate() {
            let charged = (index as u32) < self.dash_charges;
            let is_next_charge =
                !charged && (index as u32) == self.dash_charges;

            let fill_color = if charged {
                nalgebra_glm::Vec4::new(0.15, 0.5, 0.7, 0.8)
            } else if is_next_charge {
                let brightness = cooldown_fraction * 0.5;
                nalgebra_glm::Vec4::new(0.1 * brightness, 0.3 * brightness, 0.5 * brightness, 0.4)
            } else {
                nalgebra_glm::Vec4::new(0.08, 0.08, 0.1, 0.3)
            };

            if let Some(node_color) = world.ui.get_ui_node_color_mut(charge_entity) {
                node_color.colors[0] = Some(fill_color);
                node_color.computed_color = fill_color;
            }
        }
    }

    fn update_weapon_sway(&mut self, world: &mut World) {
        let Some(weapon_entity) = self.weapon_entity else {
            return;
        };
        let Some(camera_entity) = self.camera_entity else {
            return;
        };

        let forward = world
            .core
            .get_local_transform(camera_entity)
            .map(|transform| transform.forward_vector())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, -1.0));

        let current_yaw = forward.x.atan2(-forward.z);
        let current_pitch = forward.y.asin();

        let yaw_delta = current_yaw - self.weapon_previous_yaw;
        let pitch_delta = current_pitch - self.weapon_previous_pitch;
        self.weapon_previous_yaw = current_yaw;
        self.weapon_previous_pitch = current_pitch;

        let sway_strength = 0.6;
        self.weapon_sway.x -= yaw_delta * sway_strength;
        self.weapon_sway.y -= pitch_delta * sway_strength;

        let max_sway = 0.08;
        self.weapon_sway.x = self.weapon_sway.x.clamp(-max_sway, max_sway);
        self.weapon_sway.y = self.weapon_sway.y.clamp(-max_sway, max_sway);

        let delta_time = world.resources.window.timing.delta_time;
        let recovery_speed = 8.0;
        let decay = (-recovery_speed * delta_time).exp();
        self.weapon_sway.x *= decay;
        self.weapon_sway.y *= decay;

        if let Some(transform) = world.core.get_local_transform_mut(weapon_entity) {
            transform.translation =
                nalgebra_glm::vec3(0.15 + self.weapon_sway.x, -0.10 + self.weapon_sway.y, -0.25);
        }
        nightshade::ecs::transform::commands::mark_local_transform_dirty(world, weapon_entity);
    }

    fn recall_baubles(&mut self, world: &mut World) {
        for bauble in &self.baubles {
            if let Some(transform) = world.core.get_local_transform_mut(bauble.entity) {
                transform.translation = bauble.spawn_position;
            }
            world.mark_local_transform_dirty(bauble.entity);

            if let Some(rb) = world.core.get_rigid_body_mut(bauble.entity)
                && let Some(handle) = rb.handle
            {
                let physics = &mut world.resources.physics;
                if let Some(rigid_body) = physics.rigid_body_set.get_mut(handle.into()) {
                    rigid_body.set_translation(
                        rapier3d::prelude::Vector::new(
                            bauble.spawn_position.x,
                            bauble.spawn_position.y,
                            bauble.spawn_position.z,
                        ),
                        true,
                    );
                    rigid_body.set_linvel(rapier3d::prelude::Vector::zeros(), true);
                    rigid_body.set_angvel(rapier3d::prelude::Vector::zeros(), true);
                }
            }
        }
    }

    fn update_interaction_prompt(&self, world: &mut World) {
        let Some(text_index) = self.interaction_prompt_text_index else {
            return;
        };
        let Some(prompt_entity) = self.interaction_prompt_entity else {
            return;
        };

        let viewport_size = world
            .resources
            .window
            .cached_viewport_size
            .unwrap_or((800, 600));

        if self.interaction.grabbed_entity.is_some()
            || self.interaction.manipulated_door_index.is_some()
            || self.interaction.manipulated_drawer_index.is_some()
            || self.interaction.manipulated_lever_index.is_some()
            || self.interaction.manipulated_wheel_index.is_some()
            || self.interaction.manipulated_button_index.is_some()
            || self.reading_note.is_some()
        {
            world.resources.text_cache.set_text(text_index, "");
            if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
                hud_text.dirty = true;
            }
            return;
        }

        let screen_pos =
            nalgebra_glm::vec2(viewport_size.0 as f32 / 2.0, viewport_size.1 as f32 / 2.0);

        let options = PickingOptions {
            max_distance: GRAB_RANGE,
            ignore_invisible: true,
        };

        let pick_results = if self.input_mode == InputMode::Gamepad {
            self.pick_entities_cone(world, screen_pos, INTERACT_CONE_RADIUS, options)
        } else {
            pick_entities(world, screen_pos, options)
        };

        let mut can_interact = false;
        let mut can_read = false;

        for result in &pick_results {
            if self.physics_objects.contains(&result.entity) {
                can_interact = true;
                break;
            }

            for door in &self.doors {
                if result.entity == door.entity && result.distance <= INTERACT_RANGE {
                    can_interact = true;
                    break;
                }
            }
            if can_interact {
                break;
            }

            for drawer in &self.drawers {
                if result.entity == drawer.front_entity && result.distance <= INTERACT_RANGE {
                    can_interact = true;
                    break;
                }
            }
            if can_interact {
                break;
            }

            for lever in &self.levers {
                if result.entity == lever.collider_entity && result.distance <= INTERACT_RANGE {
                    can_interact = true;
                    break;
                }
            }
            if can_interact {
                break;
            }

            for wheel in &self.wheels {
                if result.entity == wheel.entity && result.distance <= INTERACT_RANGE {
                    can_interact = true;
                    break;
                }
            }
            if can_interact {
                break;
            }

            for button in &self.buttons {
                if result.entity == button.entity && result.distance <= INTERACT_RANGE {
                    can_interact = true;
                    break;
                }
            }
            if can_interact {
                break;
            }

            for note in &self.notes {
                if result.entity == note.entity && result.distance <= INTERACT_RANGE {
                    can_read = true;
                    break;
                }
            }
            if can_read {
                break;
            }
        }

        let prompt_text = if can_read {
            "Read"
        } else if can_interact {
            "Interact"
        } else {
            ""
        };

        world.resources.text_cache.set_text(text_index, prompt_text);
        if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
            hud_text.dirty = true;
        }

        let crosshair_color = if can_interact || can_read {
            nalgebra_glm::Vec4::new(0.2, 1.0, 0.2, 0.9)
        } else {
            nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 0.7)
        };
        for &arm in &self.crosshair_arms {
            if let Some(color) = world.ui.get_ui_node_color_mut(arm) {
                color.colors[0] = Some(crosshair_color);
                color.computed_color = crosshair_color;
            }
        }
    }

    fn pick_entities_cone(
        &self,
        world: &World,
        center: Vec2,
        radius: f32,
        options: PickingOptions,
    ) -> Vec<PickingResult> {
        let mut all_results: Vec<PickingResult> = Vec::new();
        let mut seen_entities = std::collections::HashSet::new();

        let offsets = [
            (0.0, 0.0),
            (1.0, 0.0),
            (-1.0, 0.0),
            (0.0, 1.0),
            (0.0, -1.0),
            (0.707, 0.707),
            (-0.707, 0.707),
            (0.707, -0.707),
            (-0.707, -0.707),
            (0.5, 0.0),
            (-0.5, 0.0),
            (0.0, 0.5),
            (0.0, -0.5),
        ];

        for (offset_x, offset_y) in offsets {
            let screen_pos =
                nalgebra_glm::vec2(center.x + offset_x * radius, center.y + offset_y * radius);

            let results = pick_entities(world, screen_pos, options);
            for result in results {
                if !seen_entities.contains(&result.entity) {
                    seen_entities.insert(result.entity);
                    all_results.push(result);
                }
            }
        }

        all_results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        all_results
    }

    fn debug_toggle_system(&mut self, world: &mut World) {
        let keyboard = &world.resources.input.keyboard;
        let key4_pressed = keyboard.is_key_pressed(KeyCode::Digit4);

        if key4_pressed && !self.key4_was_pressed {
            self.show_physics_debug = !self.show_physics_debug;
            world.resources.physics.debug_draw = self.show_physics_debug;
        }

        self.key4_was_pressed = key4_pressed;
    }

    fn note_reading_system(&mut self, world: &mut World) {
        let keyboard = &world.resources.input.keyboard;
        let f_pressed = keyboard.is_key_pressed(KeyCode::KeyF);

        let gamepad_rt_pressed = if let Some(gamepad) = query_active_gamepad(world) {
            let rt_axis = gamepad.value(gilrs::Axis::RightZ);
            let rt_button = gamepad.is_pressed(gilrs::Button::RightTrigger2);
            rt_axis > 0.5 || rt_button
        } else {
            false
        };

        let interact_pressed = f_pressed || gamepad_rt_pressed;

        if !self.note_close_key_released && !interact_pressed {
            self.note_close_key_released = true;
        }

        if self.note_close_key_released && interact_pressed {
            self.reading_note = None;
        }
    }

    fn check_fall_reset(&self, world: &mut World) {
        let Some(player_entity) = self.player_entity else {
            return;
        };

        let Some(transform) = world.core.get_local_transform(player_entity) else {
            return;
        };

        if transform.translation.y < -20.0 {
            let spawn_position = nalgebra_glm::vec3(0.0, 1.2, 8.0);

            if let Some(transform) = world.core.get_local_transform_mut(player_entity) {
                transform.translation = spawn_position;
            }

            if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
                controller.velocity = nalgebra_glm::vec3(0.0, 0.0, 0.0);
            }
        }
    }

    #[cfg(feature = "openxr")]
    fn spawn_hand_cube(&self, world: &mut World, color: Vec3) -> Entity {
        let cube_size = 0.08;
        let material = create_textured_material(color, 0.3, 0.7);

        let entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | VISIBILITY,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(entity) {
            name.0 = "HandCube".to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = nalgebra_glm::vec3(0.0, -100.0, 0.0);
            transform.scale = nalgebra_glm::vec3(cube_size, cube_size, cube_size);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
            mesh.name = "Cube".to_string();
        }

        let material_name = format!("HandCube_{}", entity.id);
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

        if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
            *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
        }

        entity
    }

    #[cfg(feature = "openxr")]
    fn spawn_bauble_gun(&mut self, world: &mut World, hand_entity: Entity) {
        let gun_body_material =
            create_textured_material(nalgebra_glm::vec3(0.15, 0.15, 0.18), 0.6, 0.8);
        let gun_barrel_material =
            create_textured_material(nalgebra_glm::vec3(0.25, 0.25, 0.28), 0.4, 0.9);
        let gun_grip_material =
            create_textured_material(nalgebra_glm::vec3(0.12, 0.08, 0.06), 0.9, 0.0);
        let gun_accent_material =
            create_textured_material(nalgebra_glm::vec3(0.9, 0.4, 0.1), 0.3, 0.7);

        let parts: Vec<(
            &str,
            &str,
            Vec3,
            Vec3,
            nightshade::ecs::material::components::Material,
        )> = vec![
            (
                "GunBody",
                "Cube",
                nalgebra_glm::vec3(0.0, 0.06, 0.0),
                nalgebra_glm::vec3(0.025, 0.015, 0.04),
                gun_body_material,
            ),
            (
                "GunBarrel",
                "Cylinder",
                nalgebra_glm::vec3(0.0, 0.12, 0.0),
                nalgebra_glm::vec3(0.008, 0.04, 0.008),
                gun_barrel_material,
            ),
            (
                "GunGrip",
                "Cube",
                nalgebra_glm::vec3(0.0, 0.0, 0.01),
                nalgebra_glm::vec3(0.015, 0.03, 0.012),
                gun_grip_material,
            ),
            (
                "GunMuzzle",
                "Sphere",
                nalgebra_glm::vec3(0.0, 0.165, 0.0),
                nalgebra_glm::vec3(0.012, 0.012, 0.012),
                gun_accent_material,
            ),
        ];

        for (name, mesh_name, offset, scale, material) in parts {
            let entity = world.spawn_entities(
                NAME | LOCAL_TRANSFORM
                    | GLOBAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | RENDER_MESH
                    | MATERIAL_REF
                    | BOUNDING_VOLUME
                    | VISIBILITY
                    | PARENT,
                1,
            )[0];

            if let Some(n) = world.core.get_name_mut(entity) {
                n.0 = name.to_string();
            }
            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                transform.translation = offset;
                transform.scale = scale;
            }
            if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
                mesh.name = mesh_name.to_string();
            }
            if let Some(parent) = world.core.get_parent_mut(entity) {
                *parent = Parent(Some(hand_entity));
            }
            if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
                *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type(mesh_name);
            }

            let material_name = format!("BaubleGun_{}_{}", name, entity.id);
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

            self.bauble_gun_entities.push(entity);
        }
    }

    #[cfg(feature = "openxr")]
    fn xr_hand_tracking_system(&mut self, world: &mut World) {
        let Some(xr_input) = world.resources.xr.input.clone() else {
            return;
        };

        if let Some(left_hand_entity) = self.left_hand_cube {
            if let Some(left_pos) = xr_input.left_hand_position() {
                if let Some(transform) = world.core.get_local_transform_mut(left_hand_entity) {
                    transform.translation = left_pos;
                }
                if let Some(rotation) = xr_input.left_hand_rotation() {
                    if let Some(transform) = world.core.get_local_transform_mut(left_hand_entity) {
                        transform.rotation = rotation;
                    }
                }
                world.mark_local_transform_dirty(left_hand_entity);
            }

            let left_trigger_pressed = xr_input.left_trigger_pressed();
            let hand_color = if left_trigger_pressed {
                [0.2, 0.9, 0.3, 1.0]
            } else {
                [0.2, 0.6, 0.9, 1.0]
            };
            let left_mat_name = world
                .core
                .get_material_ref(left_hand_entity)
                .map(|r| r.name.clone());
            if let Some(name) = left_mat_name
                && let Some(mat) = registry_entry_by_name_mut(
                    &mut world.resources.material_registry.registry,
                    &name,
                )
            {
                mat.base_color = hand_color;
            }
        }

        if let Some(right_hand_entity) = self.right_hand_cube {
            if let Some(right_pos) = xr_input.right_hand_position() {
                if let Some(transform) = world.core.get_local_transform_mut(right_hand_entity) {
                    transform.translation = right_pos;
                }
                if let Some(rotation) = xr_input.right_hand_rotation() {
                    if let Some(transform) = world.core.get_local_transform_mut(right_hand_entity) {
                        transform.rotation = rotation;
                    }
                }
                world.mark_local_transform_dirty(right_hand_entity);
            }

            let right_trigger_pressed = xr_input.right_trigger_pressed();
            let hand_color = if right_trigger_pressed {
                [0.2, 0.9, 0.3, 1.0]
            } else {
                [0.9, 0.6, 0.2, 1.0]
            };
            let right_mat_name = world
                .core
                .get_material_ref(right_hand_entity)
                .map(|r| r.name.clone());
            if let Some(name) = right_mat_name
                && let Some(mat) = registry_entry_by_name_mut(
                    &mut world.resources.material_registry.registry,
                    &name,
                )
            {
                mat.base_color = hand_color;
            }
        }
    }
}

fn spawn_weapon_part(
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
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type(mesh_name);
    }

    if let Some(p) = world.core.get_parent_mut(entity) {
        *p = Parent(Some(parent));
    }

    if let Some(render_layer) = world.core.get_render_layer_mut(entity) {
        render_layer.0 = nightshade::ecs::render_layer::components::RenderLayer::OVERLAY;
    }

    world.resources.mesh_render_state.mark_entity_added(entity);

    entity
}

fn spawn_weapon(world: &mut World, camera_entity: Entity) -> Entity {
    let root = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | PARENT,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(root) {
        name.0 = "Weapon".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(root) {
        transform.translation = nalgebra_glm::vec3(0.15, -0.10, -0.25);
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

fn build_note_overlay(world: &mut World) -> (Entity, Entity, Entity) {
    let mut tree = UiTreeBuilder::new(world);

    let panel_width = 500.0;
    let panel_height = 400.0;

    let overlay = tree
        .add_node()
        .boundary(
            Vp(nalgebra_glm::Vec2::new(50.0, 50.0))
                + Ab(nalgebra_glm::Vec2::new(
                    -panel_width / 2.0,
                    -panel_height / 2.0,
                )),
            Vp(nalgebra_glm::Vec2::new(50.0, 50.0))
                + Ab(nalgebra_glm::Vec2::new(
                    panel_width / 2.0,
                    panel_height / 2.0,
                )),
        )
        .with_rect(6.0, 2.0, nalgebra_glm::Vec4::new(0.471, 0.392, 0.275, 1.0))
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.961, 0.922, 0.824, 0.98))
        .with_visible(false)
        .without_pointer_events()
        .with_clip()
        .entity();

    tree.push_parent(overlay);

    let title_entity = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(20.0, 20.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(-20.0, 50.0)),
        )
        .with_text("", 20.0)
        .with_text_wrap()
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Top)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.196, 0.157, 0.118, 1.0))
        .without_pointer_events()
        .done();

    tree.add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(20.0, 56.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(-20.0, 57.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::zeros())
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.471, 0.392, 0.275, 0.5))
        .without_pointer_events();

    let content_entity = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(20.0, 70.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)) + Ab(nalgebra_glm::Vec2::new(-20.0, -20.0)),
        )
        .with_text("", 16.0)
        .with_text_wrap()
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.157, 0.137, 0.098, 1.0))
        .without_pointer_events()
        .done();

    tree.pop_parent();
    tree.finish();

    (overlay, title_entity, content_entity)
}

fn build_dash_hud(world: &mut World) -> (Entity, Entity, Vec<Entity>) {
    let mut tree = UiTreeBuilder::new(world);

    let panel_width = 140.0;
    let panel_height = 50.0;

    let container = tree
        .add_node()
        .boundary(
            Vp(nalgebra_glm::Vec2::new(50.0, 100.0))
                + Ab(nalgebra_glm::Vec2::new(-panel_width / 2.0, -panel_height - 15.0)),
            Vp(nalgebra_glm::Vec2::new(50.0, 100.0))
                + Ab(nalgebra_glm::Vec2::new(panel_width / 2.0, -15.0)),
        )
        .with_rect(6.0, 1.0, nalgebra_glm::Vec4::new(0.3, 0.8, 1.0, 0.3))
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.02, 0.04, 0.08, 0.6))
        .without_pointer_events()
        .entity();

    tree.push_parent(container);

    let state_text = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(0.0, 2.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 20.0)),
        )
        .with_text("GROUNDED", 11.0)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.6, 0.7, 0.8, 0.7))
        .without_pointer_events()
        .done();

    let charge_size = 20.0;
    let gap = 6.0;
    let total_width = charge_size * 2.0 + gap;
    let start_x = (panel_width - total_width) / 2.0;

    let mut charge_entities = Vec::new();
    for charge_index in 0..MAX_DASH_CHARGES {
        let offset_x = start_x + charge_index as f32 * (charge_size + gap);
        let charge = tree
            .add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(offset_x, 24.0)),
                Ab(nalgebra_glm::Vec2::new(offset_x + charge_size, 24.0 + charge_size)),
            )
            .with_rect(4.0, 1.5, nalgebra_glm::Vec4::new(0.3, 0.8, 1.0, 0.8))
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.15, 0.5, 0.7, 0.8))
            .without_pointer_events()
            .done();
        charge_entities.push(charge);
    }

    tree.pop_parent();
    tree.finish();

    (container, state_text, charge_entities)
}

fn build_crosshair(world: &mut World) -> (Entity, Vec<Entity>) {
    let mut tree = UiTreeBuilder::new(world);
    let center = nalgebra_glm::Vec2::new(50.0, 50.0);
    let color = nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 0.7);

    let container = tree
        .add_node()
        .boundary(
            Vp(center) + Ab(nalgebra_glm::Vec2::new(-10.0, -10.0)),
            Vp(center) + Ab(nalgebra_glm::Vec2::new(10.0, 10.0)),
        )
        .without_pointer_events()
        .entity();

    tree.push_parent(container);

    let left = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(2.0, 9.0)),
            Ab(nalgebra_glm::Vec2::new(7.0, 11.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::zeros())
        .with_color::<UiBase>(color)
        .without_pointer_events()
        .done();

    let right = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(13.0, 9.0)),
            Ab(nalgebra_glm::Vec2::new(18.0, 11.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::zeros())
        .with_color::<UiBase>(color)
        .without_pointer_events()
        .done();

    let top = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(9.0, 2.0)),
            Ab(nalgebra_glm::Vec2::new(11.0, 7.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::zeros())
        .with_color::<UiBase>(color)
        .without_pointer_events()
        .done();

    let bottom = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(9.0, 13.0)),
            Ab(nalgebra_glm::Vec2::new(11.0, 18.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::zeros())
        .with_color::<UiBase>(color)
        .without_pointer_events()
        .done();

    tree.pop_parent();
    tree.finish();

    (container, vec![left, right, top, bottom])
}

fn spawn_label(world: &mut World, text: &str, position: Vec3, properties: TextProperties) {
    let entity = spawn_3d_text_with_properties(world, text, position, properties);
    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.rotation =
            nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
    }
    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, entity);
}

fn spawn_flashlight(world: &mut World) -> Entity {
    let entity = world.spawn_entities(
        LIGHT | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
        1,
    )[0];

    world.core.set_light(
        entity,
        Light {
            light_type: LightType::Spot,
            color: nalgebra_glm::vec3(1.0, 0.95, 0.85),
            intensity: 60.0,
            range: 50.0,
            inner_cone_angle: 0.12,
            outer_cone_angle: 0.35,
            cast_shadows: true,
            shadow_bias: 0.0001,
        },
    );

    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);

    entity
}

fn spawn_sun_overhead(world: &mut World) -> Entity {
    use nightshade::ecs::world::components;
    use nightshade::ecs::world::{
        GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME,
    };

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | LIGHT,
        1,
    )[0];

    world
        .core
        .set_name(entity, components::Name("Sun".to_string()));
    world.core.set_local_transform(
        entity,
        components::LocalTransform {
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
        .set_local_transform_dirty(entity, components::LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, components::GlobalTransform::default());
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
