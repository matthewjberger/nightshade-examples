use nightshade::ecs::mesh::{
    create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_plane_mesh,
    create_sphere_mesh, create_torus_mesh,
};
use nightshade::prelude::*;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl ShaderVertex {
    pub const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<ShaderVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 12,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 24,
                shader_location: 2,
            },
        ],
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Cube,
    Sphere,
    Plane,
    Cylinder,
    Cone,
    Torus,
    Custom,
}

impl PrimitiveType {
    pub const ALL: &[PrimitiveType] = &[
        PrimitiveType::Cube,
        PrimitiveType::Sphere,
        PrimitiveType::Plane,
        PrimitiveType::Cylinder,
        PrimitiveType::Cone,
        PrimitiveType::Torus,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            PrimitiveType::Cube => "Cube",
            PrimitiveType::Sphere => "Sphere",
            PrimitiveType::Plane => "Plane",
            PrimitiveType::Cylinder => "Cylinder",
            PrimitiveType::Cone => "Cone",
            PrimitiveType::Torus => "Torus",
            PrimitiveType::Custom => "Custom",
        }
    }
}

#[derive(Clone)]
pub struct MeshData {
    pub vertices: Vec<ShaderVertex>,
    pub indices: Vec<u32>,
}

pub fn generate_primitive(primitive_type: PrimitiveType) -> MeshData {
    let mesh = match primitive_type {
        PrimitiveType::Cube => create_cube_mesh(),
        PrimitiveType::Sphere => create_sphere_mesh(1.0, 32),
        PrimitiveType::Plane => create_plane_mesh(2.0),
        PrimitiveType::Cylinder => create_cylinder_mesh(0.5, 2.0, 32),
        PrimitiveType::Cone => create_cone_mesh(0.5, 2.0, 32),
        PrimitiveType::Torus => create_torus_mesh(0.7, 0.3, 32, 16),
        PrimitiveType::Custom => {
            return MeshData {
                vertices: Vec::new(),
                indices: Vec::new(),
            };
        }
    };

    let vertices: Vec<ShaderVertex> = mesh
        .vertices
        .iter()
        .map(|vertex| ShaderVertex {
            position: vertex.position,
            normal: vertex.normal,
            uv: vertex.tex_coords,
        })
        .collect();

    MeshData {
        vertices,
        indices: mesh.indices,
    }
}

pub fn create_gpu_buffers(
    device: &wgpu::Device,
    mesh_data: &MeshData,
) -> (wgpu::Buffer, wgpu::Buffer, u32) {
    use wgpu::util::DeviceExt;

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Shader Studio Vertex Buffer"),
        contents: bytemuck::cast_slice(&mesh_data.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Shader Studio Index Buffer"),
        contents: bytemuck::cast_slice(&mesh_data.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let index_count = mesh_data.indices.len() as u32;

    (vertex_buffer, index_buffer, index_count)
}
