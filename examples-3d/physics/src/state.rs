use crate::ecs::GameWorld;
#[cfg(not(feature = "openxr"))]
use crate::systems::camera::{camera_look_system, crouch_camera_system, lean_system};
#[cfg(not(feature = "openxr"))]
use crate::systems::input::detect_input_mode;
use crate::systems::{
    dash::{build_dash_hud, dash_system},
    exhibits::{
        spawn_environment, spawn_exhibits, spawn_sun_overhead, setup_velocity_friction_joints,
        update_coulomb_friction_joints, update_joint_visuals, update_prismatic_sliders,
    },
    flashlight::{spawn_flashlight, update_flashlight},
    interaction::{
        check_fall_reset, interaction_system, note_reading_system, update_doors_momentum,
        update_drawers_momentum, update_interaction_prompt, update_lantern_light,
        update_levers_momentum, update_wheels_momentum,
    },
    shooting::update_shot_baubles,
    targets::{spawn_targets, update_targets, process_target_killed_events},
    ui::{build_crosshair, build_note_overlay, debug_toggle_system, update_note_overlay},
};
#[cfg(not(feature = "openxr"))]
use crate::systems::weapon::{spawn_weapon, update_weapon_sway};
use nightshade::ecs::physics::spawn_first_person_player;
use nightshade::ecs::text::commands::spawn_ui_text;
use nightshade::prelude::*;

#[derive(Default)]
pub struct PhysicsGame {
    pub game_world: GameWorld,
}

impl State for PhysicsGame {
    fn title(&self) -> &str {
        "Physics Interaction Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.atmosphere = Atmosphere::DayNight;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.use_fullscreen = true;

        world.resources.graphics.day_night.hour = 8.0;
        world.resources.graphics.day_night.speed = 0.15;
        world.resources.graphics.day_night.auto_cycle = true;
        capture_procedural_atmosphere_ibl(
            world,
            Atmosphere::DayNight,
            8.0,
        );
        capture_ibl_snapshots(
            world,
            Atmosphere::DayNight,
            vec![0.0, 6.0, 8.0, 12.0, 17.0, 18.5, 20.0],
        );

        self.game_world.resources.show_physics_debug = false;
        self.game_world.resources.player.dash_charges = self.game_world.resources.config.max_dash_charges;
        world.resources.physics.debug_draw = false;

        #[cfg(feature = "openxr")]
        {
            self.game_world.resources.input_mode = crate::ecs::InputMode::Xr;
            world.resources.xr.initial_player_position =
                Some(nalgebra_glm::vec3(0.0, 0.0, 8.0));
        }

        let sun = spawn_sun_overhead(world);
        world.resources.graphics.day_night.sun_entity = Some(sun);

        let player_position = nalgebra_glm::vec3(0.0, 1.2, 8.0);
        let (player_entity, camera_entity) = spawn_first_person_player(world, player_position);

        if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
            transform.translation.y = self.game_world.resources.config.standing_camera_height;
        }

