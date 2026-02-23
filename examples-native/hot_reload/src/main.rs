use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::texture_loader::{
    AssetLoadingState, AssetLoadingStatus, SharedTextureQueue, create_shared_queue,
    process_and_load_textures, queue_texture_from_path,
};
use nightshade::filesystem::open_directory;
use nightshade::prelude::*;
use nightshade::render::wgpu::texture_cache::texture_cache_add_reference;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(HotReloadDemo::default())?;
    Ok(())
}

struct HotReloadDemo {
    texture_queue: SharedTextureQueue,
    loading_state: AssetLoadingState,
    loaded: bool,
    texture_path: PathBuf,
    material_path: PathBuf,
    texture_reload_count: u32,
    material_reload_count: u32,
    last_texture_reload_time: Option<u64>,
    last_material_reload_time: Option<u64>,
    last_texture_modified: Option<std::time::SystemTime>,
    last_material_modified: Option<std::time::SystemTime>,
}

impl Default for HotReloadDemo {
    fn default() -> Self {
        Self {
            texture_queue: create_shared_queue(),
            loading_state: AssetLoadingState::new(1),
            loaded: false,
            texture_path: PathBuf::new(),
            material_path: PathBuf::new(),
            texture_reload_count: 0,
            material_reload_count: 0,
            last_texture_reload_time: None,
            last_material_reload_time: None,
            last_texture_modified: None,
            last_material_modified: None,
        }
    }
}

const TEXTURE_NAME: &str = "hot_reload_test.png";
const MATERIAL_NAME: &str = "hot_reload_material";
const MATERIAL_FILE_NAME: &str = "hot_reload_material.json";

fn generate_checkerboard(size: u32, tile_size: u32, color_a: [u8; 4], color_b: [u8; 4]) -> Vec<u8> {
    let mut data = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let tile_x = x / tile_size;
            let tile_y = y / tile_size;
            let color = if (tile_x + tile_y).is_multiple_of(2) {
                color_a
            } else {
                color_b
            };
            let offset = ((y * size + x) * 4) as usize;
            data[offset..offset + 4].copy_from_slice(&color);
        }
    }
    data
}

fn default_material() -> Material {
    Material {
        base_color: [1.0, 1.0, 1.0, 1.0],
        roughness: 0.5,
        metallic: 0.0,
        ..Default::default()
    }
}

fn write_material_json(path: &std::path::Path, material: &Material) {
    if let Ok(json) = serde_json::to_string_pretty(material) {
        let _ = std::fs::write(path, json);
    }
}

impl State for HotReloadDemo {
    fn title(&self) -> &str {
        "Hot Reload"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Nebula;

        load_procedural_textures(world);
        capture_procedural_atmosphere_ibl(world, Atmosphere::Nebula, 0.0);
        spawn_sun(world);

        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 1.5, 0.0),
            8.0,
            0.0,
            0.3,
            "Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);

        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let texture_path = exe_dir.join(TEXTURE_NAME);
        let material_path = exe_dir.join(MATERIAL_FILE_NAME);

        let size = 256u32;
        let rgba = generate_checkerboard(
            size,
            32,
            [60, 120, 220, 255],
            [240, 240, 240, 255],
        );
        let img = image::RgbaImage::from_raw(size, size, rgba)
            .expect("failed to create image buffer");
        img.save(&texture_path)
            .expect("failed to write test texture");

        let material = default_material();
        write_material_json(&material_path, &material);

        tracing::info!("Wrote test texture to: {}", texture_path.display());
        tracing::info!("Wrote material JSON to: {}", material_path.display());

        self.texture_path = texture_path.clone();
        self.material_path = material_path;

        let path_str = texture_path.to_string_lossy().replace('\\', "/");
        queue_texture_from_path(&self.texture_queue, &path_str);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        if !self.loaded {
            let status = process_and_load_textures(
                &self.texture_queue,
                world,
                &mut self.loading_state,
                4,
            );
            if status == AssetLoadingStatus::Complete {
                self.loaded = true;
                tracing::info!("Initial texture loaded, spawning meshes");
                self.last_texture_modified = std::fs::metadata(&self.texture_path)
                    .ok()
                    .and_then(|m| m.modified().ok());
                self.last_material_modified = std::fs::metadata(&self.material_path)
                    .ok()
                    .and_then(|m| m.modified().ok());

                let texture_name = self.texture_path.to_string_lossy().replace('\\', "/");
                spawn_demo_meshes(world, &texture_name, &self.material_path);

                world.resources.asset_watcher.track_texture(
                    texture_name,
                    self.texture_path.clone(),
                );
                world.resources.asset_watcher.track_material(
                    MATERIAL_NAME.to_string(),
                    self.material_path.clone(),
                );
            }
        }

