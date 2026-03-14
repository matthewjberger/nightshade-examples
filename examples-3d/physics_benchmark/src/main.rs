use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::physics::*;
use nightshade::ecs::picking::{PickingOptions, PickingRay, pick_entities};
use nightshade::prelude::*;
use rand::Rng;

const BALL_RADIUS: f32 = 0.15;
const BOX_SIZE: f32 = 6.0;
const BOX_WALL_THICKNESS: f32 = 0.2;
const BOX_HEIGHT: f32 = 4.0;
const SPAWN_INTERVAL_FRAMES: usize = 2;
const BALLS_PER_SPAWN: usize = 5;
const INITIAL_BALLS: usize = 0;
const CONTAINER_ALPHA: f32 = 0.6;

const GRAB_RANGE: f32 = 50.0;
const MIN_GRAB_DISTANCE: f32 = 1.0;
const MAX_GRAB_DISTANCE: f32 = 50.0;
const GRAB_STIFFNESS: f32 = 150.0;
const GRAB_DAMPING_RATIO: f32 = 1.0;
const MAX_GRAB_FORCE: f32 = 80.0;
const THROW_STRENGTH: f32 = 15.0;
const ANGULAR_DAMPING: f32 = 5.0;
const SCROLL_DISTANCE_SPEED: f32 = 0.5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(PhysicsBenchmark::default())
}

#[derive(Default)]
struct PhysicsBenchmark {
    ball_entities: Vec<Entity>,
    camera_entity: Option<Entity>,
    home_focus: Vec3,
    home_radius: f32,
    home_yaw: f32,
    home_pitch: f32,
    fps_hud_text: Option<Entity>,
    ball_count_hud_text: Option<Entity>,
    target_fps_hud_text: Option<Entity>,
    lowest_fps: f32,
    highest_fps: f32,
    frame_times: Vec<f32>,
    frame_time_index: usize,
    auto_spawn_stopped: bool,
    sustained_low_fps: f32,
    sustained_high_fps: f32,
    sustained_low_count: usize,
    sustained_high_count: usize,
    target_fps: f32,
    pending_target_fps: f32,
    frames_since_spawn: usize,
    frames_below_threshold: usize,
    frames_above_threshold: usize,
    color_index: usize,
    grabbed_entity: Option<Entity>,
    grab_distance: f32,
    continuous_spawn: bool,
}

