mod constants;
mod discovery;
mod state;
mod systems;

use std::path::Path;

use constants::STANDING_CAMERA_HEIGHT;
use discovery::{
    discover_chain_light, discover_doors, discover_levers, discover_notes,
    discover_overhead_lights, discover_physics_props,
};
use nightshade::ecs::audio::systems::load_sound_from_bytes;
use nightshade::ecs::camera::components::Projection;
use nightshade::ecs::physics::spawn_first_person_player;
use nightshade::ecs::scene::{load_scene, spawn_scene};
use nightshade::ecs::texture_loader::set_asset_search_paths;
use nightshade::prelude::*;
use state::HorrorDemo;
use systems::{
    camera_look_system, check_puzzle_state, crouch_camera_system, cutscene_system,
    detect_input_mode, interaction_system, lean_system, load_textures, monster_chase_system,
    note_reading_system, spawn_ambient_light, spawn_flashlight, spawn_ui, update_doors_momentum,
    update_flashlight, update_interaction_prompt, update_lantern_light, update_levers_momentum,
    update_objective, update_overhead_lights, update_overlays, update_temporary_message,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_asset_search_paths(vec!["".to_string()]);
    launch(HorrorDemo::default())
}

fn load_audio_assets(world: &mut World) {
    let sounds: &[(&str, &'static [u8])] = &[
        (
            "atmosphere",
            include_bytes!("../../../assets/audio/horror/atmosphere.mp3"),
        ),
        (
            "generator",
            include_bytes!("../../../assets/audio/horror/generator.mp3"),
        ),
        (
            "rubble",
            include_bytes!("../../../assets/audio/horror/rubble.mp3"),
        ),
        (
            "monster",
            include_bytes!("../../../assets/audio/horror/monster.mp3"),
        ),
        (
            "footsteps",
            include_bytes!("../../../assets/audio/horror/footsteps.mp3"),
        ),
        (
            "door_creak",
            include_bytes!("../../../assets/audio/horror/door_creak.mp3"),
        ),
    ];

    for &(name, bytes) in sounds {
        if let Ok(data) = load_sound_from_bytes(bytes) {
            world.resources.audio.load_sound(name, data);
        }
    }
}

impl State for HorrorDemo {
    fn title(&self) -> &str {
        "Horror Demo - Nightshade"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.use_fullscreen = true;
        world.resources.graphics.clear_color = [0.0, 0.0, 0.0, 1.0];

        self.power_restored = false;
        self.exit_unlocked = false;
        self.game_won = false;

        let player_position = nalgebra_glm::vec3(0.0, 1.2, 4.0);
        let (player_entity, camera_entity) = spawn_first_person_player(world, player_position);

        if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
            transform.translation.y = STANDING_CAMERA_HEIGHT;
        }

        if let Some(camera) = world.core.get_camera_mut(camera_entity)
            && let Projection::Perspective(ref mut perspective) = camera.projection
        {
            perspective.y_fov_rad = 75.0_f32.to_radians();
        }

        self.player_entity = Some(player_entity);
        self.camera_entity = Some(camera_entity);
        world.resources.active_camera = Some(camera_entity);

        world.core.add_components(camera_entity, AUDIO_LISTENER);
        world.core.set_audio_listener(camera_entity, AudioListener);

        let flashlight = spawn_flashlight(world);
        self.flashlight_entity = Some(flashlight);
        self.flashlight_on = true;

        spawn_ambient_light(world);

        load_textures(world);

        let scene = load_scene(Path::new("examples-3d/horror/horror_level.bin"))
            .expect("Failed to load horror_level.bin");

        if let Err(error) = spawn_scene(world, &scene, None) {
            tracing::error!("Failed to spawn horror scene: {}", error);
        }

        discover_doors(self, world);
        discover_levers(self, world);
        discover_notes(self, world);
        discover_physics_props(self, world);
        discover_chain_light(self, world);
        discover_overhead_lights(self, world);

        spawn_ui(self, world);

        load_audio_assets(world);

        let ambient_audio = world.spawn_entities(AUDIO_SOURCE, 1)[0];
        world.core.set_audio_source(
            ambient_audio,
            AudioSource::new("atmosphere")
                .with_volume(0.4)
                .with_looping(true),
        );
        self.ambient_audio_entity = Some(ambient_audio);

        let generator_audio = world.spawn_entities(AUDIO_SOURCE | LOCAL_TRANSFORM, 1)[0];
        world.core.set_local_transform(
            generator_audio,
            LocalTransform {
                translation: nalgebra_glm::vec3(-8.0, 1.0, -14.5),
                ..Default::default()
            },
        );
        world.core.set_audio_source(
            generator_audio,
            AudioSource::new("generator").with_spatial(true),
        );
        self.generator_audio_entity = Some(generator_audio);

        let rubble_audio = world.spawn_entities(AUDIO_SOURCE | LOCAL_TRANSFORM, 1)[0];
        world.core.set_local_transform(
            rubble_audio,
            LocalTransform {
                translation: nalgebra_glm::vec3(-4.5, 1.5, -16.0),
                ..Default::default()
            },
        );
        world.core.set_audio_source(
            rubble_audio,
            AudioSource::new("rubble")
                .with_spatial(true)
                .with_reverb(true),
        );
        self.rubble_audio_entity = Some(rubble_audio);

        let monster_audio = world.spawn_entities(AUDIO_SOURCE, 1)[0];
        world.core.set_audio_source(
            monster_audio,
            AudioSource::new("monster")
                .with_volume(0.6)
                .with_looping(true),
        );
        self.monster_audio_entity = Some(monster_audio);

        let footstep_audio = world.spawn_entities(AUDIO_SOURCE, 1)[0];
        world.core.set_audio_source(
            footstep_audio,
            AudioSource::new("footsteps")
                .with_volume(0.4)
                .with_looping(true),
        );
        self.footstep_audio_entity = Some(footstep_audio);

        let door_audio = world.spawn_entities(AUDIO_SOURCE, 1)[0];
        world
            .core
            .set_audio_source(door_audio, AudioSource::new("door_creak").with_volume(0.6));
        self.door_audio_entity = Some(door_audio);
    }

    fn run_systems(&mut self, world: &mut World) {
        update_overlays(self, world);

        if !self.audio_started
            && world.resources.audio.is_initialized()
            && let Some(entity) = self.ambient_audio_entity
            && let Some(source) = world.core.get_audio_source_mut(entity)
        {
            source.playing = true;
            self.audio_started = true;
        }

        if self.reading_note.is_some() {
            note_reading_system(self, world);
        }

        escape_key_exit_system(world);
        detect_input_mode(self, world);

        if !self.cutscene.active {
            nightshade::ecs::physics::character_controller::character_controller_input_system(
                world,
            );
            camera_look_system(self, world);
        }

        cutscene_system(self, world);
        monster_chase_system(self, world);

        lean_system(self, world);
        crouch_camera_system(self, world);

        interaction_system(self, world);
        update_doors_momentum(self, world);
        update_levers_momentum(self, world);
        update_lantern_light(self, world);
        update_flashlight(self, world);
        update_overhead_lights(self, world);
        update_interaction_prompt(self, world);
        update_objective(self, world);
        update_temporary_message(self, world);

        check_puzzle_state(self, world);

        update_footstep_audio(self, world);

        let dt = world.resources.window.timing.delta_time;
        let letterbox_speed = 3.0;
        let current = world.resources.graphics.letterbox_amount;
        let target = world.resources.graphics.letterbox_target;
        let diff = target - current;
        world.resources.graphics.letterbox_amount += diff * (letterbox_speed * dt).min(1.0);

        let fade_speed = 1.5;
        let fade_diff = self.fade_target - self.fade_amount;
        self.fade_amount += fade_diff * (fade_speed * dt).min(1.0);
    }

    fn on_keyboard_input(&mut self, world: &mut World, _key_code: KeyCode, _key_state: KeyState) {
        #[cfg(target_arch = "wasm32")]
        {
            if !world.resources.audio.is_initialized() {
                nightshade::ecs::audio::systems::lazy_initialize_audio_system(world);
            }
        }
        let _ = world;
    }

    fn on_mouse_input(&mut self, world: &mut World, _state: ElementState, _button: MouseButton) {
        #[cfg(target_arch = "wasm32")]
        {
            if !world.resources.audio.is_initialized() {
                nightshade::ecs::audio::systems::lazy_initialize_audio_system(world);
            }
        }
        let _ = world;
    }
}

