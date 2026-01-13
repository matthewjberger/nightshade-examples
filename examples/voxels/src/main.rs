use nightshade::ecs::camera::commands::spawn_camera;
use nightshade::ecs::camera::systems::fly_camera_system;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::mesh::{Mesh, Vertex};
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::prelude::*;
use noise::{NoiseFn, Perlin};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(VoxelWorld::default())?;
    Ok(())
}

freecs::ecs! {
    VoxelWorld {
        terrain_marker: TerrainMarker => TERRAIN_MARKER,
    }
    VoxelResources {
        terrain_size: usize,
        noise: Perlin,
        noise_scale: f64,
        noise_octaves: usize,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TerrainMarker;

impl State for VoxelWorld {
    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.user_interface.enabled = true;

        #[cfg(feature = "openxr")]
        {
            world.resources.xr.locomotion_speed = 20.0;
        }

        self.resources.terrain_size = 256;
        self.resources.noise = Perlin::new(42);
        self.resources.noise_scale = 0.02;
        self.resources.noise_octaves = 4;

        let camera_position = Vec3::new(128.0, 120.0, 128.0);
        let camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(camera);

        if let Some(transform) = world.get_local_transform_mut(camera) {
            let look_at = Vec3::new(100.0, 40.0, 100.0);
            let direction = (look_at - camera_position).normalize();
            let pitch = direction.y.asin();
            let yaw = direction.z.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

            transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
                * nalgebra_glm::quat_angle_axis(pitch, &Vec3::x());
        }

        spawn_sun(world);

        let sun_entity = world.spawn_entities(LIGHT | LOCAL_TRANSFORM, 1)[0];
        world.set_light(
            sun_entity,
            Light {
                color: Vec3::new(1.0, 0.95, 0.9),
                intensity: 3.0,
                light_type: LightType::Directional,
                range: 0.0,
                inner_cone_angle: 0.0,
                outer_cone_angle: 0.0,
                cast_shadows: false,
                shadow_bias: 0.007,
            },
        );
        world.set_local_transform(
            sun_entity,
            LocalTransform {
                translation: Vec3::zeros(),
                rotation: nalgebra_glm::quat_angle_axis(
                    -0.5,
                    &Vec3::new(1.0, 0.3, 0.0).normalize(),
                ),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );

        generate_terrain(world, self);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        fly_camera_system(world);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Voxel World")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.heading("Voxel Terrain");
                ui.separator();

                ui.label(format!(
                    "Terrain size: {}x{}",
                    self.resources.terrain_size, self.resources.terrain_size
                ));
                ui.label(format!("Noise scale: {:.3}", self.resources.noise_scale));
                ui.label(format!("Octaves: {}", self.resources.noise_octaves));

                ui.separator();

                if let Some(camera_entity) = world.resources.active_camera
                    && let Some(transform) = world.get_local_transform(camera_entity)
                {
                    ui.label(format!(
                        "Camera: ({:.1}, {:.1}, {:.1})",
                        transform.translation.x, transform.translation.y, transform.translation.z
                    ));
                }

                ui.separator();
                ui.label("Terrain Layers:");
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(26, 128, 217), "●");
                    ui.label("Water (< 0)");
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(242, 217, 140), "●");
                    ui.label("Sand (0-8)");
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(51, 204, 51), "●");
                    ui.label("Grass (8-20)");
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(153, 102, 51), "●");
                    ui.label("Dirt (20-30)");
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(128, 128, 140), "●");
                    ui.label("Stone (30-45)");
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 255, 255), "●");
                    ui.label("Snow (45+)");
                });

                ui.separator();
                ui.label("Controls:");
                ui.label("WASD - Move");
                ui.label("Space/Shift - Up/Down");
                ui.label("Right Mouse - Look");
                ui.label("Ctrl - Fast");
            });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VoxelType {
    Air = 0,
    Water = 1,
    Sand = 2,
    Grass = 3,
    Dirt = 4,
    Stone = 5,
    Snow = 6,
}

