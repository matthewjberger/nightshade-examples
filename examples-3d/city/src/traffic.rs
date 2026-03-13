use nightshade::ecs::world::WorldCommand;
use nightshade::prelude::*;
use rand::{Rng, SeedableRng};

use crate::city::CHUNK_SIZE;
use crate::kenney;
use crate::materials::{apply_material, spawn_city_mesh};

const MAX_CARS: usize = 40;
const BLOCK_SIZE: f32 = 16.0;
const LANE_OFFSET: f32 = 0.8;
const MIN_SPEED: f32 = 3.0;
const MAX_SPEED: f32 = 8.0;
const DESPAWN_DISTANCE: f32 = CHUNK_SIZE * 4.0;
const SPAWN_DISTANCE_MIN: f32 = CHUNK_SIZE * 2.0;
const SPAWN_DISTANCE_MAX: f32 = CHUNK_SIZE * 4.0;
const SPAWN_INTERVAL: f32 = 0.3;
const INTERSECTION_SNAP_THRESHOLD: f32 = 0.5;
const CAR_SCALE: Vec3 = Vec3::new(1.0, 1.0, 1.0);

#[derive(Clone, Copy)]
enum CardinalDirection {
    PosX,
    NegX,
    PosZ,
    NegZ,
}

impl CardinalDirection {
    fn as_vec3(self) -> Vec3 {
        match self {
            CardinalDirection::PosX => Vec3::new(1.0, 0.0, 0.0),
            CardinalDirection::NegX => Vec3::new(-1.0, 0.0, 0.0),
            CardinalDirection::PosZ => Vec3::new(0.0, 0.0, 1.0),
            CardinalDirection::NegZ => Vec3::new(0.0, 0.0, -1.0),
        }
    }

    fn yaw_angle(self) -> f32 {
        match self {
            CardinalDirection::PosX => std::f32::consts::FRAC_PI_2,
            CardinalDirection::NegX => -std::f32::consts::FRAC_PI_2,
            CardinalDirection::PosZ => std::f32::consts::PI,
            CardinalDirection::NegZ => 0.0,
        }
    }

    fn turn_left(self) -> Self {
        match self {
            CardinalDirection::PosX => CardinalDirection::NegZ,
            CardinalDirection::NegZ => CardinalDirection::NegX,
            CardinalDirection::NegX => CardinalDirection::PosZ,
            CardinalDirection::PosZ => CardinalDirection::PosX,
        }
    }

    fn turn_right(self) -> Self {
        match self {
            CardinalDirection::PosX => CardinalDirection::PosZ,
            CardinalDirection::PosZ => CardinalDirection::NegX,
            CardinalDirection::NegX => CardinalDirection::NegZ,
            CardinalDirection::NegZ => CardinalDirection::PosX,
        }
    }

    fn lane_offset_vec(self) -> Vec3 {
        match self {
            CardinalDirection::PosX => Vec3::new(0.0, 0.0, LANE_OFFSET),
            CardinalDirection::NegX => Vec3::new(0.0, 0.0, -LANE_OFFSET),
            CardinalDirection::PosZ => Vec3::new(-LANE_OFFSET, 0.0, 0.0),
            CardinalDirection::NegZ => Vec3::new(LANE_OFFSET, 0.0, 0.0),
        }
    }
}

struct TrafficCar {
    entity: Entity,
    position: Vec3,
    direction: CardinalDirection,
    speed: f32,
}

pub struct TrafficSystem {
    cars: Vec<TrafficCar>,
    rng: rand::rngs::StdRng,
    spawn_timer: f32,
    city_min: f32,
    city_max: f32,
}

impl TrafficSystem {
    pub fn new(city_half: i32) -> Self {
        let rng = rand::rngs::StdRng::seed_from_u64(777);
        let city_extent = city_half as f32 * CHUNK_SIZE;

        Self {
            cars: Vec::with_capacity(MAX_CARS),
            rng,
            spawn_timer: 0.0,
            city_min: -city_extent,
            city_max: city_extent,
        }
    }

    pub fn update(&mut self, world: &mut World, camera_pos: Vec3, delta_time: f32) {
        let mut to_remove = Vec::new();

        for (car_index, car) in self.cars.iter_mut().enumerate() {
            let movement = car.direction.as_vec3() * car.speed * delta_time;
            car.position += movement;

            let dx = car.position.x - camera_pos.x;
            let dz = car.position.z - camera_pos.z;
            let distance_sq = dx * dx + dz * dz;

            if distance_sq > DESPAWN_DISTANCE * DESPAWN_DISTANCE
                || car.position.x < self.city_min - 10.0
                || car.position.x > self.city_max + 10.0
                || car.position.z < self.city_min - 10.0
                || car.position.z > self.city_max + 10.0
            {
                to_remove.push(car_index);
                continue;
            }

            check_intersection_turn(car, &mut self.rng);

            if let Some(transform) = world.core.get_local_transform_mut(car.entity) {
                transform.translation = car.position;
                transform.rotation =
                    nalgebra_glm::quat_angle_axis(car.direction.yaw_angle(), &Vec3::y());
            }
            world.mark_local_transform_dirty(car.entity);
        }

        for &car_index in to_remove.iter().rev() {
            let car = self.cars.swap_remove(car_index);
            world.queue_command(WorldCommand::DespawnRecursive {
                entity: car.entity,
            });
        }

        self.spawn_timer -= delta_time;
        if self.spawn_timer <= 0.0 {
            self.spawn_timer = SPAWN_INTERVAL;
            self.try_spawn_car(world, camera_pos);
        }
    }

