use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(CustomPassDemo::default())?;
    Ok(())
}

struct CustomPassDemo {
    edge_pass_enabled: bool,
}

impl Default for CustomPassDemo {
    fn default() -> Self {
        Self {
            edge_pass_enabled: true,
        }
    }
}

const EDGE_DETECTION_SHADER: &str = include_str!("../shaders/edge_detection.wgsl");

struct EdgeDetectionPass {
    pipeline: wgpu::RenderPipeline,
    blit_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    cached_bind_group: Option<wgpu::BindGroup>,
}

impl EdgeDetectionPass {
    fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        blit_pipeline: wgpu::RenderPipeline,
    ) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Edge Detection Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(EDGE_DETECTION_SHADER)),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Edge Detection Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Edge Detection Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Edge Detection Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Edge Detection Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipeline,
            blit_pipeline,
            bind_group_layout,
            sampler,
            cached_bind_group: None,
        }
    }
}

impl PassNode<World> for EdgeDetectionPass {
    fn name(&self) -> &str {
        "edge_detection_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["input"]
    }

    fn writes(&self) -> Vec<&str> {
        vec!["output"]
    }

    fn invalidate_bind_groups(&mut self) {
        self.cached_bind_group = None;
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, World>,
    ) -> Result<
        Vec<nightshade::render::wgpu::rendergraph::SubGraphRunCommand<'r>>,
        nightshade::render::wgpu::rendergraph::RenderGraphError,
    > {
        if self.cached_bind_group.is_none() {
            let input_view = context.get_texture_view("input")?;

            self.cached_bind_group = Some(context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Edge Detection Bind Group"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(input_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                },
            ));
        }

        let pipeline = if context.is_pass_enabled() {
            &self.pipeline
        } else {
            &self.blit_pipeline
        };

        let (color_view, color_load_op, color_store_op) = context.get_color_attachment("output")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Custom Pass Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: color_load_op,
                        store: color_store_op,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, self.cached_bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..1);
        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}

impl State for CustomPassDemo {
    fn title(&self) -> &str {
        "Custom Pass Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;

        spawn_sun(world);

        let camera = spawn_camera(world, Vec3::new(0.0, 1.0, 5.0), "Main Camera".to_string());

        if let Some(camera_component) = world.get_camera_mut(camera) {
            camera_component.projection = Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 60.0_f32.to_radians(),
                z_far: None,
                z_near: 0.01,
            });
        }

        world.resources.active_camera = Some(camera);

        let cube1 = spawn_mesh(
            world,
            "Cube",
            Vec3::new(-2.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let cube1_material = format!("Cube1_{}", cube1.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            cube1_material.clone(),
            Material {
                base_color: [1.0, 0.3, 0.3, 1.0],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&cube1_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(cube1, MaterialRef::new(cube1_material));

        let cube2 = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let cube2_material = format!("Cube2_{}", cube2.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            cube2_material.clone(),
            Material {
                base_color: [0.3, 1.0, 0.3, 1.0],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&cube2_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(cube2, MaterialRef::new(cube2_material));

        let cube3 = spawn_mesh(
            world,
            "Cube",
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let cube3_material = format!("Cube3_{}", cube3.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            cube3_material.clone(),
            Material {
                base_color: [0.3, 0.3, 1.0, 1.0],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&cube3_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(cube3, MaterialRef::new(cube3_material));

        let sphere = spawn_mesh(
            world,
            "Sphere",
            Vec3::new(0.0, -2.0, 0.0),
            Vec3::new(1.5, 1.5, 1.5),
        );
        let sphere_material = format!("Sphere_{}", sphere.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            sphere_material.clone(),
            Material {
                base_color: [1.0, 1.0, 0.3, 1.0],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&sphere_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(sphere, MaterialRef::new(sphere_material));
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Custom Pass Controls").show(ui_context, |ui| {
            ui.checkbox(&mut self.edge_pass_enabled, "Enable Edge Detection");
        });
    }

    fn update_render_graph(&mut self, graph: &mut RenderGraph<World>, _world: &World) {
        let _ = graph.set_pass_enabled("edge_detection_pass", self.edge_pass_enabled);
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        tracing::info!("Adding custom pass to render graph");

        let blit_pipeline =
            passes::BlitPass::create_pipeline(device, wgpu::TextureFormat::Rgba8Unorm);
        let edge_pass =
            EdgeDetectionPass::new(device, wgpu::TextureFormat::Rgba8Unorm, blit_pipeline);

        graph
            .pass(Box::new(edge_pass))
            .read("input", resources.scene_color)
            .write("output", resources.swapchain);

        let final_blit = passes::BlitPass::new(device, surface_format);

        graph
            .pass(Box::new(final_blit))
            .read("input", resources.compute_output)
            .write("output", resources.swapchain);

        tracing::info!("Custom pass pipeline configured");
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        fly_camera_system(world);

        let time = world.resources.window.timing.uptime_milliseconds as f32 * 0.001;

        let rotation_speed = 0.5;
        let angle = time * rotation_speed;
        let rotation_y = nalgebra_glm::quat_angle_axis(angle, &Vec3::y());

        let entities: Vec<_> = world
            .query_entities(RENDER_MESH | LOCAL_TRANSFORM)
            .collect();
        for entity in entities {
            if let Some(transform) = world.get_local_transform_mut(entity) {
                transform.rotation = rotation_y;
            }
            mark_local_transform_dirty(world, entity);
        }
    }
}
