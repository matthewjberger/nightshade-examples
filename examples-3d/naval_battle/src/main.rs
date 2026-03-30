use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::lines::components::{Line, Lines};
use nightshade::ecs::material::components::Material;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::particles::components::{
    ColorGradient, EmitterShape, EmitterType, ParticleEmitter,
};
use nightshade::ecs::prefab::components::Prefab;
use nightshade::ecs::prefab::import_gltf_from_path;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::prefab::spawn_prefab;
use nightshade::ecs::transform::components::Parent;
use nightshade::ecs::water::{Water, sample_wave_height};
use nightshade::ecs::world::WATER;
use nightshade::prelude::*;
use std::f32::consts::TAU;
use std::path::Path;

const PLAYER_SPEED: f32 = 8.0;
const PLAYER_TURN_SPEED: f32 = 1.5;
const CANNON_COOLDOWN: f32 = 2.0;
const CANNONBALL_SPEED: f32 = 40.0;
const CANNONBALL_GRAVITY: f32 = 12.0;
const SHIP_BOB_OFFSET: f32 = 0.0;
const PLAYER_SHIP_SCALE: f32 = 0.2;
const SHIP_HALF_LENGTH: f32 = 8.0;
const SHIP_HALF_BEAM: f32 = 3.0;
const HITS_TO_KILL: u32 = 3;
const SINK_DURATION: f32 = 5.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(NavalBattle::default())
}

