use std::sync::{Arc, Mutex};

use nightshade::prelude::*;

use crate::geometry::{self, ShaderVertex};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Fullscreen,
    Geometry,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassId {
    Image,
    BufferA,
    BufferB,
    BufferC,
    BufferD,
}

impl PassId {
    pub const ALL: [PassId; 5] = [
        PassId::Image,
        PassId::BufferA,
        PassId::BufferB,
        PassId::BufferC,
        PassId::BufferD,
    ];

    pub const BUFFERS: [PassId; 4] = [
        PassId::BufferA,
        PassId::BufferB,
        PassId::BufferC,
        PassId::BufferD,
    ];

    pub fn index(self) -> usize {
        match self {
            PassId::Image => 0,
            PassId::BufferA => 1,
            PassId::BufferB => 2,
            PassId::BufferC => 3,
            PassId::BufferD => 4,
        }
    }

    pub fn buffer_index(self) -> Option<usize> {
        match self {
            PassId::BufferA => Some(0),
            PassId::BufferB => Some(1),
            PassId::BufferC => Some(2),
            PassId::BufferD => Some(3),
            PassId::Image => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PassId::Image => "Image",
            PassId::BufferA => "Buf A",
            PassId::BufferB => "Buf B",
            PassId::BufferC => "Buf C",
            PassId::BufferD => "Buf D",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChannelSource {
    None,
    Texture0,
    Texture1,
    Texture2,
    Texture3,
    BufferA,
    BufferB,
    BufferC,
    BufferD,
}

impl ChannelSource {
    pub const ALL: [ChannelSource; 9] = [
        ChannelSource::None,
        ChannelSource::Texture0,
        ChannelSource::Texture1,
        ChannelSource::Texture2,
        ChannelSource::Texture3,
        ChannelSource::BufferA,
        ChannelSource::BufferB,
        ChannelSource::BufferC,
        ChannelSource::BufferD,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ChannelSource::None => "None",
            ChannelSource::Texture0 => "Texture 0",
            ChannelSource::Texture1 => "Texture 1",
            ChannelSource::Texture2 => "Texture 2",
            ChannelSource::Texture3 => "Texture 3",
            ChannelSource::BufferA => "Buffer A",
            ChannelSource::BufferB => "Buffer B",
            ChannelSource::BufferC => "Buffer C",
            ChannelSource::BufferD => "Buffer D",
        }
    }
}

pub struct UniformData {
    pub time: f32,
    pub delta_time: f32,
    pub frame: u32,
    pub resolution: [f32; 2],
    pub mouse: [f32; 2],
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub custom: [[f32; 4]; 4],
    pub camera_position: [f32; 3],
}

impl Default for UniformData {
    fn default() -> Self {
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        Self {
            time: 0.0,
            delta_time: 0.0,
            frame: 0,
            resolution: [1920.0, 1080.0],
            mouse: [0.0, 0.0],
            model: identity,
            view: identity,
            projection: identity,
            custom: [[0.7, 0.3, 0.2, 1.0], [0.0; 4], [0.0; 4], [0.0; 4]],
            camera_position: [0.0, 0.0, 3.0],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuUniforms {
    time: f32,
    delta_time: f32,
    frame: u32,
    _pad0: u32,
    resolution: [f32; 2],
    mouse: [f32; 2],
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    custom: [[f32; 4]; 4],
    camera_position: [f32; 3],
    _pad1: f32,
}

pub struct SharedState {
    pub pass_sources: [String; 5],
    pub pass_enabled: [bool; 5],
    pub pass_needs_recompile: [bool; 5],
    pub pass_compilation_errors: [Option<String>; 5],
    pub pass_is_compiling: [bool; 5],
    pub pass_pending_validated: [Option<(String, RenderMode)>; 5],
    pub channel_bindings: [[ChannelSource; 4]; 5],
    pub channels_dirty: bool,
    pub common_source: String,
    pub common_error: Option<String>,
    pub active_tab: usize,
    pub uniforms: UniformData,
    pub render_mode: RenderMode,
    pub geometry_dirty: bool,
    pub primitive_type: crate::geometry::PrimitiveType,
    pub paused: bool,
    pub time_offset: f32,
    pub speed: f32,
    pub pending_texture_data: Vec<PendingTexture>,
    pub texture_slot_names: [Option<String>; 4],
    pub clear_texture_slot: Option<usize>,
    pub custom_mesh_data: Option<crate::geometry::MeshData>,
    pub custom_mesh_name: Option<String>,
    pub upload_custom_mesh: bool,
}

pub struct PendingTexture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub slot: usize,
}

impl Default for SharedState {
    fn default() -> Self {
        let default_source = crate::presets::default_fullscreen_source().to_string();
        Self {
            pass_sources: [
                default_source,
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ],
            pass_enabled: [true, false, false, false, false],
            pass_needs_recompile: [true, false, false, false, false],
            pass_compilation_errors: [None, None, None, None, None],
            pass_is_compiling: [false; 5],
            pass_pending_validated: [None, None, None, None, None],
            channel_bindings: [[ChannelSource::None; 4]; 5],
            channels_dirty: false,
            common_source: String::new(),
            common_error: None,
            active_tab: 0,
            uniforms: UniformData::default(),
            render_mode: RenderMode::Fullscreen,
            geometry_dirty: true,
            primitive_type: crate::geometry::PrimitiveType::Cube,
            paused: false,
            time_offset: 0.0,
            speed: 1.0,
            pending_texture_data: Vec::new(),
            texture_slot_names: [None, None, None, None],
            clear_texture_slot: None,
            custom_mesh_data: None,
            custom_mesh_name: None,
            upload_custom_mesh: false,
        }
    }
}

impl SharedState {
    pub fn active_pass_source(&self) -> &str {
        if self.active_tab == 5 {
            &self.common_source
        } else {
            &self.pass_sources[self.active_tab]
        }
    }

    pub fn active_pass_source_mut(&mut self) -> &mut String {
        if self.active_tab == 5 {
            &mut self.common_source
        } else {
            &mut self.pass_sources[self.active_tab]
        }
    }
}

pub fn validate_shader(source: &str) -> Result<RenderMode, String> {
    let module = naga::front::wgsl::parse_str(source).map_err(|err| format!("{err}"))?;

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .map_err(|err| format!("{err}"))?;

    let has_vertex = module.entry_points.iter().any(|entry_point| {
        entry_point.name == "vertex_main" && entry_point.stage == naga::ShaderStage::Vertex
    });
    let has_fragment = module.entry_points.iter().any(|entry_point| {
        entry_point.name == "fragment_main" && entry_point.stage == naga::ShaderStage::Fragment
    });

    if !has_vertex {
        return Err("Shader must have a @vertex entry point named 'vertex_main'".to_string());
    }
    if !has_fragment {
        return Err("Shader must have a @fragment entry point named 'fragment_main'".to_string());
    }

    let vertex_entry = module
        .entry_points
        .iter()
        .find(|entry_point| {
            entry_point.name == "vertex_main" && entry_point.stage == naga::ShaderStage::Vertex
        })
        .unwrap();
    let expects_vertex_inputs = vertex_entry.function.arguments.iter().any(|argument| {
        if matches!(argument.binding, Some(naga::Binding::Location { .. })) {
            return true;
        }
        if argument.binding.is_none()
            && let naga::TypeInner::Struct { ref members, .. } = module.types[argument.ty].inner
        {
            return members
                .iter()
                .any(|member| matches!(member.binding, Some(naga::Binding::Location { .. })));
        }
        false
    });

    if expects_vertex_inputs {
        Ok(RenderMode::Geometry)
    } else {
        Ok(RenderMode::Fullscreen)
    }
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

const BUFFER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

struct BufferTextures {
    _textures: [wgpu::Texture; 2],
    views: [wgpu::TextureView; 2],
    current_index: usize,
}

impl BufferTextures {
    fn create(device: &wgpu::Device, width: u32, height: u32, label: &str) -> Self {
        let create = |suffix: &str| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("{label} {suffix}")),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: BUFFER_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        };

        let texture_a = create("A");
        let texture_b = create("B");
        let view_a = texture_a.create_view(&wgpu::TextureViewDescriptor::default());
        let view_b = texture_b.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            _textures: [texture_a, texture_b],
            views: [view_a, view_b],
            current_index: 0,
        }
    }

    fn write_view(&self) -> &wgpu::TextureView {
        &self.views[self.current_index]
    }

    fn read_view(&self) -> &wgpu::TextureView {
        &self.views[self.current_index ^ 1]
    }

    fn current_view(&self) -> &wgpu::TextureView {
        &self.views[self.current_index]
    }
}

struct PassState {
    fullscreen_pipeline: Option<wgpu::RenderPipeline>,
    geometry_pipeline: Option<wgpu::RenderPipeline>,
    render_mode: RenderMode,
}

impl Default for PassState {
    fn default() -> Self {
        Self {
            fullscreen_pipeline: None,
            geometry_pipeline: None,
            render_mode: RenderMode::Fullscreen,
        }
    }
}

pub struct ShaderPass {
    shared: Arc<Mutex<SharedState>>,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    pass_states: [PassState; 5],
    per_pass_bind_groups: [Option<wgpu::BindGroup>; 5],
    buffer_textures: [Option<BufferTextures>; 4],
    buffer_size: (u32, u32),
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_count: u32,
    placeholder_texture_view: Option<wgpu::TextureView>,
    texture_views: [Option<wgpu::TextureView>; 4],
    textures_dirty: bool,
    initialized: bool,
    depth_texture_view: Option<wgpu::TextureView>,
    depth_texture_size: (u32, u32),
}

impl ShaderPass {
    pub fn new(device: &wgpu::Device, shared: Arc<Mutex<SharedState>>) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shader Studio Uniform Buffer"),
            size: std::mem::size_of::<GpuUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shader Studio Uniform BGL"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shader Studio Uniform BG"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shader Studio Texture BGL"),
                entries: &[
                    texture_entry(0),
                    sampler_entry(1),
                    texture_entry(2),
                    sampler_entry(3),
                    texture_entry(4),
                    sampler_entry(5),
                    texture_entry(6),
                    sampler_entry(7),
                ],
            });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shader Studio Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            shared,
            uniform_buffer,
            uniform_bind_group_layout,
            uniform_bind_group,
            texture_bind_group_layout,
            sampler,
            pass_states: std::array::from_fn(|_| PassState::default()),
            per_pass_bind_groups: [None, None, None, None, None],
            buffer_textures: [None, None, None, None],
            buffer_size: (0, 0),
            vertex_buffer: None,
            index_buffer: None,
            index_count: 0,
            placeholder_texture_view: None,
            texture_views: [None, None, None, None],
            textures_dirty: false,
            initialized: false,
            depth_texture_view: None,
            depth_texture_size: (0, 0),
        }
    }

    fn ensure_initialized(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.initialized {
            return;
        }
        self.initialized = true;

        let placeholder_texture = create_placeholder_texture(device, queue);
        self.placeholder_texture_view =
            Some(placeholder_texture.create_view(&wgpu::TextureViewDescriptor::default()));
    }

    fn create_pipeline_for_pass(
        &mut self,
        device: &wgpu::Device,
        source: &str,
        mode: RenderMode,
        pass_id: PassId,
    ) -> Result<(), String> {
        let target_format = BUFFER_FORMAT;

        device.push_error_scope(wgpu::ErrorFilter::Validation);

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("Shader Studio {} Shader", pass_id.label())),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(source.to_string())),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shader Studio Pipeline Layout"),
            bind_group_layouts: &[
                &self.uniform_bind_group_layout,
                &self.texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pass_index = pass_id.index();

        match mode {
            RenderMode::Fullscreen => {
                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(&format!(
                        "Shader Studio {} Fullscreen Pipeline",
                        pass_id.label()
                    )),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: Some("vertex_main"),
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        cull_mode: None,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: Some("fragment_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: target_format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    multiview: None,
                    cache: None,
                });

                if let Some(error) = pollster::block_on(device.pop_error_scope()) {
                    return Err(format!("GPU pipeline error: {error}"));
                }

                self.pass_states[pass_index].fullscreen_pipeline = Some(pipeline);
                self.pass_states[pass_index].render_mode = RenderMode::Fullscreen;
            }
            RenderMode::Geometry => {
                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(&format!(
                        "Shader Studio {} Geometry Pipeline",
                        pass_id.label()
                    )),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: Some("vertex_main"),
                        buffers: &[ShaderVertex::BUFFER_LAYOUT],
                        compilation_options: Default::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        cull_mode: Some(wgpu::Face::Back),
                        front_face: wgpu::FrontFace::Ccw,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::GreaterEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: Some("fragment_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: target_format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    multiview: None,
                    cache: None,
                });

                if let Some(error) = pollster::block_on(device.pop_error_scope()) {
                    return Err(format!("GPU pipeline error: {error}"));
                }

                self.pass_states[pass_index].geometry_pipeline = Some(pipeline);
                self.pass_states[pass_index].render_mode = RenderMode::Geometry;
            }
        }

        Ok(())
    }

    fn rebuild_bind_group_for_pass(
        &mut self,
        device: &wgpu::Device,
        pass_id: PassId,
        channel_bindings: &[ChannelSource; 4],
        is_image_pass: bool,
    ) {
        let placeholder = match &self.placeholder_texture_view {
            Some(view) => view,
            None => return,
        };

        let resolve_view = |source: ChannelSource| -> &wgpu::TextureView {
            match source {
                ChannelSource::None => placeholder,
                ChannelSource::Texture0 => self.texture_views[0].as_ref().unwrap_or(placeholder),
                ChannelSource::Texture1 => self.texture_views[1].as_ref().unwrap_or(placeholder),
                ChannelSource::Texture2 => self.texture_views[2].as_ref().unwrap_or(placeholder),
                ChannelSource::Texture3 => self.texture_views[3].as_ref().unwrap_or(placeholder),
                ChannelSource::BufferA => {
                    if let Some(buf) = &self.buffer_textures[0] {
                        if is_image_pass {
                            buf.current_view()
                        } else {
                            buf.read_view()
                        }
                    } else {
                        placeholder
                    }
                }
                ChannelSource::BufferB => {
                    if let Some(buf) = &self.buffer_textures[1] {
                        if is_image_pass {
                            buf.current_view()
                        } else {
                            buf.read_view()
                        }
                    } else {
                        placeholder
                    }
                }
                ChannelSource::BufferC => {
                    if let Some(buf) = &self.buffer_textures[2] {
                        if is_image_pass {
                            buf.current_view()
                        } else {
                            buf.read_view()
                        }
                    } else {
                        placeholder
                    }
                }
                ChannelSource::BufferD => {
                    if let Some(buf) = &self.buffer_textures[3] {
                        if is_image_pass {
                            buf.current_view()
                        } else {
                            buf.read_view()
                        }
                    } else {
                        placeholder
                    }
                }
            }
        };

        let view_0 = resolve_view(channel_bindings[0]);
        let view_1 = resolve_view(channel_bindings[1]);
        let view_2 = resolve_view(channel_bindings[2]);
        let view_3 = resolve_view(channel_bindings[3]);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Shader Studio {} Texture BG", pass_id.label())),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view_0),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(view_1),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(view_2),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(view_3),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.per_pass_bind_groups[pass_id.index()] = Some(bind_group);
    }
}

