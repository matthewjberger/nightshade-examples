mod ecs;
mod mesh;
mod systems;

use ecs::PuzzleWorld;
use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;
use std::collections::HashMap;
use systems::{
    drop_pieces_system, initialize_edge_types, input_system, load_puzzle_texture, render_system,
    reset_puzzle, reslice_puzzle, shuffle_pieces, snap_system, solve_system, spawn_board_outline,
    spawn_puzzle_pieces, start_solving, start_victory_celebration, toggle_board_outline,
    victory_system,
};

const PUZZLE_IMAGE: &[u8] = include_bytes!("../../../assets/images/logo.png");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(JigsawGame::default())
}

struct JigsawGame {
    puzzle_world: PuzzleWorld,
    camera_entity: Entity,
    initialized: bool,
    texture_loaded: bool,
    pieces_spawned: bool,
}

impl Default for JigsawGame {
    fn default() -> Self {
        let mut puzzle_world = PuzzleWorld::default();
        puzzle_world.resources.grid_cols = 4;
        puzzle_world.resources.grid_rows = 3;
        puzzle_world.resources.piece_width = 1.0;
        puzzle_world.resources.piece_height = 1.0;
        puzzle_world.resources.snap_threshold = 0.4;
        puzzle_world.resources.z_counter = 0;
        puzzle_world.resources.puzzle_complete = false;
        puzzle_world.resources.image_width = 0;
        puzzle_world.resources.image_height = 0;
        puzzle_world.resources.edge_types = HashMap::new();
        puzzle_world.resources.piece_outlines = HashMap::new();
        puzzle_world.resources.hovered_piece = None;
        puzzle_world.resources.show_board_outline = true;
        puzzle_world.resources.board_outline_entity = None;
        puzzle_world.resources.tab_depth = 0.20;
        puzzle_world.resources.tab_width = 0.32;
        puzzle_world.resources.neck_width = 0.14;
        puzzle_world.resources.is_solving = false;
        puzzle_world.resources.solve_queue = Vec::new();
        puzzle_world.resources.solve_progress = 0.0;
        puzzle_world.resources.pending_cols = 4;
        puzzle_world.resources.pending_rows = 3;
        puzzle_world.resources.pending_tab_depth = 0.20;
        puzzle_world.resources.pending_tab_width = 0.32;
        puzzle_world.resources.pending_neck_width = 0.14;
        puzzle_world.resources.has_pending_changes = false;
        puzzle_world.resources.victory_active = false;
        puzzle_world.resources.victory_flash_index = 0;
        puzzle_world.resources.victory_flash_timer = 0.0;
        puzzle_world.resources.victory_text_entity = None;
        puzzle_world.resources.victory_text_lifetime = 0.0;
        puzzle_world.resources.all_piece_entities = Vec::new();
        puzzle_world.resources.victory_lines_entity = None;
        puzzle_world.resources.victory_time = 0.0;

        Self {
            puzzle_world,
            camera_entity: Entity::default(),
            initialized: false,
            texture_loaded: false,
            pieces_spawned: false,
        }
    }
}

impl State for JigsawGame {
    fn title(&self) -> &str {
        "Jigsaw Puzzle"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::Nebula;
        world.resources.graphics.selection_outline_enabled = true;

        let sun = spawn_sun(world);
        if let Some(light) = world.core.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 2.0;
        }