freecs::ecs! {
    GameWorld {
        entity_handle: EntityHandle => ENTITY_HANDLE,
        position: Position => POSITION,
        ship: Ship => SHIP,
        cannonball: CannonballComp => CANNONBALL,
        effect: Effect => EFFECT,
    }
    GameResources {
        enemy_list: Vec<freecs::Entity>,
        cannonball_list: Vec<freecs::Entity>,
        effect_list: Vec<freecs::Entity>,
        game_speed: f32,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EntityHandle(Entity);

#[derive(Debug, Clone, Copy, Default)]
pub struct Position(Vec3);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ShipFaction {
    #[default]
    Player,
    Enemy,
}

#[derive(Debug, Clone, Default)]
pub struct Ship {
    faction: ShipFaction,
    heading: f32,
    speed: f32,
    cannon_cooldown: f32,
    root_entity: Entity,
    smooth_y: f32,
    smooth_pitch: f32,
    smooth_roll: f32,
    hits_taken: u32,
    dead: bool,
    sink_timer: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CannonballComp {
    velocity: Vec3,
    age: f32,
    trail_emitter: Entity,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Effect {
    lifetime: f32,
    age: f32,
}

struct NavalBattle {
    game: GameWorld,
    camera_entity: Option<Entity>,
    player_entity: Option<freecs::Entity>,
    water_config: Water,
    player_prefab: Option<Prefab>,
    trajectory_line_port: Option<Entity>,
    trajectory_line_starboard: Option<Entity>,
}

impl Default for NavalBattle {
    fn default() -> Self {
        Self {
            game: GameWorld::default(),
            camera_entity: None,
            player_entity: None,
            water_config: Water {
                base_color: [0.0, 0.05, 0.12, 1.0],
                water_color: [0.1, 0.25, 0.35, 1.0],
                wave_height: 0.6,
                choppy: 4.0,
                speed: 0.8,
                frequency: 0.16,
                ..Default::default()
            },
            player_prefab: None,
            trajectory_line_port: None,
            trajectory_line_starboard: None,
        }
    }
}

impl State for NavalBattle {
    fn title(&self) -> &str {
        "Naval Battle"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::CloudySky;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.bloom_enabled = false;

        self.camera_entity = Some(spawn_camera(world));
        world.resources.active_camera = self.camera_entity;

        spawn_ocean_sun(world);
        spawn_ocean(world);
        register_materials(world);
        self.load_player_model(world);

        self.game.resources.game_speed = 1.0;
        self.player_entity =
            Some(self.spawn_ship(world, vec3(0.0, 0.0, 0.0), 0.0, ShipFaction::Player));
        self.trajectory_line_port = Some(spawn_trajectory_lines(world));
        self.trajectory_line_starboard = Some(spawn_trajectory_lines(world));

        for (position, heading) in [
            (vec3(60.0, 0.0, 30.0), 2.5),
            (vec3(-50.0, 0.0, -40.0), 1.0),
            (vec3(20.0, 0.0, -70.0), 4.0),
        ] {
            self.spawn_ship(world, position, heading, ShipFaction::Enemy);
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        let delta = world.resources.window.timing.delta_time * self.game.resources.game_speed;
        let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

        self.player_input_system(world, delta);
        self.cannonball_system(world, delta);
        self.sinking_system(world, delta);
        self.ship_bob_system(world, time);
        self.update_trajectory_lines(world);
        self.camera_follow_system(world);
        self.effect_system(world, delta);

        update_particle_emitters(world, world.resources.window.timing.delta_time);
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        let cooldown = self
            .player_entity
            .and_then(|entity| self.game.get_ship(entity))
            .map_or(0.0, |s| s.cannon_cooldown);
        let ready = cooldown <= 0.0;
        let player_pos = self
            .player_entity
            .and_then(|entity| self.game.get_position(entity))
            .map_or(Vec3::zeros(), |p| p.0);
        let player_heading = self
            .player_entity
            .and_then(|entity| self.game.get_ship(entity))
            .map_or(0.0, |s| s.heading);

        let enemies_alive = self
            .game
            .resources
            .enemy_list
            .iter()
            .filter(|entity| self.game.get_ship(**entity).is_some_and(|s| !s.dead))
            .count();
        let enemies_total = self.game.resources.enemy_list.len();

        egui::Area::new(egui::Id::new("hud"))
            .fixed_pos(egui::pos2(16.0, 16.0))
            .show(ui_context, |ui| {
                egui::Frame::NONE
                    .fill(egui::Color32::from_black_alpha(160))
                    .inner_margin(egui::Margin::same(12))
                    .corner_radius(6.0)
                    .show(ui, |ui| {
                        if ready {
                            ui.label(
                                egui::RichText::new("CANNONS READY")
                                    .size(20.0)
                                    .color(egui::Color32::from_rgb(0, 255, 80)),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new(format!("Reloading: {cooldown:.1}s"))
                                    .size(20.0)
                                    .color(egui::Color32::from_rgb(255, 160, 50)),
                            );
                        }
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "Enemy ships: {enemies_alive}/{enemies_total}"
                            ))
                            .size(18.0)
                            .color(egui::Color32::WHITE),
                        );

                        ui.add_space(8.0);
                        let radar_size = 120.0;
                        let radar_range = 120.0;
                        let (response, painter) = ui.allocate_painter(
                            egui::vec2(radar_size, radar_size),
                            egui::Sense::hover(),
                        );
                        let center = response.rect.center();

                        painter.circle_filled(
                            center,
                            radar_size / 2.0,
                            egui::Color32::from_black_alpha(180),
                        );
                        painter.circle_stroke(
                            center,
                            radar_size / 2.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 100, 0)),
                        );
                        painter.circle_stroke(
                            center,
                            radar_size / 4.0,
                            egui::Stroke::new(0.5, egui::Color32::from_rgb(0, 60, 0)),
                        );

                        let heading_sin = player_heading.sin();
                        let heading_cos = player_heading.cos();
                        let fwd_len = radar_size / 2.0 - 4.0;
                        painter.line_segment(
                            [
                                center,
                                egui::pos2(
                                    center.x - heading_sin * fwd_len * 0.3,
                                    center.y - heading_cos * fwd_len * 0.3,
                                ),
                            ],
                            egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 200, 0)),
                        );

                        painter.circle_filled(center, 3.0, egui::Color32::from_rgb(0, 255, 0));

                        for &enemy in &self.game.resources.enemy_list {
                            let Some(ship) = self.game.get_ship(enemy) else {
                                continue;
                            };
                            let enemy_pos =
                                self.game.get_position(enemy).map_or(Vec3::zeros(), |p| p.0);
                            let relative_x = enemy_pos.x - player_pos.x;
                            let relative_z = enemy_pos.z - player_pos.z;

                            let screen_x = (relative_x / radar_range) * (radar_size / 2.0);
                            let screen_y = (relative_z / radar_range) * (radar_size / 2.0);

                            let dist = (screen_x * screen_x + screen_y * screen_y).sqrt();
                            if dist > radar_size / 2.0 - 4.0 {
                                continue;
                            }

                            let dot_color = if ship.dead {
                                egui::Color32::from_rgb(80, 80, 80)
                            } else {
                                egui::Color32::from_rgb(255, 50, 50)
                            };
                            let dot_pos = egui::pos2(center.x + screen_x, center.y + screen_y);
                            painter.circle_filled(dot_pos, 3.0, dot_color);
                        }

                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("Q port | E starboard")
                                .size(13.0)
                                .color(egui::Color32::from_white_alpha(140)),
                        );
                        ui.label(
                            egui::RichText::new("WA/D move/turn | Scroll zoom")
                                .size(13.0)
                                .color(egui::Color32::from_white_alpha(140)),
                        );
                    });
            });

        if enemies_total > 0 && enemies_alive == 0 {
            egui::Area::new(egui::Id::new("victory"))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui_context, |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_black_alpha(200))
                        .inner_margin(egui::Margin::same(24))
                        .corner_radius(10.0)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new("ALL ENEMIES SUNK!")
                                        .size(48.0)
                                        .color(egui::Color32::from_rgb(255, 217, 0)),
                                );
                            });
                        });
                });
        }
    }
}

