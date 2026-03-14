use nightshade::ecs::input::resources::mouse::MouseState;
use nightshade::ecs::text::components::TextProperties;
use nightshade::prelude::*;

const SLOT_CIRCLE: u32 = 0;
const SLOT_SQUARE: u32 = 1;
const SLOT_RING: u32 = 2;
const SLOT_SOFT_CIRCLE: u32 = 3;

const TAG_POSITION: u32 = 0;
const TAG_SCALE: u32 = 1;
const TAG_ALPHA: u32 = 2;
const TAG_COLOR: u32 = 3;

const SCENE_FILE: &str = "effects_scene.json";

struct SpriteShowcase {
    camera_entity: Option<Entity>,
    initialized: bool,
    uv_max_table: Vec<Vec2>,

    tween_entities: Vec<Entity>,
    emitter_entities: Vec<Entity>,

    selected_particle: usize,
    held_emitter: Option<Entity>,
    show_save_feedback: f32,
    show_load_feedback: f32,
}

impl Default for SpriteShowcase {
    fn default() -> Self {
        Self {
            camera_entity: None,
            initialized: false,
            uv_max_table: Vec::new(),
            tween_entities: Vec::new(),
            emitter_entities: Vec::new(),
            selected_particle: 0,
            held_emitter: None,
            show_save_feedback: 0.0,
            show_load_feedback: 0.0,
        }
    }
}

fn generate_circle_texture(size: u32) -> Vec<u8> {
    let mut data = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0 - 0.5;
    let radius = size as f32 / 2.0 - 1.0;
    for pixel_y in 0..size {
        for pixel_x in 0..size {
            let distance_x = pixel_x as f32 - center;
            let distance_y = pixel_y as f32 - center;
            let distance = (distance_x * distance_x + distance_y * distance_y).sqrt();
            let index = ((pixel_y * size + pixel_x) * 4) as usize;
            if distance < radius {
                data[index] = 255;
                data[index + 1] = 255;
                data[index + 2] = 255;
                data[index + 3] = 255;
            }
        }
    }
    data
}

fn generate_soft_circle_texture(size: u32) -> Vec<u8> {
    let mut data = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0 - 0.5;
    let radius = size as f32 / 2.0 - 1.0;
    for pixel_y in 0..size {
        for pixel_x in 0..size {
            let distance_x = pixel_x as f32 - center;
            let distance_y = pixel_y as f32 - center;
            let distance = (distance_x * distance_x + distance_y * distance_y).sqrt();
            let index = ((pixel_y * size + pixel_x) * 4) as usize;
            if distance < radius {
                let alpha = (1.0 - distance / radius).powf(1.5);
                data[index] = 255;
                data[index + 1] = 255;
                data[index + 2] = 255;
                data[index + 3] = (alpha * 255.0) as u8;
            }
        }
    }
    data
}

fn generate_ring_texture(size: u32) -> Vec<u8> {
    let mut data = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0 - 0.5;
    let outer_radius = size as f32 / 2.0 - 1.0;
    let inner_radius = outer_radius * 0.6;
    for pixel_y in 0..size {
        for pixel_x in 0..size {
            let distance_x = pixel_x as f32 - center;
            let distance_y = pixel_y as f32 - center;
            let distance = (distance_x * distance_x + distance_y * distance_y).sqrt();
            let index = ((pixel_y * size + pixel_x) * 4) as usize;
            if distance < outer_radius && distance > inner_radius {
                let edge_distance =
                    ((distance - inner_radius) / (outer_radius - inner_radius) - 0.5).abs() * 2.0;
                let alpha = (1.0 - edge_distance).clamp(0.0, 1.0).powf(0.5);
                data[index] = 255;
                data[index + 1] = 255;
                data[index + 2] = 255;
                data[index + 3] = (alpha * 255.0) as u8;
            }
        }
    }
    data
}