        if self.loaded {
            let current_tex_modified = std::fs::metadata(&self.texture_path)
                .ok()
                .and_then(|m| m.modified().ok());
            if current_tex_modified != self.last_texture_modified {
                self.last_texture_modified = current_tex_modified;
                self.texture_reload_count += 1;
                self.last_texture_reload_time =
                    Some(world.resources.window.timing.uptime_milliseconds);
            }

            let current_mat_modified = std::fs::metadata(&self.material_path)
                .ok()
                .and_then(|m| m.modified().ok());
            if current_mat_modified != self.last_material_modified {
                self.last_material_modified = current_mat_modified;
                self.material_reload_count += 1;
                self.last_material_reload_time =
                    Some(world.resources.window.timing.uptime_milliseconds);
            }
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Hot Reload Test")
            .default_pos([10.0, 10.0])
            .default_width(380.0)
            .show(ui_context, |ui| {
                ui.heading("Texture Hot-Reload");
                ui.separator();

                ui.label("Test texture path:");
                let tex_path_str = self.texture_path.to_string_lossy();
                ui.monospace(tex_path_str.as_ref());

                ui.horizontal(|ui| {
                    if ui.button("Copy path").clicked() {
                        ui.ctx().copy_text(tex_path_str.to_string());
                    }
                    if ui.button("Open directory").clicked() {
                        open_directory(&self.texture_path);
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Status:");
                    if self.loaded {
                        ui.colored_label(egui::Color32::GREEN, "Watching for changes");
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "Loading...");
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Texture reloads:");
                    ui.monospace(format!("{}", self.texture_reload_count));
                });

                if let Some(reload_time) = self.last_texture_reload_time {
                    let elapsed_ms = world
                        .resources
                        .window
                        .timing
                        .uptime_milliseconds
                        .saturating_sub(reload_time);
                    let elapsed_secs = elapsed_ms as f64 / 1000.0;
                    ui.horizontal(|ui| {
                        ui.label("Last texture reload:");
                        ui.monospace(format!("{elapsed_secs:.1}s ago"));
                    });
                }

                ui.separator();

                if ui.button("Regenerate as red/white checkerboard").clicked() {
                    let rgba = generate_checkerboard(
                        256,
                        32,
                        [220, 50, 50, 255],
                        [255, 255, 255, 255],
                    );
                    let img = image::RgbaImage::from_raw(256, 256, rgba).unwrap();
                    let _ = img.save(&self.texture_path);
                }

                if ui.button("Regenerate as green/black checkerboard").clicked() {
                    let rgba = generate_checkerboard(
                        256,
                        16,
                        [30, 200, 60, 255],
                        [20, 20, 20, 255],
                    );
                    let img = image::RgbaImage::from_raw(256, 256, rgba).unwrap();
                    let _ = img.save(&self.texture_path);
                }

                if ui.button("Regenerate as gradient").clicked() {
                    let size = 256u32;
                    let mut data = vec![0u8; (size * size * 4) as usize];
                    for y in 0..size {
                        for x in 0..size {
                            let offset = ((y * size + x) * 4) as usize;
                            data[offset] = (x * 255 / size) as u8;
                            data[offset + 1] = (y * 255 / size) as u8;
                            data[offset + 2] = 128;
                            data[offset + 3] = 255;
                        }
                    }
                    let img = image::RgbaImage::from_raw(size, size, data).unwrap();
                    let _ = img.save(&self.texture_path);
                }

                ui.add_space(16.0);
                ui.heading("Material Hot-Reload");
                ui.separator();

                ui.label("Material JSON path:");
                let mat_path_str = self.material_path.to_string_lossy();
                ui.monospace(mat_path_str.as_ref());

                ui.horizontal(|ui| {
                    if ui.button("Copy path").clicked() {
                        ui.ctx().copy_text(mat_path_str.to_string());
                    }
                    if ui.button("Open directory").clicked() {
                        open_directory(&self.material_path);
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Material reloads:");
                    ui.monospace(format!("{}", self.material_reload_count));
                });

                if let Some(reload_time) = self.last_material_reload_time {
                    let elapsed_ms = world
                        .resources
                        .window
                        .timing
                        .uptime_milliseconds
                        .saturating_sub(reload_time);
                    let elapsed_secs = elapsed_ms as f64 / 1000.0;
                    ui.horizontal(|ui| {
                        ui.label("Last material reload:");
                        ui.monospace(format!("{elapsed_secs:.1}s ago"));
                    });
                }

                ui.separator();
                ui.label("Quick material presets:");

                if ui.button("Make metallic (gold)").clicked() {
                    let material = Material {
                        base_color: [1.0, 0.84, 0.0, 1.0],
                        roughness: 0.2,
                        metallic: 1.0,
                        ..Default::default()
                    };
                    write_material_json(&self.material_path, &material);
                }

                if ui.button("Make rough (matte red)").clicked() {
                    let material = Material {
                        base_color: [0.8, 0.15, 0.15, 1.0],
                        roughness: 0.95,
                        metallic: 0.0,
                        ..Default::default()
                    };
                    write_material_json(&self.material_path, &material);
                }

                if ui.button("Make emissive (neon green)").clicked() {
                    let material = Material {
                        base_color: [0.1, 0.1, 0.1, 1.0],
                        emissive_factor: [0.0, 5.0, 0.0],
                        roughness: 0.5,
                        metallic: 0.0,
                        emissive_strength: 2.0,
                        ..Default::default()
                    };
                    write_material_json(&self.material_path, &material);
                }

                if ui.button("Make glossy (chrome)").clicked() {
                    let material = Material {
                        base_color: [0.9, 0.9, 0.95, 1.0],
                        roughness: 0.05,
                        metallic: 1.0,
                        ..Default::default()
                    };
                    write_material_json(&self.material_path, &material);
                }

                if ui.button("Reset to default").clicked() {
                    write_material_json(&self.material_path, &default_material());
                }
            });
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::KeyQ, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }
    }
}

fn spawn_demo_meshes(world: &mut World, texture_name: &str, material_path: &std::path::Path) {
    let material = match std::fs::read_to_string(material_path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_else(|_| default_material()),
        Err(_) => default_material(),
    };

    material_registry_insert(
        &mut world.resources.material_registry,
        MATERIAL_NAME.to_string(),
        material,
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(MATERIAL_NAME)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }

    spawn_material_mesh(world, "Torus", Vec3::new(-3.0, 1.5, 0.0), "Torus");
    spawn_material_mesh(world, "Sphere", Vec3::new(0.0, 1.5, 0.0), "Sphere");
    spawn_material_mesh(world, "Cone", Vec3::new(3.0, 1.5, 0.0), "Cone");

    spawn_textured_mesh(world, "Cube", texture_name, Vec3::new(-3.0, 1.5, -3.0), "Cube");
    spawn_textured_mesh(world, "Sphere", texture_name, Vec3::new(0.0, 1.5, -3.0), "Sphere (Textured)");
    spawn_textured_mesh(world, "Cylinder", texture_name, Vec3::new(3.0, 1.5, -3.0), "Cylinder");
}

fn spawn_material_mesh(
    world: &mut World,
    mesh_name: &str,
    position: Vec3,
    label: &str,
) {
    let entity = world.spawn_entities(
        RENDER_MESH
            | MATERIAL_REF
            | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | BOUNDING_VOLUME
            | NAME
            | VISIBILITY,
        1,
    )[0];

    world.set_render_mesh(entity, RenderMesh::new(mesh_name));

    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(MATERIAL_NAME)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(entity, MaterialRef::new(MATERIAL_NAME));

    if let Some(transform) = world.get_local_transform_mut(entity) {
        transform.translation = position;
    }

    if let Some(bounding_volume) = world.get_bounding_volume_mut(entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type(mesh_name);
    }

    if let Some(name) = world.get_name_mut(entity) {
        *name = Name(format!("Hot Reload {label}"));
    }

    world.resources.mesh_render_state.mark_entity_added(entity);
}

fn spawn_textured_mesh(
    world: &mut World,
    mesh_name: &str,
    texture_name: &str,
    position: Vec3,
    label: &str,
) {
    let entity = world.spawn_entities(
        RENDER_MESH
            | MATERIAL_REF
            | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | BOUNDING_VOLUME
            | NAME
            | VISIBILITY,
        1,
    )[0];

    world.set_render_mesh(entity, RenderMesh::new(mesh_name));

    let material_name = format!("HotReload_{}_{}", label, entity.id);
    texture_cache_add_reference(&mut world.resources.texture_cache, texture_name);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(texture_name.to_string()),
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(transform) = world.get_local_transform_mut(entity) {
        transform.translation = position;
    }

    if let Some(bounding_volume) = world.get_bounding_volume_mut(entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type(mesh_name);
    }

    if let Some(name) = world.get_name_mut(entity) {
        *name = Name(format!("Hot Reload {label}"));
    }

    world.resources.mesh_render_state.mark_entity_added(entity);
}

