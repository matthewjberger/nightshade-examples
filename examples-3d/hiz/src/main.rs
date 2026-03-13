mod observer;

use nightshade::prelude::*;
use std::collections::{HashMap, HashSet};

const CHUNK_SIZE: i32 = 20;
const CUBE_SPACING: f32 = 2.0;
const VIEW_DISTANCE: i32 = 5;
const CHUNK_HEIGHT: i32 = 20;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(HizDemo::default())?;
    Ok(())
}

struct HizDemo {
    chunks: HashMap<(i32, i32), Vec<Entity>>,
    total_cubes: usize,
    fps: f32,
    delta_time: f32,
    last_chunk_x: i32,
    last_chunk_z: i32,
    initialized: bool,
    main_camera: Option<Entity>,
    observer: Option<observer::ObserverCamera>,
    observer_enabled: bool,
}

impl Default for HizDemo {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            total_cubes: 0,
            fps: 0.0,
            delta_time: 0.0,
            last_chunk_x: i32::MIN,
            last_chunk_z: i32::MIN,
            initialized: false,
            main_camera: None,
            observer: None,
            observer_enabled: false,
        }
    }
}

impl State for HizDemo {
    fn title(&self) -> &str {
        "Hi-Z Occlusion Culling - Infinite Grid"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;

        let camera_position = Vec3::new(0.0, 20.0, 50.0);
        let main_camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(main_camera);
        self.main_camera = Some(main_camera);

        spawn_sun(world);
    }

    fn pre_render(&mut self, renderer: &mut dyn Render, world: &mut World) {
        if self.observer.is_none() {
            self.observer = Some(observer::ObserverCamera::new(renderer, world));
        }
        if let Some(observer) = &mut self.observer {
            observer.enabled = self.observer_enabled;
            if observer.enabled
                && let Some(main_camera) = self.main_camera
            {
                observer.render(renderer, world, main_camera);
            }
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        self.fps = world.resources.window.timing.frames_per_second;
        self.delta_time = world.resources.window.timing.delta_time;

        let saved_gamepad_id = if self.observer_enabled {
            world.resources.input.gamepad.gamepad.take()
        } else {
            None
        };

        fly_camera_system(world);

        if saved_gamepad_id.is_some() {
            world.resources.input.gamepad.gamepad = saved_gamepad_id;
        }

        if self.observer_enabled
            && let Some(observer) = &mut self.observer
        {
            observer.update(world, self.delta_time);
        }

        self.update_chunks(world);
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        self.render_ui(world, ctx);

        if self.observer_enabled
            && let Some(observer) = &self.observer
        {
            observer.draw_ui(ctx);
        }
    }
}

impl HizDemo {
    fn world_to_chunk(x: f32, z: f32) -> (i32, i32) {
        let chunk_world_size = CHUNK_SIZE as f32 * CUBE_SPACING;
        let chunk_x = (x / chunk_world_size).floor() as i32;
        let chunk_z = (z / chunk_world_size).floor() as i32;
        (chunk_x, chunk_z)
    }

    fn update_chunks(&mut self, world: &mut World) {
        let camera_pos = world
            .resources
            .active_camera
            .and_then(|cam| world.core.get_global_transform(cam))
            .map(|t| Vec3::new(t.0[(0, 3)], t.0[(1, 3)], t.0[(2, 3)]))
            .unwrap_or(Vec3::new(0.0, 0.0, 0.0));

        let (current_chunk_x, current_chunk_z) = Self::world_to_chunk(camera_pos.x, camera_pos.z);

        if current_chunk_x == self.last_chunk_x
            && current_chunk_z == self.last_chunk_z
            && self.initialized
        {
            return;
        }

        self.last_chunk_x = current_chunk_x;
        self.last_chunk_z = current_chunk_z;
        self.initialized = true;

        let mut desired_chunks = HashSet::new();
        for dx in -VIEW_DISTANCE..=VIEW_DISTANCE {
            for dz in -VIEW_DISTANCE..=VIEW_DISTANCE {
                desired_chunks.insert((current_chunk_x + dx, current_chunk_z + dz));
            }
        }

        let chunks_to_remove: Vec<(i32, i32)> = self
            .chunks
            .keys()
            .filter(|key| !desired_chunks.contains(key))
            .copied()
            .collect();

        for chunk_key in chunks_to_remove {
            if let Some(entities) = self.chunks.remove(&chunk_key) {
                for entity in entities {
                    despawn_recursive_immediate(world, entity);
                    self.total_cubes = self.total_cubes.saturating_sub(1);
                }
            }
        }

        for chunk_key in desired_chunks {
            if !self.chunks.contains_key(&chunk_key) {
                let entities = self.spawn_chunk(world, chunk_key.0, chunk_key.1);
                self.chunks.insert(chunk_key, entities);
            }
        }
    }