        let ground = world.spawn_entities(
            LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | RENDER_MESH
                | MATERIAL_REF
                | CASTS_SHADOW,
            1,
        )[0];
        world.core.set_local_transform(
            ground,
            LocalTransform {
                translation: Vec3::new(0.0, -0.05, 0.0),
                rotation: Quat::identity(),
                scale: Vec3::new(15.0, 0.1, 15.0),
            },
        );
        world.core.set_render_mesh(ground, RenderMesh::new("Cube"));
        let ground_material = format!("Ground_{}", ground.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            ground_material.clone(),
            Material {
                base_color: [0.25, 0.25, 0.3, 1.0],
                roughness: 0.9,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&ground_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world
            .core
            .set_material_ref(ground, MaterialRef::new(ground_material));
        world.core.set_casts_shadow(ground, CastsShadow);

        let top_down_pitch = std::f32::consts::FRAC_PI_2 - 0.01;

        self.camera_entity = spawn_pan_orbit_camera(
            world,
            nalgebra_glm::vec3(0.0, 0.0, 0.0),
            8.0,
            0.0,
            top_down_pitch,
            "Puzzle Camera".to_string(),
        );
        world.resources.active_camera = Some(self.camera_entity);

        if let Some((width, height)) = load_puzzle_texture(world, PUZZLE_IMAGE) {
            self.puzzle_world.resources.image_width = width;
            self.puzzle_world.resources.image_height = height;

            let aspect = width as f32 / height as f32;
            let cols = self.puzzle_world.resources.grid_cols as f32;
            let rows = self.puzzle_world.resources.grid_rows as f32;
            self.puzzle_world.resources.piece_width = 1.0;
            self.puzzle_world.resources.piece_height = cols / (aspect * rows);
        }

        self.initialized = true;
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        if !self.initialized {
            return;
        }

        if !self.texture_loaded {
            if world
                .resources
                .texture_cache
                .registry
                .name_to_index
                .contains_key("puzzle_texture")
            {
                self.texture_loaded = true;
            }
            return;
        }

        if !self.pieces_spawned {
            initialize_edge_types(&mut self.puzzle_world);
            spawn_board_outline(&mut self.puzzle_world, world);
            spawn_puzzle_pieces(&mut self.puzzle_world, world, "puzzle_texture");
            shuffle_pieces(&mut self.puzzle_world, world);
            self.pieces_spawned = true;
            return;
        }

        let delta_time = world.resources.window.timing.delta_time;

        let was_complete = self.puzzle_world.resources.puzzle_complete;
        solve_system(&mut self.puzzle_world, world, delta_time);

        if self.puzzle_world.resources.puzzle_complete
            && !was_complete
            && !self.puzzle_world.resources.victory_active
        {
            start_victory_celebration(&mut self.puzzle_world, world);
        }

        victory_system(&mut self.puzzle_world, world, delta_time);

        if !self.puzzle_world.resources.is_solving && !self.puzzle_world.resources.victory_active {
            input_system(&mut self.puzzle_world, world);
            snap_system(&mut self.puzzle_world, world);
        }

        if !self.puzzle_world.resources.victory_active && !self.puzzle_world.resources.is_solving {
            render_system(&self.puzzle_world, world);
            drop_pieces_system(&self.puzzle_world, world);
        }

        nightshade::ecs::text::systems::sync_text_meshes_system(world);
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Jigsaw Puzzle")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.label("Controls:");
                ui.label("  - Left click + drag: Move pieces");
                ui.label("  - Right click: Rotate piece");
                ui.label("  - Mouse wheel: Zoom");
                ui.label("  - Middle mouse: Pan camera");
                ui.label("  - C / Home: Reset camera");
                ui.label("  - B: Toggle board outline");
                ui.separator();

                let total_pieces =
                    self.puzzle_world.resources.grid_cols * self.puzzle_world.resources.grid_rows;
                let group_count = self
                    .puzzle_world
                    .query_entities(ecs::PIECE_GROUP | ecs::GROUP_MEMBERS)
                    .count();

                ui.label(format!("Groups: {} / {}", group_count, total_pieces));

                if self.puzzle_world.resources.puzzle_complete {
                    ui.separator();
                    ui.colored_label(egui::Color32::GREEN, "PUZZLE COMPLETE!");
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Reset Puzzle").clicked() {
                        self.puzzle_world.resources.is_solving = false;
                        self.puzzle_world.resources.solve_queue.clear();
                        reset_puzzle(&mut self.puzzle_world, _world);
                    }
                    if !self.puzzle_world.resources.puzzle_complete
                        && !self.puzzle_world.resources.is_solving
                        && ui.button("Solve").clicked()
                    {
                        start_solving(&mut self.puzzle_world);
                    }
                    if self.puzzle_world.resources.is_solving {
                        ui.label("Solving...");
                    }
                });

                ui.separator();
                ui.heading("Puzzle Size");

                let mut pending_cols = self.puzzle_world.resources.pending_cols as i32;
                let mut pending_rows = self.puzzle_world.resources.pending_rows as i32;

                ui.horizontal(|ui| {
                    ui.label("Columns:");
                    ui.add(egui::Slider::new(&mut pending_cols, 2..=8));
                });

                ui.horizontal(|ui| {
                    ui.label("Rows:");
                    ui.add(egui::Slider::new(&mut pending_rows, 2..=8));
                });

                self.puzzle_world.resources.pending_cols = pending_cols as u32;
                self.puzzle_world.resources.pending_rows = pending_rows as u32;

                ui.separator();
                ui.heading("Piece Shape");

                let mut apply_preset = false;
                ui.horizontal(|ui| {
                    if ui.button("Classic").clicked() {
                        self.puzzle_world.resources.pending_tab_depth = 0.20;
                        self.puzzle_world.resources.pending_tab_width = 0.32;
                        self.puzzle_world.resources.pending_neck_width = 0.14;
                        apply_preset = true;
                    }
                    if ui.button("Bulbous").clicked() {
                        self.puzzle_world.resources.pending_tab_depth = 0.25;
                        self.puzzle_world.resources.pending_tab_width = 0.40;
                        self.puzzle_world.resources.pending_neck_width = 0.12;
                        apply_preset = true;
                    }
                    if ui.button("Subtle").clicked() {
                        self.puzzle_world.resources.pending_tab_depth = 0.15;
                        self.puzzle_world.resources.pending_tab_width = 0.28;
                        self.puzzle_world.resources.pending_neck_width = 0.16;
                        apply_preset = true;
                    }
                });

