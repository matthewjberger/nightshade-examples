use super::mesh_builder::MeshBuilder;
use super::vertex::{DoomVertex, SkyVertex, SpriteVertex};
use super::visitor::{LevelWalker, PlayerStart};
use crate::wad::{Archive, Level, TextureDirectory};
use nightshade::ecs::camera::queries::query_active_camera_matrices;
use nightshade::ecs::world::World;
use nightshade::prelude::{tracing, wgpu};
use nightshade::render::wgpu::rendergraph::{PassExecutionContext, PassNode};
use wgpu::util::DeviceExt;

const GEOMETRY_SHADER: &str = include_str!("../../shaders/doom_geometry.wgsl");
const SKY_SHADER: &str = include_str!("../../shaders/doom_sky.wgsl");
const SPRITE_SHADER: &str = include_str!("../../shaders/doom_sprite.wgsl");

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GeometryUniforms {
    view_projection: [[f32; 4]; 4],
    camera_position: [f32; 4],
    time: f32,
    atlas_width: f32,
    atlas_height: f32,
    _padding: f32,
    _padding2: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SkyUniforms {
    view_projection: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
    camera_position: [f32; 4],
    time: f32,
    tiled_band_size: f32,
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteUniforms {
    view_projection: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
    camera_position: [f32; 4],
    time: f32,
    atlas_width: f32,
    atlas_height: f32,
    _padding: f32,
}

struct PendingTextureUpload {
    wall_atlas_pixels: Vec<u16>,
    wall_atlas_width: usize,
    wall_atlas_height: usize,
    flat_atlas_pixels: Vec<u8>,
    flat_atlas_width: usize,
    flat_atlas_height: usize,
    sprite_atlas_pixels: Vec<u16>,
    sprite_atlas_width: usize,
    sprite_atlas_height: usize,
    palette_pixels: Vec<u8>,
    palette_colormaps: usize,
    sky_pixels: Option<Vec<u8>>,
    sky_width: usize,
    sky_height: usize,
}

pub struct DoomPass {
    geometry_pipeline: wgpu::RenderPipeline,
    sky_pipeline: wgpu::RenderPipeline,
    sprite_pipeline: wgpu::RenderPipeline,

    wall_vertex_buffer: wgpu::Buffer,
    wall_index_buffer: wgpu::Buffer,
    flat_vertex_buffer: wgpu::Buffer,
    flat_index_buffer: wgpu::Buffer,
    sky_vertex_buffer: wgpu::Buffer,
    sky_index_buffer: wgpu::Buffer,
    sprite_vertex_buffer: wgpu::Buffer,
    sprite_index_buffer: wgpu::Buffer,

    wall_index_count: u32,
    flat_index_count: u32,
    sky_index_count: u32,
    sprite_index_count: u32,

    geometry_uniform_buffer: wgpu::Buffer,
    sky_uniform_buffer: wgpu::Buffer,
    sprite_uniform_buffer: wgpu::Buffer,

    wall_atlas_texture: wgpu::Texture,
    _wall_atlas_view: wgpu::TextureView,
    flat_atlas_texture: wgpu::Texture,
    _flat_atlas_view: wgpu::TextureView,
    sprite_atlas_texture: wgpu::Texture,
    _sprite_atlas_view: wgpu::TextureView,
    palette_texture: wgpu::Texture,
    _palette_view: wgpu::TextureView,
    sky_texture: wgpu::Texture,
    _sky_view: wgpu::TextureView,

    _sampler: wgpu::Sampler,

    wall_bind_group: wgpu::BindGroup,
    flat_bind_group: wgpu::BindGroup,
    sky_bind_group: wgpu::BindGroup,
    sprite_bind_group: wgpu::BindGroup,

    wall_atlas_width: f32,
    wall_atlas_height: f32,
    _flat_atlas_width: f32,
    _flat_atlas_height: f32,
    sprite_atlas_width: f32,
    sprite_atlas_height: f32,

    pub player_start: Option<PlayerStart>,
    time: f32,

    pending_upload: Option<PendingTextureUpload>,
}

impl DoomPass {
    pub fn new(
        device: &wgpu::Device,
        archive: &Archive,
        tex_dir: &TextureDirectory,
        level_index: usize,
        color_format: wgpu::TextureFormat,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let level = Level::from_archive(archive, level_index)?;

        let mut builder = MeshBuilder::new();

        {
            let mut walker = LevelWalker::new(&level, tex_dir, &mut builder);
            walker.walk();
        }

        let (wall_atlas, wall_bounds) =
            tex_dir.build_texture_atlas(builder.wall_texture_names.iter().copied());
        let (flat_atlas, flat_bounds) =
            tex_dir.build_flat_atlas(builder.flat_texture_names.iter().copied());
        let (sprite_atlas, sprite_bounds) = tex_dir.build_sprite_atlas_with_animations(
            builder
                .sprite_info
                .iter()
                .map(|(name, info)| (*name, info.prefix, info.sequence)),
        );

        builder.set_wall_bounds(wall_bounds);
        builder.set_flat_bounds(flat_bounds);
        builder.set_sprite_bounds(sprite_bounds);

        builder.wall_vertices.clear();
        builder.wall_indices.clear();
        builder.flat_vertices.clear();
        builder.flat_indices.clear();
        builder.sky_vertices.clear();
        builder.sky_indices.clear();
        builder.sprite_vertices.clear();
        builder.sprite_indices.clear();

        {
            let mut walker = LevelWalker::new(&level, tex_dir, &mut builder);
            walker.walk();
        }

        let wall_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Doom Wall Vertex Buffer"),
            contents: bytemuck::cast_slice(&builder.wall_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let wall_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Doom Wall Index Buffer"),
            contents: bytemuck::cast_slice(&builder.wall_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let flat_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Doom Flat Vertex Buffer"),
            contents: bytemuck::cast_slice(&builder.flat_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let flat_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Doom Flat Index Buffer"),
            contents: bytemuck::cast_slice(&builder.flat_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let dummy_sky_vertex = [SkyVertex::new([0.0, 0.0, 0.0])];
        let dummy_sky_index = [0u32];

        let sky_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Doom Sky Vertex Buffer"),
            contents: if builder.sky_vertices.is_empty() {
                bytemuck::cast_slice(&dummy_sky_vertex)
            } else {
                bytemuck::cast_slice(&builder.sky_vertices)
            },
            usage: wgpu::BufferUsages::VERTEX,
        });

        let sky_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Doom Sky Index Buffer"),
            contents: if builder.sky_indices.is_empty() {
                bytemuck::cast_slice(&dummy_sky_index)
            } else {
                bytemuck::cast_slice(&builder.sky_indices)
            },
            usage: wgpu::BufferUsages::INDEX,
        });

        let dummy_sprite_vertex = [SpriteVertex::new(
            [0.0, 0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
            [1.0, 1.0],
            0.0,
            1.0,
            1,
        )];
        let dummy_sprite_index = [0u32];

        let sprite_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Doom Sprite Vertex Buffer"),
            contents: if builder.sprite_vertices.is_empty() {
                bytemuck::cast_slice(&dummy_sprite_vertex)
            } else {
                bytemuck::cast_slice(&builder.sprite_vertices)
            },
            usage: wgpu::BufferUsages::VERTEX,
        });

        let sprite_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Doom Sprite Index Buffer"),
            contents: if builder.sprite_indices.is_empty() {
                bytemuck::cast_slice(&dummy_sprite_index)
            } else {
                bytemuck::cast_slice(&builder.sprite_indices)
            },
            usage: wgpu::BufferUsages::INDEX,
        });

        let wall_atlas_width = wall_atlas.width.max(1);
        let wall_atlas_height = wall_atlas.height.max(1);
        let flat_atlas_width = flat_atlas.width.max(1);
        let flat_atlas_height = flat_atlas.height.max(1);
        let sprite_atlas_width = sprite_atlas.width.max(1);
        let sprite_atlas_height = sprite_atlas.height.max(1);

        let mapped_palette = tex_dir.build_palette_texture(0, 0, 32);
        let sky_texture_data = tex_dir.build_sky_texture(level_index);

        let (sky_width, sky_height, sky_pixels) = match &sky_texture_data {
            Some(sky) => (
                sky.width.max(1),
                sky.height.max(1),
                Some(sky.pixels.clone()),
            ),
            None => (256, 128, None),
        };

        let pending_upload = Some(PendingTextureUpload {
            wall_atlas_pixels: wall_atlas.pixels,
            wall_atlas_width,
            wall_atlas_height,
            flat_atlas_pixels: flat_atlas.pixels,
            flat_atlas_width,
            flat_atlas_height,
            sprite_atlas_pixels: sprite_atlas.pixels,
            sprite_atlas_width,
            sprite_atlas_height,
            palette_pixels: mapped_palette.pixels,
            palette_colormaps: mapped_palette.colormaps,
            sky_pixels,
            sky_width,
            sky_height,
        });

        let wall_atlas_texture =
            create_empty_r16_texture(device, wall_atlas_width, wall_atlas_height, "Wall Atlas");
        let wall_atlas_view =
            wall_atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let flat_atlas_texture =
            create_empty_r8_texture(device, flat_atlas_width, flat_atlas_height, "Flat Atlas");
        let flat_atlas_view =
            flat_atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sprite_atlas_texture = create_empty_r16_texture(
            device,
            sprite_atlas_width,
            sprite_atlas_height,
            "Sprite Atlas",
        );
        let sprite_atlas_view =
            sprite_atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let palette_texture =
            create_empty_rgba_texture(device, 256, mapped_palette.colormaps, "Palette");
        let palette_view = palette_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sky_texture = create_empty_r8_texture(device, sky_width, sky_height, "Sky");
        let sky_view = sky_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Doom Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let geometry_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Doom Geometry Uniform Buffer"),
            size: std::mem::size_of::<GeometryUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sky_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Doom Sky Uniform Buffer"),
            size: std::mem::size_of::<SkyUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sprite_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Doom Sprite Uniform Buffer"),
            size: std::mem::size_of::<SpriteUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let geometry_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Doom Geometry Bind Group Layout"),
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
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let wall_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Doom Wall Bind Group"),
            layout: &geometry_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: geometry_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&wall_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&palette_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let flat_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Doom Flat Bind Group"),
            layout: &geometry_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: geometry_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&flat_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&palette_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let sky_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Doom Sky Bind Group Layout"),
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
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let sky_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Doom Sky Bind Group"),
            layout: &sky_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sky_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&sky_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&palette_view),
                },
            ],
        });

        let sprite_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Doom Sprite Bind Group"),
            layout: &geometry_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sprite_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&sprite_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&palette_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let geometry_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Doom Geometry Pipeline Layout"),
                bind_group_layouts: &[Some(&geometry_bind_group_layout)],
                immediate_size: 0,
            });

        let geometry_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Doom Geometry Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(GEOMETRY_SHADER)),
        });

        let geometry_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Doom Geometry Pipeline"),
            layout: Some(&geometry_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &geometry_shader,
                entry_point: Some("vertex_main"),
                buffers: &[DoomVertex::vertex_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &geometry_shader,
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
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Doom Sky Pipeline Layout"),
            bind_group_layouts: &[Some(&sky_bind_group_layout)],
            immediate_size: 0,
        });

        let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Doom Sky Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SKY_SHADER)),
        });

        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Doom Sky Pipeline"),
            layout: Some(&sky_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sky_shader,
                entry_point: Some("vertex_main"),
                buffers: &[SkyVertex::vertex_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sky_shader,
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
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let sprite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Doom Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SPRITE_SHADER)),
        });

        let sprite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Doom Sprite Pipeline"),
            layout: Some(&geometry_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sprite_shader,
                entry_point: Some("vertex_main"),
                buffers: &[SpriteVertex::vertex_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sprite_shader,
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
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let wall_index_count = builder.wall_indices.len() as u32;
        let flat_index_count = builder.flat_indices.len() as u32;
        let sky_index_count = builder.sky_indices.len() as u32;
        let sprite_index_count = builder.sprite_indices.len() as u32;

        tracing::info!(
            "DoomPass created: walls={}, flats={}, sky={}, sprites={}, wall_atlas={}x{}, sprite_atlas={}x{}",
            wall_index_count,
            flat_index_count,
            sky_index_count,
            sprite_index_count,
            wall_atlas_width,
            wall_atlas_height,
            sprite_atlas_width,
            sprite_atlas_height
        );

        Ok(Self {
            geometry_pipeline,
            sky_pipeline,
            sprite_pipeline,
            wall_vertex_buffer,
            wall_index_buffer,
            flat_vertex_buffer,
            flat_index_buffer,
            sky_vertex_buffer,
            sky_index_buffer,
            sprite_vertex_buffer,
            sprite_index_buffer,
            wall_index_count,
            flat_index_count,
            sky_index_count,
            sprite_index_count,
            geometry_uniform_buffer,
            sky_uniform_buffer,
            sprite_uniform_buffer,
            wall_atlas_texture,
            _wall_atlas_view: wall_atlas_view,
            flat_atlas_texture,
            _flat_atlas_view: flat_atlas_view,
            sprite_atlas_texture,
            _sprite_atlas_view: sprite_atlas_view,
            palette_texture,
            _palette_view: palette_view,
            sky_texture,
            _sky_view: sky_view,
            _sampler: sampler,
            wall_bind_group,
            flat_bind_group,
            sky_bind_group,
            sprite_bind_group,
            wall_atlas_width: wall_atlas_width as f32,
            wall_atlas_height: wall_atlas_height as f32,
            _flat_atlas_width: flat_atlas_width as f32,
            _flat_atlas_height: flat_atlas_height as f32,
            sprite_atlas_width: sprite_atlas_width as f32,
            sprite_atlas_height: sprite_atlas_height as f32,
            player_start: builder.player_start,
            time: 0.0,
            pending_upload,
        })
    }
}