impl State for PhysicsBenchmark {
    fn title(&self) -> &str {
        "Physics Benchmark"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.use_fullscreen = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;

        self.lowest_fps = 60.0;
        self.highest_fps = 60.0;
        self.frame_times = vec![0.0; 60];
        self.frame_time_index = 0;
        self.auto_spawn_stopped = false;
        self.sustained_low_fps = 60.0;
        self.sustained_high_fps = 60.0;
        self.sustained_low_count = 0;
        self.sustained_high_count = 0;
        self.target_fps = 60.0;
        self.pending_target_fps = 60.0;
        self.frames_since_spawn = 0;
        self.color_index = 0;
        self.continuous_spawn = true;
        self.grab_distance = 10.0;

        let sun = spawn_sun(world);
        if let Some(light) = world.core.get_light_mut(sun) {
            light.cast_shadows = false;
        }

        self.spawn_box_container(world);

        self.home_focus = Vec3::new(0.0, BOX_HEIGHT / 2.0, 0.0);
        self.home_radius = 15.0;
        self.home_yaw = 0.5;
        self.home_pitch = 0.4;

        let camera_entity = spawn_pan_orbit_camera(
            world,
            self.home_focus,
            self.home_radius,
            self.home_yaw,
            self.home_pitch,
            "Benchmark Camera".to_string(),
        );
        world.resources.active_camera = Some(camera_entity);
        self.camera_entity = Some(camera_entity);

        let fps_text = spawn_ui_text_with_properties(
            world,
            "FPS: 0",
            Vec2::zeros(),
            TextProperties {
                font_size: 48.0,
                color: Vec4::new(0.0, 1.0, 0.0, 1.0),
                ..Default::default()
            },
        );
        self.fps_hud_text = Some(fps_text);

        let ball_count_text = spawn_ui_text_with_properties(
            world,
            "Balls: 0",
            Vec2::zeros(),
            TextProperties {
                font_size: 32.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                ..Default::default()
            },
        );
        self.ball_count_hud_text = Some(ball_count_text);

        let target_fps_text = spawn_ui_text_with_properties(
            world,
            "Target FPS: 60",
            Vec2::zeros(),
            TextProperties {
                font_size: 28.0,
                color: Vec4::new(0.8, 0.8, 0.8, 1.0),
                ..Default::default()
            },
        );
        self.target_fps_hud_text = Some(target_fps_text);

        (0..INITIAL_BALLS).for_each(|_| self.spawn_ball(world));
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        if self.grabbed_entity.is_none() {
            pan_orbit_camera_system(world);
        }

        if world.resources.input.keyboard.is_key_pressed(KeyCode::KeyC)
            || world.resources.input.keyboard.is_key_pressed(KeyCode::Home)
        {
            self.reset_camera_to_home(world);
        }

        self.grab_interaction_system(world);
        self.continuous_spawn_system(world);

        let fps = world.resources.window.timing.frames_per_second;
        let target_fps = self.target_fps;
        let lower_threshold = target_fps - 4.0;
        let upper_threshold = target_fps + 4.0;

        if let Some(fps_text_entity) = self.fps_hud_text {
            let fps_color = if fps >= lower_threshold && fps <= upper_threshold {
                Vec4::new(0.0, 1.0, 0.0, 1.0)
            } else if fps > upper_threshold {
                Vec4::new(1.0, 1.0, 1.0, 1.0)
            } else {
                Vec4::new(1.0, 0.65, 0.0, 1.0)
            };

            let text_index = world.core.get_text(fps_text_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("FPS: {:.0}", fps));
                if let Some(hud_text) = world.core.get_text_mut(fps_text_entity) {
                    hud_text.properties.color = fps_color;
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(ball_count_entity) = self.ball_count_hud_text {
            let text_index = world.core.get_text(ball_count_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world.resources.text_cache.set_text(
                    text_index,
                    format!(
                        "Balls: {}",
                        format_number_with_commas(self.ball_entities.len())
                    ),
                );
                if let Some(hud_text) = world.core.get_text_mut(ball_count_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(target_fps_entity) = self.target_fps_hud_text {
            let text_index = world.core.get_text(target_fps_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("Target FPS: {:.0}", self.target_fps));
                if let Some(hud_text) = world.core.get_text_mut(target_fps_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        self.update_sustained_fps_tracking(world);
        self.auto_spawn_system(world);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        let ball_count = self.ball_entities.len();
        let fps = world.resources.window.timing.frames_per_second;

        let avg_frame_time = if !self.frame_times.is_empty() {
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
        } else {
            0.0
        };

        let resolution = if let Some(window_handle) = &world.resources.window.handle {
            let size = window_handle.inner_size();
            format!("{}x{}", size.width, size.height)
        } else {
            "Unknown".to_string()
        };

        egui::Window::new("Physics Benchmark")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.heading("Physics Ball Benchmark");
                ui.separator();

                ui.label(format!("Resolution: {}", resolution));
                ui.label(format!("Balls: {}", format_number_with_commas(ball_count)));
                ui.label(format!(
                    "FPS: {:.0} (Low: {:.0} High: {:.0})",
                    fps, self.lowest_fps, self.highest_fps
                ));
                ui.label(format!("Frame Time: {:.1}ms", avg_frame_time));

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Target FPS:");
                    ui.add(
                        egui::Slider::new(&mut self.pending_target_fps, 30.0..=144.0).step_by(1.0),
                    );

                    let apply_enabled = (self.pending_target_fps - self.target_fps).abs() > 0.1;
                    if ui
                        .add_enabled(apply_enabled, egui::Button::new("Apply"))
                        .clicked()
                    {
                        self.target_fps = self.pending_target_fps;
                        self.auto_spawn_stopped = false;
                        self.frames_below_threshold = 0;
                        self.frames_above_threshold = 0;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Set FPS:");
                    if ui.button("30").clicked() {
                        self.target_fps = 30.0;
                        self.pending_target_fps = 30.0;
                        self.auto_spawn_stopped = false;
                    }
                    if ui.button("60").clicked() {
                        self.target_fps = 60.0;
                        self.pending_target_fps = 60.0;
                        self.auto_spawn_stopped = false;
                    }
                    if ui.button("75").clicked() {
                        self.target_fps = 75.0;
                        self.pending_target_fps = 75.0;
                        self.auto_spawn_stopped = false;
                    }
                    if ui.button("90").clicked() {
                        self.target_fps = 90.0;
                        self.pending_target_fps = 90.0;
                        self.auto_spawn_stopped = false;
                    }
                    if ui.button("120").clicked() {
                        self.target_fps = 120.0;
                        self.pending_target_fps = 120.0;
                        self.auto_spawn_stopped = false;
                    }
                    if ui.button("144").clicked() {
                        self.target_fps = 144.0;
                        self.pending_target_fps = 144.0;
                        self.auto_spawn_stopped = false;
                    }
                });

                ui.separator();

                const FRAMES_REQUIRED_BELOW: usize = 45;
                const FRAMES_REQUIRED_ABOVE: usize = 60;

                if !self.auto_spawn_stopped && ball_count >= 10 {
                    if self.frames_below_threshold > 0 {
                        let progress =
                            self.frames_below_threshold as f32 / FRAMES_REQUIRED_BELOW as f32;
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::YELLOW, "Despawning");
                            ui.add(egui::ProgressBar::new(progress.min(1.0)).desired_width(80.0));
                        });
                    } else if self.frames_above_threshold > 0 {
                        let progress =
                            self.frames_above_threshold as f32 / FRAMES_REQUIRED_ABOVE as f32;
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "Spawning");
                            ui.add(egui::ProgressBar::new(progress.min(1.0)).desired_width(80.0));
                        });
                    } else {
                        ui.colored_label(egui::Color32::GREEN, "Stable");
                    }
                } else if ball_count < 10 {
                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "Spawning balls");
                }

