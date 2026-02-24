use nightshade::ecs::graphics::{DepthOfField, DepthOfFieldQuality};
use nightshade::ecs::material::material_registry_insert;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(DepthOfFieldDemo)
}

#[derive(Default)]
struct DepthOfFieldDemo;

impl State for DepthOfFieldDemo {
    fn title(&self) -> &str {
        "Depth of Field Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.bloom_enabled = true;
        world.resources.graphics.bloom_intensity = 0.2;

        world.resources.graphics.depth_of_field = DepthOfField {
            enabled: true,
            focus_distance: 10.0,
            focus_range: 5.0,
            max_blur_radius: 10.0,
            bokeh_threshold: 0.7,
            bokeh_intensity: 1.2,
            quality: DepthOfFieldQuality::Medium,
            visualize_coc: false,
            tilt_shift_enabled: false,
            tilt_shift_angle: 0.0,
            tilt_shift_center: 0.0,
            tilt_shift_blur_amount: 1.0,
            visualize_tilt_shift: false,
        };

        let camera_entity = spawn_camera(world, "Main Camera".to_string());
        world.resources.active_camera = Some(camera_entity);

        spawn_ground(world);
        spawn_scene_objects(world);
        spawn_lights(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        nightshade::ecs::camera::systems::pan_orbit_camera_system(world);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        let dof = &mut world.resources.graphics.depth_of_field;

        egui::Window::new("Depth of Field")
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
            .resizable(false)
            .collapsible(true)
            .show(ui_context, |ui| {
                let fps = world.resources.window.timing.frames_per_second;
                let fps_color = if fps >= 55.0 {
                    egui::Color32::GREEN
                } else if fps >= 30.0 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::from_rgb(255, 80, 0)
                };
                ui.colored_label(fps_color, format!("FPS: {:.0}", fps));

                ui.separator();

                ui.checkbox(&mut dof.enabled, "Enable DoF");

                ui.separator();

                ui.add_enabled_ui(dof.enabled, |ui| {
                    ui.label("Focus Distance");
                    ui.add(
                        egui::Slider::new(&mut dof.focus_distance, 0.5..=100.0).logarithmic(true),
                    );

                    ui.label("Focus Range");
                    ui.add(egui::Slider::new(&mut dof.focus_range, 0.1..=50.0).logarithmic(true));

                    ui.label("Max Blur Radius");
                    ui.add(egui::Slider::new(&mut dof.max_blur_radius, 1.0..=20.0));

                    ui.separator();

                    ui.label("Bokeh Threshold");
                    ui.add(egui::Slider::new(&mut dof.bokeh_threshold, 0.0..=1.0));

                    ui.label("Bokeh Intensity");
                    ui.add(egui::Slider::new(&mut dof.bokeh_intensity, 0.0..=3.0));

                    ui.separator();

                    ui.label("Quality");
                    egui::ComboBox::from_id_salt("quality")
                        .selected_text(dof.quality.name())
                        .show_ui(ui, |ui| {
                            for quality in DepthOfFieldQuality::ALL {
                                ui.selectable_value(&mut dof.quality, *quality, quality.name());
                            }
                        });

                    ui.separator();

                    ui.checkbox(&mut dof.visualize_coc, "Visualize CoC (Debug)");

                    ui.separator();
                    ui.heading("Tilt Shift");

                    ui.checkbox(&mut dof.tilt_shift_enabled, "Enable Tilt Shift");

                    ui.add_enabled_ui(dof.tilt_shift_enabled, |ui| {
                        ui.label("Angle");
                        ui.add(
                            egui::Slider::new(&mut dof.tilt_shift_angle, -90.0..=90.0).suffix("°"),
                        );

                        ui.label("Center");
                        ui.add(egui::Slider::new(&mut dof.tilt_shift_center, -1.0..=1.0));

                        ui.label("Blur Amount");
                        ui.add(egui::Slider::new(
                            &mut dof.tilt_shift_blur_amount,
                            0.1..=3.0,
                        ));

                        ui.checkbox(&mut dof.visualize_tilt_shift, "Visualize Focus Band");
                    });
                });

                ui.separator();

                ui.label("Presets");
                ui.horizontal(|ui| {
                    if ui.button("Portrait").clicked() {
                        *dof = DepthOfField::portrait();
                    }
                    if ui.button("Cinematic").clicked() {
                        *dof = DepthOfField::cinematic();
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Macro").clicked() {
                        *dof = DepthOfField::macro_shot();
                    }
                    if ui.button("Landscape").clicked() {
                        *dof = DepthOfField::landscape();
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Tilt Shift").clicked() {
                        *dof = DepthOfField::tilt_shift();
                    }
                    if ui.button("Off").clicked() {
                        dof.enabled = false;
                    }
                });
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

        let dof_texture = graph
            .add_color_texture("dof_output")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(width, height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", resources.scene_color)
            .write("bloom", bloom_texture);

        let dof_pass =
            passes::DepthOfFieldPass::new(device, wgpu::TextureFormat::Rgba16Float, width, height);
        graph
            .pass(Box::new(dof_pass))
            .read("hdr", resources.scene_color)
            .read("depth", resources.depth)
            .write("dof_output", dof_texture);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.2);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", dof_texture)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .write("output", resources.compute_output);

        let fxaa_output = graph
            .add_color_texture("fxaa_output")
            .format(surface_format)
            .size(
                resources.surface_width.max(1),
                resources.surface_height.max(1),
            )
            .transient();

        let fxaa_pass = passes::FxaaPass::new(device, surface_format);
        graph
            .pass(Box::new(fxaa_pass))
            .read("input", resources.compute_output)
            .write("output", fxaa_output);

        let swapchain_blit_pass =
            passes::BlitPass::new(device, surface_format).with_name("default_swapchain_blit");
        graph
            .pass(Box::new(swapchain_blit_pass))
            .read("input", fxaa_output)
            .write("output", resources.swapchain);
    }
}

fn spawn_camera(world: &mut World, name: String) -> Entity {
    let camera_entity = nightshade::ecs::camera::commands::spawn_pan_orbit_camera(
        world,
        Vec3::new(0.0, 3.0, 0.0),
        25.0,
        0.0,
        0.3,
        name,
    );

    if let Some(camera_component) = world.get_camera_mut(camera_entity) {
        camera_component.projection = Projection::Perspective(PerspectiveCamera {
            aspect_ratio: None,
            y_fov_rad: 45.0_f32.to_radians(),
            z_far: Some(500.0),
            z_near: 0.1,
        });
    }

    camera_entity
}

fn spawn_ground(world: &mut World) {
    let ground_material = Material {
        base_color: [0.3, 0.3, 0.3, 1.0],
        roughness: 0.9,
        metallic: 0.0,
        ..Default::default()
    };
    spawn_mesh_with_material(
        world,
        "Cube",
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(100.0, 1.0, 100.0),
        ground_material,
        "Ground",
    );
}

fn spawn_scene_objects(world: &mut World) {
    let colors = [
        [1.0, 0.2, 0.2, 1.0],
        [0.2, 1.0, 0.2, 1.0],
        [0.2, 0.2, 1.0, 1.0],
        [1.0, 1.0, 0.2, 1.0],
        [1.0, 0.2, 1.0, 1.0],
        [0.2, 1.0, 1.0, 1.0],
        [1.0, 0.6, 0.2, 1.0],
        [0.6, 0.2, 1.0, 1.0],
    ];

    for row in 0..5 {
        for col in 0..8 {
            let x = (col as f32 - 3.5) * 4.0;
            let z = (row as f32 - 2.0) * 8.0;
            let y = 1.0 + (row as f32 * 0.3);

            let color_index = (row * 8 + col) % colors.len();
            let material = Material {
                base_color: colors[color_index],
                roughness: 0.3 + (col as f32 * 0.08),
                metallic: 0.1 + (row as f32 * 0.15),
                ..Default::default()
            };

            let mesh = if (row + col) % 2 == 0 {
                "Sphere"
            } else {
                "Cube"
            };
            let scale = 0.8 + (col as f32 * 0.1);

            spawn_mesh_with_material(
                world,
                mesh,
                Vec3::new(x, y, z),
                Vec3::new(scale, scale, scale),
                material,
                &format!("Object_{}_{}", row, col),
            );
        }
    }

    for index in 0..6 {
        let angle = index as f32 * std::f32::consts::TAU / 6.0;
        let radius = 20.0;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        let emissive_material = Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive_factor: [
                colors[index % colors.len()][0] * 3.0,
                colors[index % colors.len()][1] * 3.0,
                colors[index % colors.len()][2] * 3.0,
            ],
            emissive_strength: 2.0,
            unlit: true,
            ..Default::default()
        };

        spawn_mesh_with_material(
            world,
            "Sphere",
            Vec3::new(x, 2.0, z),
            Vec3::new(0.5, 0.5, 0.5),
            emissive_material,
            &format!("EmissiveSphere_{}", index),
        );
    }

    let pillar_material = Material {
        base_color: [0.5, 0.45, 0.4, 1.0],
        roughness: 0.7,
        metallic: 0.0,
        ..Default::default()
    };

    for index in 0..4 {
        let x = if index % 2 == 0 { -15.0 } else { 15.0 };
        let z = if index < 2 { -20.0 } else { 20.0 };

        spawn_mesh_with_material(
            world,
            "Cylinder",
            Vec3::new(x, 5.0, z),
            Vec3::new(1.5, 10.0, 1.5),
            pillar_material.clone(),
            &format!("Pillar_{}", index),
        );
    }
}

fn spawn_lights(world: &mut World) {
    let sun_entity = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM
            | nightshade::ecs::world::LIGHT,
        1,
    )[0];

