mod components;
mod resources;

pub use components::*;
pub use resources::*;

use nightshade::prelude::Entity;
use nightshade::prelude::KeyCode;
use nightshade::prelude::Vec3;
use nightshade::prelude::freecs;
use nightshade::prelude::nalgebra_glm;

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
        target: Target => TARGET,
        velocity_friction_joint: VelocityFrictionJoint => VELOCITY_FRICTION_JOINT,
    }
    Tags {
    }
    Resources {
        config: GameConfig,
        player_entity: Option<Entity>,
        camera_entity: Option<Entity>,
        physics_objects: Vec<Entity>,
        interaction: InteractionState,
        interaction_prompt_entity: Option<Entity>,
        interaction_prompt_text_index: Option<usize>,
        lean: LeanState,
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
        player_state: PlayerState,
        dash_charges: u32,
        dash_cooldown_timer: f32,
        dash_button_was_pressed: bool,
        jump_button_was_pressed: bool,
        slide_button_was_pressed: bool,
        last_tap_key: Option<KeyCode>,
        last_tap_time_ms: u64,
        key_was_released: bool,
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