fn load_procedural_textures(world: &mut World) -> Vec<Vec2> {
    let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
    let mut uv_max_table = vec![Vec2::new(1.0, 1.0); 128];

    let texture_size = 64u32;

    let textures: [(u32, Vec<u8>); 4] = [
        (SLOT_CIRCLE, generate_circle_texture(texture_size)),
        (
            SLOT_SQUARE,
            vec![255u8; (texture_size * texture_size * 4) as usize],
        ),
        (SLOT_RING, generate_ring_texture(texture_size)),
        (SLOT_SOFT_CIRCLE, generate_soft_circle_texture(texture_size)),
    ];

    for (slot, rgba_data) in &textures {
        world
            .resources
            .command_queue
            .push(WorldCommand::UploadSpriteTexture {
                slot: *slot,
                rgba_data: rgba_data.clone(),
                width: texture_size,
                height: texture_size,
            });

        let half_texel_x = 0.5 / atlas_slot_size.0 as f32;
        let half_texel_y = 0.5 / atlas_slot_size.1 as f32;
        uv_max_table[*slot as usize] = Vec2::new(
            texture_size as f32 / atlas_slot_size.0 as f32 - half_texel_x,
            texture_size as f32 / atlas_slot_size.1 as f32 - half_texel_y,
        );
    }

    uv_max_table
}

fn uv_for_slot(uv_max_table: &[Vec2], slot: u32) -> (Vec2, Vec2) {
    let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
    let half_texel = Vec2::new(
        0.5 / atlas_slot_size.0 as f32,
        0.5 / atlas_slot_size.1 as f32,
    );
    (half_texel, uv_max_table[slot as usize])
}

fn spawn_textured_sprite(
    world: &mut World,
    position: Vec2,
    depth: f32,
    size: Vec2,
    texture_slot: u32,
    uv_max_table: &[Vec2],
) -> Entity {
    let entity = spawn_sprite(world, position, size);
    let (uv_min, uv_max) = uv_for_slot(uv_max_table, texture_slot);
    if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
        sprite.depth = depth;
        sprite.texture_index = texture_slot;
        sprite.texture_index2 = texture_slot;
        sprite.uv_min = uv_min;
        sprite.uv_max = uv_max;
    }
    entity
}

const EASING_NAMES: &[(&str, EasingFunction)] = &[
    ("Linear", EasingFunction::Linear),
    ("QuadInOut", EasingFunction::QuadInOut),
    ("CubicInOut", EasingFunction::CubicInOut),
    ("QuartInOut", EasingFunction::QuartInOut),
    ("SineInOut", EasingFunction::SineInOut),
    ("ExpoInOut", EasingFunction::ExpoInOut),
    ("CircInOut", EasingFunction::CircInOut),
    ("BackInOut", EasingFunction::BackInOut),
    ("ElasticOut", EasingFunction::ElasticOut),
    ("BounceOut", EasingFunction::BounceOut),
];

const PARTICLE_PRESETS: &[&str] = &[
    "Explosion",
    "Fire Trail",
    "Smoke",
    "Sparks",
    "Snow",
    "Fountain",
];

impl SpriteShowcase {
    fn build_tween_showcase(&mut self, world: &mut World) {
        let start_x = -350.0;
        let end_x = 350.0;
        let start_y = 200.0;
        let spacing_y = 40.0;
        let label_x = -460.0;

        for (index, &(name, easing)) in EASING_NAMES.iter().enumerate() {
            let position_y = start_y - index as f32 * spacing_y;

            let entity = spawn_textured_sprite(
                world,
                Vec2::new(start_x, position_y),
                10.0,
                Vec2::new(24.0, 24.0),
                SLOT_CIRCLE,
                &self.uv_max_table,
            );

            let hue = index as f32 / EASING_NAMES.len() as f32;
            let (red, green, blue) = hue_to_rgb(hue);

            if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                sprite.color = [red, green, blue, 1.0];
            }

            spawn_ui_text_with_properties(
                world,
                name,
                Vec2::new(label_x, position_y - 5.0),
                TextProperties {
                    font_size: 14.0,
                    color: Vec4::new(red, green, blue, 1.0),
                    ..Default::default()
                },
            );

            world.core.add_components(entity, TWEEN);
            let mut tween = Tween::new();

            tween.add_track(
                TweenTrack::new(
                    TweenValue::Vec2(Vec2::new(start_x, position_y)),
                    TweenValue::Vec2(Vec2::new(end_x, position_y)),
                    2.5,
                )
                .with_easing(easing)
                .with_loop_mode(TweenLoopMode::PingPong)
                .with_tag(TAG_POSITION),
            );

            tween.add_track(
                TweenTrack::new(TweenValue::F32(0.8), TweenValue::F32(1.6), 2.5)
                    .with_easing(easing)
                    .with_loop_mode(TweenLoopMode::PingPong)
                    .with_tag(TAG_SCALE),
            );

            world.core.set_tween(entity, tween);
            self.tween_entities.push(entity);
        }

