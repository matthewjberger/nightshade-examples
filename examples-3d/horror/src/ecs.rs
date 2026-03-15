use nightshade::prelude::Entity;
use nightshade::prelude::Vec3;
use nightshade::prelude::freecs;
use nightshade::prelude::nalgebra_glm;
use nightshade::prelude::rapier3d;

#[derive(Default, Clone, Debug)]
pub struct EngineEntity(pub Entity);

#[derive(Default, Clone, Debug)]
pub struct Door {
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

#[derive(Default, Clone, Debug)]
pub struct Lever {
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

#[derive(Default, Clone, Debug)]
pub struct Note {
    pub title: String,
    pub content: String,
}

#[derive(Default, Clone, Debug)]
pub struct OverheadLight {
    pub light_entity: Entity,
    pub base_intensity: f32,
    pub spark_timer: f32,
    pub next_spark_time: f32,
    pub is_sparking: bool,
}

#[derive(Default, Clone, Debug)]
pub struct Button {
    pub base_position: Vec3,
    pub current_press: f32,
    pub is_pressed: bool,
}

#[derive(Default, Clone, Debug)]
pub struct SparkParticle {
    pub lifetime: f32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionKind {
    #[default]
    Grab,
    Door,
    Lever,
    Button,
    Note,
}

#[derive(Default, Clone, Debug)]
pub struct Interactable {
    pub kind: InteractionKind,
    pub match_entity: Entity,
    pub range: f32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeverAction {
    #[default]
    RestorePower,
    UnlockExit,
}

#[derive(Default)]
pub struct InteractionState {
    pub grabbed_entity: Option<Entity>,
    pub grab_distance: f32,
    pub manipulated_door: Option<freecs::Entity>,
    pub manipulated_lever: Option<freecs::Entity>,
    pub manipulated_button: Option<freecs::Entity>,
    pub gamepad_rt_was_pressed: bool,
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
    pub root_entity: Option<Entity>,
    pub active: bool,
    pub speed: f32,
    pub pause_timer: f32,
    pub chasing: bool,
}

freecs::ecs! {
    GameWorld {
        engine_entity: EngineEntity => ENGINE_ENTITY,
        door: Door => DOOR,
        lever: Lever => LEVER,
        note: Note => NOTE,
        overhead_light: OverheadLight => OVERHEAD_LIGHT,
        button: Button => BUTTON,
        interactable: Interactable => INTERACTABLE,
        spark_particle: SparkParticle => SPARK_PARTICLE,
    }
    Tags {
        physics_prop => PHYSICS_PROP,
        exit_door => EXIT_DOOR,
    }
    Resources {
        player_entity: Option<Entity>,
        camera_entity: Option<Entity>,
        flashlight_entity: Option<Entity>,
        flashlight_on: bool,
        flashlight_key_was_pressed: bool,
        lantern_entity: Option<Entity>,
        lantern_light_entity: Option<Entity>,
        interaction: InteractionState,
        lean_state: LeanState,
        input_mode: InputMode,
        reading_note: Option<freecs::Entity>,
        note_close_key_released: bool,
        power_restored: bool,
        exit_unlocked: bool,
        game_won: bool,
        temporary_message: Option<String>,
        temporary_message_timer: f32,
        cutscene: CutsceneState,
        monster: MonsterState,
        exit_door: Option<freecs::Entity>,
        fade_amount: f32,
        fade_target: f32,
        ambient_audio_entity: Option<Entity>,
        audio_started: bool,
        generator_audio_entity: Option<Entity>,
        rubble_audio_entity: Option<Entity>,
        monster_audio_entity: Option<Entity>,
        footstep_audio_entity: Option<Entity>,
        was_moving: bool,
        door_audio_entity: Option<Entity>,
        interaction_prompt_entity: Option<Entity>,
        interaction_prompt_text_index: Option<usize>,
        objective_text_entity: Option<Entity>,
        objective_text_index: Option<usize>,
        death_overlay_entity: Option<Entity>,
        temporary_message_overlay_entity: Option<Entity>,
        temporary_message_text_entity: Option<Entity>,
        note_overlay_entity: Option<Entity>,
        note_title_text_entity: Option<Entity>,
        note_content_text_entity: Option<Entity>,
        win_overlay_entity: Option<Entity>,
        win_text_entity: Option<Entity>,
        last_shown_note: Option<freecs::Entity>,
        last_shown_message: Option<String>,
    }
}
