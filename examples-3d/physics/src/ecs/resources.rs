use nightshade::prelude::Entity;
use nightshade::prelude::freecs;
use nightshade::prelude::nalgebra_glm;

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
