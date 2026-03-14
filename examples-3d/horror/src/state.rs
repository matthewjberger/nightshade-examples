use nightshade::prelude::*;

#[derive(Default)]
pub struct HorrorDemo {
    pub player_entity: Option<Entity>,
    pub camera_entity: Option<Entity>,
    pub flashlight_entity: Option<Entity>,
    pub flashlight_on: bool,
    pub flashlight_key_was_pressed: bool,
    pub physics_objects: Vec<Entity>,
    pub doors: Vec<DoorState>,
    pub levers: Vec<LeverState>,
    pub buttons: Vec<ButtonState>,
    pub notes: Vec<NoteState>,
    pub lantern_entity: Option<Entity>,
    pub lantern_light_entity: Option<Entity>,
    pub interaction: InteractionState,
    pub interaction_prompt_entity: Option<Entity>,
    pub interaction_prompt_text_index: Option<usize>,
    pub objective_text_entity: Option<Entity>,
    pub objective_text_index: Option<usize>,
    pub lean_state: LeanState,
    pub input_mode: InputMode,
    pub reading_note: Option<usize>,
    pub note_close_key_released: bool,
    pub power_restored: bool,
    pub exit_unlocked: bool,
    pub game_won: bool,
    pub temporary_message: Option<String>,
    pub temporary_message_timer: f32,
    pub cutscene: CutsceneState,
    pub monster: MonsterState,
    pub overhead_lights: Vec<OverheadLightState>,
    pub exit_door_index: usize,
    pub fade_amount: f32,
    pub fade_target: f32,
    pub ambient_audio_entity: Option<Entity>,
    pub audio_started: bool,
    pub generator_audio_entity: Option<Entity>,
    pub rubble_audio_entity: Option<Entity>,
    pub monster_audio_entity: Option<Entity>,
    pub footstep_audio_entity: Option<Entity>,
    pub was_moving: bool,
    pub door_audio_entity: Option<Entity>,
    pub death_overlay_entity: Option<Entity>,
    pub temporary_message_overlay_entity: Option<Entity>,
    pub temporary_message_text_entity: Option<Entity>,
    pub note_overlay_entity: Option<Entity>,
    pub note_title_text_entity: Option<Entity>,
    pub note_content_text_entity: Option<Entity>,
    pub win_overlay_entity: Option<Entity>,
    pub win_text_entity: Option<Entity>,
    pub last_shown_note: Option<usize>,
    pub last_shown_message: Option<String>,
}

#[derive(Default)]
pub struct CutsceneState {
    pub active: bool,
    pub phase: CutscenePhase,
    pub timer: f32,
    pub saved_base_rotation: nalgebra_glm::Quat,
    pub target_rotation: nalgebra_glm::Quat,
    pub wall_break_position: Vec3,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum CutscenePhase {
    #[default]
    None,
    LookAtWall,
    WallBreaks,
    MonsterEmerges,
    ReturnControl,
    DoorSlam,
    LookAtDoor,
}

#[derive(Default)]
pub struct MonsterState {
    pub entity: Option<Entity>,
    pub body_parts: Vec<Entity>,
    pub active: bool,
    pub speed: f32,
    pub pause_timer: f32,
    pub chasing: bool,
}

pub struct OverheadLightState {
    pub entity: Entity,
    pub light_entity: Entity,
    pub base_intensity: f32,
    pub spark_timer: f32,
    pub next_spark_time: f32,
    pub is_sparking: bool,
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
}

#[derive(Default)]
pub struct InteractionState {
    pub grabbed_entity: Option<Entity>,
    pub grab_distance: f32,
    pub manipulated_door_index: Option<usize>,
    pub manipulated_lever_index: Option<usize>,
    pub manipulated_button_index: Option<usize>,
    pub gamepad_rt_was_pressed: bool,
    pub require_interact_release: bool,
}

pub struct DoorState {
    pub entity: Entity,
    pub rigid_body_handle: rapier3d::prelude::RigidBodyHandle,
    pub hinge_position: Vec3,
    pub door_half_width: f32,
    pub current_angle: f32,
    pub angular_velocity: f32,
    pub min_angle: f32,
    pub max_angle: f32,
    pub locked: bool,
    pub side_door: bool,
    pub swing_reversed: bool,
}

pub struct LeverState {
    pub pivot_entity: Entity,
    pub collider_entity: Entity,
    pub collider_rb_handle: rapier3d::prelude::RigidBodyHandle,
    pub pivot_position: Vec3,
    pub arm_half_length: f32,
    pub current_angle: f32,
    pub angular_velocity: f32,
    pub min_angle: f32,
    pub max_angle: f32,
    pub action: LeverAction,
    pub light_entity: Entity,
    pub light_material_name: String,
    pub activated: bool,
}

#[derive(Clone)]
pub enum LeverAction {
    RestorePower,
    UnlockExit,
}

pub struct ButtonState {
    pub entity: Entity,
    pub base_position: Vec3,
    pub current_press: f32,
    pub is_pressed: bool,
}

pub struct NoteState {
    pub entity: Entity,
    pub title: String,
    pub content: String,
}
