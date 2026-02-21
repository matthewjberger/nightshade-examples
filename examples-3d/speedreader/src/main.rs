use nightshade::prelude::*;

struct SpeedReaderDemo {
    rsvp_text: Option<Entity>,
    words: Vec<String>,
    current_word_index: usize,
    last_word_time: f32,
    words_per_minute: f32,
    start_time: f32,
    paused: bool,
}

impl Default for SpeedReaderDemo {
    fn default() -> Self {
        let sample_text = "The quick brown fox jumps over the lazy dog. \
            Speed reading with RSVP allows you to read much faster \
            by eliminating eye movement. Each word is displayed \
            at a fixed focal point with the optimal recognition point \
            highlighted in red. This technique can dramatically \
            increase your reading speed while maintaining comprehension.";

        let words: Vec<String> = sample_text
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        Self {
            rsvp_text: None,
            words,
            current_word_index: 0,
            last_word_time: 0.0,
            words_per_minute: 300.0,
            start_time: 0.0,
            paused: false,
        }
    }
}

fn calculate_orp_index(word: &str) -> usize {
    let len = word.chars().count();
    match len {
        0 => 0,
        1 => 0,
        2..=5 => 1,
        6..=9 => 2,
        10..=13 => 3,
        _ => 4,
    }
}

fn create_orp_colors(word: &str, orp_index: usize) -> Vec<Option<nalgebra_glm::Vec4>> {
    word.chars()
        .enumerate()
        .map(|(index, _)| {
            if index == orp_index {
                Some(nalgebra_glm::vec4(1.0, 0.2, 0.2, 1.0))
            } else {
                None
            }
        })
        .collect()
}

fn spawn_rsvp_text(world: &mut World, word: &str) -> Entity {
    let orp_index = calculate_orp_index(word);

    let props = TextProperties {
        font_size: 72.0,
        color: nalgebra_glm::vec4(1.0, 1.0, 1.0, 1.0),
        alignment: TextAlignment::Left,
        vertical_alignment: VerticalAlignment::Middle,
        outline_width: 0.02,
        outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
        anchor_character: Some(orp_index),
        ..Default::default()
    };

    let entity = spawn_hud_text_with_properties(
        world,
        word,
        HudAnchor::Center,
        nalgebra_glm::vec2(0.0, 0.0),
        props,
    );

    let colors = create_orp_colors(word, orp_index);
    world.set_text_character_colors(
        entity,
        TextCharacterColors {
            colors,
            dirty: true,
        },
    );

    entity
}

fn update_rsvp_word(world: &mut World, entity: Entity, word: &str) {
    let orp_index = calculate_orp_index(word);

    if let Some(hud_text) = world.get_hud_text(entity) {
        let text_index = hud_text.text_index;
        world.resources.text_cache.set_text(text_index, word);
    }

    if let Some(hud_text) = world.get_hud_text_mut(entity) {
        hud_text.properties.anchor_character = Some(orp_index);
        hud_text.dirty = true;
    }

    let colors = create_orp_colors(word, orp_index);
    if let Some(char_colors) = world.get_text_character_colors_mut(entity) {
        char_colors.colors = colors;
        char_colors.dirty = true;
    }
}

fn spawn_camera(world: &mut World, position: nalgebra_glm::Vec3, name: String) -> Entity {
    let cameras = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM
            | nightshade::ecs::world::CAMERA,
        1,
    );

    let camera = cameras[0];

    if let Some(camera_name) = world.get_name_mut(camera) {
        *camera_name = Name(name);
    }

    if let Some(local_transform) = world.get_local_transform_mut(camera) {
        local_transform.translation = position;
    }

    if let Some(camera_component) = world.get_camera_mut(camera) {
        *camera_component = Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 45.0_f32.to_radians(),
                z_far: None,
                z_near: 0.01,
            }),
            smoothing: Some(Smoothing::default()),
        };
    }

    camera
}

impl State for SpeedReaderDemo {
    fn title(&self) -> &str {
        "Speed Reader"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;

        let camera_position = nalgebra_glm::vec3(0.0, 2.0, 8.0);
        let main_camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(main_camera);

        let first_word = self.words.first().cloned().unwrap_or_default();
        self.rsvp_text = Some(spawn_rsvp_text(world, &first_word));

        let title_props = TextProperties {
            font_size: 24.0,
            color: nalgebra_glm::vec4(0.6, 0.6, 0.6, 1.0),
            alignment: TextAlignment::Center,
            ..Default::default()
        };

        spawn_hud_text_with_properties(
            world,
            "RSVP Speed Reader",
            HudAnchor::TopCenter,
            nalgebra_glm::vec2(0.0, 20.0),
            title_props,
        );

        let guide_left = spawn_hud_text_with_properties(
            world,
            "|",
            HudAnchor::Center,
            nalgebra_glm::vec2(-2.0, -50.0),
            TextProperties {
                font_size: 24.0,
                color: nalgebra_glm::vec4(0.3, 0.3, 0.3, 1.0),
                ..Default::default()
            },
        );
        let _ = guide_left;

        let guide_right = spawn_hud_text_with_properties(
            world,
            "|",
            HudAnchor::Center,
            nalgebra_glm::vec2(2.0, -50.0),
            TextProperties {
                font_size: 24.0,
                color: nalgebra_glm::vec4(0.3, 0.3, 0.3, 1.0),
                ..Default::default()
            },
        );
        let _ = guide_right;

        let instructions = spawn_hud_text_with_properties(
            world,
            "UP/DOWN: Adjust speed | SPACE: Pause/Resume | R: Restart",
            HudAnchor::BottomCenter,
            nalgebra_glm::vec2(0.0, -30.0),
            TextProperties {
                font_size: 16.0,
                color: nalgebra_glm::vec4(0.5, 0.5, 0.5, 1.0),
                alignment: TextAlignment::Center,
                ..Default::default()
            },
        );
        let _ = instructions;

        self.start_time = (world.resources.window.timing.uptime_milliseconds as f32) / 1000.0;
        self.last_word_time = self.start_time;
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        let elapsed = (world.resources.window.timing.uptime_milliseconds as f32) / 1000.0;
        let word_interval = 60.0 / self.words_per_minute;

        if !self.paused && elapsed - self.last_word_time >= word_interval {
            self.last_word_time = elapsed;
            self.current_word_index = (self.current_word_index + 1) % self.words.len();

            if let Some(entity) = self.rsvp_text {
                let word = self.words[self.current_word_index].clone();
                update_rsvp_word(world, entity, &word);
            }
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state == KeyState::Pressed {
            match key {
                KeyCode::Space => {
                    self.paused = !self.paused;
                }
                KeyCode::ArrowUp => {
                    self.words_per_minute = (self.words_per_minute + 50.0).min(1000.0);
                }
                KeyCode::ArrowDown => {
                    self.words_per_minute = (self.words_per_minute - 50.0).max(50.0);
                }
                KeyCode::KeyR => {
                    self.current_word_index = 0;
                    let elapsed =
                        (world.resources.window.timing.uptime_milliseconds as f32) / 1000.0;
                    self.last_word_time = elapsed;

                    if let Some(entity) = self.rsvp_text {
                        let word = self.words[self.current_word_index].clone();
                        update_rsvp_word(world, entity, &word);
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SpeedReaderDemo::default())
}