                if apply_preset {
                    self.puzzle_world.resources.grid_cols =
                        self.puzzle_world.resources.pending_cols;
                    self.puzzle_world.resources.grid_rows =
                        self.puzzle_world.resources.pending_rows;
                    self.puzzle_world.resources.tab_depth =
                        self.puzzle_world.resources.pending_tab_depth;
                    self.puzzle_world.resources.tab_width =
                        self.puzzle_world.resources.pending_tab_width;
                    self.puzzle_world.resources.neck_width =
                        self.puzzle_world.resources.pending_neck_width;

                    let image_width = self.puzzle_world.resources.image_width as f32;
                    let image_height = self.puzzle_world.resources.image_height as f32;
                    let aspect = image_width / image_height;
                    let cols = self.puzzle_world.resources.grid_cols as f32;
                    let rows = self.puzzle_world.resources.grid_rows as f32;

                    self.puzzle_world.resources.piece_width = 1.0;
                    self.puzzle_world.resources.piece_height = cols / (aspect * rows);

                    reslice_puzzle(&mut self.puzzle_world, _world, "puzzle_texture");
                }

                ui.horizontal(|ui| {
                    ui.label("Depth:");
                    ui.add(
                        egui::Slider::new(
                            &mut self.puzzle_world.resources.pending_tab_depth,
                            0.10..=0.35,
                        )
                        .step_by(0.01),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(
                        egui::Slider::new(
                            &mut self.puzzle_world.resources.pending_tab_width,
                            0.20..=0.50,
                        )
                        .step_by(0.01),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Neck:");
                    ui.add(
                        egui::Slider::new(
                            &mut self.puzzle_world.resources.pending_neck_width,
                            0.08..=0.25,
                        )
                        .step_by(0.01),
                    );
                });

                let has_changes = self.puzzle_world.resources.pending_cols
                    != self.puzzle_world.resources.grid_cols
                    || self.puzzle_world.resources.pending_rows
                        != self.puzzle_world.resources.grid_rows
                    || (self.puzzle_world.resources.pending_tab_depth
                        - self.puzzle_world.resources.tab_depth)
                        .abs()
                        > 0.001
                    || (self.puzzle_world.resources.pending_tab_width
                        - self.puzzle_world.resources.tab_width)
                        .abs()
                        > 0.001
                    || (self.puzzle_world.resources.pending_neck_width
                        - self.puzzle_world.resources.neck_width)
                        .abs()
                        > 0.001;

                self.puzzle_world.resources.has_pending_changes = has_changes;

                ui.separator();
                ui.add_enabled_ui(has_changes, |ui| {
                    if ui.button("Apply Changes").clicked() {
                        self.puzzle_world.resources.grid_cols =
                            self.puzzle_world.resources.pending_cols;
                        self.puzzle_world.resources.grid_rows =
                            self.puzzle_world.resources.pending_rows;
                        self.puzzle_world.resources.tab_depth =
                            self.puzzle_world.resources.pending_tab_depth;
                        self.puzzle_world.resources.tab_width =
                            self.puzzle_world.resources.pending_tab_width;
                        self.puzzle_world.resources.neck_width =
                            self.puzzle_world.resources.pending_neck_width;

                        let image_width = self.puzzle_world.resources.image_width as f32;
                        let image_height = self.puzzle_world.resources.image_height as f32;
                        let aspect = image_width / image_height;
                        let cols = self.puzzle_world.resources.grid_cols as f32;
                        let rows = self.puzzle_world.resources.grid_rows as f32;

                        self.puzzle_world.resources.piece_width = 1.0;
                        self.puzzle_world.resources.piece_height = cols / (aspect * rows);

                        self.puzzle_world.resources.has_pending_changes = false;
                        reslice_puzzle(&mut self.puzzle_world, _world, "puzzle_texture");
                    }
                });
            });
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            KeyCode::KeyR => {
                reset_puzzle(&mut self.puzzle_world, world);
            }
            KeyCode::KeyC | KeyCode::Home => {
                if let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(self.camera_entity) {
                    pan_orbit.target_focus = nalgebra_glm::vec3(0.0, 0.0, 0.0);
                    pan_orbit.target_radius = 8.0;
                    pan_orbit.target_pitch = std::f32::consts::FRAC_PI_2 - 0.01;
                    pan_orbit.target_yaw = 0.0;
                }
            }
            KeyCode::KeyB => {
                toggle_board_outline(&mut self.puzzle_world, world);
            }
            _ => {}
        }
    }
}