        let color_sprite = spawn_textured_sprite(
            world,
            Vec2::new(0.0, start_y + 60.0),
            10.0,
            Vec2::new(60.0, 60.0),
            SLOT_RING,
            &self.uv_max_table,
        );
        world.core.add_components(color_sprite, TWEEN);
        let mut color_tween = Tween::new();
        color_tween.add_track(
            TweenTrack::new(
                TweenValue::Vec4(Vec4::new(1.0, 0.2, 0.2, 1.0)),
                TweenValue::Vec4(Vec4::new(0.2, 0.5, 1.0, 1.0)),
                3.0,
            )
            .with_easing(EasingFunction::SineInOut)
            .with_loop_mode(TweenLoopMode::PingPong)
            .with_tag(TAG_COLOR),
        );
        color_tween.add_track(
            TweenTrack::new(TweenValue::F32(1.0), TweenValue::F32(2.5), 2.0)
                .with_easing(EasingFunction::ElasticOut)
                .with_loop_mode(TweenLoopMode::PingPong)
                .with_tag(TAG_SCALE),
        );
        world.core.set_tween(color_sprite, color_tween);
        self.tween_entities.push(color_sprite);
    }

    fn build_particle_showcase(&mut self, world: &mut World) {
        let emitter_positions = [
            Vec2::new(-300.0, -250.0),
            Vec2::new(-100.0, -250.0),
            Vec2::new(100.0, -250.0),
            Vec2::new(300.0, -250.0),
        ];

        let (uv_min, uv_max) = uv_for_slot(&self.uv_max_table, SLOT_SOFT_CIRCLE);

        let emitter_configs = [
            SpriteParticleEmitter::fire_trail(emitter_positions[0].x, emitter_positions[0].y)
                .with_texture(SLOT_SOFT_CIRCLE)
                .with_uv(uv_min, uv_max)
                .with_depth(20.0),
            SpriteParticleEmitter::smoke(emitter_positions[1].x, emitter_positions[1].y)
                .with_texture(SLOT_SOFT_CIRCLE)
                .with_uv(uv_min, uv_max)
                .with_depth(20.0),
            {
                let mut fountain = SpriteParticleEmitter {
                    enabled: true,
                    spawn_rate: 60.0,
                    max_particles: 500,
                    lifetime_min: 1.0,
                    lifetime_max: 2.0,
                    velocity_min: Vec2::new(-40.0, 100.0),
                    velocity_max: Vec2::new(40.0, 200.0),
                    gravity: Vec2::new(0.0, -200.0),
                    drag: 0.02,
                    size_start: Vec2::new(6.0, 6.0),
                    size_end: Vec2::new(3.0, 3.0),
                    color: ColorRange2D::new([0.3, 0.6, 1.0, 1.0], [0.1, 0.3, 1.0, 0.0]),
                    blend_mode: SpriteBlendMode::Additive,
                    texture_index: SLOT_SOFT_CIRCLE,
                    uv_min,
                    uv_max,
                    depth: 20.0,
                    anchor: Vec2::new(emitter_positions[2].x, emitter_positions[2].y),
                    shape: EmitterShape2D::Point,
                    ..Default::default()
                };
                fountain.rotation_speed_min = -1.0;
                fountain.rotation_speed_max = 1.0;
                fountain
            },
            {
                let mut snow = SpriteParticleEmitter {
                    enabled: true,
                    spawn_rate: 40.0,
                    max_particles: 400,
                    lifetime_min: 2.0,
                    lifetime_max: 4.0,
                    velocity_min: Vec2::new(-30.0, -20.0),
                    velocity_max: Vec2::new(30.0, -60.0),
                    gravity: Vec2::new(0.0, -10.0),
                    drag: 0.1,
                    size_start: Vec2::new(4.0, 4.0),
                    size_end: Vec2::new(2.0, 2.0),
                    color: ColorRange2D::new([1.0, 1.0, 1.0, 0.9], [0.8, 0.9, 1.0, 0.0]),
                    blend_mode: SpriteBlendMode::Alpha,
                    texture_index: SLOT_SOFT_CIRCLE,
                    uv_min,
                    uv_max,
                    depth: 20.0,
                    anchor: Vec2::new(emitter_positions[3].x, emitter_positions[3].y + 100.0),
                    shape: EmitterShape2D::Rectangle {
                        half_extents: Vec2::new(80.0, 5.0),
                    },
                    ..Default::default()
                };
                snow.rotation_speed_min = -0.5;
                snow.rotation_speed_max = 0.5;
                snow
            },
        ];

        for emitter_config in emitter_configs {
            let entity = world.spawn();
            world.sprite2d.set_sprite_particle_emitter(entity, emitter_config);
            self.emitter_entities.push(entity);
        }
    }

    fn apply_tweens(&self, world: &mut World) {
        for &entity in &self.tween_entities {
            let tween_data = world.core.get_tween(entity).cloned();
            let Some(tween) = tween_data else {
                continue;
            };

            if let Some(track) = tween.track_by_tag(TAG_POSITION) {
                let position = track.value_vec2();
                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.position = position;
                }
            }

            if let Some(track) = tween.track_by_tag(TAG_SCALE) {
                let scale = track.value_f32();
                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.scale = Vec2::new(scale, scale);
                }
            }

            if let Some(track) = tween.track_by_tag(TAG_ALPHA) {
                let alpha = track.value_f32();
                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.color[3] = alpha;
                }
            }

            if let Some(track) = tween.track_by_tag(TAG_COLOR) {
                let color = track.value_vec4();
                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.color = [color.x, color.y, color.z, color.w];
                }
            }
        }
    }

    fn screen_to_world(&self, world: &World, screen_position: Vec2) -> Vec2 {
        let window_size = world
            .resources
            .window
            .handle
            .as_ref()
            .map(|handle| {
                let size = handle.inner_size();
                Vec2::new(size.width as f32, size.height as f32)
            })
            .unwrap_or(Vec2::new(1920.0, 1080.0));

        let camera_position = self
            .camera_entity
            .and_then(|entity| world.core.get_local_transform(entity))
            .map(|transform| Vec2::new(transform.translation.x, transform.translation.y))
            .unwrap_or(Vec2::zeros());

        let half_view = self
            .camera_entity
            .and_then(|entity| world.core.get_camera(entity))
            .map(|camera| {
                if let Projection::Orthographic(ortho) = &camera.projection {
                    Vec2::new(ortho.x_mag, ortho.y_mag)
                } else {
                    Vec2::new(480.0, 270.0)
                }
            })
            .unwrap_or(Vec2::new(480.0, 270.0));

        let normalized_x = screen_position.x / window_size.x - 0.5;
        let normalized_y = -(screen_position.y / window_size.y - 0.5);

        Vec2::new(
            camera_position.x + normalized_x * 2.0 * half_view.x,
            camera_position.y + normalized_y * 2.0 * half_view.y,
        )
    }

    fn spawn_stream_emitter(
        &self,
        world: &mut World,
        position: Vec2,
        preset_index: usize,
    ) -> Entity {
        let (uv_min, uv_max) = uv_for_slot(&self.uv_max_table, SLOT_SOFT_CIRCLE);

        let emitter = match preset_index {
            0 => SpriteParticleEmitter {
                spawn_rate: 80.0,
                ..SpriteParticleEmitter::explosion(position.x, position.y)
                    .with_texture(SLOT_SOFT_CIRCLE)
                    .with_uv(uv_min, uv_max)
                    .with_depth(30.0)
            },
            1 => SpriteParticleEmitter::fire_trail(position.x, position.y)
                .with_texture(SLOT_SOFT_CIRCLE)
                .with_uv(uv_min, uv_max)
                .with_depth(30.0),
            2 => SpriteParticleEmitter::smoke(position.x, position.y)
                .with_texture(SLOT_SOFT_CIRCLE)
                .with_uv(uv_min, uv_max)
                .with_depth(30.0),
            3 => SpriteParticleEmitter {
                spawn_rate: 60.0,
                ..SpriteParticleEmitter::sparks(position.x, position.y)
                    .with_texture(SLOT_SOFT_CIRCLE)
                    .with_uv(uv_min, uv_max)
                    .with_depth(30.0)
            },
            4 => {
                let mut snow = SpriteParticleEmitter {
                    enabled: true,
                    spawn_rate: 40.0,
                    max_particles: 400,
                    lifetime_min: 1.5,
                    lifetime_max: 3.0,
                    velocity_min: Vec2::new(-60.0, -10.0),
                    velocity_max: Vec2::new(60.0, -50.0),
                    gravity: Vec2::new(0.0, -15.0),
                    drag: 0.1,
                    size_start: Vec2::new(5.0, 5.0),
                    size_end: Vec2::new(2.0, 2.0),
                    color: ColorRange2D::new([1.0, 1.0, 1.0, 1.0], [0.8, 0.9, 1.0, 0.0]),
                    blend_mode: SpriteBlendMode::Alpha,
                    texture_index: SLOT_SOFT_CIRCLE,
                    uv_min,
                    uv_max,
                    depth: 30.0,
                    anchor: Vec2::new(position.x, position.y),
                    shape: EmitterShape2D::Circle { radius: 20.0 },
                    ..Default::default()
                };
                snow.rotation_speed_min = -0.5;
                snow.rotation_speed_max = 0.5;
                snow
            }
            _ => {
                let mut fountain = SpriteParticleEmitter {
                    enabled: true,
                    spawn_rate: 60.0,
                    max_particles: 500,
                    lifetime_min: 0.8,
                    lifetime_max: 1.5,
                    velocity_min: Vec2::new(-60.0, 80.0),
                    velocity_max: Vec2::new(60.0, 200.0),
                    gravity: Vec2::new(0.0, -250.0),
                    drag: 0.05,
                    size_start: Vec2::new(5.0, 5.0),
                    size_end: Vec2::new(2.0, 2.0),
                    color: ColorRange2D::new([0.3, 0.8, 1.0, 1.0], [0.1, 0.3, 1.0, 0.0]),
                    blend_mode: SpriteBlendMode::Additive,
                    texture_index: SLOT_SOFT_CIRCLE,
                    uv_min,
                    uv_max,
                    depth: 30.0,
                    anchor: Vec2::new(position.x, position.y),
                    shape: EmitterShape2D::Point,
                    ..Default::default()
                };
                fountain.rotation_speed_min = -2.0;
                fountain.rotation_speed_max = 2.0;
                fountain
            }
        };

        let mut final_emitter = emitter;
        final_emitter.one_shot = false;
        final_emitter.burst_count = 0;

        let entity = world.spawn();
        world.sprite2d.set_sprite_particle_emitter(entity, final_emitter);
        entity
    }

    fn handle_mouse_particles(&mut self, world: &mut World) {
        let mouse_state = world.resources.input.mouse.state;
        let screen_position = Vec2::new(
            world.resources.input.mouse.position.x,
            world.resources.input.mouse.position.y,
        );

        if mouse_state.contains(MouseState::LEFT_JUST_PRESSED) {
            let world_position = self.screen_to_world(world, screen_position);
            let entity = self.spawn_stream_emitter(world, world_position, self.selected_particle);
            self.held_emitter = Some(entity);
        } else if mouse_state.contains(MouseState::LEFT_CLICKED) {
            if let Some(entity) = self.held_emitter {
                let world_position = self.screen_to_world(world, screen_position);
                if let Some(emitter) = world.sprite2d.get_sprite_particle_emitter_mut(entity) {
                    emitter.anchor = world_position;
                }
            }
        } else if let Some(entity) = self.held_emitter.take()
            && let Some(emitter) = world.sprite2d.get_sprite_particle_emitter_mut(entity)
        {
            emitter.enabled = false;
        }
    }

    fn save_scene(&mut self, world: &World) {
        let scene = world_to_scene_2d(world, Some("Sprite Showcase"));
        let path = std::path::Path::new(SCENE_FILE);
        if save_scene_2d(&scene, path).is_ok() {
            self.show_save_feedback = 2.0;
        }
    }

    fn load_scene(&mut self, world: &mut World) {
        let path = std::path::Path::new(SCENE_FILE);
        if let Ok(scene) = load_scene_2d(path)
            && spawn_scene_2d(world, &scene).is_ok()
        {
            self.show_load_feedback = 2.0;
        }
    }
}

