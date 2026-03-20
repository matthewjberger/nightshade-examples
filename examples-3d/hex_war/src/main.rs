mod camera;
mod constants;
mod ecs;
mod event_log;
mod hex;
mod hex_overlay_pass;
mod hud;
mod instancing;
mod map;
mod map_generation;
mod menu;
mod prefabs;
mod rendering;
mod selection;
mod systems;
mod tiles;
mod turn_phase;

use camera::{CameraBounds, calculate_camera_bounds, clamp_camera_to_bounds, reset_camera_to_map};
use constants::ACTIONS_PER_TURN;
use ecs::{Difficulty, FACTION_COUNT, Faction, GameEvents, GameWorld, TileType, UNIT};
use event_log::{EventLog, EventLogUi, build_event_log_ui, update_event_log_ui};
use hex::hex_to_world_position;
use hex_overlay_pass::{HexOverlayPass, SharedOverlayData};
use hud::{HudUi, build_hud_ui, update_hud};
use map_generation::{MapEntities, generate_game_map};
use menu::{
    MenuState, MenuUi, build_menu_ui, setup_game_over_display, show_menu_screen,
    update_difficulty_display,
};
use nightshade::ecs::prefab::Prefab;
use nightshade::prelude::*;
use prefabs::load_tile_prefabs;
use selection::clear_selection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use systems::{
    FireworkShell, GameResult, PendingSpawn, SpawnUnitParams, ai_turn_system, build_turn_order,
    can_end_turn, despawn_unit, end_turn, floating_popup_system, hover_outline_system,
    hover_system, input_system, movement_system, range_lines_system, selection_visual_system,
    spawn_capture_firework, spawn_capture_popup, spawn_unit, tile_highlight_system,
    tile_ownership_system, unit_text_system, unit_visual_update_system, update_firework_shells,
    valid_moves_system, victory_system,
};
use tiles::despawn_all_tiles;
use turn_phase::{TurnPhaseEvent, TurnPhaseState};

fn spawn_ocean(world: &mut World) -> Entity {
    use nightshade::ecs::water::Water;
    use nightshade::ecs::world::WATER;

    let entity = world.spawn_entities(WATER | NAME, 1)[0];
    world.core.set_name(entity, Name("Ocean".to_string()));
    world.core.set_water(
        entity,
        Water {
            base_height: -0.2,
            wave_height: 0.03,
            choppy: 2.0,
            speed: 0.5,
            frequency: 1.2,
            ..Default::default()
        },
    );
    entity
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(HexWarGame::default())
}

struct HexWarGame {
    game_world: GameWorld,
    game_events: GameEvents,
    map_entities: Option<MapEntities>,
    tile_prefabs: HashMap<TileType, Prefab>,
    menu_state: MenuState,
    selected_difficulty: Difficulty,
    event_log: EventLog,
    fps_entity: Option<Entity>,
    fps_visible: bool,
    sun_entity: Option<Entity>,
    ocean_entity: Option<Entity>,
    speech_requested: bool,
    player_faction: Faction,
    pending_spawns: Vec<PendingSpawn>,
    camera_bounds: Option<CameraBounds>,
    firework_shells: Vec<FireworkShell>,
    menu_ui: Option<MenuUi>,
    hud_ui: Option<HudUi>,
    event_log_ui: Option<EventLogUi>,
    overlay_data: SharedOverlayData,
}

impl Default for HexWarGame {
    fn default() -> Self {
        Self {
            game_world: GameWorld::default(),
            game_events: GameEvents::default(),
            map_entities: None,
            tile_prefabs: HashMap::new(),
            menu_state: MenuState::MainMenu,
            selected_difficulty: Difficulty::default(),
            event_log: EventLog::new(),
            fps_entity: None,
            fps_visible: false,
            sun_entity: None,
            ocean_entity: None,
            speech_requested: false,
            player_faction: Faction::default(),
            pending_spawns: Vec::new(),
            camera_bounds: None,
            firework_shells: Vec::new(),
            menu_ui: None,
            hud_ui: None,
            event_log_ui: None,
            overlay_data: Arc::new(Mutex::new(hex_overlay_pass::OverlayData::default())),
        }
    }
}

fn game_reset_camera(game: &HexWarGame, world: &mut World) {
    reset_camera_to_map(
        world,
        game.game_world.resources.hex_width,
        game.game_world.resources.hex_depth,
        game.game_world.resources.map_params.map_width,
        game.game_world.resources.map_params.map_height,
    );
}

