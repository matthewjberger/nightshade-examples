use nightshade::prelude::*;
use std::sync::{Arc, Mutex};

const HEX_OVERLAY_SHADER: &str = include_str!("../shaders/hex_overlay.wgsl");

fn mat4_to_arrays(matrix: &nalgebra_glm::Mat4) -> [[f32; 4]; 4] {
    let slice = matrix.as_slice();
    [
        [slice[0], slice[1], slice[2], slice[3]],
        [slice[4], slice[5], slice[6], slice[7]],
        [slice[8], slice[9], slice[10], slice[11]],
        [slice[12], slice[13], slice[14], slice[15]],
    ]
}
const MAX_TILES: usize = 256;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuUniforms {
    inverse_view_proj: [[f32; 4]; 4],
    viewport_size: [f32; 2],
    time: f32,
    tile_count: u32,
    hex_width: f32,
    hex_depth: f32,
    is_flat_top: u32,
    pad0: f32,
}

pub struct OverlayData {
    pub positions: Vec<[f32; 3]>,
    pub hex_width: f32,
    pub hex_depth: f32,
}

impl Default for OverlayData {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            hex_width: 1.0,
            hex_depth: 1.0,
        }
    }
}

pub type SharedOverlayData = Arc<Mutex<OverlayData>>;

pub struct HexOverlayPass {
    pipeline: wgpu::RenderPipeline,
    blit_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    tile_buffer: wgpu::Buffer,
    cached_bind_group: Option<wgpu::BindGroup>,
    overlay_data: SharedOverlayData,
}

impl HexOverlayPass {
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        blit_pipeline: wgpu::RenderPipeline,
        overlay_data: SharedOverlayData,
    ) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hex Overlay Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(HEX_OVERLAY_SHADER)),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hex Overlay BGL"),
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Hex Overlay Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hex Overlay Pipeline"),
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
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Hex Overlay Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hex Overlay Uniforms"),
            size: std::mem::size_of::<GpuUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let tile_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hex Overlay Tiles"),
            size: (MAX_TILES * 16) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            blit_pipeline,
            bind_group_layout,
            sampler,
            uniform_buffer,
            tile_buffer,
            cached_bind_group: None,
            overlay_data,
        }
    }
}

impl PassNode<World> for HexOverlayPass {
    fn name(&self) -> &str {
        "hex_overlay_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["scene", "depth"]
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
        let data = self.overlay_data.lock().unwrap();
        let tile_count = data.positions.len().min(MAX_TILES);

        let view_proj =
            nightshade::ecs::camera::queries::query_active_camera_matrices(context.configs)
                .map(|matrices| matrices.projection * matrices.view);

        let inverse_view_proj = view_proj
            .and_then(|vp| vp.try_inverse())
            .unwrap_or_else(nalgebra_glm::identity);

        let viewport_size = context
            .configs
            .resources
            .window
            .cached_viewport_size
            .map(|(width, height)| [width as f32, height as f32])
            .unwrap_or([1920.0, 1080.0]);

        let time = context.configs.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

        let uniforms = GpuUniforms {
            inverse_view_proj: mat4_to_arrays(&inverse_view_proj),
            viewport_size,
            time,
            tile_count: tile_count as u32,
            hex_width: data.hex_width,
            hex_depth: data.hex_depth,
            is_flat_top: u32::from(data.hex_width > data.hex_depth),
            pad0: 0.0,
        };

        context
            .queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        if tile_count > 0 {
            let mut gpu_tiles = [[0.0f32; 4]; MAX_TILES];
            for (index, pos) in data.positions.iter().take(tile_count).enumerate() {
                gpu_tiles[index] = [pos[0], pos[1], pos[2], 0.0];
            }
            context.queue.write_buffer(
                &self.tile_buffer,
                0,
                bytemuck::cast_slice(&gpu_tiles[..tile_count]),
            );
        }

        drop(data);

        if self.cached_bind_group.is_none() {
            let scene_view = context.get_texture_view("scene")?;
            let depth_view = context.get_texture_view("depth")?;

            self.cached_bind_group = Some(context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Hex Overlay Bind Group"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(scene_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(depth_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: self.uniform_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: self.tile_buffer.as_entire_binding(),
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
                label: Some("Hex Overlay Render Pass"),
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
                multiview_mask: None,
            });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, self.cached_bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..1);
        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}