                ui.separator();
                ui.checkbox(&mut self.continuous_spawn, "Continuous Spawning");

                ui.separator();
                ui.label("Controls:");
                ui.label("  Left click - Grab ball");
                ui.label("  Right drag - Orbit camera");
                ui.label("  Scroll - Zoom / Grab distance");
                ui.label("  C / Home - Reset camera");
                ui.label("  Escape - Exit");
            });
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        if matches!(key, KeyCode::KeyC | KeyCode::Home) {
            self.reset_camera_to_home(world);
        }
    }
}

impl PhysicsBenchmark {
    fn spawn_box_container(&self, world: &mut World) {
        self.spawn_transparent_physics_wall(
            world,
            Vec3::new(0.0, -BOX_WALL_THICKNESS / 2.0, 0.0),
            Vec3::new(BOX_SIZE, BOX_WALL_THICKNESS, BOX_SIZE),
            Vec3::new(0.3, 0.3, 0.35),
            "Floor",
        );

        self.spawn_transparent_physics_wall(
            world,
            Vec3::new(
                0.0,
                BOX_HEIGHT / 2.0,
                -BOX_SIZE / 2.0 - BOX_WALL_THICKNESS / 2.0,
            ),
            Vec3::new(
                BOX_SIZE + BOX_WALL_THICKNESS * 2.0,
                BOX_HEIGHT,
                BOX_WALL_THICKNESS,
            ),
            Vec3::new(0.25, 0.25, 0.28),
            "WallBack",
        );

        self.spawn_transparent_physics_wall(
            world,
            Vec3::new(
                0.0,
                BOX_HEIGHT / 2.0,
                BOX_SIZE / 2.0 + BOX_WALL_THICKNESS / 2.0,
            ),
            Vec3::new(
                BOX_SIZE + BOX_WALL_THICKNESS * 2.0,
                BOX_HEIGHT,
                BOX_WALL_THICKNESS,
            ),
            Vec3::new(0.25, 0.25, 0.28),
            "WallFront",
        );

        self.spawn_transparent_physics_wall(
            world,
            Vec3::new(
                -BOX_SIZE / 2.0 - BOX_WALL_THICKNESS / 2.0,
                BOX_HEIGHT / 2.0,
                0.0,
            ),
            Vec3::new(BOX_WALL_THICKNESS, BOX_HEIGHT, BOX_SIZE),
            Vec3::new(0.25, 0.25, 0.28),
            "WallLeft",
        );

        self.spawn_transparent_physics_wall(
            world,
            Vec3::new(
                BOX_SIZE / 2.0 + BOX_WALL_THICKNESS / 2.0,
                BOX_HEIGHT / 2.0,
                0.0,
            ),
            Vec3::new(BOX_WALL_THICKNESS, BOX_HEIGHT, BOX_SIZE),
            Vec3::new(0.25, 0.25, 0.28),
            "WallRight",
        );
    }