    world.set_name(sun_entity, Name("Sun".to_string()));
    world.set_local_transform(
        sun_entity,
        LocalTransform {
            translation: Vec3::new(50.0, 100.0, 50.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world.set_local_transform_dirty(sun_entity, LocalTransformDirty);
    world.set_global_transform(sun_entity, GlobalTransform::default());
    world.set_light(
        sun_entity,
        Light {
            light_type: LightType::Directional,
            color: Vec3::new(1.0, 0.95, 0.9),
            intensity: 2.0,
            range: 0.0,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: true,
            shadow_bias: 0.005,
        },
    );

    if let Some(transform) = world.get_local_transform_mut(sun_entity) {
        let sun_direction = Vec3::new(-0.5, -0.8, -0.3).normalize();
        let forward = -sun_direction;
        let up = Vec3::new(0.0, 1.0, 0.0);
        let right = nalgebra_glm::cross(&up, &forward).normalize();
        let corrected_up = nalgebra_glm::cross(&forward, &right);
        let rotation_matrix = nalgebra_glm::mat3(
            right.x,
            corrected_up.x,
            forward.x,
            right.y,
            corrected_up.y,
            forward.y,
            right.z,
            corrected_up.z,
            forward.z,
        );
        transform.rotation = nalgebra_glm::mat3_to_quat(&rotation_matrix);
    }
    mark_local_transform_dirty(world, sun_entity);
}

static MATERIAL_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn spawn_mesh_with_material(
    world: &mut World,
    mesh_name: &str,
    position: Vec3,
    scale: Vec3,
    material: Material,
    name: &str,
) -> Entity {
    let entity = spawn_mesh(world, mesh_name, position, scale);

    if let Some(entity_name) = world.get_name_mut(entity) {
        *entity_name = Name(name.to_string());
    }

    let material_index = MATERIAL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let material_name = format!("DoFMaterial_{}", material_index);

    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        material,
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

    world.set_material_ref(entity, MaterialRef::new(material_name));
    entity
}
