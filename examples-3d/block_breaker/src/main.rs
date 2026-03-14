use gilrs::{Axis, Button};
use nightshade::ecs::generational_registry::registry_entry_by_name;
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(BlockBreakerGame::default())
}

freecs::ecs! {
    GameWorld {
        entity_handle: EntityHandle => ENTITY_HANDLE,
        position: Position => POSITION,
        velocity: Velocity => VELOCITY,
        paddle: Paddle => PADDLE,
        ball: Ball => BALL,
        brick: Brick => BRICK,
        particle: Particle => PARTICLE,
        trail: Trail => TRAIL,
    }
    GameResources {
        score: u32,
        lives: u32,
        combo: u32,
        combo_timer: f32,
        game_state: GameState,
        shake_time: f32,
        shake_intensity: f32,
    }
}

#[derive(Default)]
struct BlockBreakerGame {
    game_world: GameWorld,
    score_text: Option<Entity>,
    lives_text: Option<Entity>,
    message_text: Option<Entity>,
    start_text: Option<Entity>,
    combo_text: Option<Entity>,
    start_button_was_pressed: bool,
    game_z_offset: f32,
}

fn spawn_paddle(game_world: &mut GameWorld, world: &mut World, z_offset: f32) -> freecs::Entity {
    let engine_entity = spawn_mesh(
        world,
        "Cube",
        Vec3::new(0.0, -6.0, z_offset),
        Vec3::new(3.0, 0.5, 0.5),
    );

    let material_name = format!("Paddle_{}", engine_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: [0.2, 0.6, 1.0, 1.0],
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
    world.core.set_material_ref(engine_entity, MaterialRef::new(material_name));

    let game_entity = game_world.spawn_entities(ENTITY_HANDLE | POSITION | PADDLE, 1)[0];
    game_world.set_entity_handle(game_entity, EntityHandle(engine_entity));
    game_world.set_position(game_entity, Position(Vec3::new(0.0, -6.0, z_offset)));
    game_world.set_paddle(game_entity, Paddle { width: 3.0 });

    game_entity
}

fn spawn_ball(game_world: &mut GameWorld, world: &mut World, z_offset: f32) -> freecs::Entity {
    let engine_entity = spawn_mesh(
        world,
        "Sphere",
        Vec3::new(0.0, -5.0, z_offset),
        Vec3::new(0.4, 0.4, 0.4),
    );

    let material_name = format!("Ball_{}", engine_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
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
    world.core.set_material_ref(engine_entity, MaterialRef::new(material_name));

    let game_entity = game_world.spawn_entities(ENTITY_HANDLE | POSITION | VELOCITY | BALL, 1)[0];
    game_world.set_entity_handle(game_entity, EntityHandle(engine_entity));
    game_world.set_position(game_entity, Position(Vec3::new(0.0, -5.0, z_offset)));
    game_world.set_velocity(game_entity, Velocity(Vec3::new(5.0, 5.0, 0.0)));
    game_world.set_ball(game_entity, Ball { radius: 0.4 });

    game_entity
}

fn spawn_brick(
    game_world: &mut GameWorld,
    world: &mut World,
    position: Vec3,
    color: [f32; 4],
    row: u32,
) -> freecs::Entity {
    let engine_entity = spawn_mesh(world, "Cube", position, Vec3::new(0.9, 0.4, 0.5));

    let material_name = format!("Brick_{}", engine_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color,
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
    world.core.set_material_ref(engine_entity, MaterialRef::new(material_name));

    let game_entity = game_world.spawn_entities(ENTITY_HANDLE | POSITION | BRICK, 1)[0];
    game_world.set_entity_handle(game_entity, EntityHandle(engine_entity));
    game_world.set_position(game_entity, Position(position));
    game_world.set_brick(
        game_entity,
        Brick {
            value: (6 - row) * 10,
            row,
        },
    );

    game_entity
}

fn spawn_particle(
    game_world: &mut GameWorld,
    world: &mut World,
    position: Vec3,
    velocity: Vec3,
    color: [f32; 4],
) -> freecs::Entity {
    let engine_entity = spawn_mesh(world, "Cube", position, Vec3::new(0.1, 0.1, 0.1));

    let material_name = format!("Particle_{}", engine_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color,
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
    world.core.set_material_ref(engine_entity, MaterialRef::new(material_name));

    let game_entity =
        game_world.spawn_entities(ENTITY_HANDLE | POSITION | VELOCITY | PARTICLE, 1)[0];
    game_world.set_entity_handle(game_entity, EntityHandle(engine_entity));
    game_world.set_position(game_entity, Position(position));
    game_world.set_velocity(game_entity, Velocity(velocity));
    game_world.set_particle(
        game_entity,
        Particle {
            lifetime: 1.0,
            initial_lifetime: 1.0,
        },
    );

    game_entity
}

fn despawn_entity(game_world: &mut GameWorld, world: &mut World, entity: freecs::Entity) {
    if let Some(handle) = game_world.get_entity_handle(entity) {
        world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
    }
    game_world.despawn_entities(&[entity]);
}

fn update_engine_transform(game_world: &GameWorld, world: &mut World, game_entity: freecs::Entity) {
    if let (Some(handle), Some(position)) = (
        game_world.get_entity_handle(game_entity),
        game_world.get_position(game_entity),
    ) {
        if let Some(transform) = world.core.get_local_transform_mut(handle.0) {
            transform.translation = position.0;
        }
        mark_local_transform_dirty(world, handle.0);
    }
}

fn update_engine_scale(
    game_world: &GameWorld,
    world: &mut World,
    game_entity: freecs::Entity,
    scale: Vec3,
) {
    if let Some(handle) = game_world.get_entity_handle(game_entity) {
        if let Some(transform) = world.core.get_local_transform_mut(handle.0) {
            transform.scale = scale;
        }
        mark_local_transform_dirty(world, handle.0);
    }
}

fn reset_game(game_world: &mut GameWorld, world: &mut World, z_offset: f32) {
    game_world.resources.score = 0;
    game_world.resources.lives = 3;
    game_world.resources.combo = 0;
    game_world.resources.combo_timer = 0.0;
    game_world.resources.game_state = GameState::WaitingToStart;
    game_world.resources.shake_time = 0.0;

    let entities_to_remove: Vec<_> = game_world.query_entities(ENTITY_HANDLE).collect();
    for entity in entities_to_remove {
        despawn_entity(game_world, world, entity);
    }

    spawn_paddle(game_world, world, z_offset);
    spawn_ball(game_world, world, z_offset);

    let colors = [
        [1.0, 0.2, 0.2, 1.0],
        [1.0, 0.6, 0.2, 1.0],
        [1.0, 1.0, 0.2, 1.0],
        [0.2, 1.0, 0.2, 1.0],
        [0.2, 0.6, 1.0, 1.0],
        [0.6, 0.2, 1.0, 1.0],
    ];

    for row in 0..6 {
        for col in -8..=8 {
            spawn_brick(
                game_world,
                world,
                Vec3::new(col as f32 * 1.05, 6.0 - row as f32 * 1.05, z_offset),
                colors[row as usize],
                row,
            );
        }
    }
}

fn camera_shake_system(game_world: &mut GameWorld, world: &mut World) {
    let real_delta = world.resources.window.timing.delta_time;

    if game_world.resources.shake_time > 0.0 {
        game_world.resources.shake_time -= real_delta;
        let decay = game_world.resources.shake_time / 0.15;
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f32
            * 0.001;
        let shake_offset = Vec3::new(
            (time * 20.0).sin() * game_world.resources.shake_intensity * decay,
            (time * 25.0).cos() * game_world.resources.shake_intensity * decay,
            0.0,
        );
        if let Some(camera) = world.resources.active_camera {
            if let Some(transform) = world.core.get_local_transform_mut(camera) {
                transform.translation = Vec3::new(0.0, 0.0, 12.0) + shake_offset;
            }
            mark_local_transform_dirty(world, camera);
        }
    } else if let Some(camera) = world.resources.active_camera {
        if let Some(transform) = world.core.get_local_transform_mut(camera) {
            transform.translation = Vec3::new(0.0, 0.0, 12.0);
        }
        mark_local_transform_dirty(world, camera);
    }
}

fn paddle_movement_system(game_world: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time;

    let keyboard_movement = (world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::ArrowRight) as i32
        - world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::ArrowLeft) as i32) as f32;

    let gamepad_movement = query_active_gamepad(world)
        .and_then(|gamepad| gamepad.axis_data(Axis::LeftStickX).map(|data| data.value()))
        .unwrap_or(0.0);

    let movement = keyboard_movement + gamepad_movement;

    let paddle_entities: Vec<_> = game_world.query_entities(PADDLE | POSITION).collect();
    for entity in paddle_entities {
        let position = game_world.get_position(entity).unwrap();
        let new_x = (position.0.x + movement * 12.0 * delta_time).clamp(-8.0, 8.0);

        if let Some(position) = game_world.get_position_mut(entity) {
            position.0.x = new_x;
        }
        update_engine_transform(game_world, world, entity);
    }
}

fn ball_physics_system(
    game_world: &mut GameWorld,
    world: &mut World,
) -> (Vec<freecs::Entity>, Vec<(freecs::Entity, Vec3, Brick)>) {
    let delta_time = world.resources.window.timing.delta_time;

    let mut balls_to_remove = Vec::new();
    let mut brick_collisions = Vec::new();

    let paddle_entities: Vec<_> = game_world.query_entities(POSITION | PADDLE).collect();
    let paddle_data: Vec<_> = paddle_entities
        .iter()
        .filter_map(|&entity| {
            match (
                game_world.get_position(entity),
                game_world.get_paddle(entity),
            ) {
                (Some(pos), Some(paddle)) => Some((pos.0, paddle.width)),
                _ => None,
            }
        })
        .collect();
    let brick_entities: Vec<_> = game_world.query_entities(POSITION | BRICK).collect();
    let brick_data: Vec<_> = brick_entities
        .iter()
        .filter_map(|&entity| {
            match (
                game_world.get_position(entity),
                game_world.get_brick(entity),
            ) {
                (Some(pos), Some(brick)) => Some((entity, pos.0, *brick)),
                _ => None,
            }
        })
        .collect();

    let ball_entities: Vec<_> = game_world
        .query_entities(BALL | POSITION | VELOCITY)
        .collect();
    let mut ball_updates = Vec::new();

    for entity in ball_entities {
        let _ball = game_world.get_ball(entity).unwrap();
        let position = game_world.get_position(entity).unwrap();
        let velocity = game_world.get_velocity(entity).unwrap();

        let mut new_position = position.0 + velocity.0 * delta_time;
        let mut new_velocity = velocity.0;

        if new_position.x.abs() > 9.5 {
            new_position.x = new_position.x.clamp(-9.5, 9.5);
            new_velocity.x = -new_velocity.x;
        }
        if new_position.y > 7.5 {
            new_position.y = 7.5;
            new_velocity.y = -new_velocity.y;
        }
        if new_position.y < -7.5 {
            balls_to_remove.push(entity);
            continue;
        }

        for (paddle_pos, paddle_width) in &paddle_data {
            let distance = new_position - paddle_pos;
            if distance.x.abs() < (paddle_width / 2.0 + 0.2)
                && distance.y.abs() < 0.45
                && new_velocity.y < 0.0
            {
                new_velocity.y = -new_velocity.y;
                new_velocity.x += (distance.x / (paddle_width / 2.0)) * 3.0;
                new_velocity *= 1.05;
                new_position.y = paddle_pos.y + 0.45;
                game_world.resources.combo = 0;
            }
        }

        for &(brick_entity, brick_pos, brick) in &brick_data {
            let distance = new_position - brick_pos;
            if distance.x.abs() < 0.65 && distance.y.abs() < 0.4 {
                brick_collisions.push((brick_entity, brick_pos, brick));
                if distance.x.abs() / 0.65 > distance.y.abs() / 0.4 {
                    new_velocity.x = -new_velocity.x;
                } else {
                    new_velocity.y = -new_velocity.y;
                }
            }
        }

        ball_updates.push((entity, new_position, new_velocity));
    }

    for (entity, new_position, new_velocity) in ball_updates {
        if let Some(position) = game_world.get_position_mut(entity) {
            position.0 = new_position;
        }
        if let Some(velocity) = game_world.get_velocity_mut(entity) {
            velocity.0 = new_velocity;
        }
        update_engine_transform(game_world, world, entity);
    }

    (balls_to_remove, brick_collisions)
}

fn brick_collision_system(
    game_world: &mut GameWorld,
    world: &mut World,
    brick_collisions: Vec<(freecs::Entity, Vec3, Brick)>,
) {
    for (brick_entity, brick_pos, brick) in brick_collisions {
        game_world.resources.score += brick.value * (game_world.resources.combo + 1);
        game_world.resources.combo += 1;
        game_world.resources.combo_timer = 2.0;

        game_world.resources.shake_time = 0.15;
        game_world.resources.shake_intensity =
            0.2 + (game_world.resources.combo as f32 * 0.05).min(0.5);

        let mut brick_color = [1.0, 1.0, 1.0, 1.0];
        if let Some(handle) = game_world.get_entity_handle(brick_entity)
            && let Some(material_ref) = world.core.get_material_ref(handle.0)
            && let Some(material) = registry_entry_by_name(
                &world.resources.material_registry.registry,
                &material_ref.name,
            )
        {
            brick_color = material.base_color;
        }

        let mut rng = rand::rng();
        for _ in 0..8 {
            let particle_vel = Vec3::new(
                (rng.random::<f32>() - 0.5) * 5.0,
                rng.random::<f32>() * 5.0,
                0.0,
            );
            let particle_pos = brick_pos
                + Vec3::new(
                    (rng.random::<f32>() - 0.5) * 0.5,
                    (rng.random::<f32>() - 0.5) * 0.5,
                    0.0,
                );
            spawn_particle(game_world, world, particle_pos, particle_vel, brick_color);
        }

        despawn_entity(game_world, world, brick_entity);
    }
}

fn ball_death_system(
    game_world: &mut GameWorld,
    world: &mut World,
    balls_to_remove: Vec<freecs::Entity>,
) {
    for entity in balls_to_remove {
        let ball_entities: Vec<_> = game_world.query_entities(BALL).collect();
        let is_main_ball = game_world.get_ball(entity).is_some() && ball_entities.len() == 1;

        if is_main_ball {
            game_world.resources.lives = game_world.resources.lives.saturating_sub(1);
            if game_world.resources.lives == 0 {
                game_world.resources.game_state = GameState::GameOver;
            } else {
                if let Some(position) = game_world.get_position_mut(entity) {
                    position.0 = Vec3::new(0.0, -5.0, 0.0);
                }
                if let Some(velocity) = game_world.get_velocity_mut(entity) {
                    velocity.0 = Vec3::new(5.0, 5.0, 0.0);
                }
                update_engine_transform(game_world, world, entity);
                game_world.resources.game_state = GameState::WaitingToStart;
            }
        } else {
            despawn_entity(game_world, world, entity);
        }
    }
}

fn particle_system(game_world: &mut GameWorld, world: &mut World) {
    let real_delta = world.resources.window.timing.delta_time;
    let mut particles_to_remove = Vec::new();
    let particle_entities: Vec<_> = game_world
        .query_entities(PARTICLE | POSITION | VELOCITY)
        .collect();
    let mut particle_updates = Vec::new();

    for entity in particle_entities {
        let particle = game_world.get_particle(entity).unwrap();
        let position = game_world.get_position(entity).unwrap();
        let velocity = game_world.get_velocity(entity).unwrap();

        let new_lifetime = particle.lifetime - real_delta;
        if new_lifetime <= 0.0 {
            particles_to_remove.push(entity);
        } else {
            let new_velocity =
                Vec3::new(velocity.0.x, velocity.0.y - 9.8 * real_delta, velocity.0.z);
            let new_position = position.0 + new_velocity * real_delta;
            let scale = 0.1 * (new_lifetime / particle.initial_lifetime);
            particle_updates.push((entity, new_position, new_velocity, new_lifetime, scale));
        }
    }

    for (entity, new_position, new_velocity, new_lifetime, scale) in particle_updates {
        if let Some(particle) = game_world.get_particle_mut(entity) {
            particle.lifetime = new_lifetime;
        }
        if let Some(position) = game_world.get_position_mut(entity) {
            position.0 = new_position;
        }
        if let Some(velocity) = game_world.get_velocity_mut(entity) {
            velocity.0 = new_velocity;
        }
        update_engine_transform(game_world, world, entity);
        update_engine_scale(game_world, world, entity, Vec3::new(scale, scale, scale));
    }

    for entity in particles_to_remove {
        despawn_entity(game_world, world, entity);
    }
}

fn victory_check_system(game_world: &mut GameWorld) {
    let brick_entities: Vec<_> = game_world.query_entities(BRICK).collect();
    if brick_entities.is_empty() {
        game_world.resources.game_state = GameState::Victory;
    }
}

impl State for BlockBreakerGame {
    fn title(&self) -> &str {
        "Block Breaker!"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.show_cursor = false;

        #[cfg(feature = "openxr")]
        {
            world.resources.graphics.atmosphere = Atmosphere::Sky;
            world.resources.xr.locomotion_enabled = false;
            world.resources.xr.initial_player_yaw = Some(std::f32::consts::PI);
            world.resources.xr.initial_player_position = Some(Vec3::new(0.0, 0.0, 12.0));
        }

        self.game_z_offset = 0.0;

        spawn_sun_without_shadows(world);

        let camera = spawn_camera(
            world,
            Vec3::new(0.0, 0.0, 12.0 + self.game_z_offset),
            "Main Camera".to_string(),
        );

        if let Some(camera_component) = world.core.get_camera_mut(camera) {
            camera_component.projection = Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 80.0_f32.to_radians(),
                z_far: None,
                z_near: 0.01,
            });
        }

        world.resources.active_camera = Some(camera);

        let wall_material = Material {
            base_color: [0.7, 0.7, 0.7, 1.0],
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            ..Default::default()
        };

        let bottom = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, -8.0, self.game_z_offset),
            Vec3::new(20.0, 0.5, 0.5),
        );
        let bottom_mat_name = format!("WallBottom_{}", bottom.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            bottom_mat_name.clone(),
            wall_material.clone(),
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&bottom_mat_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(bottom, MaterialRef::new(bottom_mat_name));

        let top = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, 8.0, self.game_z_offset),
            Vec3::new(20.0, 0.5, 0.5),
        );
        let top_mat_name = format!("WallTop_{}", top.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            top_mat_name.clone(),
            wall_material.clone(),
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&top_mat_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(top, MaterialRef::new(top_mat_name));

        let left = spawn_mesh(
            world,
            "Cube",
            Vec3::new(-10.0, 0.0, self.game_z_offset),
            Vec3::new(0.5, 16.5, 0.5),
        );
        let left_mat_name = format!("WallLeft_{}", left.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            left_mat_name.clone(),
            wall_material.clone(),
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&left_mat_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(left, MaterialRef::new(left_mat_name));

        let right = spawn_mesh(
            world,
            "Cube",
            Vec3::new(10.0, 0.0, self.game_z_offset),
            Vec3::new(0.5, 16.5, 0.5),
        );
        let right_mat_name = format!("WallRight_{}", right.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            right_mat_name.clone(),
            wall_material,
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&right_mat_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(right, MaterialRef::new(right_mat_name));

        self.game_world.resources.score = 0;
        self.game_world.resources.lives = 3;
        self.game_world.resources.combo = 0;
        self.game_world.resources.combo_timer = 0.0;
        self.game_world.resources.game_state = GameState::WaitingToStart;
        self.game_world.resources.shake_time = 0.0;
        self.game_world.resources.shake_intensity = 0.0;

        spawn_paddle(&mut self.game_world, world, self.game_z_offset);
        spawn_ball(&mut self.game_world, world, self.game_z_offset);

        let colors = [
            [1.0, 0.2, 0.2, 1.0],
            [1.0, 0.6, 0.2, 1.0],
            [1.0, 1.0, 0.2, 1.0],
            [0.2, 1.0, 0.2, 1.0],
            [0.2, 0.6, 1.0, 1.0],
            [0.6, 0.2, 1.0, 1.0],
        ];

        for row in 0..6 {
            for col in -8..=8 {
                spawn_brick(
                    &mut self.game_world,
                    world,
                    Vec3::new(
                        col as f32 * 1.05,
                        6.0 - row as f32 * 1.05,
                        self.game_z_offset,
                    ),
                    colors[row as usize],
                    row,
                );
            }
        }

        let hud_props = TextProperties {
            font_size: 24.0,
            color: nalgebra_glm::vec4(1.0, 1.0, 1.0, 1.0),
            outline_width: 0.08,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };

        self.score_text = Some(spawn_ui_text_with_properties(
            world,
            "Score: 0",
            nalgebra_glm::Vec2::zeros(),
            hud_props.clone(),
        ));

        self.lives_text = Some(spawn_ui_text_with_properties(
            world,
            "Lives: 3",
            nalgebra_glm::Vec2::zeros(),
            hud_props.clone(),
        ));

        let message_props = TextProperties {
            font_size: 48.0,
            color: nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0),
            alignment: TextAlignment::Center,
            outline_width: 0.1,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };

        self.message_text = Some(spawn_ui_text_with_properties(
            world,
            "",
            nalgebra_glm::Vec2::zeros(),
            message_props,
        ));

        let start_props = TextProperties {
            font_size: 20.0,
            color: nalgebra_glm::vec4(0.8, 0.8, 0.8, 1.0),
            alignment: TextAlignment::Center,
            outline_width: 0.05,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };

        self.start_text = Some(spawn_ui_text_with_properties(
            world,
            "Press SPACE to start",
            nalgebra_glm::Vec2::zeros(),
            start_props,
        ));

        let combo_props = TextProperties {
            font_size: 32.0,
            color: nalgebra_glm::vec4(1.0, 0.5, 0.0, 1.0),
            alignment: TextAlignment::Center,
            outline_width: 0.1,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };

        self.combo_text = Some(spawn_ui_text_with_properties(
            world,
            "",
            nalgebra_glm::Vec2::zeros(),
            combo_props,
        ));
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        if let Some(score_entity) = self.score_text {
            let text_index = world.core.get_text(score_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world.resources.text_cache.set_text(
                    text_index,
                    format!("Score: {}", self.game_world.resources.score),
                );
                if let Some(hud_text) = world.core.get_text_mut(score_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(lives_entity) = self.lives_text {
            let text_index = world.core.get_text(lives_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world.resources.text_cache.set_text(
                    text_index,
                    format!("Lives: {}", self.game_world.resources.lives),
                );
                if let Some(hud_text) = world.core.get_text_mut(lives_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(combo_entity) = self.combo_text {
            let text_index = world.core.get_text(combo_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                if self.game_world.resources.combo > 1 {
                    world.resources.text_cache.set_text(
                        text_index,
                        format!("COMBO x{}", self.game_world.resources.combo),
                    );
                } else {
                    world.resources.text_cache.set_text(text_index, "");
                }
                if let Some(hud_text) = world.core.get_text_mut(combo_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(message_entity) = self.message_text {
            let text_index = world.core.get_text(message_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                let message = match self.game_world.resources.game_state {
                    GameState::Paused => "PAUSED",
                    GameState::GameOver => "GAME OVER",
                    GameState::Victory => "VICTORY!",
                    _ => "",
                };
                world.resources.text_cache.set_text(text_index, message);
                if let Some(hud_text) = world.core.get_text_mut(message_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(start_entity) = self.start_text {
            let text_index = world.core.get_text(start_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                let start_message =
                    if self.game_world.resources.game_state == GameState::WaitingToStart {
                        "Press SPACE to start"
                    } else {
                        ""
                    };
                world
                    .resources
                    .text_cache
                    .set_text(text_index, start_message);
                if let Some(hud_text) = world.core.get_text_mut(start_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        let mut should_quit = false;

        if let Some(gamepad) = query_active_gamepad(world) {
            let start_button_pressed = gamepad.is_pressed(Button::Start);

            if gamepad.is_pressed(Button::East) {
                should_quit = true;
            }

            if self.game_world.resources.game_state == GameState::WaitingToStart
                && (start_button_pressed || gamepad.is_pressed(Button::South))
            {
                self.game_world.resources.game_state = GameState::Playing;
            } else if self.game_world.resources.game_state == GameState::Playing
                && start_button_pressed
                && !self.start_button_was_pressed
            {
                self.game_world.resources.game_state = GameState::Paused;
            } else if self.game_world.resources.game_state == GameState::Paused
                && start_button_pressed
                && !self.start_button_was_pressed
            {
                self.game_world.resources.game_state = GameState::Playing;
            }

            if gamepad.is_pressed(Button::Select) {
                reset_game(&mut self.game_world, world, self.game_z_offset);
            }

            self.start_button_was_pressed = start_button_pressed;
        }

        if should_quit {
            world.resources.window.should_exit = true;
        }

        #[cfg(feature = "openxr")]
        if let Some(xr_input) = world.resources.xr.input.clone() {
            let paddle_movement = xr_input.thumbstick.x;
            if paddle_movement.abs() > 0.1 {
                let delta_time = world.resources.window.timing.delta_time;
                let paddle_entities: Vec<_> =
                    self.game_world.query_entities(PADDLE | POSITION).collect();
                for entity in paddle_entities {
                    let position = self.game_world.get_position(entity).unwrap();
                    let new_x =
                        (position.0.x + paddle_movement * 12.0 * delta_time).clamp(-8.0, 8.0);
                    if let Some(position) = self.game_world.get_position_mut(entity) {
                        position.0.x = new_x;
                    }
                    update_engine_transform(&self.game_world, world, entity);
                }
            }

            if self.game_world.resources.game_state == GameState::WaitingToStart
                && xr_input.a_button_pressed()
            {
                self.game_world.resources.game_state = GameState::Playing;
            }

            if xr_input.right_trigger_pressed() {
                reset_game(&mut self.game_world, world, self.game_z_offset);
            }
        }

        if self.game_world.resources.game_state == GameState::WaitingToStart {
            return;
        }

        if self.game_world.resources.game_state == GameState::Paused
            || self.game_world.resources.game_state == GameState::GameOver
            || self.game_world.resources.game_state == GameState::Victory
        {
            return;
        }

        camera_shake_system(&mut self.game_world, world);
        paddle_movement_system(&mut self.game_world, world);

        let (balls_to_remove, brick_collisions) = ball_physics_system(&mut self.game_world, world);
        brick_collision_system(&mut self.game_world, world, brick_collisions);
        ball_death_system(&mut self.game_world, world, balls_to_remove);

        particle_system(&mut self.game_world, world);

        victory_check_system(&mut self.game_world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state == KeyState::Pressed {
            match key {
                KeyCode::Space
                    if self.game_world.resources.game_state == GameState::WaitingToStart =>
                {
                    self.game_world.resources.game_state = GameState::Playing;
                }
                KeyCode::KeyP => {
                    if self.game_world.resources.game_state == GameState::Playing {
                        self.game_world.resources.game_state = GameState::Paused;
                    } else if self.game_world.resources.game_state == GameState::Paused {
                        self.game_world.resources.game_state = GameState::Playing;
                    }
                }
                KeyCode::KeyR => {
                    reset_game(&mut self.game_world, world, self.game_z_offset);
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EntityHandle(pub Entity);

#[derive(Debug, Clone, Copy, Default)]
pub struct Position(pub Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity(pub Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub struct Paddle {
    pub width: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Ball {
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Brick {
    pub value: u32,
    pub row: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Particle {
    pub lifetime: f32,
    pub initial_lifetime: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Trail {
    pub index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum GameState {
    #[default]
    WaitingToStart,
    Playing,
    Paused,
    GameOver,
    Victory,
}
