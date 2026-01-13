use nightshade::ecs::texture_loader::{AssetLoadingState, SharedTextureQueue};
use nightshade::prelude::*;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum GameScreen {
    #[default]
    Title,
    Loading,
    Gameplay,
    Paused,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    MouseKeyboard,
    Gamepad,
}

#[derive(Default)]
pub struct LeanState {
    pub current_lean: f32,
    pub target_lean: f32,
    pub base_rotation: nalgebra_glm::Quat,
}

impl LeanState {
    pub fn new() -> Self {
        Self {
            current_lean: 0.0,
            target_lean: 0.0,
            base_rotation: nalgebra_glm::quat_identity(),
        }
    }
}

#[derive(Default)]
pub struct InteractionState {
    pub grabbed_entity: Option<Entity>,
    pub grab_distance: f32,
    pub gamepad_rt_was_pressed: bool,
    pub require_interact_release: bool,
}

pub struct DialogueLine {
    pub speaker: String,
    pub text: String,
}

pub struct DialogueNode {
    pub lines: Vec<DialogueLine>,
    pub choices: Vec<DialogueChoice>,
}

pub struct DialogueChoice {
    pub text: String,
    pub next_node: Option<usize>,
}

#[derive(Default)]
pub struct DialogueState {
    pub active: bool,
    pub current_node: usize,
    pub current_line: usize,
    pub nodes: Vec<DialogueNode>,
    pub speaking_npc: Option<Entity>,
    pub interact_key_was_pressed: bool,
    pub advance_key_was_pressed: bool,
}

pub struct PropState {
    pub _rigid_body_handle: rapier3d::prelude::RigidBodyHandle,
}

#[derive(Default)]
pub struct AudioState {
    pub footstep_entity: Option<Entity>,
    pub was_moving: bool,
}

pub struct LevelDemo {
    pub screen: GameScreen,
    pub input_mode: InputMode,
    pub player_entity: Option<Entity>,
    pub camera_entity: Option<Entity>,
    pub fly_camera: Option<Entity>,
    pub fly_mode: bool,
    pub hands_entity: Option<Entity>,
    pub flashlight_entity: Option<Entity>,
    pub flashlight_on: bool,
    pub flashlight_key_was_pressed: bool,
    pub lean_state: LeanState,
    pub interaction: InteractionState,
    pub dialogue: DialogueState,
    pub props: Vec<PropState>,
    pub physics_objects: Vec<Entity>,
    pub audio: AudioState,
    pub level_loaded: bool,
    pub level_entity: Option<Entity>,
    pub spawned_entities: Vec<Entity>,
    pub show_collision: bool,
    pub unlit: bool,
    pub show_navmesh: bool,
    pub texture_queue: SharedTextureQueue,
    pub loading: AssetLoadingState,
    pub level_spawned: bool,
}

impl Default for LevelDemo {
    fn default() -> Self {
        Self {
            screen: GameScreen::Title,
            input_mode: InputMode::MouseKeyboard,
            player_entity: None,
            camera_entity: None,
            fly_camera: None,
            fly_mode: false,
            hands_entity: None,
            flashlight_entity: None,
            flashlight_on: true,
            flashlight_key_was_pressed: false,
            lean_state: LeanState::new(),
            interaction: InteractionState::default(),
            dialogue: DialogueState::default(),
            props: Vec::new(),
            physics_objects: Vec::new(),
            audio: AudioState::default(),
            level_loaded: false,
            level_entity: None,
            spawned_entities: Vec::new(),
            show_collision: false,
            unlit: false,
            show_navmesh: false,
            texture_queue: nightshade::ecs::texture_loader::create_shared_queue(),
            loading: AssetLoadingState::default(),
            level_spawned: false,
        }
    }
}
