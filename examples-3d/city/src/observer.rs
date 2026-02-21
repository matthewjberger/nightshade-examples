use std::sync::atomic::{AtomicBool, Ordering};

use nightshade::ecs::camera::queries::query_camera_frustum;
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::world::WorldCommand;
use nightshade::ecs::world::resources::MouseState;
use nightshade::prelude::*;

const OBSERVER_WIDTH: u32 = 640;
const OBSERVER_HEIGHT: u32 = 360;
const MOVE_SPEED: f32 = 200.0;
const TURN_SPEED: f32 = 2.0;
const ALTITUDE_SPEED: f32 = 100.0;
const DEADZONE: f32 = 0.15;

const FRUSTUM_COLOR: Vec4 = Vec4::new(1.0, 1.0, 0.0, 1.0);
const FRUSTUM_NEAR_COLOR: Vec4 = Vec4::new(0.0, 1.0, 0.0, 1.0);
const FRUSTUM_FAR_COLOR: Vec4 = Vec4::new(1.0, 0.3, 0.3, 1.0);

pub struct ObserverCamera {
    pub enabled: bool,
    camera_entity: Entity,
    position: Vec3,
    yaw: f32,
    pitch: f32,
    _texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    egui_texture_id: Option<egui::TextureId>,
    frustum_lines_entity: Entity,
}

