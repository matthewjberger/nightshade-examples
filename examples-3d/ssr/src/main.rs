use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::prelude::*;
use nightshade::render::wgpu::passes;
use nightshade::render::wgpu::rendergraph::RenderGraph;
use nightshade::run::RenderResources;

const HDR_BYTES: &[u8] = include_bytes!("../../../assets/sky/moonrise.hdr");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SsrDemoState::default())
}

#[derive(Default)]
struct SsrDemoState {
    camera_entity: Option<Entity>,
    loaded: bool,
}

impl State for SsrDemoState {
    fn title(&self) -> &str {
        "SSR Demo"
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

        let ssr_pass = passes::SsrPass::new(device);
        graph
            .pass(Box::new(ssr_pass))
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .read("scene_color", resources.scene_color)
            .write("ssr_raw", resources.ssr_raw);

        let ssr_blur_pass = passes::SsrBlurPass::new(device);
        graph
            .pass(Box::new(ssr_blur_pass))
            .read("ssr_raw", resources.ssr_raw)
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .write("ssr", resources.ssr);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.08);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .read("ssgi", resources.ssgi)
            .read("ssr", resources.ssr)
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

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::Hdr;
        world.resources.graphics.use_fullscreen = true;

        world.resources.graphics.ssao_enabled = true;
        world.resources.graphics.ssao_radius = 0.5;
        world.resources.graphics.ssao_bias = 0.025;
        world.resources.graphics.ssao_intensity = 1.5;

        world.resources.graphics.bloom_enabled = true;
        world.resources.graphics.bloom_intensity = 0.08;

        world.resources.graphics.ssr_enabled = true;
        world.resources.graphics.ssr_max_steps = 64;
        world.resources.graphics.ssr_thickness = 0.25;
        world.resources.graphics.ssr_max_distance = 50.0;
        world.resources.graphics.ssr_stride = 1.0;
        world.resources.graphics.ssr_fade_start = 0.7;
        world.resources.graphics.ssr_fade_end = 1.0;
        world.resources.graphics.ssr_intensity = 1.0;

        load_hdr_skybox(world, HDR_BYTES.to_vec());

        let sun = spawn_sun(world);
        if let Some(light) = world.core.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 3.0;
        }

        let camera_entity = spawn_pan_orbit_camera(
            world,
            nalgebra_glm::vec3(0.0, 1.0, 0.0),
            8.0,
            -0.3,
            0.5,
            "Main Camera".to_string(),
        );
        self.camera_entity = Some(camera_entity);
        world.resources.active_camera = Some(camera_entity);

        self.spawn_ground_plane(world);
        self.spawn_objects(world);

        self.loaded = true;
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("SSR Settings")
            .default_pos(egui::pos2(10.0, 10.0))
            .default_width(300.0)
            .show(ui_context, |ui| {
                ui.heading("Screen Space Reflections");
                ui.checkbox(&mut world.resources.graphics.ssr_enabled, "Enabled");

                if world.resources.graphics.ssr_enabled {
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssr_max_steps, 8..=128)
                            .text("Max Steps"),
                    );
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssr_thickness, 0.01..=2.0)
                            .text("Thickness"),
                    );
                    ui.add(
                        egui::Slider::new(
                            &mut world.resources.graphics.ssr_max_distance,
                            1.0..=200.0,
                        )
                        .text("Max Distance"),
                    );
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssr_stride, 0.1..=4.0)
                            .text("Stride"),
                    );
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssr_fade_start, 0.0..=1.0)
                            .text("Fade Start"),
                    );
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssr_fade_end, 0.0..=1.0)
                            .text("Fade End"),
                    );
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssr_intensity, 0.0..=2.0)
                            .text("Intensity"),
                    );
                }

                ui.separator();
                ui.heading("Other Effects");

                ui.checkbox(&mut world.resources.graphics.bloom_enabled, "Bloom");
                if world.resources.graphics.bloom_enabled {
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.bloom_intensity, 0.0..=1.0)
                            .text("Bloom Intensity"),
                    );
                }

                ui.checkbox(&mut world.resources.graphics.ssao_enabled, "SSAO");
                if world.resources.graphics.ssao_enabled {
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssao_intensity, 0.0..=3.0)
                            .text("SSAO Intensity"),
                    );
                }
            });
    }
}

impl SsrDemoState {
    fn spawn_ground_plane(&self, world: &mut World) {
        let ground = world.spawn_entities(
            LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | RENDER_MESH
                | MATERIAL_REF
                | CASTS_SHADOW,
            1,
        )[0];
        world.core.set_local_transform(
            ground,
            LocalTransform {
                translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                rotation: Quat::identity(),
                scale: nalgebra_glm::vec3(20.0, 0.05, 20.0),
            },
        );
        world.core.set_render_mesh(ground, RenderMesh::new("Cube"));

        let material_name = format!("GroundMirror_{}", ground.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color: [0.02, 0.02, 0.02, 1.0],
                roughness: 0.05,
                metallic: 1.0,
                ..Default::default()
            },
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
        world.core.set_material_ref(ground, MaterialRef::new(material_name));
        world.core.set_casts_shadow(ground, CastsShadow);
    }

