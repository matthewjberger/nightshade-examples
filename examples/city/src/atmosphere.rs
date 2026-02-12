use nightshade::ecs::generational_registry::registry_entry_by_name_mut;
use nightshade::prelude::*;

fn compute_night_factor(hour: f32) -> f32 {
    if !(6.0..=18.0).contains(&hour) {
        1.0
    } else if hour < 8.0 {
        1.0 - (hour - 6.0) / 2.0
    } else if hour > 16.0 {
        (hour - 16.0) / 2.0
    } else {
        0.0
    }
}

struct EmissiveTarget {
    name: &'static str,
    day_strength: f32,
    night_strength: f32,
    day_factor: [f32; 3],
    night_factor: [f32; 3],
}

const EMISSIVE_TARGETS: &[EmissiveTarget] = &[
    EmissiveTarget {
        name: "WindowLit",
        day_strength: 0.0,
        night_strength: 1.0,
        day_factor: [0.0, 0.0, 0.0],
        night_factor: [1.5, 1.3, 0.6],
    },
    EmissiveTarget {
        name: "ShopfrontLit",
        day_strength: 0.0,
        night_strength: 0.8,
        day_factor: [0.0, 0.0, 0.0],
        night_factor: [1.2, 1.0, 0.5],
    },
    EmissiveTarget {
        name: "NeonRed",
        day_strength: 0.3,
        night_strength: 2.0,
        day_factor: [0.5, 0.05, 0.03],
        night_factor: [3.0, 0.3, 0.2],
    },
    EmissiveTarget {
        name: "NeonBlue",
        day_strength: 0.3,
        night_strength: 2.0,
        day_factor: [0.03, 0.1, 0.5],
        night_factor: [0.2, 0.6, 3.0],
    },
    EmissiveTarget {
        name: "NeonPink",
        day_strength: 0.3,
        night_strength: 2.0,
        day_factor: [0.5, 0.06, 0.2],
        night_factor: [3.0, 0.4, 1.2],
    },
    EmissiveTarget {
        name: "LampGlow",
        day_strength: 0.0,
        night_strength: 3.0,
        day_factor: [0.0, 0.0, 0.0],
        night_factor: [2.0, 1.5, 0.8],
    },
    EmissiveTarget {
        name: "BillboardWhite",
        day_strength: 0.3,
        night_strength: 1.5,
        day_factor: [0.4, 0.4, 0.36],
        night_factor: [2.0, 2.0, 1.8],
    },
    EmissiveTarget {
        name: "BillboardYellow",
        day_strength: 0.3,
        night_strength: 1.5,
        day_factor: [0.5, 0.4, 0.1],
        night_factor: [2.5, 2.0, 0.5],
    },
];

pub fn update_window_emissive(world: &mut World, hour: f32) {
    let night = compute_night_factor(hour);
    let day = 1.0 - night;

    for target in EMISSIVE_TARGETS {
        if let Some(material) =
            registry_entry_by_name_mut(&mut world.resources.material_registry.registry, target.name)
        {
            material.emissive_strength = target.day_strength * day + target.night_strength * night;
            material.emissive_factor = [
                target.day_factor[0] * day + target.night_factor[0] * night,
                target.day_factor[1] * day + target.night_factor[1] * night,
                target.day_factor[2] * day + target.night_factor[2] * night,
            ];
        }
    }
}

const CAMPFIRE_BASE_COLOR: Vec3 = Vec3::new(1.0, 0.65, 0.25);
const CAMPFIRE_BASE_INTENSITY: f32 = 6.0;

pub fn update_campfire_lights(world: &mut World, time: f32) {
    let light_entities: Vec<Entity> = world.query_entities(LIGHT).collect();

    for entity in light_entities {
        let Some(light) = world.get_light(entity) else {
            continue;
        };
        if light.light_type != LightType::Point {
            continue;
        }
        let color_matches = (light.color.x - CAMPFIRE_BASE_COLOR.x).abs() < 0.01
            && (light.color.y - CAMPFIRE_BASE_COLOR.y).abs() < 0.01
            && (light.color.z - CAMPFIRE_BASE_COLOR.z).abs() < 0.01;
        if !color_matches {
            continue;
        }

        let phase = entity.id as f32 * 2.7;
        let flicker1 = (time * 8.0 + phase).sin() * 0.2;
        let flicker2 = (time * 12.5 + phase * 1.3).sin() * 0.15;
        let flicker3 = (time * 17.0 + phase * 0.7).sin() * 0.1;

        if let Some(light) = world.get_light_mut(entity) {
            light.intensity = CAMPFIRE_BASE_INTENSITY + flicker1 + flicker2 + flicker3;
        }
    }
}

const BIRD_COUNT_PER_FLOCK: usize = 30;
const FLOCK_COUNT: usize = 4;
const BIRD_MIN_Y: f32 = 30.0;
const BIRD_MAX_Y: f32 = 80.0;
const BIRD_MIN_SPEED: f32 = 5.0;
const BIRD_MAX_SPEED: f32 = 15.0;
const BIRD_BOUNDS_RADIUS: f32 = 200.0;

