#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DoomVertex {
    pub position: [f32; 3],
    pub atlas_uv: [f32; 2],
    pub tile_uv: [f32; 2],
    pub tile_size: [f32; 2],
    pub light: f32,
    pub num_frames: f32,
    pub scroll_rate: f32,
    pub row_height: f32,
}

pub struct DoomVertexParams {
    pub position: [f32; 3],
    pub atlas_uv: [f32; 2],
    pub tile_uv: [f32; 2],
    pub tile_size: [f32; 2],
    pub light: f32,
    pub num_frames: u32,
    pub scroll_rate: f32,
    pub row_height: f32,
}

impl DoomVertex {
    pub fn new(params: DoomVertexParams) -> Self {
        Self {
            position: params.position,
            atlas_uv: params.atlas_uv,
            tile_uv: params.tile_uv,
            tile_size: params.tile_size,
            light: params.light,
            num_frames: params.num_frames as f32,
            scroll_rate: params.scroll_rate,
            row_height: params.row_height,
        }
    }

    pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DoomVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 20,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 28,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 36,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 40,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 44,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 48,
                    shader_location: 7,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkyVertex {
    pub position: [f32; 3],
    pub _padding: f32,
}

impl SkyVertex {
    pub fn new(position: [f32; 3]) -> Self {
        Self {
            position,
            _padding: 0.0,
        }
    }

    pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SkyVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub atlas_uv: [f32; 2],
    pub tile_uv: [f32; 2],
    pub tile_size: [f32; 2],
    pub local_x: f32,
    pub light: f32,
    pub num_frames: f32,
    pub _padding: f32,
}

impl SpriteVertex {
    pub fn new(
        position: [f32; 3],
        atlas_uv: [f32; 2],
        tile_uv: [f32; 2],
        tile_size: [f32; 2],
        local_x: f32,
        light: f32,
        num_frames: u32,
    ) -> Self {
        Self {
            position,
            atlas_uv,
            tile_uv,
            tile_size,
            local_x,
            light,
            num_frames: num_frames as f32,
            _padding: 0.0,
        }
    }

    pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 20,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 28,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 36,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 40,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 44,
                    shader_location: 6,
                },
            ],
        }
    }
}
