use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

const TEXTURE_WIDTH: u32 = 256;
const TEXTURE_HEIGHT: u32 = 128;
const BILLBOARD_COUNT: usize = 4;

pub const SCREEN_MATERIALS: [&str; BILLBOARD_COUNT] = [
    "BillboardScreen0",
    "BillboardScreen1",
    "BillboardScreen2",
    "BillboardScreen3",
];

struct StoredTexture {
    texture: wgpu::Texture,
    name: String,
}

pub struct BillboardTextures {
    textures: Vec<StoredTexture>,
    initialized: bool,
}

impl BillboardTextures {
    pub fn new() -> Self {
        Self {
            textures: Vec::new(),
            initialized: false,
        }
    }

    pub fn initialize(&mut self, renderer: &mut dyn Render, main_world: &mut World) {
        if self.initialized {
            return;
        }
        self.initialized = true;

        let scenes: [SceneConfig; BILLBOARD_COUNT] = [
            SceneConfig {
                clear_color: [0.01, 0.0, 0.03, 1.0],
                center_color: [1.0, 0.8, 0.2, 1.0],
                center_emissive: [3.0, 2.0, 0.5],
                orbit_colors: &[
                    ([1.0, 0.1, 0.1, 1.0], [4.0, 0.2, 0.1]),
                    ([0.1, 0.3, 1.0, 1.0], [0.3, 0.5, 4.0]),
                    ([1.0, 0.9, 0.1, 1.0], [4.0, 3.5, 0.3]),
                    ([0.1, 0.9, 1.0, 1.0], [0.3, 3.5, 4.0]),
                    ([1.0, 0.1, 0.8, 1.0], [4.0, 0.2, 3.0]),
                ],
                orbit_radius: 3.0,
                center_shape: "Sphere",
                orbit_shape: "Cube",
            },
            SceneConfig {
                clear_color: [0.0, 0.02, 0.04, 1.0],
                center_color: [0.2, 0.5, 1.0, 1.0],
                center_emissive: [0.5, 1.5, 4.0],
                orbit_colors: &[
                    ([0.1, 1.0, 0.5, 1.0], [0.2, 4.0, 1.0]),
                    ([1.0, 0.3, 0.1, 1.0], [4.0, 0.5, 0.2]),
                    ([0.8, 0.1, 1.0, 1.0], [3.0, 0.2, 4.0]),
                ],
                orbit_radius: 2.5,
                center_shape: "Cube",
                orbit_shape: "Sphere",
            },
            SceneConfig {
                clear_color: [0.03, 0.0, 0.01, 1.0],
                center_color: [1.0, 0.3, 0.1, 1.0],
                center_emissive: [4.0, 1.0, 0.2],
                orbit_colors: &[
                    ([1.0, 0.6, 0.1, 1.0], [4.0, 2.0, 0.2]),
                    ([1.0, 0.2, 0.5, 1.0], [4.0, 0.3, 1.5]),
                    ([0.2, 1.0, 0.2, 1.0], [0.3, 4.0, 0.3]),
                    ([0.9, 0.9, 0.1, 1.0], [3.5, 3.5, 0.2]),
                ],
                orbit_radius: 3.5,
                center_shape: "Cylinder",
                orbit_shape: "Cone",
            },
            SceneConfig {
                clear_color: [0.02, 0.02, 0.0, 1.0],
                center_color: [0.1, 1.0, 0.8, 1.0],
                center_emissive: [0.3, 4.0, 3.0],
                orbit_colors: &[
                    ([1.0, 1.0, 0.2, 1.0], [4.0, 4.0, 0.3]),
                    ([0.3, 0.3, 1.0, 1.0], [0.5, 0.5, 4.0]),
                    ([1.0, 0.4, 0.7, 1.0], [4.0, 1.0, 2.5]),
                    ([0.1, 0.8, 0.3, 1.0], [0.2, 3.0, 0.5]),
                    ([0.7, 0.2, 1.0, 1.0], [2.5, 0.3, 4.0]),
                    ([1.0, 0.7, 0.1, 1.0], [4.0, 2.5, 0.2]),
                ],
                orbit_radius: 3.0,
                center_shape: "Sphere",
                orbit_shape: "Cube",
            },
        ];

        for (scene_index, config) in scenes.iter().enumerate() {
            let texture_name = format!("billboard_tex_{scene_index}");

            let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Billboard Texture {scene_index}")),
                size: wgpu::Extent3d {
                    width: TEXTURE_WIDTH,
                    height: TEXTURE_HEIGHT,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: renderer.surface_format(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mut world = World::default();
            renderer.copy_fonts_to_world(&mut world);
            world.resources.world_id = 3000 + scene_index as u64;
            world.resources.graphics.atmosphere = Atmosphere::Space;
            world.resources.graphics.show_grid = false;
            world.resources.graphics.clear_color = config.clear_color;
            world.resources.graphics.bloom_enabled = false;
            world.resources.window.timing = main_world.resources.window.timing.clone();

            capture_procedural_atmosphere_ibl(&mut world, Atmosphere::Space, 0.0);

            let camera = spawn_pan_orbit_camera(
                &mut world,
                Vec3::new(0.0, 0.5, 0.0),
                8.0,
                0.0,
                0.2,
                "Billboard Camera".to_string(),
            );
            world.resources.active_camera = Some(camera);

            let sun = spawn_sun_without_shadows(&mut world);
            if let Some(light) = world.core.get_light_mut(sun) {
                light.intensity = 0.3;
            }

            let center_mat_name = format!("_bill_center_{scene_index}");
            material_registry_insert(
                &mut world.resources.material_registry,
                center_mat_name.clone(),
                Material {
                    base_color: config.center_color,
                    emissive_factor: config.center_emissive,
                    emissive_strength: 3.0,
                    unlit: true,
                    ..Default::default()
                },
            );

            let center = spawn_mesh(
                &mut world,
                config.center_shape,
                Vec3::zeros(),
                Vec3::new(0.8, 0.8, 0.8),
            );
            world.core.set_material_ref(center, MaterialRef::new(center_mat_name));

            for (orbit_index, &(base_color, emissive_factor)) in
                config.orbit_colors.iter().enumerate()
            {
                let mat_name = format!("_bill_orbit_{scene_index}_{orbit_index}");
                material_registry_insert(
                    &mut world.resources.material_registry,
                    mat_name.clone(),
                    Material {
                        base_color,
                        emissive_factor,
                        emissive_strength: 2.0,
                        unlit: true,
                        ..Default::default()
                    },
                );

                let angle =
                    (orbit_index as f32 / config.orbit_colors.len() as f32) * std::f32::consts::TAU;
                let position = Vec3::new(
                    angle.cos() * config.orbit_radius,
                    0.0,
                    angle.sin() * config.orbit_radius,
                );
                let entity = spawn_mesh(
                    &mut world,
                    config.orbit_shape,
                    position,
                    Vec3::new(0.5, 0.5, 0.5),
                );
                world.core.set_material_ref(entity, MaterialRef::new(mat_name));
            }

            update_global_transforms_system(&mut world);
            let _ = renderer.render_world_to_texture(
                &mut world,
                None,
                &texture_view,
                TEXTURE_WIDTH,
                TEXTURE_HEIGHT,
            );

            self.textures.push(StoredTexture {
                texture,
                name: texture_name,
            });
        }
    }

    pub fn register_textures(&self, renderer: &mut dyn Render) {
        for stored in &self.textures {
            let view = stored
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            renderer.register_render_texture(&stored.name, view);
        }
    }

    pub fn reset(&mut self) {
        self.textures.clear();
        self.initialized = false;
    }
}

pub fn register_screen_materials(world: &mut World) {
    for (index, material_name) in SCREEN_MATERIALS.iter().enumerate() {
        let texture_name = format!("billboard_tex_{index}");
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.to_string(),
            Material {
                base_color: [0.02, 0.02, 0.02, 1.0],
                emissive_factor: [1.0, 1.0, 1.0],
                emissive_texture: Some(texture_name),
                emissive_strength: 2.0,
                unlit: true,
                double_sided: true,
                ..Default::default()
            },
        );
    }
}

struct SceneConfig {
    clear_color: [f32; 4],
    center_color: [f32; 4],
    center_emissive: [f32; 3],
    orbit_colors: &'static [([f32; 4], [f32; 3])],
    orbit_radius: f32,
    center_shape: &'static str,
    orbit_shape: &'static str,
}