    fn spawn_transparent_physics_wall(
        &self,
        world: &mut World,
        position: Vec3,
        scale: Vec3,
        color: Vec3,
        name: &str,
    ) {
        let entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | VISIBILITY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER,
            1,
        )[0];

        if let Some(n) = world.core.get_name_mut(entity) {
            n.0 = name.to_string();
        }

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
            transform.scale = scale;
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
            mesh.name = "Cube".to_string();
        }

        let material_name = format!("{}_{}", name, entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color: [color.x, color.y, color.z, CONTAINER_ALPHA],
                alpha_mode: AlphaMode::Blend,
                roughness: 0.85,
                metallic: 0.1,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(entity, MaterialRef::new(material_name));

        if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
            *bounding_volume =
                nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(entity) {
            *rigid_body = RigidBodyComponent::new_static()
                .with_translation(position.x, position.y, position.z);
        }

        if let Some(collider) = world.core.get_collider_mut(entity) {
            *collider = ColliderComponent::new_cuboid(scale.x / 2.0, scale.y / 2.0, scale.z / 2.0)
                .with_friction(0.5)
                .with_restitution(0.3);
        }

        let rigid_body_comp = world.core.get_rigid_body(entity).cloned().unwrap();
        let collider_comp = world.core.get_collider(entity).cloned();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(collider_comp) = collider_comp {
            let collider = collider_comp.to_rapier_collider();
            world.resources.physics.add_collider(collider, handle);
        }
        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(entity) {
            rigid_body_mut.handle = Some(handle.into());
        }
    }

    fn spawn_ball(&mut self, world: &mut World) {
        let mut rng = rand::rng();

        let inner_margin = BOX_SIZE / 2.0 - BALL_RADIUS * 2.0 - 0.3;
        let spawn_x = rng.random_range(-inner_margin..inner_margin);
        let spawn_z = rng.random_range(-inner_margin..inner_margin);
        let spawn_y = BOX_HEIGHT + 0.5;

        let color = get_pool_color(self.color_index);
        self.color_index += 1;

        let material = Material {
            base_color: [color.x, color.y, color.z, 1.0],
            roughness: 0.3,
            metallic: 0.6,
            ..Default::default()
        };

        let entity = spawn_dynamic_physics_sphere_with_material(
            world,
            Vec3::new(spawn_x, spawn_y, spawn_z),
            BALL_RADIUS,
            1.0,
            material,
        );

        world.resources.mesh_render_state.mark_entity_added(entity);

        if let Some(collider) = world.core.get_collider_mut(entity) {
            collider.friction = 0.5;
            collider.restitution = 0.7;
        }

        if let Some(rigid_body) = world.core.get_rigid_body(entity)
            && let Some(handle) = rigid_body.handle
        {
            let velocity_x = rng.random_range(-0.3..0.3);
            let velocity_z = rng.random_range(-0.3..0.3);
            let velocity_y = rng.random_range(2.0..4.0);
            if let Some(rb) = world
                .resources
                .physics
                .rigid_body_set
                .get_mut(handle.into())
            {
                rb.set_linvel(
                    rapier3d::prelude::Vector::new(velocity_x, velocity_y, velocity_z),
                    true,
                );
            }
        }

        self.ball_entities.push(entity);
    }

    fn despawn_ball(&mut self, world: &mut World) {
        if let Some(entity) = self.ball_entities.pop() {
            if let Some(rigid_body) = world.core.get_rigid_body(entity)
                && let Some(handle) = rigid_body.handle
            {
                world.resources.physics.remove_rigid_body(handle.into());
            }
            world.despawn_entities(&[entity]);
        }
    }

    fn reset_camera_to_home(&self, world: &mut World) {
        let Some(camera_entity) = self.camera_entity else {
            return;
        };

        let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(camera_entity) else {
            return;
        };

        pan_orbit.target_focus = self.home_focus;
        pan_orbit.target_radius = self.home_radius;
        pan_orbit.target_yaw = self.home_yaw;
        pan_orbit.target_pitch = self.home_pitch;
    }

    fn update_sustained_fps_tracking(&mut self, world: &World) {
        if self.frame_times.is_empty() {
            return;
        }

        let fps = world.resources.window.timing.frames_per_second;
        let frame_time = world.resources.window.timing.raw_delta_time * 1000.0;
        let ball_count = self.ball_entities.len();

        if ball_count > 10 {
            if fps < self.sustained_low_fps {
                self.sustained_low_count += 1;
                if self.sustained_low_count >= 20 {
                    self.lowest_fps = fps.min(self.lowest_fps);
                    self.sustained_low_fps = fps;
                }
            } else {
                self.sustained_low_count = 0;
                self.sustained_low_fps = fps;
            }

            if fps > self.sustained_high_fps {
                self.sustained_high_count += 1;
                if self.sustained_high_count >= 20 {
                    self.highest_fps = fps.max(self.highest_fps);
                    self.sustained_high_fps = fps;
                }
            } else {
                self.sustained_high_count = 0;
                self.sustained_high_fps = fps;
            }
        }

        self.frame_times[self.frame_time_index] = frame_time;
        self.frame_time_index = (self.frame_time_index + 1) % self.frame_times.len();
    }

    fn continuous_spawn_system(&mut self, world: &mut World) {
        if !self.continuous_spawn {
            return;
        }

        for _ in 0..BALLS_PER_SPAWN {
            self.spawn_ball(world);
        }
    }

    fn grab_interaction_system(&mut self, world: &mut World) {
        let left_just_pressed = world
            .resources
            .input
            .mouse
            .state
            .contains(nightshade::ecs::input::resources::MouseState::LEFT_JUST_PRESSED);
        let left_clicked = world
            .resources
            .input
            .mouse
            .state
            .contains(nightshade::ecs::input::resources::MouseState::LEFT_CLICKED);
        let scroll_delta = world.resources.input.mouse.wheel_delta.y;

        let mouse_pos = world.resources.input.mouse.position;
        let screen_pos = Vec2::new(mouse_pos.x, mouse_pos.y);

        let Some(ray) = PickingRay::from_screen_position(world, screen_pos) else {
            return;
        };

        if self.grabbed_entity.is_some() {
            if left_clicked {
                self.update_grabbed_object(world, ray.origin, ray.direction, scroll_delta);
            } else {
                self.throw_grabbed_object(world, ray.direction);
            }
        } else if left_just_pressed {
            self.try_grab(world);
        }
    }

    fn try_grab(&mut self, world: &mut World) {
        let mouse_pos = world.resources.input.mouse.position;
        let screen_pos = Vec2::new(mouse_pos.x, mouse_pos.y);

        let options = PickingOptions {
            max_distance: GRAB_RANGE,
            ..Default::default()
        };

        let pick_results = pick_entities(world, screen_pos, options);

        for result in &pick_results {
            if let Some(rigid_body) = world.core.get_rigid_body(result.entity)
                && rigid_body.body_type == RigidBodyType::Dynamic
            {
                self.grabbed_entity = Some(result.entity);
                self.grab_distance = result.distance.clamp(MIN_GRAB_DISTANCE, MAX_GRAB_DISTANCE);
                return;
            }
        }
    }

    fn update_grabbed_object(
        &mut self,
        world: &mut World,
        camera_position: Vec3,
        camera_forward: Vec3,
        scroll_delta: f32,
    ) {
        self.grab_distance = (self.grab_distance + scroll_delta * SCROLL_DISTANCE_SPEED)
            .clamp(MIN_GRAB_DISTANCE, MAX_GRAB_DISTANCE);

        let target_position = camera_position + camera_forward * self.grab_distance;

        let Some(grabbed_entity) = self.grabbed_entity else {
            return;
        };

        let Some(rigid_body_component) = world.core.get_rigid_body(grabbed_entity) else {
            self.grabbed_entity = None;
            return;
        };
        let Some(handle) = rigid_body_component.handle else {
            self.grabbed_entity = None;
            return;
        };
        let Some(rigid_body) = world
            .resources
            .physics
            .rigid_body_set
            .get_mut(handle.into())
        else {
            self.grabbed_entity = None;
            return;
        };

        let current_pos = rigid_body.translation();
        let current_position = nalgebra_glm::vec3(current_pos.x, current_pos.y, current_pos.z);

        let displacement = target_position - current_position;

        let current_vel = rigid_body.linvel();
        let current_velocity = nalgebra_glm::vec3(current_vel.x, current_vel.y, current_vel.z);

        let mass = rigid_body.mass();
        let critical_damping = 2.0 * (GRAB_STIFFNESS * mass).sqrt();
        let damping = critical_damping * GRAB_DAMPING_RATIO;

        let spring_force = displacement * GRAB_STIFFNESS;
        let damping_force = -current_velocity * damping;
        let mut total_force = spring_force + damping_force;

        let force_magnitude = nalgebra_glm::length(&total_force);
        let max_force_for_mass = MAX_GRAB_FORCE * mass.max(0.5);
        if force_magnitude > max_force_for_mass {
            total_force *= max_force_for_mass / force_magnitude;
        }

        let acceleration = total_force / mass;
        let dt = world.resources.physics.fixed_timestep;
        let new_velocity = current_velocity + acceleration * dt;

        rigid_body.set_linvel(
            rapier3d::prelude::Vector::new(new_velocity.x, new_velocity.y, new_velocity.z),
            true,
        );

        let current_angvel = rigid_body.angvel();
        let angular_decay = (-ANGULAR_DAMPING * dt * 60.0).exp();
        rigid_body.set_angvel(current_angvel * angular_decay, true);
    }

    fn throw_grabbed_object(&mut self, world: &mut World, camera_forward: Vec3) {
        let Some(grabbed_entity) = self.grabbed_entity else {
            return;
        };

        let Some(rigid_body_component) = world.core.get_rigid_body(grabbed_entity) else {
            self.grabbed_entity = None;
            return;
        };
        let Some(handle) = rigid_body_component.handle else {
            self.grabbed_entity = None;
            return;
        };
        let Some(rigid_body) = world
            .resources
            .physics
            .rigid_body_set
            .get_mut(handle.into())
        else {
            self.grabbed_entity = None;
            return;
        };

        let throw_velocity = camera_forward * THROW_STRENGTH;
        rigid_body.set_linvel(
            rapier3d::prelude::Vector::new(throw_velocity.x, throw_velocity.y, throw_velocity.z),
            true,
        );

        self.grabbed_entity = None;
    }

    fn auto_spawn_system(&mut self, world: &mut World) {
        if self.auto_spawn_stopped {
            return;
        }

        self.frames_since_spawn += 1;

        let current_count = self.ball_entities.len();

        if current_count < 10 {
            for _ in 0..10 {
                self.spawn_ball(world);
            }
            self.frames_since_spawn = 0;
            self.frames_below_threshold = 0;
            self.frames_above_threshold = 0;
            return;
        }

        if self.frame_times.is_empty() {
            return;
        }

        let avg_frame_time: f32 =
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;

        if avg_frame_time < 0.001 {
            return;
        }

        let avg_fps = 1000.0 / avg_frame_time;
        let target_fps = self.target_fps;
        let lower_threshold = target_fps - 4.0;
        let upper_threshold = target_fps + 4.0;

        const MIN_FRAMES_BETWEEN_CHANGES: usize = 30;
        const FRAMES_REQUIRED_BELOW: usize = 45;
        const FRAMES_REQUIRED_ABOVE: usize = 60;

        if avg_fps < lower_threshold {
            self.frames_below_threshold += 1;
            self.frames_above_threshold = 0;
        } else if avg_fps > upper_threshold {
            self.frames_above_threshold += 1;
            self.frames_below_threshold = 0;
        } else {
            self.frames_below_threshold = 0;
            self.frames_above_threshold = 0;
        }

        if self.frames_since_spawn < MIN_FRAMES_BETWEEN_CHANGES {
            if self.frames_since_spawn >= SPAWN_INTERVAL_FRAMES && avg_fps > upper_threshold {
                for _ in 0..BALLS_PER_SPAWN {
                    self.spawn_ball(world);
                }
                self.frames_since_spawn = 0;
            }
            return;
        }

        if self.frames_below_threshold >= FRAMES_REQUIRED_BELOW {
            let fps_deficit = lower_threshold - avg_fps;

            let despawn_percentage = if fps_deficit > 15.0 {
                0.15
            } else if fps_deficit > 10.0 {
                0.10
            } else if fps_deficit > 5.0 {
                0.05
            } else if fps_deficit > 2.0 {
                0.02
            } else {
                0.01
            };

            let min_despawn = if fps_deficit > 10.0 {
                20
            } else if fps_deficit > 5.0 {
                10
            } else if fps_deficit > 2.0 {
                5
            } else {
                2
            };

            let despawn_count =
                ((current_count as f32 * despawn_percentage).max(min_despawn as f32)) as usize;
            let despawn_count = despawn_count.min(current_count.saturating_sub(1));

            for _ in 0..despawn_count {
                self.despawn_ball(world);
            }

            self.frames_since_spawn = 0;
            self.frames_below_threshold = 0;
        } else if self.frames_above_threshold >= FRAMES_REQUIRED_ABOVE {
            let fps_surplus = avg_fps - upper_threshold;

            let spawn_count = if fps_surplus > 30.0 {
                50
            } else if fps_surplus > 20.0 {
                30
            } else if fps_surplus > 10.0 {
                15
            } else if fps_surplus > 5.0 {
                8
            } else if fps_surplus > 2.0 {
                4
            } else {
                2
            };

            for _ in 0..spawn_count {
                self.spawn_ball(world);
            }

            self.frames_since_spawn = 0;
            self.frames_above_threshold = 0;
        }
    }
}

const COLOR_PALETTE: &[[f32; 3]] = &[
    [1.0, 0.2, 0.2],
    [0.2, 1.0, 0.2],
    [0.2, 0.4, 1.0],
    [1.0, 1.0, 0.2],
    [1.0, 0.2, 1.0],
    [0.2, 1.0, 1.0],
    [1.0, 0.6, 0.2],
    [0.6, 0.2, 1.0],
    [0.2, 1.0, 0.6],
    [1.0, 0.4, 0.6],
    [0.4, 0.6, 1.0],
    [0.8, 0.8, 0.8],
];

fn get_pool_color(index: usize) -> Vec3 {
    let color = COLOR_PALETTE[index % COLOR_PALETTE.len()];
    Vec3::new(color[0], color[1], color[2])
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