impl ObserverCamera {
    pub fn new(renderer: &dyn Render, world: &mut World) -> Self {
        let position = Vec3::new(0.0, 300.0, 0.0);
        let yaw = 0.0;
        let pitch: f32 = -70.0_f32.to_radians();

        let camera_entity = spawn_camera(world, position, "Observer Camera".to_string());
        if let Some(camera) = world.get_camera_mut(camera_entity) {
            camera.projection = Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 90.0_f32.to_radians(),
                z_far: Some(3000.0),
                z_near: 1.0,
            });
        }

        let rotation = Self::compute_rotation(yaw, pitch);
        if let Some(transform) = world.get_local_transform_mut(camera_entity) {
            transform.rotation = rotation;
        }
        mark_local_transform_dirty(world, camera_entity);

        let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Observer Camera Texture"),
            size: wgpu::Extent3d {
                width: OBSERVER_WIDTH,
                height: OBSERVER_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: renderer.surface_format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let frustum_lines_entity = world.spawn_entities(
            LINES | VISIBILITY | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
            1,
        )[0];
        world.set_lines(frustum_lines_entity, Lines::new(Vec::new()));

        Self {
            enabled: false,
            camera_entity,
            position,
            yaw,
            pitch,
            _texture: texture,
            texture_view,
            egui_texture_id: None,
            frustum_lines_entity,
        }
    }

    fn compute_rotation(yaw: f32, pitch: f32) -> Quat {
        let yaw_quat = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
        let pitch_quat = nalgebra_glm::quat_angle_axis(pitch, &Vec3::x());
        yaw_quat * pitch_quat
    }

    fn apply_deadzone(value: f32) -> f32 {
        if value.abs() > DEADZONE {
            value.signum() * (value.abs() - DEADZONE) / (1.0 - DEADZONE)
        } else {
            0.0
        }
    }

    fn update_frustum_lines(&self, world: &mut World, main_camera: Entity) {
        let Some(frustum) = query_camera_frustum(world, main_camera) else {
            return;
        };

        let ntl = frustum.near_top_left;
        let ntr = frustum.near_top_right;
        let nbl = frustum.near_bottom_left;
        let nbr = frustum.near_bottom_right;
        let ftl = frustum.far_top_left;
        let ftr = frustum.far_top_right;
        let fbl = frustum.far_bottom_left;
        let fbr = frustum.far_bottom_right;

        let hatch_steps = 8;
        let mut lines = Vec::with_capacity(12 + 4 + hatch_steps * 4);

        lines.push(Line {
            start: ntl,
            end: ntr,
            color: FRUSTUM_NEAR_COLOR,
        });
        lines.push(Line {
            start: ntr,
            end: nbr,
            color: FRUSTUM_NEAR_COLOR,
        });
        lines.push(Line {
            start: nbr,
            end: nbl,
            color: FRUSTUM_NEAR_COLOR,
        });
        lines.push(Line {
            start: nbl,
            end: ntl,
            color: FRUSTUM_NEAR_COLOR,
        });

        lines.push(Line {
            start: ftl,
            end: ftr,
            color: FRUSTUM_FAR_COLOR,
        });
        lines.push(Line {
            start: ftr,
            end: fbr,
            color: FRUSTUM_FAR_COLOR,
        });
        lines.push(Line {
            start: fbr,
            end: fbl,
            color: FRUSTUM_FAR_COLOR,
        });
        lines.push(Line {
            start: fbl,
            end: ftl,
            color: FRUSTUM_FAR_COLOR,
        });

        lines.push(Line {
            start: ntl,
            end: ftl,
            color: FRUSTUM_COLOR,
        });
        lines.push(Line {
            start: ntr,
            end: ftr,
            color: FRUSTUM_COLOR,
        });
        lines.push(Line {
            start: nbl,
            end: fbl,
            color: FRUSTUM_COLOR,
        });
        lines.push(Line {
            start: nbr,
            end: fbr,
            color: FRUSTUM_COLOR,
        });

        lines.push(Line {
            start: ntl,
            end: nbr,
            color: FRUSTUM_NEAR_COLOR,
        });
        lines.push(Line {
            start: ntr,
            end: nbl,
            color: FRUSTUM_NEAR_COLOR,
        });
        lines.push(Line {
            start: ftl,
            end: fbr,
            color: FRUSTUM_FAR_COLOR,
        });
        lines.push(Line {
            start: ftr,
            end: fbl,
            color: FRUSTUM_FAR_COLOR,
        });

        let hatch_color = Vec4::new(1.0, 1.0, 0.0, 0.4);
        for step in 1..=hatch_steps {
            let t = step as f32 / (hatch_steps + 1) as f32;
            let left_top = nalgebra_glm::lerp(&ntl, &ftl, t);
            let right_top = nalgebra_glm::lerp(&ntr, &ftr, t);
            let left_bottom = nalgebra_glm::lerp(&nbl, &fbl, t);
            let right_bottom = nalgebra_glm::lerp(&nbr, &fbr, t);
            lines.push(Line {
                start: left_top,
                end: right_top,
                color: hatch_color,
            });
            lines.push(Line {
                start: left_bottom,
                end: right_bottom,
                color: hatch_color,
            });
            lines.push(Line {
                start: left_top,
                end: left_bottom,
                color: hatch_color,
            });
            lines.push(Line {
                start: right_top,
                end: right_bottom,
                color: hatch_color,
            });
        }

        if let Some(lines_component) = world.get_lines_mut(self.frustum_lines_entity) {
            lines_component.lines = lines;
            lines_component.mark_dirty();
        }
    }

    pub fn update(&mut self, world: &mut World, delta_time: f32) {
        let (left_x, left_y, right_x, right_y, trigger_up, trigger_down) = {
            let Some(gamepad) = query_active_gamepad(world) else {
                return;
            };
            (
                Self::apply_deadzone(
                    gamepad
                        .axis_data(gilrs::Axis::LeftStickX)
                        .map(|a| a.value())
                        .unwrap_or(0.0),
                ),
                Self::apply_deadzone(
                    gamepad
                        .axis_data(gilrs::Axis::LeftStickY)
                        .map(|a| a.value())
                        .unwrap_or(0.0),
                ),
                Self::apply_deadzone(
                    gamepad
                        .axis_data(gilrs::Axis::RightStickX)
                        .map(|a| a.value())
                        .unwrap_or(0.0),
                ),
                Self::apply_deadzone(
                    gamepad
                        .axis_data(gilrs::Axis::RightStickY)
                        .map(|a| a.value())
                        .unwrap_or(0.0),
                ),
                if gamepad.is_pressed(gilrs::Button::RightTrigger2) {
                    1.0_f32
                } else {
                    0.0
                },
                if gamepad.is_pressed(gilrs::Button::LeftTrigger2) {
                    1.0_f32
                } else {
                    0.0
                },
            )
        };

        self.yaw -= right_x * TURN_SPEED * delta_time;
        self.pitch += right_y * TURN_SPEED * delta_time;
        self.pitch = self
            .pitch
            .clamp(-80.0_f32.to_radians(), -10.0_f32.to_radians());

        let forward = Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos());
        let right_dir = Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin());

        self.position += forward * left_y * MOVE_SPEED * delta_time;
        self.position += right_dir * left_x * MOVE_SPEED * delta_time;
        self.position.y += (trigger_up - trigger_down) * ALTITUDE_SPEED * delta_time;

        let rotation = Self::compute_rotation(self.yaw, self.pitch);
        if let Some(transform) = world.get_local_transform_mut(self.camera_entity) {
            transform.translation = self.position;
            transform.rotation = rotation;
        }
        mark_local_transform_dirty(world, self.camera_entity);
    }

    pub fn render(&mut self, renderer: &mut dyn Render, world: &mut World, main_camera: Entity) {
        if self.egui_texture_id.is_none() {
            self.egui_texture_id = renderer.register_egui_texture(&self.texture_view);
        }

        self.update_frustum_lines(world, main_camera);

        let saved_camera = world.resources.active_camera;
        let saved_fog = world.resources.graphics.fog.take();

        world.resources.active_camera = Some(self.camera_entity);
        world.resources.graphics.culling_camera_override = Some(main_camera);

        let _ = renderer.render_world_to_texture(
            world,
            None,
            &self.texture_view,
            OBSERVER_WIDTH,
            OBSERVER_HEIGHT,
        );

        world.resources.active_camera = saved_camera;
        world.resources.graphics.fog = saved_fog;
        world.resources.graphics.culling_camera_override = None;

        if let Some(lines_component) = world.get_lines_mut(self.frustum_lines_entity) {
            lines_component.lines.clear();
            lines_component.mark_dirty();
        }
    }

    pub fn despawn(self, world: &mut World) {
        world.queue_command(WorldCommand::DespawnRecursive {
            entity: self.camera_entity,
        });
        world.queue_command(WorldCommand::DespawnRecursive {
            entity: self.frustum_lines_entity,
        });
    }

    pub fn draw_ui(&self, ui_context: &egui::Context, minimap_enabled: bool) {
        let Some(texture_id) = self.egui_texture_id else {
            return;
        };

        let margin = 10.0;
        let pip_width = 480.0;
        let pip_height = pip_width * (OBSERVER_HEIGHT as f32 / OBSERVER_WIDTH as f32);

        let minimap_offset = if minimap_enabled { 220.0 + 14.0 } else { 0.0 };

        egui::Area::new(egui::Id::new("observer_pip"))
            .anchor(
                egui::Align2::RIGHT_BOTTOM,
                [-margin, -margin - minimap_offset],
            )
            .interactable(false)
            .order(egui::Order::Foreground)
            .show(ui_context, |ui| {
                egui::Frame::new()
                    .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        ui.image(egui::load::SizedTexture::new(
                            texture_id,
                            [pip_width, pip_height],
                        ));
                    });
            });
    }
}

