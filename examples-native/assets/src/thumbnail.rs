use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::prefab::mesh_cache_insert;
use nightshade::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_WORLD_ID: AtomicU64 = AtomicU64::new(10000);

pub struct GpuThumbnail {
    pub _texture: wgpu::Texture,
    pub _texture_view: wgpu::TextureView,
    pub egui_texture_id: egui::TextureId,
}

pub fn generate_gpu_thumbnail(
    renderer: &mut dyn Render,
    result: nightshade::ecs::prefab::GltfLoadResult,
    size: u32,
) -> Option<GpuThumbnail> {
    if result.meshes.is_empty() {
        return None;
    }

    let (aabb_min, aabb_max) = compute_mesh_aabb(&result.meshes)?;
    let center = (aabb_min + aabb_max) * 0.5;
    let extent = aabb_max - aabb_min;
    let diagonal = nalgebra_glm::length(&extent);

    if diagonal < 1e-6 {
        return None;
    }

    let mut world = World::default();
    renderer.copy_fonts_to_world(&mut world);

    world.resources.world_id = NEXT_WORLD_ID.fetch_add(1, Ordering::Relaxed);
    world.resources.graphics.atmosphere = Atmosphere::None;
    world.resources.graphics.show_grid = false;
    world.resources.graphics.clear_color = [0.15, 0.15, 0.19, 1.0];

    for (name, mesh) in result.meshes {
        mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
    }

    if let Some(prefab) = result.prefabs.first() {
        if !result.skins.is_empty() {
            nightshade::ecs::prefab::spawn_prefab_with_skins(
                &mut world,
                prefab,
                &result.animations,
                &result.skins,
                Vec3::zeros(),
            );
        } else {
            nightshade::ecs::prefab::spawn_prefab(&mut world, prefab, Vec3::zeros());
        }
    }

    let sun_entity = spawn_sun(&mut world);
    if let Some(transform) = world.get_local_transform_mut(sun_entity) {
        transform.translation = Vec3::new(10.0, 20.0, 10.0);
    }

    let camera_entity = spawn_pan_orbit_camera(
        &mut world,
        center,
        diagonal * 1.2,
        0.5,
        0.3,
        "ThumbnailCamera".to_string(),
    );
    world.resources.active_camera = Some(camera_entity);

    nightshade::ecs::transform::systems::run_systems(&mut world);

    let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("gpu_thumbnail"),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: renderer.surface_format(),
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    renderer
        .render_world_to_texture(&mut world, None, &texture_view, size, size)
        .ok()?;

    let egui_texture_id = renderer.register_egui_texture(&texture_view)?;

    Some(GpuThumbnail {
        _texture: texture,
        _texture_view: texture_view,
        egui_texture_id,
    })
}

pub fn compute_mesh_aabb(
    meshes: &std::collections::HashMap<String, nightshade::ecs::mesh::Mesh>,
) -> Option<(Vec3, Vec3)> {
    let mut first = true;
    let mut aabb_min = Vec3::zeros();
    let mut aabb_max = Vec3::zeros();

    for mesh in meshes.values() {
        for vertex in &mesh.vertices {
            let position = Vec3::new(vertex.position[0], vertex.position[1], vertex.position[2]);
            if first {
                aabb_min = position;
                aabb_max = position;
                first = false;
            } else {
                aabb_min.x = aabb_min.x.min(position.x);
                aabb_min.y = aabb_min.y.min(position.y);
                aabb_min.z = aabb_min.z.min(position.z);
                aabb_max.x = aabb_max.x.max(position.x);
                aabb_max.y = aabb_max.y.max(position.y);
                aabb_max.z = aabb_max.z.max(position.z);
            }
        }
    }

    if first {
        None
    } else {
        Some((aabb_min, aabb_max))
    }
}