impl NavalBattle {
    fn load_player_model(&mut self, world: &mut World) {
        let model_path = Path::new("assets/models/uss_newport_news_war_thunder.glb");
        match import_gltf_from_path(model_path) {
            Ok(result) => {
                for (name, (rgba_data, width, height)) in result.textures {
                    world.queue_command(WorldCommand::LoadTexture {
                        name,
                        rgba_data,
                        width,
                        height,
                    });
                }
                for (name, mesh) in result.meshes {
                    mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
                }
                self.player_prefab = result.prefabs.into_iter().next();
            }
            Err(error) => {
                tracing::error!("Failed to load ship model: {error}");
            }
        }
    }

    fn spawn_ship(
        &mut self,
        world: &mut World,
        position: Vec3,
        heading: f32,
        faction: ShipFaction,
    ) -> freecs::Entity {
        let game_entity = self.game.spawn_entities(ENTITY_HANDLE | POSITION | SHIP, 1)[0];

        let root_entity = if let Some(prefab) = &self.player_prefab {
            let entity = spawn_prefab(world, prefab, position);
            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                transform.scale =
                    Vec3::new(PLAYER_SHIP_SCALE, PLAYER_SHIP_SCALE, PLAYER_SHIP_SCALE);
            }
            mark_local_transform_dirty(world, entity);
            entity
        } else {
            spawn_entity_with_material(
                world,
                "Cube",
                position,
                vec3(3.0, 1.0, 16.0),
                "ship_hull_fallback",
            )
        };

        if faction == ShipFaction::Enemy {
            tint_prefab_red(world, root_entity);
        }

        self.game
            .set_entity_handle(game_entity, EntityHandle(root_entity));
        self.game.set_position(game_entity, Position(position));
        self.game.set_ship(
            game_entity,
            Ship {
                faction,
                heading,
                speed: PLAYER_SPEED,
                cannon_cooldown: 0.0,
                root_entity,
                smooth_y: 0.0,
                smooth_pitch: 0.0,
                smooth_roll: 0.0,
                hits_taken: 0,
                dead: false,
                sink_timer: 0.0,
            },
        );