fn game_regenerate_map(game: &mut HexWarGame, world: &mut World) {
    game_cleanup_map(game, world);
    game.map_entities = Some(generate_game_map(
        &mut game.game_world,
        world,
        &game.tile_prefabs,
    ));
    game.camera_bounds = Some(calculate_camera_bounds(
        game.game_world.resources.hex_width,
        game.game_world.resources.hex_depth,
        game.game_world.resources.map_params.map_width,
        game.game_world.resources.map_params.map_height,
    ));
}

fn game_cleanup_map(game: &mut HexWarGame, world: &mut World) {
    if let Some(mut entities) = game.map_entities.take() {
        map_generation::despawn_map_entities(world, &mut entities);
    }

    let unit_entities: Vec<_> = game.game_world.query_entities(UNIT).collect();
    for entity in unit_entities {
        despawn_unit(&mut game.game_world, world, entity);
    }

    despawn_all_tiles(&mut game.game_world);
    clear_selection(&mut game.game_world);
    game.game_world.resources.hovered_tile = None;
    game.game_world.resources.frame_cache = Default::default();
    game.game_world.resources.unit_position_map.clear();
}

fn advance_turn_phase(phase: &mut TurnPhaseState, event: TurnPhaseEvent) {
    if let Some(next) = phase.process_event(event) {
        *phase = next;
    }
}

fn game_end_turn(game: &mut HexWarGame) {
    let transition = end_turn(&mut game.game_world, &mut game.game_events);
    game.event_log
        .add_turn_start(transition.turn_number, transition.new_faction);
    game.pending_spawns = transition.pending_spawns;
}

fn game_cleanup_game_world(game: &mut HexWarGame, world: &mut World) {
    world.resources.graphics.atmosphere = Atmosphere::None;

    if let Some(sun) = game.sun_entity.take() {
        world.queue_command(WorldCommand::DespawnRecursive { entity: sun });
    }
    if let Some(ocean) = game.ocean_entity.take() {
        world.queue_command(WorldCommand::DespawnRecursive { entity: ocean });
    }

    game_cleanup_map(game, world);
}

fn game_range_lines_entity(game: &HexWarGame) -> Option<Entity> {
    game.map_entities.as_ref().map(|e| e.range_lines_entity)
}

fn game_hover_outline_entity(game: &HexWarGame) -> Option<Entity> {
    game.map_entities.as_ref().map(|e| e.hover_outline_entity)
}

fn build_fps_label(world: &mut World) -> Entity {
    let mut tree = UiTreeBuilder::new(world);
    let label = tree
        .add_node()
        .window(
            Ab(Vec2::new(-10.0, 10.0)),
            Ab(Vec2::new(150.0, 24.0)),
            Anchor::TopRight,
        )
        .with_text("", 14.0)
        .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
        .with_color::<UiBase>(Vec4::new(1.0, 1.0, 1.0, 1.0))
        .with_visible(false)
        .without_pointer_events()
        .done();
    tree.finish();
    label
}

fn enter_map_setup(game: &mut HexWarGame, world: &mut World) {
    world.resources.graphics.atmosphere = Atmosphere::CloudySky;
    game.sun_entity = Some(spawn_sun(world));
    game.ocean_entity = Some(spawn_ocean(world));
    game.map_entities = Some(generate_game_map(
        &mut game.game_world,
        world,
        &game.tile_prefabs,
    ));
    game.camera_bounds = Some(calculate_camera_bounds(
        game.game_world.resources.hex_width,
        game.game_world.resources.hex_depth,
        game.game_world.resources.map_params.map_width,
        game.game_world.resources.map_params.map_height,
    ));
    game_reset_camera(game, world);

    set_menu_state(game, world, MenuState::MapSetup);
    if let Some(ui) = &game.menu_ui {
        update_difficulty_display(world, ui, game.selected_difficulty);
    }
}

fn set_gameplay_ui_visible(game: &HexWarGame, world: &mut World, visible: bool) {
    if let Some(hud) = &game.hud_ui {
        world.ui_set_visible(hud.screen, visible);
    }
    if let Some(log_ui) = &game.event_log_ui {
        world.ui_set_visible(log_ui.screen, visible);
    }
}

