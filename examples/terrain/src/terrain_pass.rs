use crate::chunk::{ChunkManager, PatchInput};
use crate::config::TerrainConfig;
use nightshade::ecs::camera::queries::query_active_camera_matrices;
use nightshade::ecs::world::World;
use nightshade::prelude::{Vec3, wgpu};
use nightshade::render::wgpu::rendergraph::{PassExecutionContext, PassNode};
use std::sync::atomic::{AtomicBool, Ordering};
use wgpu::util::DeviceExt;

pub static WIREFRAME_ENABLED: AtomicBool = AtomicBool::new(false);

const TESSELLATE_SHADER: &str = include_str!("../shaders/terrain_tessellate.wgsl");
const RENDER_SHADER: &str = include_str!("../shaders/terrain_render.wgsl");

const MAX_VERTICES: u32 = 2_000_000;
const MAX_INDICES: u32 = 6_000_000;
const WORKGROUP_SIZE: u32 = 64;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TerrainUniforms {
    view_projection: [[f32; 4]; 4],
    camera_position: [f32; 4],
    height_scale: f32,
    noise_frequency: f32,
    noise_octaves: u32,
    patch_count: u32,
    lod_distances: [f32; 4],
    lod_distance_4: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct RenderUniforms {
    view_projection: [[f32; 4]; 4],
    camera_position: [f32; 4],
    sun_direction: [f32; 4],
    height_scale: f32,
    fog_start: f32,
    fog_end: f32,
    _padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TerrainVertex {
    position: [f32; 4],
    normal: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Counters {
    vertex_count: u32,
    index_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DrawIndexedIndirect {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
}

pub struct TerrainPass {
    config: TerrainConfig,
    chunk_manager: ChunkManager,

    compute_uniform_buffer: wgpu::Buffer,
    patch_buffer: wgpu::Buffer,
    _vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    counter_buffer: wgpu::Buffer,
    counter_reset_buffer: wgpu::Buffer,
    draw_indirect_buffer: wgpu::Buffer,
    render_uniform_buffer: wgpu::Buffer,

    tessellate_pipeline: wgpu::ComputePipeline,
    reset_pipeline: wgpu::ComputePipeline,
    finalize_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    wireframe_pipeline: wgpu::RenderPipeline,

    _compute_bind_group_layout: wgpu::BindGroupLayout,
    compute_bind_group: wgpu::BindGroup,
    _render_bind_group_layout: wgpu::BindGroupLayout,
    render_bind_group: wgpu::BindGroup,

    current_patch_count: u32,
    patches_dirty: bool,
    pub wireframe: bool,
}

impl TerrainPass {
    pub fn new(
        device: &wgpu::Device,
        config: TerrainConfig,
        color_format: wgpu::TextureFormat,
    ) -> Self {
        let compute_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Terrain Compute Uniform Buffer"),
            size: std::mem::size_of::<TerrainUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let max_patches = config.max_patches();
        let patch_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Terrain Patch Buffer"),
            size: (std::mem::size_of::<PatchInput>() * max_patches as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Terrain Vertex Buffer"),
            size: (std::mem::size_of::<TerrainVertex>() * MAX_VERTICES as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Terrain Index Buffer"),
            size: (std::mem::size_of::<u32>() * MAX_INDICES as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDEX,
            mapped_at_creation: false,
        });

        let counter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Terrain Counter Buffer"),
            size: std::mem::size_of::<Counters>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let counter_reset_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain Counter Reset Buffer"),
            contents: bytemuck::cast_slice(&[Counters {
                vertex_count: 0,
                index_count: 0,
            }]),
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        let draw_indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain Draw Indirect Buffer"),
            contents: bytemuck::cast_slice(&[DrawIndexedIndirect {
                index_count: 0,
                instance_count: 1,
                first_index: 0,
                base_vertex: 0,
                first_instance: 0,
            }]),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
        });

        let render_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Terrain Render Uniform Buffer"),
            size: std::mem::size_of::<RenderUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Terrain Compute Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Terrain Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: compute_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: patch_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: index_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: counter_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: draw_indirect_buffer.as_entire_binding(),
                },
            ],
        });

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Terrain Render Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Terrain Render Bind Group"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: render_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: vertex_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Terrain Tessellate Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(TESSELLATE_SHADER)),
        });

        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Terrain Render Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(RENDER_SHADER)),
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Terrain Compute Pipeline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let tessellate_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Terrain Tessellate Pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &compute_shader,
                entry_point: Some("tessellate"),
                compilation_options: Default::default(),
                cache: None,
            });

        let reset_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Terrain Reset Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("reset_counters"),
            compilation_options: Default::default(),
            cache: None,
        });

        let finalize_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Terrain Finalize Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("finalize_indirect"),
            compilation_options: Default::default(),
            cache: None,
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Terrain Render Pipeline Layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Terrain Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        #[cfg(target_arch = "wasm32")]
        let wireframe_polygon_mode = wgpu::PolygonMode::Fill;
        #[cfg(not(target_arch = "wasm32"))]
        let wireframe_polygon_mode = wgpu::PolygonMode::Line;

        let wireframe_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Terrain Wireframe Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: Some("fragment_wireframe"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wireframe_polygon_mode,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            config,
            chunk_manager: ChunkManager::new(),
            compute_uniform_buffer,
            patch_buffer,
            _vertex_buffer: vertex_buffer,
            index_buffer,
            counter_buffer,
            counter_reset_buffer,
            draw_indirect_buffer,
            render_uniform_buffer,
            tessellate_pipeline,
            reset_pipeline,
            finalize_pipeline,
            render_pipeline,
            wireframe_pipeline,
            _compute_bind_group_layout: compute_bind_group_layout,
            compute_bind_group,
            _render_bind_group_layout: render_bind_group_layout,
            render_bind_group,
            current_patch_count: 0,
            patches_dirty: true,
            wireframe: false,
        }
    }
}

