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
        Grounded + Slide = Sliding,
        Sliding + Release = Grounded,
        Sliding + Jump = Airborne,
        Sliding + Dash = GroundDash,
        Sliding + BecomeAirborne = Airborne,
        GroundDash + Jump = Airborne,
        GroundDash + Land = Grounded,
        GroundDash + BecomeAirborne = Airborne,
        Airborne + DoubleJump = DoubleJumped,
        Airborne + Dash = AirDash,
        DoubleJumped + Dash = AirDash,
        AirDash + DashEnd = Falling,
        AirDash + Land = Grounded,
        Falling + Dash = AirDash,
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
    pub standing_camera_height: f32,
    #[cfg(not(feature = "openxr"))]
    pub crouching_camera_height: f32,
    #[cfg(not(feature = "openxr"))]
    pub lean_amount: f32,
    #[cfg(not(feature = "openxr"))]
    pub lean_angle: f32,
    #[cfg(not(feature = "openxr"))]
    pub lean_speed: f32,
    pub max_shot_baubles: usize,
    pub bauble_lifetime_ms: u64,
    pub bauble_shrink_duration_ms: u64,
    pub slide_boost: f32,
    pub slide_friction: f32,
    pub slide_min_speed: f32,
    #[cfg(not(feature = "openxr"))]
    pub slide_camera_tilt: f32,
    pub dash_impulse: f32,
    pub dash_air_impulse: f32,
    pub dash_friction: f32,
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
            standing_camera_height: 0.8,
            #[cfg(not(feature = "openxr"))]
            crouching_camera_height: 0.3,
            #[cfg(not(feature = "openxr"))]
            lean_amount: 0.4,
            #[cfg(not(feature = "openxr"))]
            lean_angle: 0.15,
            #[cfg(not(feature = "openxr"))]
            lean_speed: 8.0,
            max_shot_baubles: 200,
            bauble_lifetime_ms: 30000,
            bauble_shrink_duration_ms: 2000,
            slide_boost: 8.0,
            slide_friction: 1.2,
            slide_min_speed: 5.0,
            #[cfg(not(feature = "openxr"))]
            slide_camera_tilt: 0.05,
            dash_impulse: 25.0,
            dash_air_impulse: 18.0,
            dash_friction: 0.5,
            double_jump_impulse: 5.5,
            max_dash_charges: 2,
            dash_cooldown: 1.5,
        }
    }
}

#[derive(Default)]
pub struct PlayerResources {
    pub entity: Option<Entity>,
    pub camera_entity: Option<Entity>,
    pub state: PlayerState,
    pub dash_charges: u32,
    pub dash_cooldown_timer: f32,
}

#[cfg(not(feature = "openxr"))]
#[derive(Default)]
pub struct WeaponState {
    pub entity: Option<Entity>,
    pub aiming_down_sights: bool,
    pub aim_blend: f32,
    pub sway: nalgebra_glm::Vec2,
    pub previous_yaw: f32,
    pub previous_pitch: f32,
}

#[derive(Default)]
pub struct FlashlightState {
    pub entity: Option<Entity>,
    pub on: bool,
    pub key_was_pressed: bool,
}

#[derive(Default)]
pub struct PromptCache {
    pub camera_position: nalgebra_glm::Vec3,
    pub camera_forward: nalgebra_glm::Vec3,
    pub can_interact: bool,
    pub can_read: bool,
}

#[derive(Default)]
pub struct UiHandles {
    pub crosshair_entity: Option<Entity>,
    pub crosshair_arms: Vec<Entity>,
    pub note_overlay_entity: Option<Entity>,
    pub note_title_entity: Option<Entity>,
    pub note_content_entity: Option<Entity>,
    pub last_shown_note: Option<freecs::Entity>,
    pub reading_note: Option<freecs::Entity>,
    pub note_close_key_released: bool,
    pub interaction_prompt_entity: Option<Entity>,
    pub interaction_prompt_text_index: Option<usize>,
    pub input_mode_text_entity: Option<Entity>,
    pub input_mode_text_index: Option<usize>,
    pub dash_hud_entity: Option<Entity>,
    pub dash_hud_state_text_entity: Option<Entity>,
    pub dash_hud_charge_entities: Vec<Entity>,
}

#[derive(Default, Clone, Copy)]
pub struct ActionEdge {
    held: bool,
    previous: bool,
}

impl ActionEdge {
    pub fn update(&mut self, pressed: bool) {
        self.previous = self.held;
        self.held = pressed;
    }

    pub fn just_pressed(&self) -> bool {
        self.held && !self.previous
    }

    pub fn held(&self) -> bool {
        self.held
    }
}

#[derive(Default)]
pub struct InputActions {
    pub dash: ActionEdge,
    pub jump: ActionEdge,
    pub slide: ActionEdge,
}

#[derive(Default)]
pub struct InteractionState {
    pub grabbed_entity: Option<Entity>,
    pub grab_distance: f32,
    pub manipulated: Option<(freecs::Entity, super::InteractableKind)>,
    pub gamepad_rt_was_pressed: bool,
    pub shoot_was_pressed: bool,
    pub shoot_hold_start_ms: Option<u64>,
    pub last_rapid_fire_ms: u64,
    pub require_interact_release: bool,
}

impl InteractionState {
    pub fn is_any_active(&self) -> bool {
        self.grabbed_entity.is_some() || self.manipulated.is_some()
    }

    pub fn manipulated_entity_of_kind(&self, kind: &super::InteractableKind) -> Option<freecs::Entity> {
        self.manipulated.as_ref().and_then(|(entity, k)| {
            if std::mem::discriminant(k) == std::mem::discriminant(kind) {
                Some(*entity)
            } else {
                None
            }
        })
    }
}

#[cfg(not(feature = "openxr"))]
pub struct LeanState {
    pub current_lean: f32,
    pub base_rotation: nalgebra_glm::Quat,
}

#[cfg(not(feature = "openxr"))]
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
