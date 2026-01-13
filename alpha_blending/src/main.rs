use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(AlphaBlendingDemo)?;
    Ok(())
}

struct AlphaBlendingDemo;

impl State for AlphaBlendingDemo {
    fn title(&self) -> &str {
        "Alpha Blending Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;

        spawn_sun_without_shadows(world);

        let camera_position = Vec3::new(0.0, 3.0, 12.0);
        let main_camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(main_camera);

        let background_cube = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, 0.0, -3.0),
            Vec3::new(5.0, 5.0, 0.5),
        );
        let bg_cube_mat = format!("BackgroundCube_{}", background_cube.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            bg_cube_mat.clone(),
            Material {
                base_color: [0.8, 0.8, 0.8, 1.0],
                alpha_mode: AlphaMode::Opaque,
                alpha_cutoff: 0.5,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&bg_cube_mat)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(background_cube, MaterialRef::new(bg_cube_mat));

        let opaque_sphere = spawn_mesh(
            world,
            "Sphere",
            Vec3::new(-4.0, 2.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let opaque_mat = format!("OpaqueSphere_{}", opaque_sphere.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            opaque_mat.clone(),
            Material {
                base_color: [1.0, 0.0, 0.0, 1.0],
                alpha_mode: AlphaMode::Opaque,
                alpha_cutoff: 0.5,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&opaque_mat)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(opaque_sphere, MaterialRef::new(opaque_mat));

        let mask_sphere = spawn_mesh(
            world,
            "Sphere",
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let mask_mat = format!("MaskSphere_{}", mask_sphere.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            mask_mat.clone(),
            Material {
                base_color: [0.0, 1.0, 0.0, 0.5],
                alpha_mode: AlphaMode::Mask,
                alpha_cutoff: 0.3,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&mask_mat)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(mask_sphere, MaterialRef::new(mask_mat));

        let blend_sphere_1 = spawn_mesh(
            world,
            "Sphere",
            Vec3::new(4.0, 2.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let blend1_mat = format!("BlendSphere1_{}", blend_sphere_1.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            blend1_mat.clone(),
            Material {
                base_color: [0.0, 0.0, 1.0, 0.5],
                alpha_mode: AlphaMode::Blend,
                alpha_cutoff: 0.5,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&blend1_mat)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(blend_sphere_1, MaterialRef::new(blend1_mat));

        let blend_sphere_2 = spawn_mesh(
            world,
            "Sphere",
            Vec3::new(-2.0, 0.0, 2.0),
            Vec3::new(1.5, 1.5, 1.5),
        );
        let blend2_mat = format!("BlendSphere2_{}", blend_sphere_2.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            blend2_mat.clone(),
            Material {
                base_color: [1.0, 1.0, 0.0, 0.3],
                alpha_mode: AlphaMode::Blend,
                alpha_cutoff: 0.5,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&blend2_mat)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(blend_sphere_2, MaterialRef::new(blend2_mat));

        let blend_sphere_3 = spawn_mesh(
            world,
            "Sphere",
            Vec3::new(2.0, 0.0, 2.0),
            Vec3::new(1.5, 1.5, 1.5),
        );
        let blend3_mat = format!("BlendSphere3_{}", blend_sphere_3.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            blend3_mat.clone(),
            Material {
                base_color: [1.0, 0.0, 1.0, 0.4],
                alpha_mode: AlphaMode::Blend,
                alpha_cutoff: 0.5,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&blend3_mat)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(blend_sphere_3, MaterialRef::new(blend3_mat));

        let blend_cube = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, -1.0, 4.0),
            Vec3::new(3.0, 3.0, 0.5),
        );
        let blend_cube_mat = format!("BlendCube_{}", blend_cube.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            blend_cube_mat.clone(),
            Material {
                base_color: [0.0, 1.0, 1.0, 0.6],
                alpha_mode: AlphaMode::Blend,
                alpha_cutoff: 0.5,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&blend_cube_mat)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(blend_cube, MaterialRef::new(blend_cube_mat));
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Alpha Blending Demo").show(ui_context, |ui| {
            ui.heading("Alpha Blending Demo");
            ui.separator();
            ui.label("This demo showcases three alpha modes:");
            ui.label("• Opaque (red sphere, left): No transparency");
            ui.label("• Mask (green sphere, center): Binary cutoff");
            ui.label("• Blend (blue/yellow/magenta spheres): Smooth transparency");
            ui.separator();
            ui.label("Controls:");
            ui.label("• WASD: Move camera");
            ui.label("• Mouse: Look around");
            ui.label("• Q: Exit");
        });
    }

    fn run_systems(&mut self, world: &mut World) {
        fly_camera_system(world);
    }

    fn handle_event(&mut self, _world: &mut World, message: &Message) {
        match message {
            Message::Input { event } => {
                tracing::debug!("Input event: {:?}", event);
            }
            Message::App { type_name, .. } => {
                tracing::debug!("App event: {}", type_name);
            }
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::KeyQ, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }
    }
}