const SEPARATION_RADIUS: f32 = 2.0;
const SEPARATION_WEIGHT: f32 = 1.5;
const ALIGNMENT_RADIUS: f32 = 8.0;
const ALIGNMENT_WEIGHT: f32 = 1.0;
const COHESION_RADIUS: f32 = 12.0;
const COHESION_WEIGHT: f32 = 0.8;
const BOUNDS_WEIGHT: f32 = 0.5;
const HEIGHT_WEIGHT: f32 = 0.8;

pub struct BirdFlock {
    positions: Vec<Vec3>,
    velocities: Vec<Vec3>,
    entity: Option<Entity>,
}

pub struct BirdSystem {
    flocks: Vec<BirdFlock>,
    initialized: bool,
}

impl BirdSystem {
    pub fn new() -> Self {
        Self {
            flocks: Vec::new(),
            initialized: false,
        }
    }

    pub fn initialize(&mut self, world: &mut World, city_center: Vec3) {
        if self.initialized {
            return;
        }
        self.initialized = true;

        for flock_index in 0..FLOCK_COUNT {
            let angle_offset = flock_index as f32 * std::f32::consts::TAU / FLOCK_COUNT as f32;
            let flock_center = city_center
                + Vec3::new(
                    angle_offset.cos() * 60.0,
                    40.0 + flock_index as f32 * 10.0,
                    angle_offset.sin() * 60.0,
                );

            let mut positions = Vec::with_capacity(BIRD_COUNT_PER_FLOCK);
            let mut velocities = Vec::with_capacity(BIRD_COUNT_PER_FLOCK);
            let mut instances = Vec::with_capacity(BIRD_COUNT_PER_FLOCK);

            for bird_index in 0..BIRD_COUNT_PER_FLOCK {
                let spread_angle =
                    bird_index as f32 * std::f32::consts::TAU / BIRD_COUNT_PER_FLOCK as f32;
                let spread_radius = 3.0 + (bird_index as f32 * 1.7).sin().abs() * 5.0;
                let position = flock_center
                    + Vec3::new(
                        spread_angle.cos() * spread_radius,
                        (bird_index as f32 * 0.5).sin() * 2.0,
                        spread_angle.sin() * spread_radius,
                    );

                let speed = 8.0 + (bird_index as f32 * 0.3).sin() * 2.0;
                let velocity_angle = angle_offset + std::f32::consts::FRAC_PI_4;
                let velocity = Vec3::new(
                    velocity_angle.cos() * speed,
                    0.0,
                    velocity_angle.sin() * speed,
                );

                positions.push(position);
                velocities.push(velocity);
                instances.push(InstanceTransform::from_translation_scale(
                    position,
                    Vec3::new(0.15, 0.15, 0.3),
                ));
            }

            let entity = spawn_instanced_mesh_with_material(world, "Cone", instances, "Silhouette");
            world.remove_casts_shadow(entity);

            self.flocks.push(BirdFlock {
                positions,
                velocities,
                entity: Some(entity),
            });
        }

        world
            .resources
            .mesh_render_state
            .mark_instanced_meshes_changed();
    }

    pub fn update(&mut self, world: &mut World, delta_time: f32, city_center: Vec3) {
        if !self.initialized {
            return;
        }

        for flock in &mut self.flocks {
            update_boid_flock(
                &mut flock.positions,
                &mut flock.velocities,
                delta_time,
                city_center,
            );

            if let Some(entity) = flock.entity
                && let Some(instanced_mesh) = world.get_instanced_mesh_mut(entity)
            {
                for (bird_index, position) in flock.positions.iter().enumerate() {
                    let velocity = &flock.velocities[bird_index];
                    let speed = nalgebra_glm::length(velocity);
                    let rotation = if speed > 0.01 {
                        let forward = velocity / speed;
                        let yaw = forward.z.atan2(forward.x);
                        let pitch = (-forward.y)
                            .atan2((forward.x * forward.x + forward.z * forward.z).sqrt());
                        nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
                            * nalgebra_glm::quat_angle_axis(pitch, &Vec3::x())
                    } else {
                        nalgebra_glm::quat_identity()
                    };

                    instanced_mesh.set_instance_transform(
                        bird_index,
                        InstanceTransform::new(*position, rotation, Vec3::new(0.15, 0.15, 0.3)),
                    );
                }
            }
        }
    }

    pub fn despawn(&mut self, world: &mut World) {
        for flock in &mut self.flocks {
            if let Some(entity) = flock.entity.take() {
                world.queue_command(nightshade::ecs::world::WorldCommand::DespawnRecursive {
                    entity,
                });
            }
        }
        self.flocks.clear();
        self.initialized = false;

        world
            .resources
            .mesh_render_state
            .mark_instanced_meshes_changed();
    }
}