        if faction == ShipFaction::Enemy {
            self.game.resources.enemy_list.push(game_entity);
        }
        game_entity
    }

    fn player_input_system(&mut self, world: &mut World, delta: f32) {
        let Some(player) = self.player_entity else {
            return;
        };
        let Some(mut ship) = self.game.get_ship(player).cloned() else {
            return;
        };

        let keyboard = &world.resources.input.keyboard;
        let mut turn = 0.0;
        let mut throttle = 0.0;

        if keyboard.is_key_pressed(KeyCode::KeyA) || keyboard.is_key_pressed(KeyCode::ArrowLeft) {
            turn += 1.0
        }
        if keyboard.is_key_pressed(KeyCode::KeyD) || keyboard.is_key_pressed(KeyCode::ArrowRight) {
            turn -= 1.0
        }
        if keyboard.is_key_pressed(KeyCode::KeyW) || keyboard.is_key_pressed(KeyCode::ArrowUp) {
            throttle += 1.0
        }

        ship.heading += turn * PLAYER_TURN_SPEED * delta;
        ship.heading = (ship.heading + TAU) % TAU;

        let forward = vec3(ship.heading.sin(), 0.0, ship.heading.cos());
        ship.cannon_cooldown = (ship.cannon_cooldown - delta).max(0.0);

        let position = self
            .game
            .get_position(player)
            .map_or(Vec3::zeros(), |p| p.0);
        let new_position = position + forward * throttle * ship.speed * delta;
        if let Some(pos) = self.game.get_position_mut(player) {
            pos.0 = new_position
        }

        let heading = ship.heading;
        let fire_left = keyboard.is_key_pressed(KeyCode::KeyQ);
        let fire_right = keyboard.is_key_pressed(KeyCode::KeyE);

        if ship.cannon_cooldown <= 0.0 {
            let fired = if fire_left {
                let side = vec3(-heading.cos(), 0.0, heading.sin());
                self.fire_cannonball(
                    world,
                    new_position + side * 4.0 + vec3(0.0, 4.0, 0.0),
                    vec3(side.x, 0.3, side.z),
                );
                true
            } else if fire_right {
                let side = vec3(heading.cos(), 0.0, -heading.sin());
                self.fire_cannonball(
                    world,
                    new_position + side * 4.0 + vec3(0.0, 4.0, 0.0),
                    vec3(side.x, 0.3, side.z),
                );
                true
            } else {
                false
            };
            if fired {
                ship.cannon_cooldown = CANNON_COOLDOWN
            }
        }

        self.game.set_ship(player, ship);
    }

    fn fire_cannonball(&mut self, world: &mut World, position: Vec3, direction: Vec3) {
        let game_entity = self
            .game
            .spawn_entities(ENTITY_HANDLE | POSITION | CANNONBALL, 1)[0];
        let velocity = direction.normalize() * CANNONBALL_SPEED;

        let ball_entity = spawn_entity_with_material(
            world,
            "Sphere",
            position,
            vec3(0.8, 0.8, 0.8),
            "cannonball_glow",
        );

        let trail_emitter = spawn_particle_emitter(
            world,
            position,
            ParticleEmitter {
                emitter_type: EmitterType::Smoke,
                shape: EmitterShape::Point,
                direction: Vec3::new(0.0, 0.2, 0.0),
                spawn_rate: 50.0,
                particle_lifetime_min: 0.5,
                particle_lifetime_max: 1.0,
                initial_velocity_min: 0.3,
                initial_velocity_max: 0.8,
                velocity_spread: 0.5,
                gravity: vec3(0.0, 0.5, 0.0),
                drag: 1.5,
                size_start: 0.4,
                size_end: 1.2,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, nalgebra_glm::vec4(0.4, 0.4, 0.4, 0.5)),
                        (1.0, nalgebra_glm::vec4(0.2, 0.2, 0.2, 0.0)),
                    ],
                },
                ..Default::default()
            },
        );

        self.game
            .set_entity_handle(game_entity, EntityHandle(ball_entity));
        self.game.set_position(game_entity, Position(position));
        self.game.set_cannonball(
            game_entity,
            CannonballComp {
                velocity,
                age: 0.0,
                trail_emitter,
            },
        );
        self.game.resources.cannonball_list.push(game_entity);
    }

    fn cannonball_system(&mut self, world: &mut World, delta: f32) {
        let mut to_remove: Vec<freecs::Entity> = Vec::new();
        let cannonball_list: Vec<freecs::Entity> = self.game.resources.cannonball_list.clone();

        for &ball_entity in &cannonball_list {
            let Some(ball) = self.game.get_cannonball(ball_entity).copied() else {
                to_remove.push(ball_entity);
                continue;
            };

            let position = self
                .game
                .get_position(ball_entity)
                .map_or(Vec3::zeros(), |p| p.0);
            let mut new_velocity = ball.velocity;
            new_velocity.y -= CANNONBALL_GRAVITY * delta;
            let new_position = position + new_velocity * delta;

            if let Some(pos) = self.game.get_position_mut(ball_entity) {
                pos.0 = new_position
            }
            if let Some(b) = self.game.get_cannonball_mut(ball_entity) {
                b.velocity = new_velocity;
                b.age += delta;
            }

            if let Some(handle) = self.game.get_entity_handle(ball_entity).map(|h| h.0) {
                if let Some(transform) = world.core.get_local_transform_mut(handle) {
                    transform.translation = new_position;
                }
                mark_local_transform_dirty(world, handle);
            }

            if let Some(transform) = world.core.get_local_transform_mut(ball.trail_emitter) {
                transform.translation = new_position;
            }
            mark_local_transform_dirty(world, ball.trail_emitter);

            if new_position.y < -1.0 || ball.age > 5.0 {
                self.spawn_splash(world, vec3(new_position.x, 0.0, new_position.z));
                to_remove.push(ball_entity);
                continue;
            }

            if let Some(hit_enemy) = self.check_enemy_hit(new_position) {
                self.register_hit(world, hit_enemy, new_position);
                to_remove.push(ball_entity);
            }
        }

        for entity in &to_remove {
            let ball = self.game.get_cannonball(*entity).copied();
            if let Some(handle) = self.game.get_entity_handle(*entity) {
                despawn_recursive_immediate(world, handle.0);
            }
            if let Some(ball) = ball {
                despawn_recursive_immediate(world, ball.trail_emitter);
            }
            self.game.despawn_entities(&[*entity]);
            self.game.resources.cannonball_list.retain(|e| e != entity);
        }
    }

    fn check_enemy_hit(&self, ball_pos: Vec3) -> Option<freecs::Entity> {
        let hit_radius = 6.0;
        for &enemy in &self.game.resources.enemy_list {
            let Some(ship) = self.game.get_ship(enemy) else {
                continue;
            };
            if ship.dead || ship.faction != ShipFaction::Enemy {
                continue;
            }
            let enemy_pos = self.game.get_position(enemy).map_or(Vec3::zeros(), |p| p.0);
            let diff = ball_pos - enemy_pos;
            let xz_dist = (diff.x * diff.x + diff.z * diff.z).sqrt();
            if xz_dist < hit_radius && diff.y.abs() < 5.0 {
                return Some(enemy);
            }
        }
        None
    }

    fn register_hit(&mut self, world: &mut World, enemy: freecs::Entity, hit_pos: Vec3) {
        let Some(ship) = self.game.get_ship(enemy).cloned() else {
            return;
        };

        let marker =
            spawn_entity_with_material(world, "Cube", hit_pos, vec3(0.5, 0.5, 0.5), "hit_marker");
        parent_to(world, marker, ship.root_entity);

        let smoke = spawn_particle_emitter(
            world,
            hit_pos,
            ParticleEmitter {
                emitter_type: EmitterType::Smoke,
                shape: EmitterShape::Sphere { radius: 0.2 },
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 8.0,
                particle_lifetime_min: 1.0,
                particle_lifetime_max: 2.0,
                initial_velocity_min: 0.5,
                initial_velocity_max: 1.5,
                velocity_spread: 0.4,
                gravity: vec3(0.0, 0.3, 0.0),
                drag: 0.5,
                size_start: 0.4,
                size_end: 1.5,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, nalgebra_glm::vec4(0.2, 0.2, 0.2, 0.6)),
                        (1.0, nalgebra_glm::vec4(0.1, 0.1, 0.1, 0.0)),
                    ],
                },
                ..Default::default()
            },
        );
        parent_to(world, smoke, ship.root_entity);

        self.spawn_explosion(world, hit_pos);

        let Some(ship_mut) = self.game.get_ship_mut(enemy) else {
            return;
        };
        ship_mut.hits_taken += 1;
        if ship_mut.hits_taken >= HITS_TO_KILL {
            ship_mut.dead = true;
        }
    }

    fn sinking_system(&mut self, world: &mut World, delta: f32) {
        let mut to_remove: Vec<freecs::Entity> = Vec::new();

        for &enemy in &self.game.resources.enemy_list.clone() {
            let Some(mut ship) = self.game.get_ship(enemy).cloned() else {
                continue;
            };
            if !ship.dead {
                continue;
            }

            ship.sink_timer += delta;
            let sink_progress = (ship.sink_timer / SINK_DURATION).min(1.0);
            let position = self.game.get_position(enemy).map_or(Vec3::zeros(), |p| p.0);
            let sink_y = ship.smooth_y - sink_progress * 8.0;
            let tilt = sink_progress * 0.4;

            let heading_rot = nalgebra_glm::quat_angle_axis(ship.heading, &Vec3::y());
            let tilt_rot = nalgebra_glm::quat_angle_axis(tilt, &Vec3::z());

            if let Some(transform) = world.core.get_local_transform_mut(ship.root_entity) {
                transform.translation = vec3(position.x, sink_y, position.z);
                transform.rotation = heading_rot * tilt_rot;
                transform.scale =
                    Vec3::new(PLAYER_SHIP_SCALE, PLAYER_SHIP_SCALE, PLAYER_SHIP_SCALE);
            }
            mark_local_transform_dirty(world, ship.root_entity);

            if ship.sink_timer > SINK_DURATION {
                to_remove.push(enemy);
            }

            self.game.set_ship(enemy, ship);
        }

        for entity in &to_remove {
            if let Some(ship) = self.game.get_ship(*entity).cloned() {
                despawn_recursive_immediate(world, ship.root_entity);
            }
            self.game.resources.enemy_list.retain(|e| e != entity);
            self.game.despawn_entities(&[*entity]);
        }
    }

    fn spawn_explosion(&mut self, world: &mut World, position: Vec3) {
        let game_entity = self
            .game
            .spawn_entities(ENTITY_HANDLE | POSITION | EFFECT, 1)[0];
        let emitter_entity = spawn_particle_emitter(
            world,
            position,
            ParticleEmitter {
                emitter_type: EmitterType::Fire,
                shape: EmitterShape::Sphere { radius: 0.5 },
                direction: Vec3::new(0.0, 1.0, 0.0),
                burst_count: 25,
                particle_lifetime_min: 0.5,
                particle_lifetime_max: 1.0,
                initial_velocity_min: 3.0,
                initial_velocity_max: 6.0,
                velocity_spread: 1.0,
                gravity: vec3(0.0, -2.0, 0.0),
                drag: 0.5,
                size_start: 0.5,
                size_end: 0.05,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, nalgebra_glm::vec4(1.0, 0.8, 0.2, 1.0)),
                        (0.5, nalgebra_glm::vec4(1.0, 0.3, 0.0, 0.8)),
                        (1.0, nalgebra_glm::vec4(0.2, 0.05, 0.0, 0.0)),
                    ],
                },
                emissive_strength: 12.0,
                one_shot: true,
                ..Default::default()
            },
        );
        self.game
            .set_entity_handle(game_entity, EntityHandle(emitter_entity));
        self.game.set_position(game_entity, Position(position));
        self.game.set_effect(
            game_entity,
            Effect {
                lifetime: 1.5,
                age: 0.0,
            },
        );
        self.game.resources.effect_list.push(game_entity);
    }

    fn ship_bob_system(&mut self, world: &mut World, time: f32) {
        let mut all_ships: Vec<freecs::Entity> = self.game.resources.enemy_list.clone();
        if let Some(player) = self.player_entity {
            all_ships.push(player);
        }

        for &ship_entity in &all_ships.clone() {
            let Some(mut ship) = self.game.get_ship(ship_entity).cloned() else {
                continue;
            };
            if ship.dead {
                continue;
            }

            let position = self
                .game
                .get_position(ship_entity)
                .map_or(Vec3::zeros(), |p| p.0);
            let delta = world.resources.window.timing.delta_time;
            let y_smoothing = 1.0 - (-2.0 * delta).exp();
            let angle_smoothing = 1.0 - (-1.2 * delta).exp();

            let heading_sin = ship.heading.sin();
            let heading_cos = ship.heading.cos();

            let height_bow = sample_wave_height(
                &self.water_config,
                position.x - heading_sin * SHIP_HALF_LENGTH,
                position.z - heading_cos * SHIP_HALF_LENGTH,
                time,
            );
            let height_stern = sample_wave_height(
                &self.water_config,
                position.x + heading_sin * SHIP_HALF_LENGTH,
                position.z + heading_cos * SHIP_HALF_LENGTH,
                time,
            );
            let height_port = sample_wave_height(
                &self.water_config,
                position.x - heading_cos * SHIP_HALF_BEAM,
                position.z + heading_sin * SHIP_HALF_BEAM,
                time,
            );
            let height_starboard = sample_wave_height(
                &self.water_config,
                position.x + heading_cos * SHIP_HALF_BEAM,
                position.z - heading_sin * SHIP_HALF_BEAM,
                time,
            );
            let height_center =
                sample_wave_height(&self.water_config, position.x, position.z, time);

            let target_y =
                (height_bow + height_stern + height_port + height_starboard + height_center) / 5.0
                    + SHIP_BOB_OFFSET;
            let target_pitch = ((height_stern - height_bow) / (SHIP_HALF_LENGTH * 2.0)).atan();
            let target_roll = ((height_starboard - height_port) / (SHIP_HALF_BEAM * 2.0)).atan();

            ship.smooth_y += (target_y - ship.smooth_y) * y_smoothing;
            ship.smooth_pitch += (target_pitch - ship.smooth_pitch) * angle_smoothing;
            ship.smooth_roll += (target_roll - ship.smooth_roll) * angle_smoothing;

            let heading_rot = nalgebra_glm::quat_angle_axis(ship.heading, &Vec3::y());
            let pitch_rot = nalgebra_glm::quat_angle_axis(ship.smooth_pitch, &Vec3::x());
            let roll_rot = nalgebra_glm::quat_angle_axis(ship.smooth_roll, &Vec3::z());
            let rotation = heading_rot * pitch_rot * roll_rot;

            if let Some(transform) = world.core.get_local_transform_mut(ship.root_entity) {
                transform.translation = vec3(position.x, ship.smooth_y, position.z);
                transform.rotation = rotation;
                transform.scale =
                    Vec3::new(PLAYER_SHIP_SCALE, PLAYER_SHIP_SCALE, PLAYER_SHIP_SCALE);
            }
            mark_local_transform_dirty(world, ship.root_entity);

            self.game.set_ship(ship_entity, ship);
        }
    }

    fn update_trajectory_lines(&self, world: &mut World) {
        let Some(player) = self.player_entity else {
            return;
        };
        let Some(ship) = self.game.get_ship(player) else {
            return;
        };
        let position = self
            .game
            .get_position(player)
            .map_or(Vec3::zeros(), |p| p.0);
        let heading = ship.heading;

        let port_side = vec3(-heading.cos(), 0.0, heading.sin());
        let starboard_side = vec3(heading.cos(), 0.0, -heading.sin());

        if let Some(entity) = self.trajectory_line_port {
            let lines = compute_trajectory_arc(
                position + port_side * 4.0 + vec3(0.0, 4.0, 0.0),
                vec3(port_side.x, 0.3, port_side.z),
            );
            if let Some(lines_comp) = world.core.get_lines_mut(entity) {
                lines_comp.lines = lines;
                lines_comp.version += 1;
            }
        }
        if let Some(entity) = self.trajectory_line_starboard {
            let lines = compute_trajectory_arc(
                position + starboard_side * 4.0 + vec3(0.0, 4.0, 0.0),
                vec3(starboard_side.x, 0.3, starboard_side.z),
            );
            if let Some(lines_comp) = world.core.get_lines_mut(entity) {
                lines_comp.lines = lines;
                lines_comp.version += 1;
            }
        }
    }

    fn camera_follow_system(&mut self, world: &mut World) {
        let Some(player) = self.player_entity else {
            return;
        };
        let Some(camera) = self.camera_entity else {
            return;
        };
        let position = self
            .game
            .get_position(player)
            .map_or(Vec3::zeros(), |p| p.0);
        if let Some(orbit) = world.core.get_pan_orbit_camera_mut(camera) {
            orbit.target_focus = vec3(position.x, 2.0, position.z);
            orbit.pitch = orbit.pitch.max(0.05);
            orbit.target_pitch = orbit.target_pitch.max(0.05);
        }
    }

    fn spawn_splash(&mut self, world: &mut World, position: Vec3) {
        let game_entity = self
            .game
            .spawn_entities(ENTITY_HANDLE | POSITION | EFFECT, 1)[0];
        let emitter_entity = spawn_particle_emitter(
            world,
            position,
            ParticleEmitter {
                emitter_type: EmitterType::Sparks,
                shape: EmitterShape::Sphere { radius: 0.3 },
                direction: Vec3::new(0.0, 1.0, 0.0),
                burst_count: 15,
                particle_lifetime_min: 0.4,
                particle_lifetime_max: 0.7,
                initial_velocity_min: 2.0,
                initial_velocity_max: 5.0,
                velocity_spread: 0.8,
                gravity: vec3(0.0, -8.0, 0.0),
                drag: 1.0,
                size_start: 0.3,
                size_end: 0.02,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, nalgebra_glm::vec4(0.7, 0.85, 1.0, 0.9)),
                        (1.0, nalgebra_glm::vec4(0.3, 0.5, 0.7, 0.0)),
                    ],
                },
                emissive_strength: 3.0,
                one_shot: true,
                ..Default::default()
            },
        );
        self.game
            .set_entity_handle(game_entity, EntityHandle(emitter_entity));
        self.game.set_position(game_entity, Position(position));
        self.game.set_effect(
            game_entity,
            Effect {
                lifetime: 1.0,
                age: 0.0,
            },
        );
        self.game.resources.effect_list.push(game_entity);
    }

    fn effect_system(&mut self, world: &mut World, delta: f32) {
        let mut to_remove: Vec<freecs::Entity> = Vec::new();
        let effect_list: Vec<freecs::Entity> = self.game.resources.effect_list.clone();
        for &effect_entity in &effect_list {
            let Some(effect) = self.game.get_effect_mut(effect_entity) else {
                to_remove.push(effect_entity);
                continue;
            };
            effect.age += delta;
            if effect.age >= effect.lifetime {
                to_remove.push(effect_entity)
            }
        }
        for entity in &to_remove {
            if let Some(handle) = self.game.get_entity_handle(*entity) {
                despawn_recursive_immediate(world, handle.0);
            }
            self.game.despawn_entities(&[*entity]);
            self.game.resources.effect_list.retain(|e| e != entity);
        }
    }
}