impl PassNode<World> for ShaderPass {
    fn name(&self) -> &str {
        "shader_studio_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec![]
    }

    fn writes(&self) -> Vec<&str> {
        vec![]
    }

    fn reads_writes(&self) -> Vec<&str> {
        vec!["hdr"]
    }

    fn invalidate_bind_groups(&mut self) {}

    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, _configs: &World) {
        self.ensure_initialized(device, queue);

        for buf in self.buffer_textures.iter_mut().flatten() {
            buf.current_index ^= 1;
        }

        let mut shared = self.shared.lock().unwrap();

        let width = shared.uniforms.resolution[0] as u32;
        let height = shared.uniforms.resolution[1] as u32;

        if width > 0 && height > 0 && (width, height) != self.depth_texture_size {
            let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Shader Studio Depth Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            self.depth_texture_view =
                Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));
            self.depth_texture_size = (width, height);
        }

        if width > 0 && height > 0 && (width, height) != self.buffer_size {
            let labels = ["Buffer A", "Buffer B", "Buffer C", "Buffer D"];
            for (buffer_index, label) in labels.iter().enumerate() {
                self.buffer_textures[buffer_index] =
                    Some(BufferTextures::create(device, width, height, label));
            }
            self.buffer_size = (width, height);
            shared.channels_dirty = true;
        }

        let mut any_pipeline_changed = false;
        let pending_sources: Vec<(usize, Option<(String, RenderMode)>)> = PassId::ALL
            .iter()
            .map(|pass| {
                let index = pass.index();
                (index, shared.pass_pending_validated[index].take())
            })
            .collect();

        let channels = shared.channel_bindings;
        let channels_dirty = shared.channels_dirty;
        shared.channels_dirty = false;

        drop(shared);

        for (pass_index, pending) in pending_sources {
            if let Some((source, mode)) = pending {
                let pass_id = PassId::ALL[pass_index];
                match self.create_pipeline_for_pass(device, &source, mode, pass_id) {
                    Ok(()) => {
                        let mut shared = self.shared.lock().unwrap();
                        shared.pass_compilation_errors[pass_index] = None;
                        if pass_id == PassId::Image {
                            shared.render_mode = mode;
                        }
                    }
                    Err(error) => {
                        let mut shared = self.shared.lock().unwrap();
                        shared.pass_compilation_errors[pass_index] = Some(error);
                    }
                }
                any_pipeline_changed = true;
            }
        }

        let mut shared = self.shared.lock().unwrap();

        if shared.upload_custom_mesh {
            shared.upload_custom_mesh = false;
            if let Some(mesh_data) = &shared.custom_mesh_data {
                let (vertex_buffer, index_buffer, index_count) =
                    geometry::create_gpu_buffers(device, mesh_data);
                self.vertex_buffer = Some(vertex_buffer);
                self.index_buffer = Some(index_buffer);
                self.index_count = index_count;
            }
        }

        if shared.geometry_dirty {
            shared.geometry_dirty = false;
            let primitive_type = shared.primitive_type;
            if primitive_type != crate::geometry::PrimitiveType::Custom {
                let mesh_data = geometry::generate_primitive(primitive_type);
                let (vertex_buffer, index_buffer, index_count) =
                    geometry::create_gpu_buffers(device, &mesh_data);
                self.vertex_buffer = Some(vertex_buffer);
                self.index_buffer = Some(index_buffer);
                self.index_count = index_count;
            }
        }

        if let Some(slot) = shared.clear_texture_slot.take()
            && slot < 4
        {
            self.texture_views[slot] = None;
            self.textures_dirty = true;
        }

        let pending: Vec<PendingTexture> = shared.pending_texture_data.drain(..).collect();
        drop(shared);

        for pending_texture in pending {
            if pending_texture.slot < 4 {
                let mip_levels = generate_mip_levels(
                    pending_texture.width,
                    pending_texture.height,
                    &pending_texture.data,
                );
                let mip_count = mip_levels.len() as u32;

                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Shader Studio User Texture"),
                    size: wgpu::Extent3d {
                        width: pending_texture.width,
                        height: pending_texture.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: mip_count,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });

                for (mip_level, (mip_width, mip_height, mip_data)) in mip_levels.iter().enumerate()
                {
                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &texture,
                            mip_level: mip_level as u32,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        mip_data,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * mip_width),
                            rows_per_image: Some(*mip_height),
                        },
                        wgpu::Extent3d {
                            width: *mip_width,
                            height: *mip_height,
                            depth_or_array_layers: 1,
                        },
                    );
                }

                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                self.texture_views[pending_texture.slot] = Some(view);
                self.textures_dirty = true;
            }
        }

        let any_buffer_channel = channels.iter().any(|pass_channels| {
            pass_channels.iter().any(|source| {
                matches!(
                    source,
                    ChannelSource::BufferA
                        | ChannelSource::BufferB
                        | ChannelSource::BufferC
                        | ChannelSource::BufferD
                )
            })
        });

        if self.textures_dirty || channels_dirty || any_pipeline_changed || any_buffer_channel {
            self.textures_dirty = false;
            for pass_id in &PassId::ALL {
                let is_image = *pass_id == PassId::Image;
                self.rebuild_bind_group_for_pass(
                    device,
                    *pass_id,
                    &channels[pass_id.index()],
                    is_image,
                );
            }
        }

        let shared = self.shared.lock().unwrap();
        let gpu_uniforms = GpuUniforms {
            time: shared.uniforms.time,
            delta_time: shared.uniforms.delta_time,
            frame: shared.uniforms.frame,
            _pad0: 0,
            resolution: shared.uniforms.resolution,
            mouse: shared.uniforms.mouse,
            model: shared.uniforms.model,
            view: shared.uniforms.view,
            projection: shared.uniforms.projection,
            custom: shared.uniforms.custom,
            camera_position: shared.uniforms.camera_position,
            _pad1: 0.0,
        };
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[gpu_uniforms]),
        );
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, World>,
    ) -> Result<
        Vec<nightshade::render::wgpu::rendergraph::SubGraphRunCommand<'r>>,
        nightshade::render::wgpu::rendergraph::RenderGraphError,
    > {
        if !context.is_pass_enabled() {
            return Ok(context.into_sub_graph_commands());
        }

        let pass_enabled = {
            let shared = self.shared.lock().unwrap();
            shared.pass_enabled
        };

        for pass_id in &PassId::BUFFERS {
            let pass_index = pass_id.index();
            let buffer_index = pass_id.buffer_index().unwrap();

            if !pass_enabled[pass_index] {
                continue;
            }

            let pass_state = &self.pass_states[pass_index];
            let bind_group = &self.per_pass_bind_groups[pass_index];

            let pipeline = match pass_state.render_mode {
                RenderMode::Fullscreen => pass_state.fullscreen_pipeline.as_ref(),
                RenderMode::Geometry => pass_state.geometry_pipeline.as_ref(),
            };

            let (Some(pipeline), Some(bind_group)) = (pipeline, bind_group) else {
                continue;
            };

            let Some(buf) = &self.buffer_textures[buffer_index] else {
                continue;
            };
            let target_view = buf.write_view();

            let mut render_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(&format!("Shader Studio {} Pass", pass_id.label())),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, bind_group, &[]);

            match pass_state.render_mode {
                RenderMode::Fullscreen => {
                    render_pass.draw(0..3, 0..1);
                }
                RenderMode::Geometry => {
                    if let (Some(vertex_buffer), Some(index_buffer)) =
                        (&self.vertex_buffer, &self.index_buffer)
                    {
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
                    }
                }
            }

            drop(render_pass);
        }

        let image_pass_state = &self.pass_states[PassId::Image.index()];
        let image_bind_group = &self.per_pass_bind_groups[PassId::Image.index()];

        let image_pipeline = match image_pass_state.render_mode {
            RenderMode::Fullscreen => image_pass_state.fullscreen_pipeline.as_ref(),
            RenderMode::Geometry => image_pass_state.geometry_pipeline.as_ref(),
        };

        if let (Some(pipeline), Some(bind_group)) = (image_pipeline, image_bind_group) {
            let (color_view, _color_load_op, color_store_op) =
                context.get_color_attachment("hdr")?;

            if image_pass_state.render_mode == RenderMode::Geometry {
                let hdr_size = context.get_texture_size("hdr").unwrap_or((0, 0));
                if hdr_size.0 > 0 && hdr_size.1 > 0 && hdr_size != self.depth_texture_size {
                    let depth_texture = context.device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Shader Studio Depth Texture"),
                        size: wgpu::Extent3d {
                            width: hdr_size.0,
                            height: hdr_size.1,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: DEPTH_FORMAT,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                    });
                    self.depth_texture_view =
                        Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));
                    self.depth_texture_size = hdr_size;
                }
            }

            let atmosphere = context.configs.resources.graphics.atmosphere;
            let has_background = image_pass_state.render_mode == RenderMode::Geometry
                && atmosphere != Atmosphere::None;

            let depth_attachment = if image_pass_state.render_mode == RenderMode::Geometry {
                self.depth_texture_view.as_ref().map(|view| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0.0),
                            store: wgpu::StoreOp::Discard,
                        }),
                        stencil_ops: None,
                    }
                })
            } else {
                None
            };

            let image_color_load = if has_background {
                wgpu::LoadOp::Load
            } else {
                wgpu::LoadOp::Clear(wgpu::Color::BLACK)
            };

            let mut render_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Shader Studio Image Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: color_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: image_color_load,
                            store: color_store_op,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: depth_attachment,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, bind_group, &[]);

            match image_pass_state.render_mode {
                RenderMode::Fullscreen => {
                    render_pass.draw(0..3, 0..1);
                }
                RenderMode::Geometry => {
                    if let (Some(vertex_buffer), Some(index_buffer)) =
                        (&self.vertex_buffer, &self.index_buffer)
                    {
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
                    }
                }
            }

            drop(render_pass);
        }

        Ok(context.into_sub_graph_commands())
    }
}

