use nightshade::prelude::*;
use rand::Rng;

const GRAVITY: f32 = -500.0;
const MIN_X: f32 = 0.0;
const MIN_Y: f32 = 0.0;
const BUNNY_SIZE: f32 = 32.0;
const INITIAL_BUNNIES: usize = 0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(BunnyWorld::default())?;
    Ok(())
}

freecs::ecs! {
    BunnyWorld {
        entity_handle: EntityHandle => ENTITY_HANDLE,
        bunny_physics: BunnyPhysics => BUNNY_PHYSICS,
    }
    BunnyResources {
        max_x: f32,
        max_y: f32,
        lowest_fps: f32,
        highest_fps: f32,
        texture_count: u32,
        frame_times: Vec<f32>,
        frame_time_index: usize,
        bunny_counter: usize,
        auto_spawn_stopped: bool,
        sustained_low_fps: f32,
        sustained_high_fps: f32,
        sustained_low_count: usize,
        sustained_high_count: usize,
        target_fps: f32,
        pending_target_fps: f32,
        fps_hud_text: Option<Entity>,
        bunny_count_hud_text: Option<Entity>,
        target_fps_hud_text: Option<Entity>,
    }
}

impl State for BunnyWorld {
    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.user_interface.enabled = true;

        if let Some(window_handle) = &world.resources.window.handle {
            let size = window_handle.inner_size();
            self.resources.max_x = size.width as f32;
            self.resources.max_y = size.height as f32;
        }

        let half_x = self.resources.max_x / 2.0;
        let half_y = self.resources.max_y / 2.0;
        let camera = spawn_ortho_camera(world, Vec2::new(half_x, half_y));
        if let Some(camera_data) = world.get_camera_mut(camera)
            && let Projection::Orthographic(ref mut ortho) = camera_data.projection
        {
            ortho.x_mag = half_x;
            ortho.y_mag = half_y;
        }

        self.resources.lowest_fps = 60.0;
        self.resources.highest_fps = 60.0;
        self.resources.texture_count = 128;
        self.resources.frame_times = vec![0.0; 60];
        self.resources.frame_time_index = 0;
        self.resources.bunny_counter = 0;
        self.resources.auto_spawn_stopped = false;
        self.resources.sustained_low_fps = 60.0;
        self.resources.sustained_high_fps = 60.0;
        self.resources.sustained_low_count = 0;
        self.resources.sustained_high_count = 0;
        self.resources.target_fps = 30.0;
        self.resources.pending_target_fps = 30.0;

        let fps_text = spawn_hud_text_with_properties(
            world,
            "FPS: 0",
            HudAnchor::TopRight,
            Vec2::new(-10.0, 10.0),
            TextProperties {
                font_size: 48.0,
                color: Vec4::new(0.0, 1.0, 0.0, 1.0),
                ..Default::default()
            },
        );
        self.resources.fps_hud_text = Some(fps_text);

        let bunny_count_text = spawn_hud_text_with_properties(
            world,
            "Bunnies: 0",
            HudAnchor::TopRight,
            Vec2::new(-10.0, 70.0),
            TextProperties {
                font_size: 32.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                ..Default::default()
            },
        );
        self.resources.bunny_count_hud_text = Some(bunny_count_text);

        let target_fps_text = spawn_hud_text_with_properties(
            world,
            "Target FPS: 30",
            HudAnchor::TopRight,
            Vec2::new(-10.0, 115.0),
            TextProperties {
                font_size: 28.0,
                color: Vec4::new(0.8, 0.8, 0.8, 1.0),
                ..Default::default()
            },
        );
        self.resources.target_fps_hud_text = Some(target_fps_text);

        generate_gradient_textures(world);

        spawn_bunnies(world, self, INITIAL_BUNNIES);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        if let Some(window_handle) = &world.resources.window.handle {
            let size = window_handle.inner_size();
            self.resources.max_x = size.width as f32;
            self.resources.max_y = size.height as f32;
        }

