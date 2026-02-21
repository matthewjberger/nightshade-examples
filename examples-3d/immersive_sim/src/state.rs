use crate::data::levels::LevelId;
use crate::data::player::PlayerProgress;
use crate::systems::combat::CombatState;
use crate::systems::level_loader::LoadedLevel;
use crate::systems::particles::ParticleSystem;
use nightshade::ecs::gpu_picking::GpuPickResult;
use nightshade::ecs::texture_loader::{AssetLoadingState, SharedTextureQueue};
use nightshade::prelude::*;
use nightshade::shell::Command;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum GameScreen {
    #[default]
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
    pub npc_name: String,
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

#[derive(Default)]
pub struct ShellContext {
    pub noclip: bool,
    pub psx: bool,
    pub unlit: bool,
    pub pending_level: Option<LevelId>,
}

pub struct NoclipCommand;

impl Command<ShellContext> for NoclipCommand {
    fn name(&self) -> &str {
        "noclip"
    }

    fn description(&self) -> &str {
        "Toggle noclip fly camera mode (disables player physics)"
    }

    fn usage(&self) -> &str {
        "noclip"
    }

    fn execute(&self, _args: &[&str], _world: &mut World, context: &mut ShellContext) -> String {
        context.noclip = !context.noclip;
        if context.noclip {
            "Noclip enabled - WASD to move, right-click drag to look".to_string()
        } else {
            "Noclip disabled - player controls restored".to_string()
        }
    }
}

pub struct PsxCommand;

impl Command<ShellContext> for PsxCommand {
    fn name(&self) -> &str {
        "psx"
    }

    fn description(&self) -> &str {
        "Toggle PS1-style rendering effects (vertex snapping, affine textures, fog)"
    }

    fn usage(&self) -> &str {
        "psx"
    }

    fn execute(&self, _args: &[&str], world: &mut World, context: &mut ShellContext) -> String {
        context.psx = !context.psx;
        if context.psx {
            world.resources.graphics.vertex_snap = Some(VertexSnap {
                resolution: [320.0, 240.0],
            });
            world.resources.graphics.affine_texture_mapping = true;
            world.resources.graphics.fog = Some(Fog {
                start: 2.0,
                end: 20.0,
                color: [0.1, 0.1, 0.15],
            });
            "PSX mode enabled - vertex snapping, affine textures, fog".to_string()
        } else {
            world.resources.graphics.vertex_snap = None;
            world.resources.graphics.affine_texture_mapping = false;
            world.resources.graphics.fog = None;
            "PSX mode disabled - modern rendering restored".to_string()
        }
    }
}

pub struct UnlitCommand;

impl Command<ShellContext> for UnlitCommand {
    fn name(&self) -> &str {
        "unlit"
    }

    fn description(&self) -> &str {
        "Toggle unlit rendering (disable lighting calculations)"
    }

    fn usage(&self) -> &str {
        "unlit"
    }

    fn execute(&self, _args: &[&str], world: &mut World, context: &mut ShellContext) -> String {
        context.unlit = !context.unlit;
        for material in world
            .resources
            .material_registry
            .registry
            .entries
            .iter_mut()
            .flatten()
        {
            material.unlit = context.unlit;
        }
        if context.unlit {
            "Unlit mode enabled - lighting disabled".to_string()
        } else {
            "Unlit mode disabled - lighting restored".to_string()
        }
    }
}

pub struct LoadLevelCommand;

impl Command<ShellContext> for LoadLevelCommand {
    fn name(&self) -> &str {
        "level"
    }

    fn description(&self) -> &str {
        "Load a level (hub, dungeon, forest, castle, arena)"
    }

    fn usage(&self) -> &str {
        "level <name>"
    }

    fn execute(&self, args: &[&str], _world: &mut World, context: &mut ShellContext) -> String {
        if args.is_empty() {
            return "Usage: level <hub|dungeon|forest|castle|arena>".to_string();
        }

        let level_name = args[0].to_lowercase();
        let level_id = match level_name.as_str() {
            "hub" => Some(LevelId::Hub),
            "dungeon" => Some(LevelId::Dungeon),
            "forest" => Some(LevelId::Forest),
            "castle" => Some(LevelId::Castle),
            "arena" | "final" | "boss" => Some(LevelId::FinalArena),
            _ => None,
        };

        match level_id {
            Some(id) => {
                context.pending_level = Some(id);
                format!("Loading level: {:?}", id)
            }
            None => "Unknown level. Options: hub, dungeon, forest, castle, arena".to_string(),
        }
    }
}

pub struct ImmersiveSim {
    pub screen: GameScreen,
    pub input_mode: InputMode,
    pub player_entity: Option<Entity>,
    pub camera_entity: Option<Entity>,
    pub hands_entity: Option<Entity>,
    pub flashlight_entity: Option<Entity>,
    pub flashlight_on: bool,
    pub flashlight_key_was_pressed: bool,
    pub lean_state: LeanState,
    pub interaction: InteractionState,
    pub dialogue: DialogueState,
    pub props: Vec<PropState>,
    pub physics_objects: Vec<Entity>,
    pub npc_entities: Vec<Entity>,
    pub audio: AudioState,
    pub level_loaded: bool,
    pub level_entity: Option<Entity>,
    pub spawned_entities: Vec<Entity>,
    pub texture_queue: SharedTextureQueue,
    pub loading: AssetLoadingState,
    pub level_spawned: bool,
    pub fps_hud_text: Option<Entity>,
    pub shell: ShellState<ShellContext>,
    pub noclip_was_active: bool,
    pub player_progress: PlayerProgress,
    pub combat_state: CombatState,
    pub particle_system: ParticleSystem,
    pub loaded_level: LoadedLevel,
    pub current_level: LevelId,
    pub is_dead: bool,
    pub game_time: f32,
    pub skill_keys_pressed: [bool; 8],
    pub last_pick_result: Option<GpuPickResult>,
}

impl Default for ImmersiveSim {
    fn default() -> Self {
        let mut shell = ShellState::new(ShellContext::default());
        shell.register_builtin_commands();
        shell.register_command(Box::new(NoclipCommand));
        shell.register_command(Box::new(PsxCommand));
        shell.register_command(Box::new(UnlitCommand));
        shell.register_command(Box::new(LoadLevelCommand));

        Self {
            screen: GameScreen::Loading,
            input_mode: InputMode::MouseKeyboard,
            player_entity: None,
            camera_entity: None,
            hands_entity: None,
            flashlight_entity: None,
            flashlight_on: true,
            flashlight_key_was_pressed: false,
            lean_state: LeanState::new(),
            interaction: InteractionState::default(),
            dialogue: DialogueState::default(),
            props: Vec::new(),
            physics_objects: Vec::new(),
            npc_entities: Vec::new(),
            audio: AudioState::default(),
            level_loaded: false,
            level_entity: None,
            spawned_entities: Vec::new(),
            texture_queue: nightshade::ecs::texture_loader::create_shared_queue(),
            loading: AssetLoadingState::default(),
            level_spawned: false,
            fps_hud_text: None,
            shell,
            noclip_was_active: false,
            player_progress: PlayerProgress::default(),
            combat_state: CombatState::default(),
            particle_system: ParticleSystem::default(),
            loaded_level: LoadedLevel::default(),
            current_level: LevelId::Hub,
            is_dead: false,
            game_time: 0.0,
            skill_keys_pressed: [false; 8],
            last_pick_result: None,
        }
    }
}