    fn try_spawn_car(&mut self, world: &mut World, camera_pos: Vec3) {
        if self.cars.len() >= MAX_CARS {
            return;
        }

        let direction = match self.rng.random_range(0u32..4) {
            0 => CardinalDirection::PosX,
            1 => CardinalDirection::NegX,
            2 => CardinalDirection::PosZ,
            _ => CardinalDirection::NegZ,
        };

        let distance = self.rng.random_range(SPAWN_DISTANCE_MIN..SPAWN_DISTANCE_MAX);
        let lateral_offset = self.rng.random_range(-distance..distance);

        let spawn_pos = match direction {
            CardinalDirection::PosX => {
                let road_z = snap_to_road_grid(camera_pos.z + lateral_offset);
                Vec3::new(camera_pos.x - distance, 0.0, road_z)
            }
            CardinalDirection::NegX => {
                let road_z = snap_to_road_grid(camera_pos.z + lateral_offset);
                Vec3::new(camera_pos.x + distance, 0.0, road_z)
            }
            CardinalDirection::PosZ => {
                let road_x = snap_to_road_grid(camera_pos.x + lateral_offset);
                Vec3::new(road_x, 0.0, camera_pos.z - distance)
            }
            CardinalDirection::NegZ => {
                let road_x = snap_to_road_grid(camera_pos.x + lateral_offset);
                Vec3::new(road_x, 0.0, camera_pos.z + distance)
            }
        };

        if spawn_pos.x < self.city_min
            || spawn_pos.x > self.city_max
            || spawn_pos.z < self.city_min
            || spawn_pos.z > self.city_max
        {
            return;
        }

        let position = spawn_pos + direction.lane_offset_vec();
        let model = kenney::CAR_MODELS[self.rng.random_range(0..kenney::CAR_MODELS.len())];
        let speed = self.rng.random_range(MIN_SPEED..MAX_SPEED);

        let entity = spawn_city_mesh(world, model, position, CAR_SCALE);
        apply_material(world, entity, kenney::MAT_CAR);
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.rotation =
                nalgebra_glm::quat_angle_axis(direction.yaw_angle(), &Vec3::y());
        }
        world.mark_local_transform_dirty(entity);
        world.resources.mesh_render_state.mark_entity_added(entity);

        self.cars.push(TrafficCar {
            entity,
            position,
            direction,
            speed,
        });
    }

    pub fn despawn_all(&mut self, world: &mut World) {
        for car in self.cars.drain(..) {
            world.queue_command(WorldCommand::DespawnRecursive {
                entity: car.entity,
            });
        }
    }
}

fn snap_to_road_grid(value: f32) -> f32 {
    (value / BLOCK_SIZE).round() * BLOCK_SIZE
}

fn check_intersection_turn(car: &mut TrafficCar, rng: &mut impl Rng) {
    let (along_axis, cross_axis) = match car.direction {
        CardinalDirection::PosX | CardinalDirection::NegX => (car.position.x, car.position.z),
        CardinalDirection::PosZ | CardinalDirection::NegZ => (car.position.z, car.position.x),
    };

    let nearest_grid = (along_axis / BLOCK_SIZE).round() * BLOCK_SIZE;
    let distance_to_grid = (along_axis - nearest_grid).abs();

    if distance_to_grid < INTERSECTION_SNAP_THRESHOLD {
        let roll: f32 = rng.random_range(0.0..1.0);
        if roll < 0.25 {
            car.direction = car.direction.turn_left();
            match car.direction {
                CardinalDirection::PosX | CardinalDirection::NegX => {
                    car.position.x = nearest_grid;
                    car.position.z = cross_axis + car.direction.lane_offset_vec().z;
                }
                CardinalDirection::PosZ | CardinalDirection::NegZ => {
                    car.position.z = nearest_grid;
                    car.position.x = cross_axis + car.direction.lane_offset_vec().x;
                }
            }
        } else if roll < 0.50 {
            car.direction = car.direction.turn_right();
            match car.direction {
                CardinalDirection::PosX | CardinalDirection::NegX => {
                    car.position.x = nearest_grid;
                    car.position.z = cross_axis + car.direction.lane_offset_vec().z;
                }
                CardinalDirection::PosZ | CardinalDirection::NegZ => {
                    car.position.z = nearest_grid;
                    car.position.x = cross_axis + car.direction.lane_offset_vec().x;
                }
            }
        }
    }
}