    fn spawn_chunk(&mut self, world: &mut World, chunk_x: i32, chunk_z: i32) -> Vec<Entity> {
        let mut entities = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT) as usize);
        let chunk_world_size = CHUNK_SIZE as f32 * CUBE_SPACING;
        let base_x = chunk_x as f32 * chunk_world_size;
        let base_z = chunk_z as f32 * chunk_world_size;

        for local_x in 0..CHUNK_SIZE {
            for local_z in 0..CHUNK_SIZE {
                for local_y in 0..CHUNK_HEIGHT {
                    let position = Vec3::new(
                        base_x + local_x as f32 * CUBE_SPACING,
                        local_y as f32 * CUBE_SPACING,
                        base_z + local_z as f32 * CUBE_SPACING,
                    );

                    let cube = spawn_cube_at(world, position);
                    world.core.remove_components(cube, CASTS_SHADOW);

                    let material_index =
                        ((chunk_x + chunk_z + local_x + local_y + local_z) % 6).abs();
                    let material = match material_index {
                        0 => "Red",
                        1 => "Green",
                        2 => "Blue",
                        3 => "Yellow",
                        4 => "Magenta",
                        _ => "Cyan",
                    };
                    world.core.set_material_ref(cube, MaterialRef::new(material));

                    entities.push(cube);
                    self.total_cubes += 1;
                }
            }
        }

        entities
    }

    fn render_ui(&mut self, _world: &mut World, ctx: &egui::Context) {
        egui::Window::new("Hi-Z Occlusion Culling - Infinite Grid")
            .default_pos([10.0, 10.0])
            .default_width(350.0)
            .show(ctx, |ui| {
                ui.heading("Scene Statistics");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Total Cubes:");
                    ui.strong(format!("{}", self.total_cubes));
                });

                ui.horizontal(|ui| {
                    ui.label("Loaded Chunks:");
                    ui.strong(format!("{}", self.chunks.len()));
                });

                ui.horizontal(|ui| {
                    ui.label("Cubes per Chunk:");
                    ui.strong(format!("{}", CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT));
                });

                ui.horizontal(|ui| {
                    ui.label("View Distance:");
                    ui.strong(format!("{} chunks", VIEW_DISTANCE));
                });

                ui.separator();
                ui.heading("Performance");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("FPS:");
                    ui.strong(format!("{:.1}", self.fps));
                });

                let frame_time_ms = self.delta_time * 1000.0;
                ui.horizontal(|ui| {
                    ui.label("Frame Time:");
                    ui.strong(format!("{:.2} ms", frame_time_ms));
                });

                ui.horizontal(|ui| {
                    ui.label("Camera Chunk:");
                    ui.strong(format!("({}, {})", self.last_chunk_x, self.last_chunk_z));
                });

                ui.separator();
                ui.checkbox(&mut self.observer_enabled, "Observer Camera");

                ui.separator();
                ui.heading("Controls");
                ui.label("WASD: Move camera");
                ui.label("Mouse: Look around (hold click)");
                ui.label("Space/Shift: Up/Down");
                if self.observer_enabled {
                    ui.label("Gamepad: Control observer camera");
                }
            });
    }
}
