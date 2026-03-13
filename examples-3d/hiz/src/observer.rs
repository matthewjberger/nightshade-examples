use nightshade::ecs::camera::queries::{query_camera_matrices, query_window_aspect_ratio};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::prelude::*;

const OBSERVER_WIDTH: u32 = 960;
const OBSERVER_HEIGHT: u32 = 540;
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
        if let Some(camera) = world.core.get_camera_mut(camera_entity) {
            camera.projection = Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 90.0_f32.to_radians(),
                z_far: Some(3000.0),
                z_near: 1.0,
            });
        }

        let rotation = Self::compute_rotation(yaw, pitch);
        if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
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
        world.core.set_lines(frustum_lines_entity, Lines::new(Vec::new()));

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
        let Some(matrices) = query_camera_matrices(world, main_camera) else {
            return;
        };

        let visualization_far = 500.0;
        let projection = if let Some(camera) = world.core.get_camera(main_camera) {
            match &camera.projection {
                Projection::Perspective(persp) if persp.z_far.is_none() => {
                    let aspect_ratio = persp
                        .aspect_ratio
                        .unwrap_or_else(|| query_window_aspect_ratio(world).unwrap_or(16.0 / 9.0));
                    PerspectiveCamera {
                        z_far: Some(visualization_far),
                        ..*persp
                    }
                    .matrix_with_aspect(aspect_ratio)
                }
                _ => matrices.projection,
            }
        } else {
            matrices.projection
        };

        let view_proj = projection * matrices.view;
        let Some(inv_view_proj) = view_proj.try_inverse() else {
            return;
        };

        let unproject = |ndc: Vec3| -> Vec3 {
            let clip = inv_view_proj * Vec4::new(ndc.x, ndc.y, ndc.z, 1.0);
            Vec3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w)
        };

        let near_z = 1.0_f32;
        let far_z = 0.0_f32;

        let ntl = unproject(Vec3::new(-1.0, 1.0, near_z));
        let ntr = unproject(Vec3::new(1.0, 1.0, near_z));
        let nbl = unproject(Vec3::new(-1.0, -1.0, near_z));
        let nbr = unproject(Vec3::new(1.0, -1.0, near_z));
        let ftl = unproject(Vec3::new(-1.0, 1.0, far_z));
        let ftr = unproject(Vec3::new(1.0, 1.0, far_z));
        let fbl = unproject(Vec3::new(-1.0, -1.0, far_z));
        let fbr = unproject(Vec3::new(1.0, -1.0, far_z));

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

        if let Some(lines_component) = world.core.get_lines_mut(self.frustum_lines_entity) {
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
        if let Some(transform) = world.core.get_local_transform_mut(self.camera_entity) {
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
        world.resources.graphics.culling_camera_override = None;

        if let Some(lines_component) = world.core.get_lines_mut(self.frustum_lines_entity) {
            lines_component.lines.clear();
            lines_component.mark_dirty();
        }
    }

    pub fn draw_ui(&self, ui_context: &egui::Context) {
        let Some(texture_id) = self.egui_texture_id else {
            return;
        };

        let margin = 10.0;
        let pip_width = 640.0;
        let pip_height = pip_width * (OBSERVER_HEIGHT as f32 / OBSERVER_WIDTH as f32);

        egui::Area::new(egui::Id::new("observer_pip"))
            .anchor(egui::Align2::RIGHT_BOTTOM, [-margin, -margin])
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
