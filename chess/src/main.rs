mod board;
mod ecs;
mod pieces;
mod selection;
mod systems;

use ecs::ChessWorld;
use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::picking::register_entity_hierarchy_for_trimesh_picking;
use nightshade::ecs::prefab::{Prefab, import_gltf_from_bytes};
use nightshade::prelude::*;
use pieces::{PiecePrefabs, load_piece_prefabs, spawn_all_pieces};
use selection::clear_selection;
use systems::{highlight_system, hover_system, input_system, spawn_piece_system};

const CHESS_GLB: &[u8] = include_bytes!("../../assets/models/chess.glb");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(ChessGame::default())
}

struct ChessGame {
    chess_world: ChessWorld,
    piece_prefabs: Option<PiecePrefabs>,
    board_entity: Option<Entity>,
    loaded: bool,
    camera_entity: Option<Entity>,
    picking_registered: bool,
    show_picking_colliders: bool,
    debug_lines_entity: Option<Entity>,
}

impl Default for ChessGame {
    fn default() -> Self {
        let mut chess_world = ChessWorld::default();
        chess_world.resources.square_size = 1.0;
        Self {
            chess_world,
            piece_prefabs: None,
            board_entity: None,
            loaded: false,
            camera_entity: None,
            picking_registered: false,
            show_picking_colliders: false,
            debug_lines_entity: None,
        }
    }
}

impl State for ChessGame {
    fn title(&self) -> &str {
        "Chess"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::Nebula;
        world.resources.graphics.selection_outline_enabled = true;

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 2.0;
        }

        let camera_entity = spawn_pan_orbit_camera(
            world,
            nalgebra_glm::vec3(0.0035, 0.0, 0.0035),
            0.15,
            0.0,
            0.8,
            "Chess Camera".to_string(),
        );
        self.camera_entity = Some(camera_entity);
        world.resources.active_camera = Some(camera_entity);

        if let Some(pan_orbit) = world.get_pan_orbit_camera_mut(camera_entity) {
            pan_orbit.zoom_lower_limit = 0.002;
            pan_orbit.zoom_upper_limit = Some(0.5);
            pan_orbit.pitch_lower_limit = 0.3;
            pan_orbit.pitch_upper_limit = std::f32::consts::FRAC_PI_2 - 0.1;
        }

        tracing::info!("Loading chess GLTF model...");
        match import_gltf_from_bytes(CHESS_GLB) {
            Ok(result) => {
                tracing::info!("Successfully loaded chess GLTF");
                tracing::info!("Meshes: {:?}", result.meshes.keys().collect::<Vec<_>>());
                tracing::info!(
                    "Prefabs: {:?}",
                    result.prefabs.iter().map(|p| &p.name).collect::<Vec<_>>()
                );

                for prefab in &result.prefabs {
                    log_prefab_structure(prefab, 0);
                }

                if let Some(prefabs) = load_piece_prefabs(world, &result) {
                    self.piece_prefabs = Some(prefabs);

                    board::spawn_board_squares(&mut self.chess_world);

                    let scale = 0.001;
                    self.chess_world.resources.square_size = scale;

                    if let Some(scene_entity) =
                        pieces::spawn_full_scene(world, self.piece_prefabs.as_ref().unwrap(), scale)
                    {
                        self.board_entity = Some(scene_entity);
                        tracing::info!("Spawned full chess scene at scale {}", scale);
                    }

                    self.loaded = true;
                    reset_camera_to_board(world, scale);
                    tracing::info!("Chess game initialized!");
                    tracing::info!("Controls:");
                    tracing::info!("  - Left click + drag: Move pieces");
                    tracing::info!("  - Space: Spawn piece at hovered square");
                    tracing::info!("  - Q/E: Cycle piece type");
                    tracing::info!("  - W: Toggle piece color");
                    tracing::info!("  - R: Reset board");
                    tracing::info!("  - Home/C: Reset camera");
                }
            }
            Err(error) => {
                tracing::error!("Failed to load chess GLTF: {}", error);
            }
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        if !self.loaded {
            return;
        }

        if !self.picking_registered
            && world.resources.children_cache_valid
            && let Some(board_entity) = self.board_entity
        {
            register_entity_hierarchy_for_trimesh_picking(world, board_entity);
            self.picking_registered = true;
            tracing::info!("Registered chess pieces for trimesh picking");
        }

        hover_system(&mut self.chess_world, world);
        input_system(&mut self.chess_world, world);
        highlight_system(&self.chess_world, world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed || !self.loaded {
            return;
        }

        let Some(prefabs) = &self.piece_prefabs else {
            return;
        };

        match key {
            KeyCode::Escape => {
                clear_selection(&mut self.chess_world);
            }
            KeyCode::Space => {
                spawn_piece_system(&mut self.chess_world, world, prefabs);
            }
            KeyCode::KeyQ => {
                self.chess_world.resources.selected_piece_type =
                    self.chess_world.resources.selected_piece_type.previous();
                tracing::info!(
                    "Selected piece: {:?} {:?}",
                    self.chess_world.resources.selected_piece_color,
                    self.chess_world.resources.selected_piece_type
                );
            }
            KeyCode::KeyE => {
                self.chess_world.resources.selected_piece_type =
                    self.chess_world.resources.selected_piece_type.next();
                tracing::info!(
                    "Selected piece: {:?} {:?}",
                    self.chess_world.resources.selected_piece_color,
                    self.chess_world.resources.selected_piece_type
                );
            }
            KeyCode::KeyW => {
                self.chess_world.resources.selected_piece_color =
                    self.chess_world.resources.selected_piece_color.opposite();
                tracing::info!(
                    "Selected piece: {:?} {:?}",
                    self.chess_world.resources.selected_piece_color,
                    self.chess_world.resources.selected_piece_type
                );
            }
            KeyCode::KeyR => {
                reset_board(&mut self.chess_world, world, prefabs);
            }
            KeyCode::Home | KeyCode::KeyC => {
                reset_camera_to_board(world, self.chess_world.resources.square_size);
            }
            KeyCode::Digit4 => {
                self.show_picking_colliders = !self.show_picking_colliders;
                tracing::info!(
                    "Picking collider visualization: {}",
                    if self.show_picking_colliders {
                        "ON"
                    } else {
                        "OFF"
                    }
                );

                if self.show_picking_colliders {
                    if self.debug_lines_entity.is_none() {
                        let entity = world.spawn_entities(
                            nightshade::ecs::world::LINES
                                | nightshade::ecs::world::LOCAL_TRANSFORM
                                | nightshade::ecs::world::GLOBAL_TRANSFORM
                                | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
                                | nightshade::ecs::world::VISIBILITY,
                            1,
                        )[0];
                        world.set_visibility(
                            entity,
                            nightshade::ecs::world::components::Visibility { visible: true },
                        );
                        self.debug_lines_entity = Some(entity);
                    }
                    update_picking_collider_lines(world, self.debug_lines_entity.unwrap());
                } else if let Some(entity) = self.debug_lines_entity {
                    world.set_lines(entity, nightshade::ecs::world::components::Lines::default());
                }
            }
            _ => {}
        }
    }
}

fn update_picking_collider_lines(world: &mut World, lines_entity: Entity) {
    use nightshade::ecs::world::components::{Line, Lines};

    let mut lines = Vec::new();
    let color = nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0);