fn generate_mip_levels(width: u32, height: u32, data: &[u8]) -> Vec<(u32, u32, Vec<u8>)> {
    let mut levels = vec![(width, height, data.to_vec())];
    let mut current_width = width;
    let mut current_height = height;
    let mut current_data = data.to_vec();

    while current_width > 1 || current_height > 1 {
        let new_width = (current_width / 2).max(1);
        let new_height = (current_height / 2).max(1);
        let mut new_data = Vec::with_capacity((new_width * new_height * 4) as usize);

        for y in 0..new_height {
            for x in 0..new_width {
                let mut red = 0u32;
                let mut green = 0u32;
                let mut blue = 0u32;
                let mut alpha = 0u32;

                for dy in 0..2u32 {
                    for dx in 0..2u32 {
                        let source_x = (x * 2 + dx).min(current_width - 1);
                        let source_y = (y * 2 + dy).min(current_height - 1);
                        let index = ((source_y * current_width + source_x) * 4) as usize;
                        red += current_data[index] as u32;
                        green += current_data[index + 1] as u32;
                        blue += current_data[index + 2] as u32;
                        alpha += current_data[index + 3] as u32;
                    }
                }

                new_data.push((red / 4) as u8);
                new_data.push((green / 4) as u8);
                new_data.push((blue / 4) as u8);
                new_data.push((alpha / 4) as u8);
            }
        }

        levels.push((new_width, new_height, new_data.clone()));
        current_width = new_width;
        current_height = new_height;
        current_data = new_data;
    }

    levels
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn create_placeholder_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Shader Studio Placeholder Texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &[255, 0, 255, 255],
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );

    texture
}
