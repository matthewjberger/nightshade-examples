use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(RenderLayersDemo::default())?;
    Ok(())
}

#[derive(Default)]
struct RenderLayersDemo {
    world_cube: Entity,
    overlay_cube: Entity,
    overlay_cube2: Entity,
}

impl State for RenderLayersDemo {
    fn title(&self) -> &str {
        "Render Layers Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.graphics.show_grid = false;

        spawn_sun(world);

        let position = Vec3::new(5.0, 3.0, 5.0);
        let camera_entity = spawn_camera(world, position, "Main Camera".to_string());
        world.resources.active_camera = Some(camera_entity);

        let floor = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(10.0, 0.1, 10.0),
        );
        let floor_material = format!("Floor_{}", floor.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            floor_material.clone(),
            Material {
                base_color: [0.5, 0.5, 0.5, 1.0],
                roughness: 0.8,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&floor_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(floor, MaterialRef::new(floor_material));
        if let Some(name) = world.get_name_mut(floor) {
            name.0 = "Floor".to_string();
        }

        let wall = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, 2.0, -3.0),
            Vec3::new(10.0, 5.0, 0.1),
        );
        let wall_material = format!("Wall_{}", wall.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            wall_material.clone(),
            Material {
                base_color: [0.7, 0.7, 0.7, 1.0],
                roughness: 0.8,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&wall_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(wall, MaterialRef::new(wall_material));
        if let Some(name) = world.get_name_mut(wall) {
            name.0 = "Wall".to_string();
        }

        self.world_cube = spawn_mesh(
            world,
            "Cube",
            Vec3::new(-2.0, 0.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let world_cube_material = format!("WorldCube_{}", self.world_cube.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            world_cube_material.clone(),
            Material {
                base_color: [1.0, 0.0, 0.0, 1.0],
                roughness: 0.5,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&world_cube_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(self.world_cube, MaterialRef::new(world_cube_material));
        if let Some(name) = world.get_name_mut(self.world_cube) {
            name.0 = "World Cube (Red)".to_string();
        }

        self.overlay_cube = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let overlay_cube_material = format!("OverlayCube1_{}", self.overlay_cube.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            overlay_cube_material.clone(),
            Material {
                base_color: [0.0, 1.0, 0.0, 1.0],
                roughness: 0.5,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&overlay_cube_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(self.overlay_cube, MaterialRef::new(overlay_cube_material));
        world.add_render_layer(self.overlay_cube);
        if let Some(layer) = world.get_render_layer_mut(self.overlay_cube) {
            layer.0 = RenderLayer::OVERLAY;
        }
        if let Some(name) = world.get_name_mut(self.overlay_cube) {
            name.0 = "Overlay Cube 1 (Green)".to_string();
        }

        self.overlay_cube2 = spawn_mesh(
            world,
            "Cube",
            Vec3::new(1.5, 0.0, -0.5),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let overlay_cube2_material = format!("OverlayCube2_{}", self.overlay_cube2.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            overlay_cube2_material.clone(),
            Material {
                base_color: [0.0, 0.0, 1.0, 1.0],
                roughness: 0.5,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&overlay_cube2_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(self.overlay_cube2, MaterialRef::new(overlay_cube2_material));
        world.add_render_layer(self.overlay_cube2);
        if let Some(layer) = world.get_render_layer_mut(self.overlay_cube2) {
            layer.0 = RenderLayer::OVERLAY;
        }
        if let Some(name) = world.get_name_mut(self.overlay_cube2) {
            name.0 = "Overlay Cube 2 (Blue)".to_string();
        }

        let occluder = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(4.0, 2.0, 0.5),
        );
        let occluder_material = format!("Occluder_{}", occluder.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            occluder_material.clone(),
            Material {
                base_color: [0.3, 0.3, 0.3, 1.0],
                roughness: 0.9,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&occluder_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(occluder, MaterialRef::new(occluder_material));
        if let Some(name) = world.get_name_mut(occluder) {
            name.0 = "Occluder".to_string();
        }

        println!("Render Layers Demo");
        println!("==================");
        println!("Red cube: World layer (renders normally, can be occluded)");
        println!("Green & Blue cubes: Overlay layer (always render on top of world)");
        println!("Gray wall: Occluder to show overlay behavior");
        println!();
        println!("Note: Both overlay cubes respect depth testing with each other!");
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        fly_camera_system(world);

        let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

        if let Some(transform) = world.get_local_transform_mut(self.world_cube) {
            transform.rotation = nalgebra_glm::quat_angle_axis(time, &Vec3::new(0.0, 1.0, 0.0));
        }

        if let Some(transform) = world.get_local_transform_mut(self.overlay_cube) {
            transform.rotation =
                nalgebra_glm::quat_angle_axis(-time * 0.8, &Vec3::new(1.0, 0.0, 0.0));
            transform.translation.z = time.sin() * 2.0;
        }

        if let Some(transform) = world.get_local_transform_mut(self.overlay_cube2) {
            transform.rotation =
                nalgebra_glm::quat_angle_axis(time * 1.2, &Vec3::new(0.0, 0.0, 1.0));
            transform.translation.z = -time.cos() * 2.0;
            transform.translation.x = 1.5 + time.sin() * 0.5;
        }
    }

    fn ui(&mut self, world: &mut World, ui: &egui::Context) {
        egui::Window::new("Render Layers").show(ui, |ui| {
            ui.heading("Layer Visibility");
            ui.separator();

            ui.checkbox(
                &mut world.resources.graphics.render_layer_world_enabled,
                "World Layer",
            );
            ui.checkbox(
                &mut world.resources.graphics.render_layer_overlay_enabled,
                "Overlay Layer",
            );

            ui.separator();
            ui.label("Info:");
            ui.label("• Red: World layer (can be occluded)");
            ui.label("• Green & Blue: Overlay (render on top)");
            ui.label("  Note: Overlay cubes occlude each other!");
        });
    }
}
