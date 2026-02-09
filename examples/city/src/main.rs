mod building;
mod camera;
mod chunk;
mod city;
mod descriptors;
mod materials;
mod minimap;
mod waterfront;

use chunk::ChunkManager;
use nightshade::ecs::graphics::resources::{DepthOfField, DepthOfFieldQuality};
use nightshade::ecs::water::Water;
use nightshade::ecs::world::WATER;
use nightshade::prelude::*;

const MIN_CAMERA_Y: f32 = 2.0;

const FOG_STREET_START: f32 = 20.0;
const FOG_STREET_END: f32 = 180.0;
const FOG_SKY_START: f32 = 200.0;
const FOG_SKY_END: f32 = 700.0;

const FOG_HEIGHT_LOW: f32 = 5.0;
const FOG_HEIGHT_HIGH: f32 = 60.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(CityDemo::default())
}

struct CityDemo {
    camera_entity: Option<Entity>,
    sun_entity: Option<Entity>,
    chunk_manager: Option<ChunkManager>,
    dof_enabled: bool,
    ssgi_enabled: bool,
    minimap_enabled: bool,
    camera_controller: Option<camera::CameraController>,
    current_hour: f32,
    time_speed: f32,
    auto_time: bool,
}

impl Default for CityDemo {
    fn default() -> Self {
        Self {
            camera_entity: None,
            sun_entity: None,
            chunk_manager: None,
            dof_enabled: false,
            ssgi_enabled: false,
            minimap_enabled: false,
            camera_controller: None,
            current_hour: 18.0,
            time_speed: 0.5,
            auto_time: true,
        }
    }
}