        if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
            controller.max_speed = 2.5;
            controller.sprint_speed_multiplier = 2.0;
        }

        self.game_world.resources.player.entity = Some(player_entity);
        self.game_world.resources.player.camera_entity = Some(camera_entity);

        world.resources.graphics.render_layer_world_enabled = true;
        world.resources.graphics.render_layer_overlay_enabled = true;

        #[cfg(not(feature = "openxr"))]
        {
            let weapon = spawn_weapon(world, camera_entity);
            self.game_world.resources.weapon.entity = Some(weapon);
        }

        let flashlight = spawn_flashlight(world);
        self.game_world.resources.flashlight.entity = Some(flashlight);
        self.game_world.resources.flashlight.on = false;
        if let Some(light) = world.core.get_light_mut(flashlight) {
            light.intensity = 0.0;
        }

        spawn_environment(world);
        spawn_exhibits(&mut self.game_world, world);
        spawn_targets(&mut self.game_world, world);

        let prompt_entity = spawn_ui_text(world, "", nalgebra_glm::Vec2::zeros());
        if let Some(hud_text) = world.core.get_text(prompt_entity) {
            self.game_world.resources.ui.interaction_prompt_text_index = Some(hud_text.text_index);
        }
        self.game_world.resources.ui.interaction_prompt_entity = Some(prompt_entity);

        let input_mode_entity =
            spawn_ui_text(world, "Mouse/Keyboard", nalgebra_glm::Vec2::zeros());
        if let Some(hud_text) = world.core.get_text(input_mode_entity) {
            self.game_world.resources.ui.input_mode_text_index = Some(hud_text.text_index);
        }
        self.game_world.resources.ui.input_mode_text_entity = Some(input_mode_entity);

        #[cfg(feature = "openxr")]
        {
            let left_hand =
                crate::systems::xr::spawn_hand_cube(world, nalgebra_glm::vec3(0.2, 0.6, 0.9));
            self.game_world.resources.left_hand_cube = Some(left_hand);
            let gun_root =
                crate::systems::xr::spawn_weapon(&mut self.game_world, world);
            self.game_world.resources.gun_root_entity = Some(gun_root);
        }

        world.resources.retained_ui.enabled = true;
        let (crosshair, crosshair_arms) = build_crosshair(world);
        self.game_world.resources.ui.crosshair_entity = Some(crosshair);
        self.game_world.resources.ui.crosshair_arms = crosshair_arms;
        let (note_overlay, note_title, note_content) = build_note_overlay(world);
        self.game_world.resources.ui.note_overlay_entity = Some(note_overlay);
        self.game_world.resources.ui.note_title_entity = Some(note_title);
        self.game_world.resources.ui.note_content_entity = Some(note_content);

        let (dash_hud, dash_state_text, dash_charges) = build_dash_hud(world, self.game_world.resources.config.max_dash_charges);
        self.game_world.resources.ui.dash_hud_entity = Some(dash_hud);
        self.game_world.resources.ui.dash_hud_state_text_entity = Some(dash_state_text);
        self.game_world.resources.ui.dash_hud_charge_entities = dash_charges;
    }

    fn run_systems(&mut self, world: &mut World) {
        self.game_world.step();

        update_note_overlay(&mut self.game_world, world);

        if self.game_world.resources.ui.reading_note.is_some() {
            note_reading_system(&mut self.game_world, world);
        }

        escape_key_exit_system(world);
        if let Some(gamepad) = nightshade::ecs::input::queries::query_active_gamepad(world)
            && gamepad.is_pressed(gilrs::Button::Select)
        {
            world.resources.window.should_exit = true;
        }
        debug_toggle_system(&mut self.game_world, world);
        #[cfg(not(feature = "openxr"))]
        detect_input_mode(&mut self.game_world, world);
        check_fall_reset(&self.game_world, world);
        #[cfg(not(feature = "openxr"))]
        camera_look_system(&mut self.game_world, world);
        #[cfg(not(feature = "openxr"))]
        lean_system(&mut self.game_world, world);
        #[cfg(not(feature = "openxr"))]
        crouch_camera_system(&self.game_world, world);
        #[cfg(feature = "openxr")]
        crate::systems::xr::xr_hand_tracking_system(&mut self.game_world, world);
        dash_system(&mut self.game_world, world);
        #[cfg(not(feature = "openxr"))]
        update_weapon_sway(&mut self.game_world, world);
        nightshade::ecs::transform::systems::update_global_transforms_system(world);
        interaction_system(&mut self.game_world, world);
        update_shot_baubles(&mut self.game_world, world);
        update_targets(&mut self.game_world, world);
        process_target_killed_events(&mut self.game_world, world);
        update_doors_momentum(&mut self.game_world, world);
        update_drawers_momentum(&mut self.game_world, world);
        update_levers_momentum(&mut self.game_world, world);
        update_wheels_momentum(&mut self.game_world, world);
        update_lantern_light(&self.game_world, world);
        update_flashlight(&mut self.game_world, world);
        update_interaction_prompt(&mut self.game_world, world);
        update_prismatic_sliders(&mut self.game_world, world);
        update_joint_visuals(&self.game_world, world);
        update_coulomb_friction_joints(&self.game_world, world);
        setup_velocity_friction_joints(&mut self.game_world, world);
    }
}