impl VoxelType {
    fn color(self) -> [f32; 4] {
        match self {
            VoxelType::Air => [0.0, 0.0, 0.0, 0.0],
            VoxelType::Water => [0.1, 0.5, 0.85, 0.8],
            VoxelType::Sand => [0.95, 0.85, 0.55, 1.0],
            VoxelType::Grass => [0.2, 0.8, 0.2, 1.0],
            VoxelType::Dirt => [0.6, 0.4, 0.2, 1.0],
            VoxelType::Stone => [0.5, 0.5, 0.55, 1.0],
            VoxelType::Snow => [1.0, 1.0, 1.0, 1.0],
        }
    }

    fn from_height(world_y: i32) -> Self {
        match world_y {
            y if y < 0 => VoxelType::Water,
            y if y < 8 => VoxelType::Sand,
            y if y < 20 => VoxelType::Grass,
            y if y < 30 => VoxelType::Dirt,
            y if y < 45 => VoxelType::Stone,
            _ => VoxelType::Snow,
        }
    }
}

fn generate_terrain(world: &mut World, voxel_world: &mut VoxelWorld) {
    let terrain_size = voxel_world.resources.terrain_size;
    let noise = &voxel_world.resources.noise;
    let noise_scale = voxel_world.resources.noise_scale;
    let noise_octaves = voxel_world.resources.noise_octaves;

    use std::collections::HashMap;
    let mut meshes_by_type: HashMap<VoxelType, (Vec<Vertex>, Vec<u32>)> = HashMap::new();

    tracing::info!("Generating {}x{} terrain...", terrain_size, terrain_size);

    let mut height_map = vec![0i32; terrain_size * terrain_size];
    for z in 0..terrain_size {
        for x in 0..terrain_size {
            let height =
                sample_terrain_height(noise, x as f64, z as f64, noise_scale, noise_octaves);
            height_map[x + z * terrain_size] = height;
        }
    }

    for z in 0..terrain_size {
        for x in 0..terrain_size {
            let height = height_map[x + z * terrain_size];

            let height_left = if x > 0 {
                height_map[(x - 1) + z * terrain_size]
            } else {
                -1
            };
            let height_right = if x < terrain_size - 1 {
                height_map[(x + 1) + z * terrain_size]
            } else {
                -1
            };
            let height_back = if z > 0 {
                height_map[x + (z - 1) * terrain_size]
            } else {
                -1
            };
            let height_front = if z < terrain_size - 1 {
                height_map[x + (z + 1) * terrain_size]
            } else {
                -1
            };

            let min_y = -50;
            for y in min_y..=height {
                let voxel_type = VoxelType::from_height(y);
                let pos = Vec3::new(x as f32, y as f32, z as f32);

                let (vertices, indices) = meshes_by_type
                    .entry(voxel_type)
                    .or_insert((Vec::new(), Vec::new()));

                add_cube_faces(
                    vertices,
                    indices,
                    VoxelFaceParams {
                        pos,
                        y,
                        height,
                        height_left,
                        height_right,
                        height_back,
                        height_front,
                        min_y,
                        voxel_type,
                    },
                );
            }
        }

        if z % 100 == 0 {
            tracing::info!("Progress: {}%", (z * 100) / terrain_size);
        }
    }

    tracing::info!("Creating meshes for {} voxel types", meshes_by_type.len());

    for (voxel_type, (vertices, indices)) in meshes_by_type {
        if vertices.is_empty() {
            continue;
        }

        let mesh_name = format!("terrain_{:?}", voxel_type);
        let mesh = Mesh {
            vertices,
            indices,
            bounding_volume: None,
            skin_data: None,
            morph_targets: None,
        };

        mesh_cache_insert(&mut world.resources.mesh_cache, mesh_name.clone(), mesh);

        let entity = world.spawn_entities(
            RENDER_MESH
                | LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | VISIBILITY
                | MATERIAL_REF,
            1,
        )[0];

        world.set_render_mesh(entity, RenderMesh::new(mesh_name));

        let color = voxel_type.color();
        let voxel_material = format!("Voxel_{}", entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            voxel_material.clone(),
            Material {
                base_color: color,
                metallic: 0.0,
                roughness: 0.9,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&voxel_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(entity, MaterialRef::new(voxel_material));

        world.set_local_transform(
            entity,
            LocalTransform {
                translation: Vec3::zeros(),
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );

        world.set_visibility(entity, Visibility { visible: true });
    }

    tracing::info!("Terrain meshes created");
}

struct VoxelFaceParams {
    pos: Vec3,
    y: i32,
    height: i32,
    height_left: i32,
    height_right: i32,
    height_back: i32,
    height_front: i32,
    min_y: i32,
    voxel_type: VoxelType,
}

fn add_cube_faces(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, params: VoxelFaceParams) {
    let voxel_above_type = if params.y < params.height {
        VoxelType::from_height(params.y + 1)
    } else {
        VoxelType::Air
    };
    let voxel_below_type = if params.y > params.min_y {
        VoxelType::from_height(params.y - 1)
    } else {
        VoxelType::Air
    };

    if params.y == params.height || voxel_above_type != params.voxel_type {
        let base_index = vertices.len() as u32;
        vertices.push(Vertex {
            position: [params.pos.x, params.pos.y + 1.0, params.pos.z + 1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x + 1.0, params.pos.y + 1.0, params.pos.z + 1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x + 1.0, params.pos.y + 1.0, params.pos.z],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x, params.pos.y + 1.0, params.pos.z],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    if params.y == params.min_y || voxel_below_type != params.voxel_type {
        let base_index = vertices.len() as u32;
        vertices.push(Vertex {
            position: [params.pos.x, params.pos.y, params.pos.z],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x + 1.0, params.pos.y, params.pos.z],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [1.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x + 1.0, params.pos.y, params.pos.z + 1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [1.0, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x, params.pos.y, params.pos.z + 1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [0.0, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    if params.y > params.height_left {
        let base_index = vertices.len() as u32;
        vertices.push(Vertex {
            position: [params.pos.x, params.pos.y, params.pos.z + 1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            tangent: [0.0, 0.0, -1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x, params.pos.y + 1.0, params.pos.z + 1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            tangent: [0.0, 0.0, -1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x, params.pos.y + 1.0, params.pos.z],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            tangent: [0.0, 0.0, -1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x, params.pos.y, params.pos.z],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            tangent: [0.0, 0.0, -1.0, 1.0],
        });
        indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    if params.y > params.height_right {
        let base_index = vertices.len() as u32;
        vertices.push(Vertex {
            position: [params.pos.x + 1.0, params.pos.y, params.pos.z],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            tangent: [0.0, 0.0, 1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x + 1.0, params.pos.y + 1.0, params.pos.z],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            tangent: [0.0, 0.0, 1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x + 1.0, params.pos.y + 1.0, params.pos.z + 1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            tangent: [0.0, 0.0, 1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x + 1.0, params.pos.y, params.pos.z + 1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            tangent: [0.0, 0.0, 1.0, 1.0],
        });
        indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    if params.y > params.height_front {
        let base_index = vertices.len() as u32;
        vertices.push(Vertex {
            position: [params.pos.x + 1.0, params.pos.y, params.pos.z + 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 0.0],
            tangent: [-1.0, 0.0, 0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x + 1.0, params.pos.y + 1.0, params.pos.z + 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
            tangent: [-1.0, 0.0, 0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x, params.pos.y + 1.0, params.pos.z + 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 1.0],
            tangent: [-1.0, 0.0, 0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x, params.pos.y, params.pos.z + 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            tangent: [-1.0, 0.0, 0.0, 1.0],
        });
        indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    if params.y > params.height_back {
        let base_index = vertices.len() as u32;
        vertices.push(Vertex {
            position: [params.pos.x, params.pos.y, params.pos.z],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x, params.pos.y + 1.0, params.pos.z],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x + 1.0, params.pos.y + 1.0, params.pos.z],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [1.0, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [params.pos.x + 1.0, params.pos.y, params.pos.z],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [1.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }
}

fn sample_terrain_height(
    noise: &Perlin,
    world_x: f64,
    world_z: f64,
    scale: f64,
    octaves: usize,
) -> i32 {
    let mut height = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        let sample_x = world_x * scale * frequency;
        let sample_z = world_z * scale * frequency;

        let noise_value = noise.get([sample_x, sample_z]);
        height += noise_value * amplitude;

        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    height /= max_value;

    let base_height = 10.0;
    let mountain_height = 50.0;

    (base_height + height * mountain_height) as i32
}