static FLY_CAM_HAS_CURSOR: AtomicBool = AtomicBool::new(false);

pub fn fly_camera_keyboard_mouse_only(world: &mut World) {
    fly_cam_look(world);
    fly_cam_wasd(world);
}

fn fly_cam_look(world: &mut World) {
    let Some(camera_entity) = world.resources.active_camera else {
        return;
    };

    let delta_time = world.resources.window.timing.delta_time;

    let right_clicked = world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::RIGHT_CLICKED);

    if right_clicked {
        if !FLY_CAM_HAS_CURSOR.load(Ordering::Relaxed) {
            if let Some(window_handle) = &world.resources.window.handle {
                if window_handle
                    .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                    .is_err()
                {
                    let _ = window_handle.set_cursor_grab(winit::window::CursorGrabMode::Confined);
                }
                window_handle.set_cursor_visible(false);
            }
            FLY_CAM_HAS_CURSOR.store(true, Ordering::Relaxed);
        }

        let raw_delta = world.resources.input.mouse.raw_mouse_delta;

        let Some(camera) = world.get_camera_mut(camera_entity) else {
            return;
        };
        let Some(smoothing) = camera.smoothing.as_mut() else {
            return;
        };

        let smoothing_factor = if smoothing.mouse_smoothness > 0.0 {
            1.0 - smoothing.mouse_smoothness.powi(7).powf(delta_time)
        } else {
            1.0
        };
        smoothing.smoothed_mouse_delta = smoothing.smoothed_mouse_delta * (1.0 - smoothing_factor)
            + raw_delta * smoothing_factor;

        let pixels_to_radians = (std::f32::consts::PI / 1000.0) * smoothing.mouse_dpi_scale;
        let mut delta =
            smoothing.smoothed_mouse_delta * smoothing.mouse_sensitivity * pixels_to_radians;
        delta.x *= -1.0;
        delta.y *= -1.0;

        let Some(local_transform) = world.get_local_transform_mut(camera_entity) else {
            return;
        };

        let yaw = nalgebra_glm::quat_angle_axis(delta.x, &Vec3::y());
        local_transform.rotation = yaw * local_transform.rotation;

        let forward = local_transform.forward_vector();
        let current_pitch = forward.y.asin();
        let new_pitch = current_pitch + delta.y;
        if new_pitch.abs() <= 89_f32.to_radians() {
            let pitch = nalgebra_glm::quat_angle_axis(delta.y, &Vec3::x());
            local_transform.rotation *= pitch;
        }

        mark_local_transform_dirty(world, camera_entity);
    } else {
        if FLY_CAM_HAS_CURSOR.load(Ordering::Relaxed) {
            if let Some(window_handle) = &world.resources.window.handle {
                let _ = window_handle.set_cursor_grab(winit::window::CursorGrabMode::None);
                window_handle.set_cursor_visible(true);
            }
            FLY_CAM_HAS_CURSOR.store(false, Ordering::Relaxed);
        }

        if let Some(Camera {
            smoothing:
                Some(Smoothing {
                    smoothed_mouse_delta,
                    mouse_smoothness,
                    ..
                }),
            ..
        }) = world.get_camera_mut(camera_entity)
        {
            let decay_smoothness = (*mouse_smoothness * 0.5).max(0.01);
            let smoothing_factor = 1.0 - decay_smoothness.powi(7).powf(delta_time);
            *smoothed_mouse_delta =
                *smoothed_mouse_delta * (1.0 - smoothing_factor) + Vec2::zeros() * smoothing_factor;
        }
    }

    if world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::MIDDLE_CLICKED)
    {
        let (right, up) = {
            let Some(local_transform) = world.get_local_transform(camera_entity) else {
                return;
            };
            (local_transform.right_vector(), local_transform.up_vector())
        };

        let mut delta =
            world.resources.input.mouse.position_delta * world.resources.window.timing.delta_time;
        delta.x *= -1.0;
        delta.y *= -1.0;

        let Some(local_transform) = world.get_local_transform_mut(camera_entity) else {
            return;
        };
        let translation_right = right * delta.x;
        let translation_up = up * delta.y;

        local_transform.translation += translation_right;
        local_transform.translation += translation_up;

        let changed = translation_right.magnitude() > 0.0 || translation_up.magnitude() > 0.0;
        if changed {
            mark_local_transform_dirty(world, camera_entity);
        }
    }
}