    for (_handle, collider) in world.resources.picking_world.collider_set.iter() {
        let position = collider.position();

        if let Some(trimesh) = collider.shape().as_trimesh() {
            for triangle_index in 0..trimesh.num_triangles() {
                let triangle = trimesh.triangle(triangle_index as u32);

                let v0 = position.transform_point(&triangle.a);
                let v1 = position.transform_point(&triangle.b);
                let v2 = position.transform_point(&triangle.c);

                lines.push(Line {
                    start: nalgebra_glm::vec3(v0.x, v0.y, v0.z),
                    end: nalgebra_glm::vec3(v1.x, v1.y, v1.z),
                    color,
                });
                lines.push(Line {
                    start: nalgebra_glm::vec3(v1.x, v1.y, v1.z),
                    end: nalgebra_glm::vec3(v2.x, v2.y, v2.z),
                    color,
                });
                lines.push(Line {
                    start: nalgebra_glm::vec3(v2.x, v2.y, v2.z),
                    end: nalgebra_glm::vec3(v0.x, v0.y, v0.z),
                    color,
                });
            }
        }
    }

    world.set_lines(lines_entity, Lines::new(lines));
}

fn reset_camera_to_board(world: &mut World, square_size: f32) {
    let Some(camera_entity) = world.resources.active_camera else {
        return;
    };

    let board_size = 8.0 * square_size;
    let center = 3.5 * square_size;

    let y_fov_rad = if let Some(camera) = world.get_camera(camera_entity) {
        match &camera.projection {
            Projection::Perspective(persp) => persp.y_fov_rad,
            Projection::Orthographic(_) => std::f32::consts::FRAC_PI_4,
        }
    } else {
        std::f32::consts::FRAC_PI_4
    };

    let (viewport_width, viewport_height) = world
        .resources
        .window
        .cached_viewport_size
        .unwrap_or((1920, 1080));
    let aspect_ratio = viewport_width as f32 / viewport_height as f32;

    let half_fov_tan = (y_fov_rad / 2.0).tan();
    let radius_for_height = (board_size / 2.0) / half_fov_tan;
    let radius_for_width = (board_size / 2.0) / (half_fov_tan * aspect_ratio);
    let radius = radius_for_height.max(radius_for_width) * 3.0;

    let Some(pan_orbit) = world.get_pan_orbit_camera_mut(camera_entity) else {
        return;
    };

    pan_orbit.target_focus = nalgebra_glm::vec3(center, 0.0, center);
    pan_orbit.target_radius = radius;
    pan_orbit.target_yaw = 0.0;
    pan_orbit.target_pitch = 0.8;
}

fn reset_board(chess_world: &mut ChessWorld, world: &mut World, prefabs: &PiecePrefabs) {
    use crate::ecs::{ENGINE_ENTITY, PIECE};

    let pieces: Vec<_> = chess_world.query_entities(ENGINE_ENTITY | PIECE).collect();
    for entity in pieces {
        pieces::despawn_piece(chess_world, world, entity);
    }

    spawn_all_pieces(chess_world, world, prefabs);
    tracing::info!("Board reset!");
}

fn log_prefab_structure(prefab: &Prefab, indent: usize) {
    let indent_str = "  ".repeat(indent);
    tracing::info!("{}Prefab: {}", indent_str, prefab.name);
    for node in &prefab.root_nodes {
        log_node_structure(node, indent + 1);
    }
}

fn log_node_structure(node: &nightshade::ecs::prefab::PrefabNode, indent: usize) {
    let indent_str = "  ".repeat(indent);
    let name = node
        .components
        .name
        .as_ref()
        .map(|n| n.0.as_str())
        .unwrap_or("unnamed");
    let has_mesh = node.components.render_mesh.is_some();
    tracing::info!("{}Node: {} (mesh: {})", indent_str, name, has_mesh);
    for child in &node.children {
        log_node_structure(child, indent + 1);
    }
}
