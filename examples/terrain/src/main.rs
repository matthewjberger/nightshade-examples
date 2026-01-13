mod chunk;
mod config;
mod terrain_pass;

use config::TerrainConfig;
use nightshade::prelude::*;
use std::sync::atomic::Ordering;
use terrain_pass::{TerrainPass, WIREFRAME_ENABLED};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(TerrainDemo::default())?;
    Ok(())
}

#[derive(Default)]
struct TerrainDemo {
    config: TerrainConfig,
    camera_entity: Option<Entity>,
    wireframe: bool,
}

impl State for TerrainDemo {
    fn title(&self) -> &str {
        "Infinite Procedural Terrain"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::CloudySky;

        let camera_pos = Vec3::new(0.0, 100.0, 0.0);
        let camera = spawn_camera(world, camera_pos, "Main Camera".to_string());

        if let Some(camera_component) = world.get_camera_mut(camera) {
            camera_component.projection = Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 60.0_f32.to_radians(),
                z_far: Some(5000.0),
                z_near: 0.1,
            });
        }

        world.resources.active_camera = Some(camera);
        self.camera_entity = Some(camera);
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let terrain_pass = TerrainPass::new(
            device,
            self.config.clone(),
            wgpu::TextureFormat::Rgba16Float,
        );

        graph
            .pass(Box::new(terrain_pass))
            .slot("color", resources.scene_color)
            .slot("depth", resources.depth);

        let blit_pass = passes::BlitPass::new(device, surface_format);
        graph
            .pass(Box::new(blit_pass))
            .read("input", resources.scene_color)
            .write("output", resources.swapchain);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        fly_camera_system(world);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Terrain Controls").show(ui_context, |ui| {
            if let Some(camera) = self.camera_entity
                && let Some(transform) = world.get_local_transform(camera)
            {
                ui.label(format!(
                    "Camera: ({:.1}, {:.1}, {:.1})",
                    transform.translation.x, transform.translation.y, transform.translation.z
                ));
            }

            ui.separator();

            ui.label("Noise Settings:");
            ui.add(
                egui::Slider::new(&mut self.config.height_scale, 10.0..=200.0).text("Height Scale"),
            );
            ui.add(
                egui::Slider::new(&mut self.config.noise_frequency, 0.001..=0.1)
                    .text("Frequency")
                    .logarithmic(true),
            );
            ui.add(egui::Slider::new(&mut self.config.noise_octaves, 1..=8).text("Octaves"));

            ui.separator();

            ui.label("LOD Settings:");
            ui.add(
                egui::Slider::new(&mut self.config.view_distance, 2..=16)
                    .text("View Distance (chunks)"),
            );

            ui.separator();

            ui.label("Debug:");
            if ui.checkbox(&mut self.wireframe, "Wireframe").changed() {
                WIREFRAME_ENABLED.store(self.wireframe, Ordering::Relaxed);
            }

            ui.separator();

            let fps = 1.0 / world.resources.window.timing.delta_time.max(0.001);
            ui.label(format!("FPS: {:.1}", fps));
        });
    }
}

impl Clone for TerrainConfig {
    fn clone(&self) -> Self {
        Self {
            chunk_size: self.chunk_size,
            view_distance: self.view_distance,
            patches_per_chunk_side: self.patches_per_chunk_side,
            max_tessellation: self.max_tessellation,
            min_tessellation: self.min_tessellation,
            height_scale: self.height_scale,
            noise_frequency: self.noise_frequency,
            noise_octaves: self.noise_octaves,
            lod_distances: self.lod_distances,
        }
    }
}