fn tint_prefab_red(world: &mut World, root: Entity) {
    let entities: Vec<Entity> = world
        .core
        .query_entities(RENDER_MESH | MATERIAL_REF)
        .collect();
    for entity in entities {
        if entity != root && !is_descendant_of(world, entity, root) {
            continue;
        }
        let material_name = format!("enemy_tint_{}", entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color: [0.6, 0.1, 0.1, 1.0],
                roughness: 0.7,
                metallic: 0.1,
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
        world
            .core
            .set_material_ref(entity, MaterialRef::new(&material_name));
    }
}

fn is_descendant_of(world: &World, entity: Entity, ancestor: Entity) -> bool {
    let mut current = entity;
    for _ in 0..32 {
        if let Some(parent) = world.core.get_parent(current)
            && let Some(parent_entity) = parent.0
        {
            if parent_entity == ancestor {
                return true;
            }
            current = parent_entity;
        } else {
            return false;
        }
    }
    false
}

fn parent_to(world: &mut World, child: Entity, parent: Entity) {
    world
        .core
        .add_components(child, nightshade::ecs::world::PARENT);
    world.core.set_parent(child, Parent(Some(parent)));
    mark_local_transform_dirty(world, child);
}

fn spawn_camera(world: &mut World) -> Entity {
    use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
    let camera = spawn_pan_orbit_camera(
        world,
        vec3(0.0, 2.0, 0.0),
        45.0,
        0.0,
        0.4,
        "Main Camera".to_string(),
    );
    if let Some(cam) = world.core.get_camera_mut(camera) {
        cam.projection = Projection::Perspective(PerspectiveCamera {
            aspect_ratio: None,
            y_fov_rad: 60.0_f32.to_radians(),
            z_far: Some(2000.0),
            z_near: 0.1,
        });
    }
    camera
}

fn spawn_ocean_sun(world: &mut World) {
    let sun = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | LIGHT,
        1,
    )[0];
    world.core.set_name(sun, Name("Sun".to_string()));
    world.core.set_local_transform(
        sun,
        LocalTransform {
            translation: Vec3::new(100.0, 80.0, 50.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(sun, LocalTransformDirty);
    world
        .core
        .set_global_transform(sun, GlobalTransform::default());
    world.core.set_light(
        sun,
        Light {
            light_type: LightType::Directional,
            color: Vec3::new(1.0, 0.92, 0.75),
            intensity: 3.5,
            range: 0.0,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: true,
            shadow_bias: 0.005,
        },
    );
    if let Some(transform) = world.core.get_local_transform_mut(sun) {
        let dir = Vec3::new(-0.4, -0.7, -0.3).normalize();
        let fwd = -dir;
        let up = Vec3::new(0.0, 1.0, 0.0);
        let right = nalgebra_glm::cross(&up, &fwd).normalize();
        let cup = nalgebra_glm::cross(&fwd, &right);
        transform.rotation = nalgebra_glm::mat3_to_quat(&nalgebra_glm::mat3(
            right.x, cup.x, fwd.x, right.y, cup.y, fwd.y, right.z, cup.z, fwd.z,
        ));
    }
    mark_local_transform_dirty(world, sun);
}

fn spawn_ocean(world: &mut World) {
    let entity = world.spawn_entities(WATER | NAME, 1)[0];
    world.core.set_name(entity, Name("Ocean".to_string()));
    world.core.set_water(
        entity,
        Water {
            base_color: [0.0, 0.05, 0.12, 1.0],
            water_color: [0.1, 0.25, 0.35, 1.0],
            wave_height: 0.6,
            choppy: 4.0,
            speed: 0.8,
            frequency: 0.16,
            ..Default::default()
        },
    );
}

fn register_materials(world: &mut World) {
    register_material(
        world,
        "ship_hull_fallback",
        [0.35, 0.20, 0.10, 1.0],
        0.8,
        0.0,
        [0.0; 3],
    );
    register_material(
        world,
        "cannonball_glow",
        [0.15, 0.12, 0.1, 1.0],
        0.4,
        0.8,
        [0.0; 3],
    );
    register_material(
        world,
        "hit_marker",
        [0.8, 0.05, 0.05, 1.0],
        0.9,
        0.0,
        [0.5, 0.0, 0.0],
    );
}

fn register_material(
    world: &mut World,
    name: &str,
    color: [f32; 4],
    roughness: f32,
    metallic: f32,
    emissive: [f32; 3],
) {
    material_registry_insert(
        &mut world.resources.material_registry,
        name.to_string(),
        Material {
            base_color: color,
            roughness,
            metallic,
            emissive_factor: emissive,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
}

fn spawn_trajectory_lines(world: &mut World) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | GLOBAL_TRANSFORM
            | nightshade::ecs::world::LINES,
        1,
    )[0];
    world.core.set_name(entity, Name("Trajectory".to_string()));
    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: Vec3::zeros(),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world.core.set_lines(entity, Lines::new(Vec::new()));
    entity
}

fn compute_trajectory_arc(origin: Vec3, direction: Vec3) -> Vec<Line> {
    let velocity = direction.normalize() * CANNONBALL_SPEED;
    let steps = 30;
    let dt = 0.1;
    let mut lines = Vec::with_capacity(steps);
    let mut pos = origin;
    let mut vel = velocity;
    let color = nalgebra_glm::vec4(1.0, 1.0, 1.0, 0.3);

    for _ in 0..steps {
        let prev = pos;
        vel.y -= CANNONBALL_GRAVITY * dt;
        pos += vel * dt;
        if pos.y < 0.0 {
            break;
        }
        lines.push(Line {
            start: prev,
            end: pos,
            color,
        });
    }
    lines
}

fn spawn_entity_with_material(
    world: &mut World,
    mesh_name: &str,
    position: Vec3,
    scale: Vec3,
    material_name: &str,
) -> Entity {
    let entity = spawn_mesh(world, mesh_name, position, scale);
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material_name));
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    entity
}

fn spawn_particle_emitter(world: &mut World, position: Vec3, emitter: ParticleEmitter) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | GLOBAL_TRANSFORM
            | nightshade::ecs::world::PARTICLE_EMITTER,
        1,
    )[0];
    world.core.set_name(entity, Name("Particle".to_string()));
    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world.core.set_particle_emitter(entity, emitter);
    entity
}