fn update_footstep_audio(demo: &mut HorrorDemo, world: &mut World) {
    let Some(player_entity) = demo.player_entity else {
        return;
    };
    let Some(footstep_entity) = demo.footstep_audio_entity else {
        return;
    };

    let keyboard = &world.resources.input.keyboard;
    let w_pressed = keyboard.is_key_pressed(KeyCode::KeyW);
    let a_pressed = keyboard.is_key_pressed(KeyCode::KeyA);
    let s_pressed = keyboard.is_key_pressed(KeyCode::KeyS);
    let d_pressed = keyboard.is_key_pressed(KeyCode::KeyD);
    let movement_keys_pressed = w_pressed || a_pressed || s_pressed || d_pressed;

    let is_grounded = world
        .core
        .get_character_controller(player_entity)
        .map(|cc| cc.grounded)
        .unwrap_or(false);

    let is_moving = movement_keys_pressed && is_grounded && !demo.cutscene.active;

    if is_moving && !demo.was_moving {
        if let Some(source) = world.core.get_audio_source_mut(footstep_entity) {
            source.playing = true;
        }
    } else if !is_moving
        && demo.was_moving
        && let Some(source) = world.core.get_audio_source_mut(footstep_entity)
    {
        source.playing = false;
    }

    demo.was_moving = is_moving;
}