fn fly_cam_wasd(world: &mut World) {
    if let Some(gui_state) = &mut world.resources.user_interface.state
        && gui_state.egui_ctx().wants_keyboard_input()
    {
        return;
    }

    let Some(camera_entity) = world.resources.active_camera else {
        return;
    };
    let delta_time = world.resources.window.timing.delta_time;

    let (left, right, forward_key, backward, up, shift) = {
        let keyboard = &world.resources.input.keyboard;
        (
            keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyA),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyD),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyW),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyS),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::Space),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::ShiftLeft)
                || keyboard.is_key_pressed(winit::keyboard::KeyCode::ShiftRight),
        )
    };

    let base_speed = if shift { 60.0 } else { 20.0 };

    let mut target_movement = Vec3::zeros();
    if forward_key {
        target_movement.z += 1.0;
    }
    if backward {
        target_movement.z -= 1.0;
    }
    if left {
        target_movement.x -= 1.0;
    }
    if right {
        target_movement.x += 1.0;
    }
    if up {
        target_movement.y += 1.0;
    }

    if target_movement.magnitude() > 0.0 {
        target_movement = target_movement.normalize();
    }

    let Some(camera) = world.get_camera_mut(camera_entity) else {
        return;
    };
    let Some(smoothing) = camera.smoothing.as_mut() else {
        return;
    };

    let smoothing_factor = if smoothing.keyboard_smoothness > 0.0 {
        1.0 - smoothing.keyboard_smoothness.powi(7).powf(delta_time)
    } else {
        1.0
    };
    smoothing.smoothed_movement =
        smoothing.smoothed_movement * (1.0 - smoothing_factor) + target_movement * smoothing_factor;

    let movement = smoothing.smoothed_movement;

    let Some(local_transform) = world.get_local_transform_mut(camera_entity) else {
        return;
    };
    let forward = local_transform.forward_vector();
    let right = local_transform.right_vector();
    let up = local_transform.up_vector();

    let forward_translation = forward * movement.z * base_speed * delta_time;
    let right_translation = right * movement.x * base_speed * delta_time;
    let up_translation = up * movement.y * base_speed * delta_time;

    local_transform.translation += forward_translation;
    local_transform.translation += right_translation;
    local_transform.translation += up_translation;

    let changed = forward_translation.magnitude() > 0.0
        || right_translation.magnitude() > 0.0
        || up_translation.magnitude() > 0.0;

    if changed {
        mark_local_transform_dirty(world, camera_entity);
    }
}
