mod streamer;
mod terrain;

use nightshade::ecs::camera::commands::spawn_camera;
use nightshade::ecs::camera::systems::fly_camera_system;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

use streamer::ChunkStreamer;
use terrain::VoxelType;

const FOG_GROUND_START: f32 = 120.0;
const FOG_GROUND_END: f32 = 550.0;
const FOG_SKY_START: f32 = 200.0;
const FOG_SKY_END: f32 = 620.0;
const FOG_COLOR: [f32; 3] = [0.6, 0.65, 0.75];
const FOG_HEIGHT_LOW: f32 = 0.0;
const FOG_HEIGHT_HIGH: f32 = 100.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(VoxelWorld::default())?;
    Ok(())
}

#[derive(Default)]
struct VoxelWorld {
    chunk_streamer: Option<ChunkStreamer>,
    camera_entity: Option<Entity>,
}

impl State for VoxelWorld {
    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.user_interface.enabled = true;

        #[cfg(feature = "openxr")]
        {
            world.resources.xr.locomotion_speed = 20.0;
        }

        let camera_position = Vec3::new(0.0, 80.0, 0.0);
        let camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(camera);
        self.camera_entity = Some(camera);

        if let Some(transform) = world.core.get_local_transform_mut(camera) {
            let look_at = Vec3::new(30.0, 40.0, 30.0);
            let direction = (look_at - camera_position).normalize();
            let pitch = direction.y.asin();
            let yaw = direction.z.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
            transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
                * nalgebra_glm::quat_angle_axis(pitch, &Vec3::x());
        }

        let sun = spawn_sun(world);
        if let Some(light) = world.core.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 3.0;
            light.shadow_bias = 0.007;
        }

        register_voxel_materials(world);

        world.resources.graphics.fog = Some(Fog {
            start: FOG_GROUND_START,
            end: FOG_GROUND_END,
            color: FOG_COLOR,
        });

        let mut streamer = ChunkStreamer::new(42, 0.02, 4);
        streamer.initialize(world);
        self.chunk_streamer = Some(streamer);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        fly_camera_system(world);

        let (camera_pos, camera_forward) = if let Some(camera) = self.camera_entity
            && let Some(transform) = world.core.get_local_transform(camera)
        {
            let forward =
                nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::new(0.0, 0.0, -1.0));
            (transform.translation, forward)
        } else {
            (Vec3::zeros(), Vec3::new(0.0, 0.0, -1.0))
        };

        if let Some(streamer) = &mut self.chunk_streamer {
            streamer.update(world, camera_pos, camera_forward);
        }

        let height_factor =
            ((camera_pos.y - FOG_HEIGHT_LOW) / (FOG_HEIGHT_HIGH - FOG_HEIGHT_LOW)).clamp(0.0, 1.0);
        let fog_start = FOG_GROUND_START + (FOG_SKY_START - FOG_GROUND_START) * height_factor;
        let fog_end = FOG_GROUND_END + (FOG_SKY_END - FOG_GROUND_END) * height_factor;

        world.resources.graphics.fog = Some(Fog {
            start: fog_start,
            end: fog_end,
            color: FOG_COLOR,
        });
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Voxel World")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.heading("Infinite Voxel Terrain");
                ui.separator();

                if let Some(camera) = self.camera_entity
                    && let Some(transform) = world.core.get_local_transform(camera)
                {
                    ui.label(format!(
                        "Camera: ({:.1}, {:.1}, {:.1})",
                        transform.translation.x, transform.translation.y, transform.translation.z
                    ));
                }

                if let Some(streamer) = &self.chunk_streamer {
                    ui.label(format!("Chunks: {}", streamer.loaded_chunk_count()));
                    ui.label(format!("Instances: {}", streamer.instance_count()));
                }

                let fps = world.resources.window.timing.frames_per_second;
                let fps_color = if fps >= 56.0 {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::from_rgb(255, 165, 0)
                };
                ui.colored_label(fps_color, format!("FPS: {:.0}", fps));

                ui.separator();
                ui.label("Terrain Layers:");
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(26, 128, 217), "●");
                    ui.label("Water (< 0)");
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(242, 217, 140), "●");
                    ui.label("Sand (0-8)");
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(51, 204, 51), "●");
                    ui.label("Grass (8-20)");
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(153, 102, 51), "●");
                    ui.label("Dirt (20-30)");
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(128, 128, 140), "●");
                    ui.label("Stone (30-45)");
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 255, 255), "●");
                    ui.label("Snow (45+)");
                });

                ui.separator();
                ui.label("Controls:");
                ui.label("WASD - Move");
                ui.label("Space/Shift - Up/Down");
                ui.label("Right Mouse - Look");
                ui.label("Ctrl - Fast");
            });
    }
}

fn register_voxel_materials(world: &mut World) {
    for voxel_type in VoxelType::ALL_SOLID {
        let color = voxel_type.color();
        material_registry_insert(
            &mut world.resources.material_registry,
            voxel_type.material_name().to_string(),
            Material {
                base_color: color,
                metallic: 0.0,
                roughness: 0.9,
                ..Default::default()
            },
        );
    }
}
