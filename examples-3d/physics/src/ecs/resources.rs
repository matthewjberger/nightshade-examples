use nightshade::prelude::Entity;
use nightshade::prelude::freecs;
use nightshade::prelude::nalgebra_glm;

stateless::statemachine! {
    name: Player,
    transitions: {
        *Grounded + Jump = Airborne,
        Grounded + Dash = GroundDash,
        Grounded + LeanLeft = LeaningLeft,
        Grounded + LeanRight = LeaningRight,
        LeaningLeft + Release = Grounded,
        LeaningRight + Release = Grounded,
        LeaningLeft + Jump = Airborne,
        LeaningRight + Jump = Airborne,
        LeaningLeft + Dash = GroundDash,
        LeaningRight + Dash = GroundDash,
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

pub struct GameConfig {
    pub grab_range: f32,
    pub interact_range: f32,
    pub interact_cone_radius: f32,
    pub min_grab_distance: f32,
    pub max_grab_distance: f32,
    pub scroll_distance_speed: f32,
    pub throw_strength: f32,
    pub grab_stiffness: f32,
    pub grab_damping_ratio: f32,
    pub max_grab_force: f32,
    pub angular_damping: f32,
    pub standing_camera_height: f32,
    pub crouching_camera_height: f32,
    pub lean_amount: f32,
    pub lean_angle: f32,
    pub lean_speed: f32,
    pub max_shot_baubles: usize,
    pub bauble_lifetime_ms: u64,
    pub bauble_shrink_duration_ms: u64,
    pub dash_impulse: f32,
    pub dash_air_impulse: f32,
    pub double_jump_impulse: f32,
    pub max_dash_charges: u32,
    pub dash_cooldown: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            grab_range: 3.0,
            interact_range: 2.5,
            interact_cone_radius: 40.0,
            min_grab_distance: 0.8,
            max_grab_distance: 3.0,
            scroll_distance_speed: 0.3,
            throw_strength: 12.0,
            grab_stiffness: 150.0,
            grab_damping_ratio: 1.0,
            max_grab_force: 80.0,
            angular_damping: 0.95,
            standing_camera_height: 0.8,
            crouching_camera_height: 0.3,
            lean_amount: 0.4,
            lean_angle: 0.15,
            lean_speed: 8.0,
            max_shot_baubles: 200,
            bauble_lifetime_ms: 30000,
            bauble_shrink_duration_ms: 2000,
            dash_impulse: 45.0,
            dash_air_impulse: 40.0,
            double_jump_impulse: 6.0,
            max_dash_charges: 2,
            dash_cooldown: 1.5,
        }
    }
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
    pub base_rotation: nalgebra_glm::Quat,
}

impl Default for LeanState {
    fn default() -> Self {
        Self {
            current_lean: 0.0,
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
