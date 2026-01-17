use nightshade::ecs::camera::{pan_orbit_camera_system, spawn_pan_orbit_camera};
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

        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 2.0, 0.0),
            15.0,
            0.5,
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
                ui.add_space(10.0);

                ui.add(egui::Slider::new(&mut graphics.ssao_radius, 0.1..=2.0).text("Radius"));

                ui.add(egui::Slider::new(&mut graphics.ssao_bias, 0.001..=0.1).text("Bias"));

                ui.add(
                    egui::Slider::new(&mut graphics.ssao_intensity, 0.5..=3.0).text("Intensity"),
                );

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.label("Camera Controls:");
                ui.label("  Left mouse drag: Rotate camera");
                ui.label("  Mouse scroll: Zoom in/out");
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
            .write("ssao", resources.ssao);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.3);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .write("output", resources.compute_output);

        let swapchain_blit_pass =
            passes::BlitPass::new(device, surface_format).with_name("swapchain_blit_pass");
        graph
            .pass(Box::new(swapchain_blit_pass))
            .read("input", resources.compute_output)
            .write("output", resources.swapchain);
    }
}

fn create_scene(world: &mut World) {
    let white_material = Material {
        base_color: [0.9, 0.9, 0.9, 1.0],
        roughness: 0.8,
        metallic: 0.0,
        ..Default::default()
    };

    let red_material = Material {
        base_color: [0.8, 0.2, 0.2, 1.0],
        roughness: 0.6,
        metallic: 0.0,
        ..Default::default()
    };

    let green_material = Material {
        base_color: [0.2, 0.8, 0.2, 1.0],
        roughness: 0.6,
        metallic: 0.0,
        ..Default::default()
    };

    let blue_material = Material {
        base_color: [0.2, 0.2, 0.8, 1.0],
        roughness: 0.6,
        metallic: 0.0,
        ..Default::default()
    };

    let floor = spawn_mesh_at(
        world,
        "Plane",
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(20.0, 1.0, 20.0),
    );
    spawn_material(
        world,
        floor,
        "white_floor".to_string(),
        white_material.clone(),
    );

    let left_wall = spawn_mesh_at(
        world,
        "Cube",
        Vec3::new(-10.0, 5.0, 0.0),
        Vec3::new(1.0, 10.0, 20.0),
    );
    spawn_material(
        world,
        left_wall,
        "white_left_wall".to_string(),
        white_material.clone(),
    );

    let right_wall = spawn_mesh_at(
        world,
        "Cube",
        Vec3::new(10.0, 5.0, 0.0),
        Vec3::new(1.0, 10.0, 20.0),
    );
    spawn_material(
        world,
        right_wall,
        "white_right_wall".to_string(),
        white_material.clone(),
    );

    let back_wall = spawn_mesh_at(
        world,
        "Cube",
        Vec3::new(0.0, 5.0, -10.0),
        Vec3::new(20.0, 10.0, 1.0),
    );
    spawn_material(
        world,
        back_wall,
        "white_back_wall".to_string(),
        white_material.clone(),
    );

    let front_wall = spawn_mesh_at(
        world,
        "Cube",
        Vec3::new(0.0, 5.0, 10.0),
        Vec3::new(20.0, 10.0, 1.0),
    );
    spawn_material(
        world,
        front_wall,
        "white_front_wall".to_string(),
        white_material,
    );

    let sphere1 = spawn_mesh_at(
        world,
        "Sphere",
        Vec3::new(-7.0, 1.5, -7.0),
        Vec3::new(1.5, 1.5, 1.5),
    );
    spawn_material(
        world,
        sphere1,
        "red_sphere1".to_string(),
        red_material.clone(),
    );

    let sphere2 = spawn_mesh_at(
        world,
        "Sphere",
        Vec3::new(7.0, 1.5, -7.0),
        Vec3::new(1.5, 1.5, 1.5),
    );
    spawn_material(
        world,
        sphere2,
        "green_sphere".to_string(),
        green_material.clone(),
    );

    let sphere3 = spawn_mesh_at(
        world,
        "Sphere",
        Vec3::new(-7.0, 1.5, 7.0),
        Vec3::new(1.5, 1.5, 1.5),
    );
    spawn_material(
        world,
        sphere3,
        "blue_sphere".to_string(),
        blue_material.clone(),
    );

    let sphere4 = spawn_mesh_at(
        world,
        "Sphere",
        Vec3::new(7.0, 1.5, 7.0),
        Vec3::new(1.5, 1.5, 1.5),
    );
    spawn_material(world, sphere4, "red_sphere2".to_string(), red_material);

    let cube1 = spawn_mesh_at(
        world,
        "Cube",
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(2.0, 2.0, 2.0),
    );
    spawn_material(world, cube1, "green_cube".to_string(), green_material);

    let cube2 = spawn_mesh_at(
        world,
        "Cube",
        Vec3::new(3.0, 0.75, 2.0),
        Vec3::new(1.5, 1.5, 1.5),
    );
    spawn_material(world, cube2, "blue_cube".to_string(), blue_material);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SsaoDemo)
}
