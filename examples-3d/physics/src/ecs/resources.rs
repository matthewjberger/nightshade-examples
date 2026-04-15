use nightshade::prelude::Entity;
use nightshade::prelude::freecs;

stateless::statemachine! {
    name: Player,
    transitions: {
        *Grounded + Jump = Airborne,
        Grounded + Sprint = Sprinting,
        Grounded + Crouch = Crouching,
        Sprinting + Release = Grounded,
        Sprinting + Jump = Airborne,
        Crouching + Release = Grounded,
        Crouching + Jump = Airborne,
        Airborne + Land = Grounded,
        _ + Reset = Grounded,
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
    pub max_shot_baubles: usize,
    pub bauble_lifetime_ms: u64,
    pub bauble_shrink_duration_ms: u64,
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
            max_shot_baubles: 200,
            bauble_lifetime_ms: 30000,
            bauble_shrink_duration_ms: 2000,
        }
    }
}

#[derive(Default)]
pub struct PlayerResources {
    pub entity: Option<Entity>,
    pub camera_entity: Option<Entity>,
    pub state: PlayerState,
}

#[cfg(not(feature = "openxr"))]
#[derive(Default)]
pub struct WeaponState {
    pub entity: Option<Entity>,
}

#[derive(Default)]
pub struct FlashlightState {
    pub entity: Option<Entity>,
    pub on: bool,
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
    pub interaction_prompt_entity: Option<Entity>,
    pub interaction_prompt_text_index: Option<usize>,
    pub input_mode_text_entity: Option<Entity>,
    pub input_mode_text_index: Option<usize>,
    pub player_state_text_entity: Option<Entity>,
}

#[derive(Default)]
pub struct InteractionState {
    pub manipulated: Option<(freecs::Entity, super::InteractableKind)>,
    pub shoot_hold_start_ms: Option<u64>,
    pub last_rapid_fire_ms: u64,
    pub require_interact_release: bool,
}

impl InteractionState {
    pub fn manipulated_entity_of_kind(
        &self,
        kind: &super::InteractableKind,
    ) -> Option<freecs::Entity> {
        self.manipulated.as_ref().and_then(|(entity, k)| {
            if std::mem::discriminant(k) == std::mem::discriminant(kind) {
                Some(*entity)
            } else {
                None
            }
        })
    }
}