fn update_boid_flock(
    positions: &mut [Vec3],
    velocities: &mut [Vec3],
    delta_time: f32,
    city_center: Vec3,
) {
    let count = positions.len();
    let mut accelerations = vec![Vec3::zeros(); count];

    for bird_index in 0..count {
        let position = positions[bird_index];
        let velocity = velocities[bird_index];

        let mut separation = Vec3::zeros();
        let mut alignment = Vec3::zeros();
        let mut cohesion_center = Vec3::zeros();
        let mut separation_count = 0;
        let mut alignment_count = 0;
        let mut cohesion_count = 0;

        for other_index in 0..count {
            if other_index == bird_index {
                continue;
            }
            let diff = position - positions[other_index];
            let distance = nalgebra_glm::length(&diff);

            if distance < SEPARATION_RADIUS && distance > 0.001 {
                separation += diff / (distance * distance);
                separation_count += 1;
            }
            if distance < ALIGNMENT_RADIUS {
                alignment += velocities[other_index];
                alignment_count += 1;
            }
            if distance < COHESION_RADIUS {
                cohesion_center += positions[other_index];
                cohesion_count += 1;
            }
        }

        let mut acceleration = Vec3::zeros();

        if separation_count > 0 {
            acceleration += separation * SEPARATION_WEIGHT;
        }
        if alignment_count > 0 {
            let avg_vel = alignment / alignment_count as f32;
            acceleration += (avg_vel - velocity) * ALIGNMENT_WEIGHT;
        }
        if cohesion_count > 0 {
            let center = cohesion_center / cohesion_count as f32;
            acceleration += (center - position) * COHESION_WEIGHT * 0.1;
        }

        let to_center = city_center + Vec3::new(0.0, 50.0, 0.0) - position;
        let horizontal_dist = (to_center.x * to_center.x + to_center.z * to_center.z).sqrt();
        if horizontal_dist > BIRD_BOUNDS_RADIUS {
            let overshoot = (horizontal_dist - BIRD_BOUNDS_RADIUS) / BIRD_BOUNDS_RADIUS;
            acceleration +=
                Vec3::new(to_center.x, 0.0, to_center.z).normalize() * overshoot * BOUNDS_WEIGHT;
        }

        if position.y < BIRD_MIN_Y {
            acceleration.y += (BIRD_MIN_Y - position.y) * HEIGHT_WEIGHT;
        } else if position.y > BIRD_MAX_Y {
            acceleration.y += (BIRD_MAX_Y - position.y) * HEIGHT_WEIGHT;
        }

        accelerations[bird_index] = acceleration;
    }

    for bird_index in 0..count {
        velocities[bird_index] += accelerations[bird_index] * delta_time;

        let speed = nalgebra_glm::length(&velocities[bird_index]);
        if speed > BIRD_MAX_SPEED {
            velocities[bird_index] = velocities[bird_index] / speed * BIRD_MAX_SPEED;
        } else if speed < BIRD_MIN_SPEED && speed > 0.001 {
            velocities[bird_index] = velocities[bird_index] / speed * BIRD_MIN_SPEED;
        }

        positions[bird_index] += velocities[bird_index] * delta_time;
    }
}

pub struct LeafSystem {
    emitter_entities: Vec<Entity>,
    initialized: bool,
}

impl LeafSystem {
    pub fn new() -> Self {
        Self {
            emitter_entities: Vec::new(),
            initialized: false,
        }
    }

    pub fn initialize(&mut self, world: &mut World) {
        if self.initialized {
            return;
        }
        self.initialized = true;

        for _ in 0..3 {
            let entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
            let emitter = ParticleEmitter {
                spawn_rate: 4.0,
                particle_lifetime_min: 4.0,
                particle_lifetime_max: 8.0,
                initial_velocity_min: 0.5,
                initial_velocity_max: 1.5,
                velocity_spread: std::f32::consts::PI,
                gravity: Vec3::new(0.3, -0.5, 0.1),
                drag: 0.1,
                size_start: 0.15,
                size_end: 0.1,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, Vec4::new(0.3, 0.6, 0.15, 0.8)),
                        (0.4, Vec4::new(0.5, 0.55, 0.1, 0.7)),
                        (0.8, Vec4::new(0.6, 0.4, 0.1, 0.5)),
                        (1.0, Vec4::new(0.5, 0.3, 0.1, 0.0)),
                    ],
                },
                shape: EmitterShape::Box {
                    half_extents: Vec3::new(30.0, 5.0, 30.0),
                },
                turbulence_strength: 1.5,
                turbulence_frequency: 0.3,
                ..Default::default()
            };
            world.set_particle_emitter(entity, emitter);
            self.emitter_entities.push(entity);
        }
    }

    pub fn update(&self, world: &mut World, camera_position: Vec3) {
        if !self.initialized {
            return;
        }

        for (emitter_index, &entity) in self.emitter_entities.iter().enumerate() {
            let offset = Vec3::new(
                (emitter_index as f32 * 2.1).sin() * 15.0,
                10.0 + emitter_index as f32 * 3.0,
                (emitter_index as f32 * 1.7).cos() * 15.0,
            );
            if let Some(emitter) = world.get_particle_emitter_mut(entity) {
                emitter.position = camera_position + offset;
            }
        }
    }

    pub fn despawn(&mut self, world: &mut World) {
        for entity in self.emitter_entities.drain(..) {
            world.queue_command(nightshade::ecs::world::WorldCommand::DespawnRecursive { entity });
        }
        self.initialized = false;
    }
}