    fn spawn_objects(&self, world: &mut World) {
        const GLTF_DATA: &[u8] = include_bytes!("../../../assets/gltf/DamagedHelmet.glb");
        let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(GLTF_DATA);

        match load_result {
            Ok(result) => {
                for (name, (rgba_data, width, height)) in result.textures {
                    world.queue_command(WorldCommand::LoadTexture {
                        name,
                        rgba_data,
                        width,
                        height,
                    });
                }

                for (name, mesh) in result.meshes {
                    mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
                }

                for prefab in &result.prefabs {
                    nightshade::ecs::prefab::spawn_prefab(
                        world,
                        prefab,
                        nalgebra_glm::vec3(0.0, 2.0, 0.0),
                    );
                }
            }
            Err(error) => {
                tracing::error!("Failed to load GLTF: {}", error);
            }
        }

        let sphere_positions = [
            (
                nalgebra_glm::vec3(-3.0, 1.0, -2.0),
                [1.0, 0.2, 0.2, 1.0],
                0.95,
                0.1,
            ),
            (
                nalgebra_glm::vec3(-1.5, 1.0, -2.0),
                [0.2, 1.0, 0.2, 1.0],
                0.7,
                0.3,
            ),
            (
                nalgebra_glm::vec3(0.0, 1.0, -2.0),
                [0.2, 0.2, 1.0, 1.0],
                0.5,
                0.5,
            ),
            (
                nalgebra_glm::vec3(1.5, 1.0, -2.0),
                [0.9, 0.9, 0.1, 1.0],
                0.3,
                0.7,
            ),
            (
                nalgebra_glm::vec3(3.0, 1.0, -2.0),
                [0.8, 0.8, 0.8, 1.0],
                0.05,
                1.0,
            ),
        ];

        for (index, (position, color, roughness, metallic)) in sphere_positions.iter().enumerate() {
            let entity = world.spawn_entities(
                LOCAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | GLOBAL_TRANSFORM
                    | RENDER_MESH
                    | MATERIAL_REF
                    | CASTS_SHADOW,
                1,
            )[0];
            world.core.set_local_transform(
                entity,
                LocalTransform {
                    translation: *position,
                    rotation: Quat::identity(),
                    scale: nalgebra_glm::vec3(0.5, 0.5, 0.5),
                },
            );
            world.core.set_render_mesh(entity, RenderMesh::new("Sphere"));

            let material_name = format!("Sphere_{}_{}", index, entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: *color,
                    roughness: *roughness,
                    metallic: *metallic,
                    ..Default::default()
                },
            );
            if let Some(&mat_index) = world
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
                    .add_reference(mat_index);
            }
            world.core.set_material_ref(entity, MaterialRef::new(material_name));
            world.core.set_casts_shadow(entity, CastsShadow);
        }

        let emissive_positions = [
            (nalgebra_glm::vec3(-4.0, 0.5, 2.0), [5.0, 0.5, 0.5]),
            (nalgebra_glm::vec3(0.0, 0.5, 3.0), [0.5, 5.0, 0.5]),
            (nalgebra_glm::vec3(4.0, 0.5, 2.0), [0.5, 0.5, 5.0]),
        ];

        for (index, (position, emissive_color)) in emissive_positions.iter().enumerate() {
            let entity = world.spawn_entities(
                LOCAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | GLOBAL_TRANSFORM
                    | RENDER_MESH
                    | MATERIAL_REF
                    | CASTS_SHADOW,
                1,
            )[0];
            world.core.set_local_transform(
                entity,
                LocalTransform {
                    translation: *position,
                    rotation: Quat::identity(),
                    scale: nalgebra_glm::vec3(0.3, 0.3, 0.3),
                },
            );
            world.core.set_render_mesh(entity, RenderMesh::new("Sphere"));

            let material_name = format!("Emissive_{}_{}", index, entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.1, 0.1, 0.1, 1.0],
                    roughness: 1.0,
                    metallic: 0.0,
                    emissive_factor: *emissive_color,
                    ..Default::default()
                },
            );
            if let Some(&mat_index) = world
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
                    .add_reference(mat_index);
            }
            world.core.set_material_ref(entity, MaterialRef::new(material_name));
            world.core.set_casts_shadow(entity, CastsShadow);
        }

        let cube_positions = [
            (nalgebra_glm::vec3(-2.0, 0.75, 1.0), 0.1, 0.9),
            (nalgebra_glm::vec3(2.0, 0.75, 1.0), 0.3, 0.7),
        ];

        for (index, (position, roughness, metallic)) in cube_positions.iter().enumerate() {
            let entity = world.spawn_entities(
                LOCAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | GLOBAL_TRANSFORM
                    | RENDER_MESH
                    | MATERIAL_REF
                    | CASTS_SHADOW,
                1,
            )[0];
            world.core.set_local_transform(
                entity,
                LocalTransform {
                    translation: *position,
                    rotation: nalgebra_glm::quat_angle_axis(0.5, &nalgebra_glm::Vec3::y()),
                    scale: nalgebra_glm::vec3(0.75, 0.75, 0.75),
                },
            );
            world.core.set_render_mesh(entity, RenderMesh::new("Cube"));

            let material_name = format!("Cube_{}_{}", index, entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.7, 0.7, 0.7, 1.0],
                    roughness: *roughness,
                    metallic: *metallic,
                    ..Default::default()
                },
            );
            if let Some(&mat_index) = world
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
                    .add_reference(mat_index);
            }
            world.core.set_material_ref(entity, MaterialRef::new(material_name));
            world.core.set_casts_shadow(entity, CastsShadow);
        }
    }
}
