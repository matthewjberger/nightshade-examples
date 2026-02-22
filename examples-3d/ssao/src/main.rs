use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::world::commands::{spawn_material, spawn_sun_without_shadows};
use nightshade::prelude::*;

struct SsaoDemo;

impl State for SsaoDemo {
    fn title(&self) -> &str {
        "SSAO Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.ssao_enabled = true;
        world.resources.graphics.ssao_radius = 0.5;
        world.resources.graphics.ssao_bias = 0.025;
        world.resources.graphics.ssao_intensity = 1.5;
        world.resources.graphics.ssao_sample_count = 64;

        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 3.0, 0.0),
            12.0,
            0.4,
            0.3,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);

        spawn_sun_without_shadows(world);

        create_scene(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        pan_orbit_camera_system(world);
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        egui::Window::new("SSAO Settings")
            .default_pos([10.0, 10.0])
            .show(ctx, |ui| {
                let graphics = &mut world.resources.graphics;

                ui.checkbox(&mut graphics.ssao_enabled, "SSAO Enabled");
                ui.checkbox(&mut graphics.ssao_visualization, "Show AO Buffer");

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.add(egui::Slider::new(&mut graphics.ssao_radius, 0.1..=5.0).text("Radius"));
                ui.add(egui::Slider::new(&mut graphics.ssao_bias, 0.001..=0.1).text("Bias"));
                ui.add(
                    egui::Slider::new(&mut graphics.ssao_intensity, 0.5..=5.0).text("Intensity"),
                );

                let mut sample_count_f32 = graphics.ssao_sample_count as f32;
                if ui
                    .add(
                        egui::Slider::new(&mut sample_count_f32, 16.0..=64.0)
                            .step_by(16.0)
                            .text("Samples"),
                    )
                    .changed()
                {
                    graphics.ssao_sample_count = sample_count_f32 as u32;
                }

                ui.add_space(8.0);
                ui.separator();
                ui.label("Presets:");
                ui.horizontal(|ui| {
                    if ui.button("Subtle").clicked() {
                        graphics.ssao_radius = 0.3;
                        graphics.ssao_bias = 0.025;
                        graphics.ssao_intensity = 1.0;
                        graphics.ssao_sample_count = 32;
                    }
                    if ui.button("Medium").clicked() {
                        graphics.ssao_radius = 0.5;
                        graphics.ssao_bias = 0.025;
                        graphics.ssao_intensity = 1.5;
                        graphics.ssao_sample_count = 64;
                    }
                    if ui.button("Strong").clicked() {
                        graphics.ssao_radius = 1.0;
                        graphics.ssao_bias = 0.01;
                        graphics.ssao_intensity = 3.0;
                        graphics.ssao_sample_count = 64;
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label("Camera Controls:");
                ui.label("  Left drag: Rotate");
                ui.label("  Scroll: Zoom");
            });
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
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .write("ssao", resources.ssao);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.3);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .write("output", resources.compute_output);

        let fxaa_output = graph
            .add_color_texture("fxaa_output")
            .format(surface_format)
            .size(resources.surface_width.max(1), resources.surface_height.max(1))
            .transient();

        let fxaa_pass = passes::FxaaPass::new(device, surface_format);
        graph
            .pass(Box::new(fxaa_pass))
            .read("input", resources.compute_output)
            .write("output", fxaa_output);

        let swapchain_blit_pass = passes::BlitPass::new(device, surface_format)
            .with_name("default_swapchain_blit");
        graph
            .pass(Box::new(swapchain_blit_pass))
            .read("input", fxaa_output)
            .write("output", resources.swapchain);
    }
}

fn create_scene(world: &mut World) {
    let white = Material {
        base_color: [0.85, 0.85, 0.85, 1.0],
        roughness: 0.9,
        metallic: 0.0,
        ..Default::default()
    };

    let red = Material {
        base_color: [0.65, 0.05, 0.05, 1.0],
        roughness: 0.9,
        metallic: 0.0,
        ..Default::default()
    };

    let green = Material {
        base_color: [0.12, 0.45, 0.15, 1.0],
        roughness: 0.9,
        metallic: 0.0,
        ..Default::default()
    };

    let grey = Material {
        base_color: [0.6, 0.6, 0.6, 1.0],
        roughness: 0.7,
        metallic: 0.0,
        ..Default::default()
    };

    let floor = spawn_mesh_at(
        world,
        "Plane",
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 1.0, 10.0),
    );
    spawn_material(world, floor, "floor".to_string(), white.clone());

    let ceiling = spawn_mesh_at(
        world,
        "Plane",
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::new(10.0, 1.0, 10.0),
    );
    spawn_material(world, ceiling, "ceiling".to_string(), white.clone());

    let back_wall = spawn_mesh_at(
        world,
        "Cube",
        Vec3::new(0.0, 5.0, -5.0),
        Vec3::new(10.0, 10.0, 0.5),
    );
    spawn_material(world, back_wall, "back_wall".to_string(), white.clone());

    let left_wall = spawn_mesh_at(
        world,
        "Cube",
        Vec3::new(-5.0, 5.0, 0.0),
        Vec3::new(0.5, 10.0, 10.0),
    );
    spawn_material(world, left_wall, "left_wall".to_string(), red);

    let right_wall = spawn_mesh_at(
        world,
        "Cube",
        Vec3::new(5.0, 5.0, 0.0),
        Vec3::new(0.5, 10.0, 10.0),
    );
    spawn_material(world, right_wall, "right_wall".to_string(), green);

    let tall_box = spawn_mesh_at(
        world,
        "Cube",
        Vec3::new(1.5, 3.0, -1.5),
        Vec3::new(3.0, 6.0, 3.0),
    );
    spawn_material(world, tall_box, "tall_box".to_string(), white.clone());

    let short_box = spawn_mesh_at(
        world,
        "Cube",
        Vec3::new(-1.5, 1.5, 1.5),
        Vec3::new(3.0, 3.0, 3.0),
    );
    spawn_material(world, short_box, "short_box".to_string(), white.clone());

    let corner_sphere = spawn_mesh_at(
        world,
        "Sphere",
        Vec3::new(-3.5, 0.8, -3.5),
        Vec3::new(0.8, 0.8, 0.8),
    );
    spawn_material(
        world,
        corner_sphere,
        "corner_sphere".to_string(),
        grey.clone(),
    );

    let wall_sphere = spawn_mesh_at(
        world,
        "Sphere",
        Vec3::new(3.5, 1.0, 2.0),
        Vec3::new(1.0, 1.0, 1.0),
    );
    spawn_material(world, wall_sphere, "wall_sphere".to_string(), grey.clone());

    let cylinder = spawn_mesh_at(
        world,
        "Cylinder",
        Vec3::new(-2.5, 1.0, -1.0),
        Vec3::new(0.8, 2.0, 0.8),
    );
    spawn_material(world, cylinder, "cylinder".to_string(), grey.clone());

    let top_sphere = spawn_mesh_at(
        world,
        "Sphere",
        Vec3::new(-1.5, 3.7, 1.5),
        Vec3::new(0.7, 0.7, 0.7),
    );
    spawn_material(world, top_sphere, "top_sphere".to_string(), grey.clone());

    let torus = spawn_mesh_at(
        world,
        "Torus",
        Vec3::new(2.0, 0.4, 3.0),
        Vec3::new(1.0, 1.0, 1.0),
    );
    spawn_material(world, torus, "torus".to_string(), grey);

    let small_cube = spawn_mesh_at(
        world,
        "Cube",
        Vec3::new(-4.0, 0.5, 3.5),
        Vec3::new(1.0, 1.0, 1.0),
    );
    spawn_material(world, small_cube, "small_cube".to_string(), white);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SsaoDemo)
}