impl State for CityDemo {
    fn title(&self) -> &str {
        "Procedural City"
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let (width, height) = (1920, 1080);
        let bloom_width = width / 2;
        let bloom_height = height / 2;

        let bloom_texture = graph
            .add_color_texture("bloom")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(bloom_width, bloom_height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", resources.scene_color)
            .write("bloom", bloom_texture);

        let ssao_pass = passes::SsaoPass::new(device);
        graph
            .pass(Box::new(ssao_pass))
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .write("ssao_raw", resources.ssao_raw);

        let ssao_blur_pass = passes::SsaoBlurPass::new(device);
        graph
            .pass(Box::new(ssao_blur_pass))
            .read("ssao_raw", resources.ssao_raw)
            .write("ssao", resources.ssao);

        let ssgi_pass = passes::SsgiPass::new(device);
        graph
            .pass(Box::new(ssgi_pass))
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .read("scene_color", resources.scene_color)
            .write("ssgi_raw", resources.ssgi_raw);

        let ssgi_blur_pass = passes::SsgiBlurPass::new(device);
        graph
            .pass(Box::new(ssgi_blur_pass))
            .read("ssgi_raw", resources.ssgi_raw)
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .write("ssgi", resources.ssgi);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.01);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .read("ssgi", resources.ssgi)
            .write("output", resources.swapchain);
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.atmosphere = Atmosphere::DayNight;
        world.resources.graphics.day_night_hour = self.current_hour;
        capture_ibl_snapshots(world, Atmosphere::DayNight, vec![0.0, 8.0, 12.0, 18.0]);
        world.resources.graphics.show_grid = false;
        world.resources.user_interface.enabled = true;
        world.resources.graphics.fog = Some(Fog {
            start: FOG_SKY_START,
            end: FOG_SKY_END,
            color: [0.75, 0.55, 0.45],
        });

        world.resources.graphics.bloom_enabled = true;
        world.resources.graphics.bloom_intensity = 0.01;
        world.resources.graphics.ssao_enabled = true;
        world.resources.graphics.ssao_radius = 0.5;
        world.resources.graphics.ssao_intensity = 0.5;
        world.resources.graphics.ambient_light = [0.25, 0.22, 0.20, 1.0];
        world.resources.graphics.occlusion_culling_enabled = false;
        world.resources.graphics.color_grading = ColorGradingPreset::Cinematic.to_color_grading();
        world.resources.graphics.depth_of_field = DepthOfField {
            enabled: true,
            focus_distance: 50.0,
            focus_range: 30.0,
            max_blur_radius: 10.0,
            bokeh_threshold: 0.7,
            bokeh_intensity: 1.0,
            quality: DepthOfFieldQuality::Medium,
            visualize_coc: false,
            tilt_shift_enabled: false,
            tilt_shift_angle: 0.0,
            tilt_shift_center: 0.0,
            tilt_shift_blur_amount: 1.0,
            visualize_tilt_shift: false,
        };

        world.resources.graphics.ssgi_enabled = true;
        world.resources.graphics.ssgi_radius = 2.0;
        world.resources.graphics.ssgi_intensity = 0.5;
        world.resources.graphics.ssgi_max_steps = 16;

        self.dof_enabled = true;
        self.ssgi_enabled = true;
        self.minimap_enabled = true;

        materials::create_materials(world);

        let camera = spawn_camera(world, Vec3::new(0.0, 30.0, 0.0), "City Camera".to_string());
        if let Some(camera_component) = world.get_camera_mut(camera) {
            camera_component.projection = Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 60.0_f32.to_radians(),
                z_far: Some(2000.0),
                z_near: 1.0,
            });
        }
        world.resources.active_camera = Some(camera);
        self.camera_entity = Some(camera);

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 3.5;
            light.shadow_bias = 0.008;
        }
        self.sun_entity = Some(sun);
        self.update_sun_for_hour(world);

        let ocean = world.spawn_entities(WATER | NAME, 1)[0];
        world.set_name(ocean, Name("Ocean".to_string()));
        world.set_water(
            ocean,
            Water {
                base_height: -2.0,
                wave_height: 0.3,
                choppy: 2.0,
                speed: 0.5,
                frequency: 0.12,
                ..Default::default()
            },
        );

        let mut manager = ChunkManager::new();
        manager.update(world, Vec3::new(0.0, 30.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        self.chunk_manager = Some(manager);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        let delta_time = world.resources.window.timing.delta_time;

        if self.auto_time {
            self.current_hour += self.time_speed * delta_time;
            if self.current_hour >= 24.0 {
                self.current_hour -= 24.0;
            }
        }

        world.resources.graphics.day_night_hour = self.current_hour;
        self.update_sun_for_hour(world);
        self.update_environment_for_hour(world);

        if let Some(camera) = self.camera_entity {
            if let Some(controller) = &mut self.camera_controller {
                let (position, rotation) = controller.update(delta_time);
                if let Some(transform) = world.get_local_transform_mut(camera) {
                    transform.translation = position;
                    transform.rotation = rotation;
                }
            } else {
                fly_camera_system(world);
                if let Some(transform) = world.get_local_transform_mut(camera)
                    && transform.translation.y < MIN_CAMERA_Y
                {
                    transform.translation.y = MIN_CAMERA_Y;
                }
            }
            mark_local_transform_dirty(world, camera);

            let camera_pos = world
                .get_local_transform(camera)
                .map(|t| t.translation)
                .unwrap_or(Vec3::zeros());

            if let Some(manager) = &mut self.chunk_manager {
                let camera_forward = world
                    .get_local_transform(camera)
                    .map(|t| {
                        nalgebra_glm::quat_rotate_vec3(&t.rotation, &Vec3::new(0.0, 0.0, -1.0))
                    })
                    .unwrap_or(Vec3::new(0.0, 0.0, -1.0));
                manager.update(world, camera_pos, camera_forward);
            }
        }

        update_particle_emitters(world, delta_time);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        let camera_pos = self
            .camera_entity
            .and_then(|entity| world.get_local_transform(entity))
            .map(|t| t.translation)
            .unwrap_or(Vec3::zeros());

        let camera_forward = self
            .camera_entity
            .and_then(|entity| world.get_local_transform(entity))
            .map(|t| nalgebra_glm::quat_rotate_vec3(&t.rotation, &Vec3::new(0.0, 0.0, -1.0)))
            .unwrap_or(Vec3::new(0.0, 0.0, -1.0));

        egui::Window::new("City Info").show(ui_context, |ui| {
            ui.label(format!(
                "Position: ({:.1}, {:.1}, {:.1})",
                camera_pos.x, camera_pos.y, camera_pos.z
            ));

            if let Some(manager) = &self.chunk_manager {
                ui.label(format!("Chunks: {}", manager.loaded_chunk_count()));
                ui.label(format!("Entities: {}", manager.entity_count()));
            }

            let fps = world.resources.window.timing.frames_per_second;
            let fps_color = if fps >= 56.0 {
                egui::Color32::GREEN
            } else {
                egui::Color32::from_rgb(255, 165, 0)
            };
            ui.colored_label(fps_color, format!("FPS: {:.0}", fps));

            ui.separator();
            ui.label("Time of Day");
            ui.add(egui::Slider::new(&mut self.current_hour, 0.0..=24.0).text("Hour"));
            ui.checkbox(&mut self.auto_time, "Auto Time");
            if self.auto_time {
                ui.add(egui::Slider::new(&mut self.time_speed, 0.0..=5.0).text("Speed (h/s)"));
            }

            ui.separator();

            let mut dof = self.dof_enabled;
            if ui.checkbox(&mut dof, "Depth of Field").changed() {
                self.dof_enabled = dof;
                world.resources.graphics.depth_of_field.enabled = dof;
            }

            let mut ssgi = self.ssgi_enabled;
            if ui.checkbox(&mut ssgi, "Global Illumination").changed() {
                self.ssgi_enabled = ssgi;
                world.resources.graphics.ssgi_enabled = ssgi;
            }

            let mut minimap_on = self.minimap_enabled;
            if ui.checkbox(&mut minimap_on, "Minimap").changed() {
                self.minimap_enabled = minimap_on;
            }

            ui.separator();

            let auto_camera_active = self.camera_controller.is_some();
            let mut auto_cam = auto_camera_active;
            if ui.checkbox(&mut auto_cam, "Auto Camera").changed() {
                if auto_cam {
                    self.camera_controller = Some(camera::CameraController::new(
                        camera::CinematicMode::Drive,
                        camera_pos,
                    ));
                } else {
                    self.camera_controller = None;
                }
            }

            if let Some(controller) = &mut self.camera_controller {
                for &mode in camera::CinematicMode::ALL {
                    if ui.radio(controller.mode() == mode, mode.label()).clicked() {
                        controller.set_mode(mode, camera_pos);
                    }
                }
            }
        });

        if self.minimap_enabled
            && let Some(manager) = &self.chunk_manager
        {
            minimap::draw(
                ui_context,
                manager.layouts(),
                &minimap::MinimapState {
                    camera_x: camera_pos.x,
                    camera_z: camera_pos.z,
                    camera_forward_x: camera_forward.x,
                    camera_forward_z: camera_forward.z,
                    city_min: chunk::CITY_MIN,
                    city_max: chunk::CITY_MAX,
                },
            );
        }
    }
}