fn set_menu_state(game: &mut HexWarGame, world: &mut World, state: MenuState) {
    game.menu_state = state;
    let playing = state == MenuState::Playing;
    set_gameplay_ui_visible(game, world, playing);
    if let Some(ui) = &game.menu_ui {
        show_menu_screen(world, ui, state);
    }
}

fn start_game(game: &mut HexWarGame, world: &mut World) {
    game.game_world.resources.current_faction = Faction::Redosia;
    game.game_world.resources.actions_remaining = ACTIONS_PER_TURN;
    game.game_world.resources.turn_number = 1;
    game.game_world.resources.faction_eliminated = [false; FACTION_COUNT];
    game.game_world.resources.game_speed = 1.0;
    game.game_world.resources.difficulty = game.selected_difficulty;

    build_turn_order(&mut game.game_world);

    game.event_log = EventLog::new();
    game.event_log.add_turn_start(1, Faction::Redosia);

    set_menu_state(game, world, MenuState::Playing);
}

fn pause_game(game: &mut HexWarGame, world: &mut World) {
    set_menu_state(game, world, MenuState::Paused);
}

fn resume_game(game: &mut HexWarGame, world: &mut World) {
    set_menu_state(game, world, MenuState::Playing);
}

fn return_to_main_menu(game: &mut HexWarGame, world: &mut World) {
    set_gameplay_ui_visible(game, world, false);
    game_cleanup_game_world(game, world);
    set_menu_state(game, world, MenuState::MainMenu);
}