impl PassNode<World> for DoomPass {
    fn name(&self) -> &str {
        "doom_pass"
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
        if let Some(upload) = self.pending_upload.take() {
            upload_r16_texture(
                queue,
                &self.wall_atlas_texture,
                upload.wall_atlas_width,
                upload.wall_atlas_height,
                &upload.wall_atlas_pixels,
            );
            upload_r8_texture(
                queue,
                &self.flat_atlas_texture,
                upload.flat_atlas_width,
                upload.flat_atlas_height,
                &upload.flat_atlas_pixels,
            );
            upload_r16_texture(
                queue,
                &self.sprite_atlas_texture,
                upload.sprite_atlas_width,
                upload.sprite_atlas_height,
                &upload.sprite_atlas_pixels,
            );
            upload_rgba_texture(
                queue,
                &self.palette_texture,
                256,
                upload.palette_colormaps,
                &upload.palette_pixels,
            );
            if let Some(sky_pixels) = &upload.sky_pixels {
                upload_r8_texture(
                    queue,
                    &self.sky_texture,
                    upload.sky_width,
                    upload.sky_height,
                    sky_pixels,
                );
            }
        }

        let camera_matrices = match query_active_camera_matrices(world) {
            Some(m) => m,
            None => return,
        };

        let view = camera_matrices.view;
        let projection = camera_matrices.projection;
        let view_projection = projection * view;

        let view_inverse = nightshade::prelude::nalgebra_glm::inverse(&view);
        let camera_position = [
            view_inverse[(0, 3)],
            view_inverse[(1, 3)],
            view_inverse[(2, 3)],
            1.0,
        ];

        self.time += world.resources.window.timing.delta_time;

        let geometry_uniforms = GeometryUniforms {
            view_projection: view_projection.into(),
            camera_position,
            time: self.time,
            atlas_width: self.wall_atlas_width,
            atlas_height: self.wall_atlas_height,
            _padding: 0.0,
            _padding2: [0.0, 0.0, 0.0, 0.0],
        };

        let sky_uniforms = SkyUniforms {
            view_projection: view_projection.into(),
            view_matrix: view.into(),
            camera_position,
            time: self.time,
            tiled_band_size: 0.186,
            _padding: [0.0, 0.0],
        };

        let sprite_uniforms = SpriteUniforms {
            view_projection: view_projection.into(),
            view_matrix: view.into(),
            camera_position,
            time: self.time,
            atlas_width: self.sprite_atlas_width,
            atlas_height: self.sprite_atlas_height,
            _padding: 0.0,
        };

        queue.write_buffer(
            &self.geometry_uniform_buffer,
            0,
            bytemuck::cast_slice(&[geometry_uniforms]),
        );
        queue.write_buffer(
            &self.sky_uniform_buffer,
            0,
            bytemuck::cast_slice(&[sky_uniforms]),
        );
        queue.write_buffer(
            &self.sprite_uniform_buffer,
            0,
            bytemuck::cast_slice(&[sprite_uniforms]),
        );
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, World>,
    ) -> nightshade::render::wgpu::rendergraph::Result<
        Vec<nightshade::render::wgpu::rendergraph::SubGraphRunCommand<'r>>,
    > {
        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;

        {
            let mut render_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Doom Render Pass"),
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
                    multiview_mask: None,
                });

