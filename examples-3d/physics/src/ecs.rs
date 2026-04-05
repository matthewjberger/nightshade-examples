use nightshade::prelude::Entity;
use nightshade::prelude::Vec3;
use nightshade::prelude::freecs;
use nightshade::prelude::nalgebra_glm;
use nightshade::prelude::rapier3d;

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

#[derive(Default, Clone, Debug)]
pub struct Door {
    pub entity: Entity,
    pub rigid_body_handle: rapier3d::prelude::RigidBodyHandle,
    pub hinge_position: Vec3,
    pub door_half_width: f32,
    pub current_angle: f32,
    pub angular_velocity: f32,
    pub min_angle: f32,
    pub max_angle: f32,
}

#[derive(Default, Clone, Debug)]
pub struct Drawer {
    pub entity: Entity,
    pub front_entity: Entity,
    pub rigid_body_handle: rapier3d::prelude::RigidBodyHandle,
    pub closed_position: Vec3,
    pub current_offset: f32,
    pub velocity: f32,
    pub max_offset: f32,
}

#[derive(Default, Clone, Debug)]
pub struct Lever {
    pub pivot_entity: Entity,
    pub collider_entity: Entity,
    pub collider_rb_handle: rapier3d::prelude::RigidBodyHandle,
    pub pivot_position: Vec3,
    pub arm_half_length: f32,
    pub current_angle: f32,
    pub angular_velocity: f32,
    pub min_angle: f32,
    pub max_angle: f32,
}

#[derive(Default, Clone, Debug)]
pub struct Wheel {
    pub entity: Entity,
    pub spoke_entities: Vec<Entity>,
    pub rigid_body_handle: rapier3d::prelude::RigidBodyHandle,
    pub center_position: Vec3,
    pub current_angle: f32,
    pub angular_velocity: f32,
}

#[derive(Default, Clone, Debug)]
pub struct Button {
    pub entity: Entity,
    pub base_position: Vec3,
    pub current_press: f32,
    pub is_pressed: bool,
    pub action: ButtonAction,
}

#[derive(Default, Clone, Debug)]
pub enum ButtonAction {
    #[default]
    RecallBaubles,
}

#[derive(Default, Clone, Debug)]
pub struct Note {
    pub entity: Entity,
    pub title: String,
    pub content: String,
}

#[derive(Default, Clone, Debug)]
pub struct BaubleSpawn {
    pub entity: Entity,
    pub spawn_position: Vec3,
}

#[derive(Default, Clone, Debug)]
pub struct ShotBauble {
    pub entity: Entity,
    pub spawn_time_ms: u64,
    pub original_scale: f32,
    pub landed: bool,
}

#[derive(Default, Clone, Debug)]
pub struct PrismaticSlider {
    pub entity: Entity,
    pub time_accumulator: f32,
}

#[derive(Default, Clone, Debug)]
pub struct SphericalJointVisual {
    pub anchor_entity: Entity,
    pub ball_entity: Entity,
    pub rod_entity: Entity,
}

#[derive(Default, Clone, Debug)]
pub struct RopeJointVisual {
    pub anchor_entity: Entity,
    pub ball_entity: Entity,
    pub rope_entity: Entity,
}

#[derive(Default, Clone, Debug)]
pub struct SpringJointVisual {
    pub anchor_entity: Entity,
    pub object_entity: Entity,
    pub spring_entities: Vec<Entity>,
}

#[derive(Default, Clone, Debug)]
pub struct CoulombFrictionJoint {
    pub arm_entity: Entity,
    pub friction_torque: f32,
}

#[derive(Default, Clone, Debug)]
pub struct VelocityFrictionJoint {
    pub arm_entity: Entity,
    pub damping_factor: f32,
    pub initialized: bool,
}

#[derive(Default)]
pub struct InteractionState {
    pub grabbed_entity: Option<Entity>,
    pub grab_distance: f32,
    pub manipulated_door: Option<freecs::Entity>,
    pub manipulated_drawer: Option<freecs::Entity>,
    pub manipulated_lever: Option<freecs::Entity>,
    pub manipulated_wheel: Option<freecs::Entity>,
    pub manipulated_button: Option<freecs::Entity>,
    pub gamepad_rt_was_pressed: bool,
    pub shoot_was_pressed: bool,
    pub shoot_hold_start_ms: Option<u64>,
    pub last_rapid_fire_ms: u64,
    pub require_interact_release: bool,
}

pub struct LeanState {
    pub current_lean: f32,
    pub target_lean: f32,
    pub base_rotation: nalgebra_glm::Quat,
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
pub enum InputMode {
    #[default]
    MouseKeyboard,
    Gamepad,
    #[cfg(feature = "openxr")]
    Xr,
}

freecs::ecs! {
    GameWorld {
        door: Door => DOOR,
        drawer: Drawer => DRAWER,
        lever: Lever => LEVER,
        wheel: Wheel => WHEEL,
        button: Button => BUTTON,
        note: Note => NOTE,
        bauble_spawn: BaubleSpawn => BAUBLE_SPAWN,
        shot_bauble: ShotBauble => SHOT_BAUBLE,
        prismatic_slider: PrismaticSlider => PRISMATIC_SLIDER,
        spherical_joint_visual: SphericalJointVisual => SPHERICAL_JOINT_VISUAL,
        rope_joint_visual: RopeJointVisual => ROPE_JOINT_VISUAL,
        spring_joint_visual: SpringJointVisual => SPRING_JOINT_VISUAL,
        coulomb_friction_joint: CoulombFrictionJoint => COULOMB_FRICTION_JOINT,
        velocity_friction_joint: VelocityFrictionJoint => VELOCITY_FRICTION_JOINT,
    }
    Tags {
    }
    Resources {
        player_entity: Option<Entity>,
        camera_entity: Option<Entity>,
        sun_entity: Option<Entity>,
        current_hour: f32,
        time_speed: f32,
        physics_objects: Vec<Entity>,
        interaction: InteractionState,
        interaction_prompt_entity: Option<Entity>,
        interaction_prompt_text_index: Option<usize>,
        lean_state: LeanState,
        input_mode: InputMode,
        input_mode_text_entity: Option<Entity>,
        input_mode_text_index: Option<usize>,
        show_physics_debug: bool,
        key4_was_pressed: bool,
        reading_note: Option<freecs::Entity>,
        note_close_key_released: bool,
        bauble_table_center: Vec3,
        bauble_table_top_y: f32,
        lantern_entity: Option<Entity>,
        lantern_light_entity: Option<Entity>,
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
        last_shown_note: Option<freecs::Entity>,
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
}