impl State for HexWarGame {
    fn title(&self) -> &str {
        "Hex War"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.retained_ui.enabled = true;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;

        if let Some(loaded) = load_tile_prefabs(world) {
            self.tile_prefabs = loaded.tile_prefabs;
            self.game_world.resources.hex_width = loaded.hex_width;
            self.game_world.resources.hex_depth = loaded.hex_depth;
        }

        let camera_entity = spawn_pan_orbit_camera(
            world,
            nalgebra_glm::vec3(0.0, 0.0, 0.0),
            50.0,
            0.0,
            std::f32::consts::FRAC_PI_2 - 0.01,
            "Hex War Camera".to_string(),
        );
        if let Some(camera) = world.core.get_camera_mut(camera_entity) {
            camera.projection = Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 45.0_f32.to_radians(),
                z_far: Some(2000.0),
                z_near: 0.1,
            });
        }
        world.resources.active_camera = Some(camera_entity);

        self.menu_ui = Some(build_menu_ui(world));
        self.hud_ui = Some(build_hud_ui(world));
        self.event_log_ui = Some(build_event_log_ui(world));
        self.fps_entity = Some(build_fps_label(world));
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.menu_ui.is_none() {
            return;
        }

        match self.menu_state {
            MenuState::MainMenu => {
                let ui = self.menu_ui.as_ref().unwrap();
                if world.ui_clicked(ui.new_game_button) {
                    enter_map_setup(self, world);
                } else if world.ui_clicked(ui.quit_button) {
                    world.resources.window.should_exit = true;
                }
                return;
            }
            MenuState::MapSetup => {
                pan_orbit_camera_system(world);
                if let Some(bounds) = &self.camera_bounds {
                    clamp_camera_to_bounds(world, bounds);
                }

                let ui = self.menu_ui.as_ref().unwrap();
                if world.ui_clicked(ui.easy_button) {
                    self.selected_difficulty = Difficulty::Easy;
                    update_difficulty_display(world, ui, Difficulty::Easy);
                } else if world.ui_clicked(ui.normal_button) {
                    self.selected_difficulty = Difficulty::Normal;
                    update_difficulty_display(world, ui, Difficulty::Normal);
                } else if world.ui_clicked(ui.hard_button) {
                    self.selected_difficulty = Difficulty::Hard;
                    update_difficulty_display(world, ui, Difficulty::Hard);
                } else if world.ui_clicked(ui.new_map_button) {
                    game_regenerate_map(self, world);
                } else if world.ui_clicked(ui.start_button) {
                    start_game(self, world);
                } else if world.ui_clicked(ui.setup_back_button) {
                    game_cleanup_game_world(self, world);
                    set_menu_state(self, world, MenuState::MainMenu);
                }
                return;
            }
            MenuState::Paused => {
                let ui = self.menu_ui.as_ref().unwrap();
                if world.ui_clicked(ui.resume_button) {
                    resume_game(self, world);
                } else if world.ui_clicked(ui.pause_main_menu_button) {
                    return_to_main_menu(self, world);
                }
                return;
            }
            MenuState::GameOver => {
                pan_orbit_camera_system(world);
                if let Some(bounds) = &self.camera_bounds {
                    clamp_camera_to_bounds(world, bounds);
                }

                let ui = self.menu_ui.as_ref().unwrap();
                if world.ui_clicked(ui.game_over_new_game_button) {
                    set_gameplay_ui_visible(self, world, false);
                    game_cleanup_game_world(self, world);
                    enter_map_setup(self, world);
                } else if world.ui_clicked(ui.game_over_main_menu_button) {
                    return_to_main_menu(self, world);
                }
                return;
            }
            MenuState::Playing => {}
        }

        pan_orbit_camera_system(world);
        if let Some(bounds) = &self.camera_bounds {
            clamp_camera_to_bounds(world, bounds);
        }

        let hex_width = self.game_world.resources.hex_width;
        let hex_depth = self.game_world.resources.hex_depth;
        let delta_time = world.resources.window.timing.delta_time;
        update_particle_emitters(world, delta_time);
        update_firework_shells(&mut self.firework_shells, world, delta_time);
        movement_system(&mut self.game_world, world, delta_time);

        let is_ai_turn = self.game_world.resources.current_faction != self.player_faction;

        match self.game_world.resources.turn_phase {
            TurnPhaseState::Reinforcement => {
                for pending in self.pending_spawns.drain(..) {
                    spawn_unit(
                        &mut self.game_world,
                        world,
                        SpawnUnitParams {
                            coord: pending.coord,
                            hex_width,
                            hex_depth,
                            faction: pending.faction,
                            soldiers: pending.soldiers,
                            unit_type: pending.unit_type,
                        },
                    );
                }
                advance_turn_phase(
                    &mut self.game_world.resources.turn_phase,
                    TurnPhaseEvent::SpawnsProcessed,
                );
            }
            TurnPhaseState::Action => {
                if is_ai_turn {
                    let ai_done = ai_turn_system(
                        &mut self.game_world,
                        world,
                        self.player_faction,
                        &mut self.game_events,
                    );
                    if ai_done && can_end_turn(&self.game_world) {
                        advance_turn_phase(
                            &mut self.game_world.resources.turn_phase,
                            TurnPhaseEvent::ActionsExhausted,
                        );
                    }
                } else {
                    hover_system(&mut self.game_world, world);
                    input_system(&mut self.game_world, world, &mut self.game_events);
                    if self.speech_requested {
                        systems::execute_action(
                            &mut self.game_world,
                            world,
                            systems::GameAction::Speech,
                            &mut self.game_events,
                        );
                        self.speech_requested = false;
                    }
                }
            }
            TurnPhaseState::End => {
                game_end_turn(self);
                advance_turn_phase(
                    &mut self.game_world.resources.turn_phase,
                    TurnPhaseEvent::TurnAdvanced,
                );
            }
        }

        if self.fps_visible
            && let Some(fps_entity) = self.fps_entity
        {
            let fps = world.resources.window.timing.frames_per_second;
            world.ui_set_text(fps_entity, &format!("FPS: {:.0}", fps));
        }

        let range_lines_entity = game_range_lines_entity(self);
        let hover_outline_entity = game_hover_outline_entity(self);

        let captures = tile_ownership_system(&mut self.game_world);
        for capture in captures {
            let position = hex_to_world_position(
                capture.coord.column,
                capture.coord.row,
                hex_width,
                hex_depth,
            );
            spawn_capture_popup(&mut self.game_world, world, position, capture.tile_type);
            spawn_capture_firework(
                &mut self.firework_shells,
                world,
                position,
                capture.tile_type,
                capture.faction,
            );
        }

        selection_visual_system(&self.game_world, world);
        valid_moves_system(&mut self.game_world);
        range_lines_system(&mut self.game_world, world, range_lines_entity);

        tile_highlight_system(&mut self.game_world, &self.overlay_data);
        hover_outline_system(&mut self.game_world, world, hover_outline_entity);
        unit_text_system(&self.game_world, world);
        unit_visual_update_system(&self.game_world, world);
        floating_popup_system(&mut self.game_world, world, delta_time);
        nightshade::ecs::text::systems::sync_text_meshes_system(world);

        if let Some(hud) = &self.hud_ui {
            update_hud(hud, &mut self.game_world, world, self.player_faction);
        }

        let game_result = victory_system(&mut self.game_world, world, &mut self.game_events);

        self.event_log.drain_events(&mut self.game_events);
        self.event_log.scroll_system(world);
        if let Some(log_ui) = &self.event_log_ui {
            update_event_log_ui(
                world,
                &self.event_log,
                log_ui,
                &mut self.game_world.resources.frame_cache.previous_log_scroll,
                &mut self.game_world.resources.frame_cache.previous_log_count,
            );
        }

        if let GameResult::Victory(winner) = game_result {
            let is_player_winner = winner == self.player_faction;
            if let Some(ui) = &self.menu_ui {
                setup_game_over_display(world, ui, winner, is_player_winner);
            }
            set_menu_state(self, world, MenuState::GameOver);
        }

        self.game_world.step();
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            KeyCode::KeyP => match self.menu_state {
                MenuState::Playing => {
                    pause_game(self, world);
                }
                MenuState::Paused => {
                    resume_game(self, world);
                }
                MenuState::MainMenu | MenuState::MapSetup | MenuState::GameOver => {}
            },
            KeyCode::Space if self.menu_state == MenuState::Playing => {
                let is_player_turn =
                    self.game_world.resources.current_faction == self.player_faction;
                if is_player_turn {
                    advance_turn_phase(
                        &mut self.game_world.resources.turn_phase,
                        TurnPhaseEvent::EndTurnPressed,
                    );
                }
            }
            KeyCode::KeyS if self.menu_state == MenuState::Playing => {
                let is_player_turn =
                    self.game_world.resources.current_faction == self.player_faction;
                if is_player_turn {
                    self.speech_requested = true;
                }
            }
            KeyCode::Home | KeyCode::KeyC if self.menu_state == MenuState::Playing => {
                game_reset_camera(self, world);
            }
            KeyCode::KeyF => {
                self.fps_visible = !self.fps_visible;
                if let Some(fps_entity) = self.fps_entity {
                    world.ui_set_visible(fps_entity, self.fps_visible);
                    if !self.fps_visible {
                        world.ui_set_text(fps_entity, "");
                    }
                }
            }
            KeyCode::BracketRight | KeyCode::Equal if self.menu_state == MenuState::Playing => {
                let current = self.game_world.resources.game_speed;
                self.game_world.resources.game_speed = (current * 2.0).min(8.0);
            }
            KeyCode::BracketLeft | KeyCode::Minus if self.menu_state == MenuState::Playing => {
                let current = self.game_world.resources.game_speed;
                self.game_world.resources.game_speed = (current / 2.0).max(0.25);
            }
            _ => {}
        }
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let particle_pass = passes::ParticlePass::new(device, wgpu::TextureFormat::Rgba16Float);
        graph
            .pass(Box::new(particle_pass))
            .slot("color", resources.scene_color)
            .slot("depth", resources.depth);

        let overlay_output = graph
            .add_color_texture("overlay_output")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(
                resources.surface_width.max(1),
                resources.surface_height.max(1),
            )
            .transient();

        let blit_pipeline =
            passes::BlitPass::create_pipeline(device, wgpu::TextureFormat::Rgba16Float);
        let hex_overlay = HexOverlayPass::new(
            device,
            wgpu::TextureFormat::Rgba16Float,
            blit_pipeline,
            self.overlay_data.clone(),
        );
        graph
            .pass(Box::new(hex_overlay))
            .read("scene", resources.scene_color)
            .read("depth", resources.depth)
            .write("output", overlay_output);

        let (width, height) = (1920, 1080);
        let bloom_width = width / 2;
        let bloom_height = height / 2;

        let bloom_texture = graph
            .add_color_texture("bloom")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(bloom_width, bloom_height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", overlay_output)
            .write("bloom", bloom_texture);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.005);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", overlay_output)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .write("output", resources.compute_output);

        let fxaa_output = graph
            .add_color_texture("fxaa_output")
            .format(surface_format)
            .size(
                resources.surface_width.max(1),
                resources.surface_height.max(1),
            )
            .transient();

        let fxaa_pass = passes::FxaaPass::new(device, surface_format);
        graph
            .pass(Box::new(fxaa_pass))
            .read("input", resources.compute_output)
            .write("output", fxaa_output);

        let swapchain_blit_pass =
            passes::BlitPass::new(device, surface_format).with_name("default_swapchain_blit");
        graph
            .pass(Box::new(swapchain_blit_pass))
            .read("input", fxaa_output)
            .write("output", resources.swapchain);
    }
}