impl CityDemo {
    fn get_sun_direction(hour: f32) -> Vec3 {
        let pi = std::f32::consts::PI;
        if !(6.0..=18.0).contains(&hour) {
            Vec3::new(0.0, -1.0, 0.0)
        } else {
            let sun_angle = (hour - 6.0) / 12.0 * pi;
            nalgebra_glm::normalize(&Vec3::new(-sun_angle.cos(), sun_angle.sin(), -0.3))
        }
    }

    fn update_sun_for_hour(&self, world: &mut World) {
        let sun = match self.sun_entity {
            Some(entity) => entity,
            None => return,
        };

        let sun_dir = Self::get_sun_direction(self.current_hour);
        let is_night = !(6.0..=18.0).contains(&self.current_hour);

        let sun_intensity = if is_night {
            0.0
        } else {
            let elevation = sun_dir.y.max(0.0);
            3.5 * elevation.sqrt()
        };

        let sun_color = if is_night {
            Vec3::new(0.0, 0.0, 0.0)
        } else if !(7.5..=16.5).contains(&self.current_hour) {
            Vec3::new(1.0, 0.7, 0.4)
        } else {
            Vec3::new(1.0, 0.95, 0.8)
        };

        if let Some(light) = world.get_light_mut(sun) {
            light.intensity = sun_intensity;
            light.color = sun_color;
        }

        let sun_position = sun_dir * 100.0;
        if let Some(transform) = world.get_local_transform_mut(sun) {
            transform.translation = sun_position;
            let direction = -sun_dir;
            let up = Vec3::y();
            let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&direction, &up));
            if right.norm() > 0.001 {
                let corrected_up = nalgebra_glm::cross(&right, &direction);
                transform.rotation =
                    nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[
                        right,
                        corrected_up,
                        -direction,
                    ]));
            }
        }
        mark_local_transform_dirty(world, sun);
    }

    fn update_environment_for_hour(&self, world: &mut World) {
        let hour = self.current_hour;
        let is_night = !(6.0..=18.0).contains(&hour);

        let (ambient_intensity, ambient_color) = if is_night {
            (0.05, [0.05, 0.05, 0.1, 1.0])
        } else if !(7.5..=16.5).contains(&hour) {
            let t = if hour < 7.5 {
                (hour - 6.0) / 1.5
            } else {
                (18.0 - hour) / 1.5
            };
            let intensity = 0.05 + 0.20 * t;
            (
                intensity,
                [0.05 + 0.20 * t, 0.05 + 0.17 * t, 0.10 + 0.10 * t, 1.0],
            )
        } else {
            (0.25, [0.25, 0.22, 0.20, 1.0])
        };
        let _ = ambient_intensity;
        world.resources.graphics.ambient_light = ambient_color;

        let (fog_color, fog_density_mult) = if is_night {
            ([0.02, 0.02, 0.05], 0.5)
        } else if !(7.5..=16.5).contains(&hour) {
            ([0.65, 0.45, 0.35], 1.0)
        } else {
            ([0.7, 0.75, 0.8], 1.2)
        };

        let camera_y = self
            .camera_entity
            .and_then(|entity| world.get_local_transform(entity))
            .map(|t| t.translation.y)
            .unwrap_or(30.0);

        let height_factor =
            ((camera_y - FOG_HEIGHT_LOW) / (FOG_HEIGHT_HIGH - FOG_HEIGHT_LOW)).clamp(0.0, 1.0);
        let base_start = FOG_STREET_START + (FOG_SKY_START - FOG_STREET_START) * height_factor;
        let base_end = FOG_STREET_END + (FOG_SKY_END - FOG_STREET_END) * height_factor;

        world.resources.graphics.fog = Some(Fog {
            start: base_start * fog_density_mult,
            end: base_end * fog_density_mult,
            color: fog_color,
        });
    }
}