impl PassNode<World> for TerrainPass {
    fn name(&self) -> &str {
        "terrain_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec![]
    }

    fn writes(&self) -> Vec<&str> {
        vec![]
    }

    fn reads_writes(&self) -> Vec<&str> {
        vec!["color", "depth"]
    }

    fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, world: &World) {
        let camera_matrices = match query_active_camera_matrices(world) {
            Some(m) => m,
            None => return,
        };

        let view = camera_matrices.view;
        let projection = camera_matrices.projection;
        let view_projection = projection * view;

        let view_inverse = nightshade::prelude::nalgebra_glm::inverse(&view);
        let camera_position = Vec3::new(
            view_inverse[(0, 3)],
            view_inverse[(1, 3)],
            view_inverse[(2, 3)],
        );

        let chunks_changed =
            self.chunk_manager
                .update(camera_position.x, camera_position.z, &self.config);

        if chunks_changed || self.patches_dirty {
            let patches = self.chunk_manager.generate_patches(&self.config);
            self.current_patch_count = patches.len() as u32;

            if !patches.is_empty() {
                queue.write_buffer(&self.patch_buffer, 0, bytemuck::cast_slice(&patches));
            }

            self.patches_dirty = false;
        }

        let compute_uniforms = TerrainUniforms {
            view_projection: view_projection.into(),
            camera_position: [camera_position.x, camera_position.y, camera_position.z, 1.0],
            height_scale: self.config.height_scale,
            noise_frequency: self.config.noise_frequency,
            noise_octaves: self.config.noise_octaves,
            patch_count: self.current_patch_count,
            lod_distances: [
                self.config.lod_distances[0],
                self.config.lod_distances[1],
                self.config.lod_distances[2],
                self.config.lod_distances[3],
            ],
            lod_distance_4: self.config.lod_distances[4],
            _padding1: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
        };
        queue.write_buffer(
            &self.compute_uniform_buffer,
            0,
            bytemuck::cast_slice(&[compute_uniforms]),
        );

        let sun_direction = Vec3::new(0.5, 0.8, 0.3).normalize();
        let render_uniforms = RenderUniforms {
            view_projection: view_projection.into(),
            camera_position: [camera_position.x, camera_position.y, camera_position.z, 1.0],
            sun_direction: [sun_direction.x, sun_direction.y, sun_direction.z, 0.0],
            height_scale: self.config.height_scale,
            fog_start: self.config.chunk_size * (self.config.view_distance as f32 - 2.0),
            fog_end: self.config.chunk_size * self.config.view_distance as f32,
            _padding: 0.0,
        };
        queue.write_buffer(
            &self.render_uniform_buffer,
            0,
            bytemuck::cast_slice(&[render_uniforms]),
        );

        self.wireframe = WIREFRAME_ENABLED.load(Ordering::Relaxed);
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, World>,
    ) -> nightshade::render::wgpu::rendergraph::Result<
        Vec<nightshade::render::wgpu::rendergraph::SubGraphRunCommand<'r>>,
    > {
        if self.current_patch_count == 0 {
            return Ok(context.into_sub_graph_commands());
        }

        context.encoder.copy_buffer_to_buffer(
            &self.counter_reset_buffer,
            0,
            &self.counter_buffer,
            0,
            std::mem::size_of::<Counters>() as u64,
        );

        {
            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Terrain Reset Pass"),
                        timestamp_writes: None,
                    });
            compute_pass.set_pipeline(&self.reset_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }

        {
            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Terrain Tessellate Pass"),
                        timestamp_writes: None,
                    });
            compute_pass.set_pipeline(&self.tessellate_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            let workgroups = self.current_patch_count.div_ceil(WORKGROUP_SIZE);
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        {
            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Terrain Finalize Pass"),
                        timestamp_writes: None,
                    });
            compute_pass.set_pipeline(&self.finalize_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;

        {
            let mut render_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Terrain Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: color_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: color_load,
                            store: color_store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: depth_load,
                            store: depth_store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            if self.wireframe {
                render_pass.set_pipeline(&self.wireframe_pipeline);
            } else {
                render_pass.set_pipeline(&self.render_pipeline);
            }
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed_indirect(&self.draw_indirect_buffer, 0);
        }

        Ok(context.into_sub_graph_commands())
    }
}