        if let Some(fps_text_entity) = self.resources.fps_hud_text {
            let fps = world.resources.window.timing.frames_per_second;
            let target_fps = self.resources.target_fps;
            let lower_threshold = target_fps - 4.0;
            let upper_threshold = target_fps + 4.0;

            let fps_color = if fps >= lower_threshold && fps <= upper_threshold {
                Vec4::new(0.0, 1.0, 0.0, 1.0)
            } else if fps > upper_threshold {
                Vec4::new(1.0, 1.0, 1.0, 1.0)
            } else {
                Vec4::new(1.0, 0.65, 0.0, 1.0)
            };

            let text_index = world.get_hud_text(fps_text_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("FPS: {:.0}", fps));
                if let Some(hud_text) = world.get_hud_text_mut(fps_text_entity) {
                    hud_text.properties.color = fps_color;
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(bunny_count_entity) = self.resources.bunny_count_hud_text {
            let bunny_count: Vec<_> = self.query_entities(BUNNY_PHYSICS).collect();
            let bunny_count = bunny_count.len();
            let text_index = world.get_hud_text(bunny_count_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world.resources.text_cache.set_text(
                    text_index,
                    format!("Bunnies: {}", format_number_with_commas(bunny_count)),
                );
                if let Some(hud_text) = world.get_hud_text_mut(bunny_count_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(target_fps_entity) = self.resources.target_fps_hud_text {
            let target_fps = self.resources.target_fps;
            let text_index = world.get_hud_text(target_fps_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("Target FPS: {:.0}", target_fps));
                if let Some(hud_text) = world.get_hud_text_mut(target_fps_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        update_sustained_fps_tracking(world, self);
        auto_spawn_system(world, self);
        update_bunnies_system(world, self);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        let bunny_count: Vec<_> = self.query_entities(BUNNY_PHYSICS).collect();
        let bunny_count = bunny_count.len();

        let fps = world.resources.window.timing.frames_per_second;

        let avg_frame_time = if !self.resources.frame_times.is_empty() {
            self.resources.frame_times.iter().sum::<f32>() / self.resources.frame_times.len() as f32
        } else {
            0.0
        };

        let avg_fps = if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            0.0
        };

        let target_fps = self.resources.target_fps;
        let lower_threshold = target_fps - 4.0;
        let upper_threshold = target_fps + 4.0;

        let resolution = if let Some(window_handle) = &world.resources.window.handle {
            let size = window_handle.inner_size();
            format!("{}x{}", size.width, size.height)
        } else {
            "Unknown".to_string()
        };

        egui::Window::new("Bunnymark")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.heading("Sprite Rendering Benchmark");
                ui.separator();

                ui.label(format!("Resolution: {}", resolution));
                ui.label(format!(
                    "Bunnies: {}",
                    format_number_with_commas(bunny_count)
                ));
                ui.label(format!(
                    "FPS: {:.0} (Low: {:.0} High: {:.0})",
                    fps, self.resources.lowest_fps, self.resources.highest_fps
                ));
                ui.label(format!("Frame Time: {:.1}ms", avg_frame_time));

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Target FPS:");
                    ui.add(
                        egui::Slider::new(&mut self.resources.pending_target_fps, 30.0..=144.0)
                            .step_by(1.0),
                    );

                    let apply_enabled =
                        (self.resources.pending_target_fps - self.resources.target_fps).abs() > 0.1;
                    if ui
                        .add_enabled(apply_enabled, egui::Button::new("Apply"))
                        .clicked()
                    {
                        self.resources.target_fps = self.resources.pending_target_fps;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Set FPS:");
                    if ui.button("30").clicked() {
                        self.resources.target_fps = 30.0;
                        self.resources.pending_target_fps = 30.0;
                    }
                    if ui.button("60").clicked() {
                        self.resources.target_fps = 60.0;
                        self.resources.pending_target_fps = 60.0;
                    }
                    if ui.button("75").clicked() {
                        self.resources.target_fps = 75.0;
                        self.resources.pending_target_fps = 75.0;
                    }
                    if ui.button("90").clicked() {
                        self.resources.target_fps = 90.0;
                        self.resources.pending_target_fps = 90.0;
                    }
                    if ui.button("120").clicked() {
                        self.resources.target_fps = 120.0;
                        self.resources.pending_target_fps = 120.0;
                    }
                    if ui.button("144").clicked() {
                        self.resources.target_fps = 144.0;
                        self.resources.pending_target_fps = 144.0;
                    }
                });

                ui.separator();

                if !self.resources.auto_spawn_stopped && bunny_count >= 1000 {
                    if avg_fps < lower_threshold {
                        ui.colored_label(egui::Color32::YELLOW, "Despawning bunnies");
                    } else if avg_fps > upper_threshold {
                        ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "Spawning bunnies");
                    } else {
                        ui.colored_label(egui::Color32::GREEN, "Stable");
                    }
                } else if bunny_count < 1000 {
                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "Spawning bunnies");
                }
            });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EntityHandle(pub Entity);

#[derive(Debug, Clone, Copy, Default)]
pub struct BunnyPhysics {
    pub velocity: Vec2,
    pub rotation_angle: f32,
}

fn spawn_bunny(world: &mut World, bunny_world: &mut BunnyWorld, position: Vec2) -> freecs::Entity {
    let mut rng = rand::rng();

    let engine_entity = world.spawn_entities(
        SPRITE | VISIBILITY | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
        1,
    )[0];

    if let Some(sprite) = world.get_sprite_mut(engine_entity) {
        sprite.size = Vec2::new(BUNNY_SIZE, BUNNY_SIZE);
        sprite.texture_index = 0;

        let texture_count = bunny_world.resources.texture_count.max(2);
        sprite.texture_index2 = rng.random_range(1..texture_count);
        sprite.blend_factor = rng.random::<f32>() * 0.7;

        sprite.color = [
            0.7 + rng.random::<f32>() * 0.3,
            0.7 + rng.random::<f32>() * 0.3,
            0.7 + rng.random::<f32>() * 0.3,
            1.0,
        ];

        let texture_size = 64.0;
        let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
        sprite.uv_min = Vec2::new(0.0, 0.0);
        sprite.uv_max = Vec2::new(
            texture_size / atlas_slot_size.0 as f32,
            texture_size / atlas_slot_size.1 as f32,
        );
    }

    let rotation_angle = rng.random::<f32>() * std::f32::consts::TAU;
    let scale = 0.5 + rng.random::<f32>() * 1.0;

    let depth = rng.random::<f32>() * 100.0;

    if let Some(transform) = world.get_local_transform_mut(engine_entity) {
        transform.translation = Vec3::new(position.x, position.y, depth);
        transform.rotation = nalgebra_glm::quat_angle_axis(rotation_angle, &Vec3::z());
        transform.scale = Vec3::new(scale, scale, 1.0);
    }
    mark_local_transform_dirty(world, engine_entity);

    bunny_world.resources.bunny_counter += 1;

    let velocity = Vec2::new(
        rng.random_range(-250.0..250.0),
        rng.random_range(0.0..500.0),
    );

    let game_entity = bunny_world.spawn_entities(ENTITY_HANDLE | BUNNY_PHYSICS, 1)[0];

    bunny_world.set_entity_handle(game_entity, EntityHandle(engine_entity));
    bunny_world.set_bunny_physics(
        game_entity,
        BunnyPhysics {
            velocity,
            rotation_angle,
        },
    );

    game_entity
}

fn spawn_bunnies(world: &mut World, bunny_world: &mut BunnyWorld, count: usize) {
    let spawn_x = bunny_world.resources.max_x * 0.5;
    let spawn_y = bunny_world.resources.max_y * 0.75;

    for _ in 0..count {
        spawn_bunny(world, bunny_world, Vec2::new(spawn_x, spawn_y));
    }
}

fn update_bunnies_system(world: &mut World, bunny_world: &mut BunnyWorld) {
    let delta_time = world.resources.window.timing.delta_time;
    let mut rng = rand::rng();

    let entities: Vec<_> = bunny_world
        .query_entities(BUNNY_PHYSICS | ENTITY_HANDLE)
        .collect();

    let max_x = bunny_world.resources.max_x;
    let max_y = bunny_world.resources.max_y;

    for entity in entities {
        let handle = bunny_world.get_entity_handle(entity).copied();

        if let (Some(physics), Some(handle)) = (bunny_world.get_bunny_physics_mut(entity), handle)
            && let Some(transform) = world.get_local_transform_mut(handle.0)
        {
            physics.velocity.y += GRAVITY * delta_time;

            transform.translation.x += physics.velocity.x * delta_time;
            transform.translation.y += physics.velocity.y * delta_time;
            physics.rotation_angle += physics.velocity.x * 0.01 * delta_time;
            transform.rotation = nalgebra_glm::quat_angle_axis(physics.rotation_angle, &Vec3::z());

            if transform.translation.x + BUNNY_SIZE > max_x {
                transform.translation.x = max_x - BUNNY_SIZE;
                physics.velocity.x *= -0.85;
                if rng.random::<f32>() > 0.5 {
                    physics.velocity.y = rng.random_range(0.0..300.0);
                }
            } else if transform.translation.x < MIN_X {
                transform.translation.x = MIN_X;
                physics.velocity.x *= -0.85;
                if rng.random::<f32>() > 0.5 {
                    physics.velocity.y = rng.random_range(0.0..300.0);
                }
            }

            if transform.translation.y < MIN_Y {
                transform.translation.y = MIN_Y;
                physics.velocity.y *= -0.85;

                if rng.random::<f32>() > 0.5 {
                    physics.velocity.y = rng.random_range(200.0..500.0);
                }

                if physics.velocity.y.abs() < 100.0 && rng.random::<f32>() > 0.8 {
                    physics.velocity.x = rng.random_range(-100.0..100.0);
                }
            } else if transform.translation.y + BUNNY_SIZE > max_y {
                transform.translation.y = max_y - BUNNY_SIZE;
                physics.velocity.y = 0.0;
            }

            mark_local_transform_dirty(world, handle.0);
        }
    }
}

fn update_sustained_fps_tracking(world: &mut World, bunny_world: &mut BunnyWorld) {
    if bunny_world.resources.frame_times.is_empty() {
        return;
    }

    let fps = world.resources.window.timing.frames_per_second;
    let frame_time = world.resources.window.timing.raw_delta_time * 1000.0;
    let bunny_count: Vec<_> = bunny_world.query_entities(BUNNY_PHYSICS).collect();
    let bunny_count = bunny_count.len();

    if bunny_count > 1000 {
        if fps < bunny_world.resources.sustained_low_fps {
            bunny_world.resources.sustained_low_count += 1;
            if bunny_world.resources.sustained_low_count >= 20 {
                bunny_world.resources.lowest_fps = fps.min(bunny_world.resources.lowest_fps);
                bunny_world.resources.sustained_low_fps = fps;
            }
        } else {
            bunny_world.resources.sustained_low_count = 0;
            bunny_world.resources.sustained_low_fps = fps;
        }

        if fps > bunny_world.resources.sustained_high_fps {
            bunny_world.resources.sustained_high_count += 1;
            if bunny_world.resources.sustained_high_count >= 20 {
                bunny_world.resources.highest_fps = fps.max(bunny_world.resources.highest_fps);
                bunny_world.resources.sustained_high_fps = fps;
            }
        } else {
            bunny_world.resources.sustained_high_count = 0;
            bunny_world.resources.sustained_high_fps = fps;
        }
    }

    bunny_world.resources.frame_times[bunny_world.resources.frame_time_index] = frame_time;
    bunny_world.resources.frame_time_index =
        (bunny_world.resources.frame_time_index + 1) % bunny_world.resources.frame_times.len();
}

fn auto_spawn_system(world: &mut World, bunny_world: &mut BunnyWorld) {
    if bunny_world.resources.auto_spawn_stopped {
        return;
    }

    let current_entities: Vec<_> = bunny_world
        .query_entities(BUNNY_PHYSICS | ENTITY_HANDLE)
        .collect();
    let current_count = current_entities.len();

    if current_count < 1000 {
        spawn_bunnies(world, bunny_world, 1000);
        return;
    }

    if bunny_world.resources.frame_times.is_empty() {
        return;
    }

    let avg_frame_time: f32 = bunny_world.resources.frame_times.iter().sum::<f32>()
        / bunny_world.resources.frame_times.len() as f32;

    if avg_frame_time < 0.001 {
        return;
    }

    let avg_fps = 1000.0 / avg_frame_time;
    let target_fps = bunny_world.resources.target_fps;
    let lower_threshold = target_fps - 4.0;
    let upper_threshold = target_fps + 4.0;

    if avg_fps < lower_threshold {
        let fps_deficit = lower_threshold - avg_fps;

        let despawn_percentage = if fps_deficit > 15.0 {
            0.10
        } else if fps_deficit > 10.0 {
            0.05
        } else if fps_deficit > 5.0 {
            0.02
        } else if fps_deficit > 2.0 {
            0.01
        } else {
            0.005
        };

        let min_despawn = if fps_deficit > 10.0 {
            1000
        } else if fps_deficit > 5.0 {
            200
        } else if fps_deficit > 2.0 {
            50
        } else if fps_deficit > 1.0 {
            20
        } else {
            5
        };

        let despawn_count =
            ((current_count as f32 * despawn_percentage).max(min_despawn as f32)) as usize;
        let despawn_count = despawn_count.min(current_count);
        despawn_bunnies(world, bunny_world, &current_entities, despawn_count);
    } else if avg_fps > upper_threshold {
        let fps_surplus = avg_fps - upper_threshold;

        let spawn_count = if fps_surplus > 30.0 {
            10000
        } else if fps_surplus > 20.0 {
            5000
        } else if fps_surplus > 10.0 {
            2000
        } else if fps_surplus > 5.0 {
            500
        } else if fps_surplus > 2.0 {
            100
        } else if fps_surplus > 1.0 {
            25
        } else {
            5
        };

        spawn_bunnies(world, bunny_world, spawn_count);
    }
}

fn despawn_bunnies(
    world: &mut World,
    bunny_world: &mut BunnyWorld,
    entities: &[freecs::Entity],
    count: usize,
) {
    let to_despawn = entities.iter().take(count).copied().collect::<Vec<_>>();

    for entity in &to_despawn {
        if let Some(handle) = bunny_world.get_entity_handle(*entity).copied() {
            world.despawn_entities(&[handle.0]);
        }
    }

    bunny_world.despawn_entities(&to_despawn);
}

fn generate_gradient_textures(world: &mut World) {
    let size = 64;

    for slot in 0..128 {
        let mut pixels = vec![0u8; size * size * 4];

        let gradient_type = slot % 20;
        let hue_offset = (slot as f32 / 127.0) * 360.0;
        let saturation_var = 0.4 + (slot as f32 / 127.0) * 0.6;
        let frequency_var = 1.0 + (slot as f32 / 64.0) * 3.0;

        for y in 0..size {
            for x in 0..size {
                let index = (y * size + x) * 4;
                let fx = x as f32 / (size - 1) as f32;
                let fy = y as f32 / (size - 1) as f32;

                let (r, g, b) = match gradient_type {
                    0 => {
                        let value = fx;
                        hsv_to_rgb(hue_offset + value * 60.0, saturation_var, 0.9)
                    }
                    1 => {
                        let value = fy;
                        hsv_to_rgb(hue_offset + value * 60.0, saturation_var, 0.9)
                    }
                    2 => {
                        let value = (fx + fy) * 0.5;
                        hsv_to_rgb(hue_offset + value * 90.0, saturation_var * 0.9, 0.85)
                    }
                    3 => {
                        let dx = fx - 0.5;
                        let dy = fy - 0.5;
                        let value = 1.0 - (dx * dx + dy * dy).sqrt() * 1.414;
                        hsv_to_rgb(hue_offset + value * 120.0, saturation_var, value.max(0.3))
                    }
                    4 => {
                        let value = ((fx * std::f32::consts::PI * frequency_var).sin() + 1.0) * 0.5;
                        hsv_to_rgb(hue_offset + value * 80.0, saturation_var, 0.8 + value * 0.2)
                    }
                    5 => {
                        let value = ((fy * std::f32::consts::PI * frequency_var).cos() + 1.0) * 0.5;
                        hsv_to_rgb(hue_offset + value * 80.0, saturation_var, 0.8 + value * 0.2)
                    }
                    6 => {
                        let value = ((fx * fy * std::f32::consts::PI * frequency_var * 2.0).sin()
                            + 1.0)
                            * 0.5;
                        hsv_to_rgb(
                            hue_offset + value * 100.0,
                            saturation_var * 0.8,
                            0.7 + value * 0.3,
                        )
                    }
                    7 => {
                        let dx = (fx - 0.5).abs() * 2.0;
                        let dy = (fy - 0.5).abs() * 2.0;
                        let value = 1.0 - dx.max(dy);
                        hsv_to_rgb(hue_offset + value * 70.0, saturation_var, 0.6 + value * 0.4)
                    }
                    8 => {
                        let angle = fy.atan2(fx);
                        let value = (angle / std::f32::consts::TAU + 0.5).fract();
                        hsv_to_rgb(hue_offset + value * 360.0, saturation_var, 0.85)
                    }
                    9 => {
                        let dist = ((fx - 0.5).powi(2) + (fy - 0.5).powi(2)).sqrt();
                        let value = (dist * frequency_var * 4.0).sin() * 0.5 + 0.5;
                        hsv_to_rgb(
                            hue_offset + value * 120.0,
                            saturation_var,
                            0.7 + value * 0.3,
                        )
                    }
                    10 => {
                        let value = ((fx * frequency_var * 2.0).sin()
                            * (fy * frequency_var * 2.0).cos()
                            + 1.0)
                            * 0.5;
                        hsv_to_rgb(hue_offset + value * 150.0, saturation_var * 0.9, 0.75)
                    }
                    11 => {
                        let checkerboard = ((fx * 8.0) as i32 ^ (fy * 8.0) as i32) & 1;
                        let value = checkerboard as f32;
                        hsv_to_rgb(
                            hue_offset + value * 180.0,
                            saturation_var,
                            0.5 + value * 0.5,
                        )
                    }
                    12 => {
                        let noise = ((fx * 31.0 + fy * 37.0) * frequency_var).sin();
                        let value = (noise + 1.0) * 0.5;
                        hsv_to_rgb(
                            hue_offset + value * 90.0,
                            saturation_var * 0.7,
                            0.6 + value * 0.4,
                        )
                    }
                    13 => {
                        let dist = ((fx - 0.5).powi(2) + (fy - 0.5).powi(2)).sqrt();
                        let spiral =
                            ((fx - 0.5).atan2(fy - 0.5) + dist * frequency_var * 10.0).sin() * 0.5
                                + 0.5;
                        hsv_to_rgb(
                            hue_offset + spiral * 180.0,
                            saturation_var,
                            0.6 + spiral * 0.4,
                        )
                    }
                    14 => {
                        let wave1 = (fx * frequency_var * std::f32::consts::PI * 3.0).sin();
                        let wave2 = (fy * frequency_var * std::f32::consts::PI * 3.0).cos();
                        let value = (wave1 * wave2 + 1.0) * 0.5;
                        hsv_to_rgb(
                            hue_offset + value * 120.0,
                            saturation_var,
                            0.65 + value * 0.35,
                        )
                    }
                    15 => {
                        let dist_x = (fx - 0.5).abs();
                        let dist_y = (fy - 0.5).abs();
                        let value = 1.0 - (dist_x + dist_y);
                        hsv_to_rgb(
                            hue_offset + value * 100.0,
                            saturation_var * 0.85,
                            0.5 + value * 0.5,
                        )
                    }
                    16 => {
                        let dist = ((fx - 0.5).powi(2) + (fy - 0.5).powi(2)).sqrt();
                        let ripple =
                            ((dist * frequency_var * 8.0 - slot as f32 * 0.1).sin() + 1.0) * 0.5;
                        hsv_to_rgb(
                            hue_offset + ripple * 150.0,
                            saturation_var,
                            0.6 + ripple * 0.4,
                        )
                    }
                    17 => {
                        let grid = ((fx * frequency_var * 5.0).sin().abs()
                            + (fy * frequency_var * 5.0).sin().abs())
                            * 0.5;
                        hsv_to_rgb(
                            hue_offset + grid * 90.0,
                            saturation_var * 0.9,
                            0.7 + grid * 0.3,
                        )
                    }
                    18 => {
                        let plasma = (fx * frequency_var).sin()
                            + (fy * frequency_var).sin()
                            + ((fx + fy) * frequency_var * 0.5).sin();
                        let value = (plasma / 3.0 + 0.5).clamp(0.0, 1.0);
                        hsv_to_rgb(
                            hue_offset + value * 270.0,
                            saturation_var,
                            0.6 + value * 0.4,
                        )
                    }
                    _ => {
                        let perlin = ((fx * frequency_var * 7.0).sin()
                            * (fy * frequency_var * 7.0).cos()
                            + (fx * frequency_var * 13.0).cos()
                                * (fy * frequency_var * 13.0).sin())
                            * 0.25
                            + 0.5;
                        hsv_to_rgb(
                            hue_offset + perlin * 200.0,
                            saturation_var,
                            0.5 + perlin * 0.5,
                        )
                    }
                };

                pixels[index] = (r * 255.0) as u8;
                pixels[index + 1] = (g * 255.0) as u8;
                pixels[index + 2] = (b * 255.0) as u8;
                pixels[index + 3] = 255;
            }
        }

        world
            .resources
            .command_queue
            .push(WorldCommand::UploadSpriteTexture {
                slot,
                rgba_data: pixels,
                width: size as u32,
                height: size as u32,
            });
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
}

fn format_number_with_commas(number: usize) -> String {
    let number_str = number.to_string();
    let mut result = String::new();
    let chars: Vec<char> = number_str.chars().collect();

    for (index, character) in chars.iter().enumerate() {
        if index > 0 && (chars.len() - index).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*character);
    }

    result
}