            if self.sky_index_count > 0 {
                render_pass.set_pipeline(&self.sky_pipeline);
                render_pass.set_bind_group(0, &self.sky_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.sky_vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.sky_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.sky_index_count, 0, 0..1);
            }

            render_pass.set_pipeline(&self.geometry_pipeline);

            if self.wall_index_count > 0 {
                render_pass.set_bind_group(0, &self.wall_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.wall_vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.wall_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.wall_index_count, 0, 0..1);
            }

            if self.flat_index_count > 0 {
                render_pass.set_bind_group(0, &self.flat_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.flat_vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.flat_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.flat_index_count, 0, 0..1);
            }

            if self.sprite_index_count > 0 {
                render_pass.set_pipeline(&self.sprite_pipeline);
                render_pass.set_bind_group(0, &self.sprite_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.sprite_vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.sprite_index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                render_pass.draw_indexed(0..self.sprite_index_count, 0, 0..1);
            }
        }

        Ok(context.into_sub_graph_commands())
    }
}

fn create_empty_r16_texture(
    device: &wgpu::Device,
    width: usize,
    height: usize,
    label: &str,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R16Uint,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn create_empty_r8_texture(
    device: &wgpu::Device,
    width: usize,
    height: usize,
    label: &str,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Uint,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn create_empty_rgba_texture(
    device: &wgpu::Device,
    width: usize,
    height: usize,
    label: &str,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn upload_r16_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: usize,
    height: usize,
    pixels: &[u16],
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(pixels),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some((width * 2) as u32),
            rows_per_image: Some(height as u32),
        },
        wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
    );
}

fn upload_r8_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: usize,
    height: usize,
    pixels: &[u8],
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width as u32),
            rows_per_image: Some(height as u32),
        },
        wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
    );
}

fn upload_rgba_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: usize,
    height: usize,
    pixels: &[u8],
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some((width * 4) as u32),
            rows_per_image: Some(height as u32),
        },
        wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
    );
}