fn hue_to_rgb(hue: f32) -> (f32, f32, f32) {
    let hue = hue.fract();
    let segment = hue * 6.0;
    let fraction = segment.fract();
    match segment as u32 {
        0 => (1.0, fraction, 0.0),
        1 => (1.0 - fraction, 1.0, 0.0),
        2 => (0.0, 1.0, fraction),
        3 => (0.0, 1.0 - fraction, 1.0),
        4 => (fraction, 0.0, 1.0),
        _ => (1.0, 0.0, 1.0 - fraction),
    }
}

impl State for SpriteShowcase {
    fn title(&self) -> &str {
        "Sprite Showcase"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.clear_color = [0.05, 0.05, 0.12, 1.0];
        world.resources.user_interface.enabled = true;

        let camera = spawn_ortho_camera(world, Vec2::new(0.0, 0.0));
        self.camera_entity = Some(camera);

        if let Some(camera_data) = world.core.get_camera_mut(camera)
            && let Projection::Orthographic(ref mut ortho) = camera_data.projection
        {
            ortho.x_mag = 480.0;
            ortho.y_mag = 270.0;
        }

        self.uv_max_table = load_procedural_textures(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        if !self.initialized {
            self.initialized = true;
            self.build_tween_showcase(world);
            self.build_particle_showcase(world);
        }

        let delta_time = world.resources.window.timing.delta_time;

        escape_key_exit_system(world);

        self.apply_tweens(world);
        self.handle_mouse_particles(world);

        if self.show_save_feedback > 0.0 {
            self.show_save_feedback -= delta_time;
        }
        if self.show_load_feedback > 0.0 {
            self.show_load_feedback -= delta_time;
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Sprite Showcase")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                let fps = world.resources.window.timing.frames_per_second;
                ui.label(format!("FPS: {:.0}", fps));
                ui.separator();

                ui.heading("Tweening");
                ui.label("10 easing curves animating position and scale simultaneously.");
                ui.label("Top ring cycles color via SineInOut + scale via ElasticOut.");
                ui.separator();

                ui.heading("Particles");
                ui.label("4 continuous emitters: fire, smoke, fountain, snow.");
                ui.label("Click and hold to stream particles. Release to stop.");

                ui.horizontal(|ui| {
                    ui.label("Click spawns:");
                    for (index, &name) in PARTICLE_PRESETS.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_particle == index, name)
                            .clicked()
                        {
                            self.selected_particle = index;
                        }
                    }
                });
                ui.separator();

                ui.heading("Scene Serialization");
                ui.horizontal(|ui| {
                    if ui.button("Save Scene").clicked() {
                        self.save_scene(world);
                    }
                    if ui.button("Load Scene").clicked() {
                        self.load_scene(world);
                    }
                });
                if self.show_save_feedback > 0.0 {
                    ui.colored_label(egui::Color32::GREEN, format!("Saved to {SCENE_FILE}"));
                }
                if self.show_load_feedback > 0.0 {
                    ui.colored_label(egui::Color32::GREEN, format!("Loaded from {SCENE_FILE}"));
                }
                ui.label("Saves all sprites, tweens, and emitters to JSON.");
            });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SpriteShowcase::default())
}
