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
        day_strength: 0.2,
        night_strength: 1.5,
        day_factor: [0.3, 0.03, 0.02],
        night_factor: [1.8, 0.2, 0.12],
    },
    EmissiveTarget {
        name: "NeonBlue",
        day_strength: 0.2,
        night_strength: 1.5,
        day_factor: [0.02, 0.06, 0.3],
        night_factor: [0.12, 0.36, 1.8],
    },
    EmissiveTarget {
        name: "NeonPink",
        day_strength: 0.2,
        night_strength: 1.5,
        day_factor: [0.3, 0.04, 0.12],
        night_factor: [1.8, 0.24, 0.72],
    },
    EmissiveTarget {
        name: "LampGlow",
        day_strength: 0.0,
        night_strength: 2.0,
        day_factor: [0.0, 0.0, 0.0],
        night_factor: [1.5, 1.1, 0.6],
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

const TRAFFIC_CYCLE_DURATION: f32 = 30.0;

struct TrafficLightPhase {
    red: f32,
    yellow: f32,
    green: f32,
}

fn traffic_light_phase(time: f32) -> TrafficLightPhase {
    let cycle_time = time % TRAFFIC_CYCLE_DURATION;
    if cycle_time < 12.0 {
        TrafficLightPhase {
            red: 0.0,
            yellow: 0.0,
            green: 1.0,
        }
    } else if cycle_time < 15.0 {
        TrafficLightPhase {
            red: 0.0,
            yellow: 1.0,
            green: 0.0,
        }
    } else if cycle_time < 27.0 {
        TrafficLightPhase {
            red: 1.0,
            yellow: 0.0,
            green: 0.0,
        }
    } else {
        TrafficLightPhase {
            red: 0.0,
            yellow: 1.0,
            green: 0.0,
        }
    }
}

pub fn update_traffic_lights(world: &mut World, time: f32) {
    let phase = traffic_light_phase(time);

    let inactive_factor = [0.02, 0.02, 0.02];
    let inactive_strength = 0.05;

    if let Some(material) = registry_entry_by_name_mut(
        &mut world.resources.material_registry.registry,
        "TrafficRed",
    ) {
        if phase.red > 0.5 {
            material.emissive_factor = [1.5, 0.05, 0.02];
            material.emissive_strength = 1.0;
        } else {
            material.emissive_factor = inactive_factor;
            material.emissive_strength = inactive_strength;
        }
    }

    if let Some(material) = registry_entry_by_name_mut(
        &mut world.resources.material_registry.registry,
        "TrafficYellow",
    ) {
        if phase.yellow > 0.5 {
            material.emissive_factor = [1.5, 1.2, 0.1];
            material.emissive_strength = 1.0;
        } else {
            material.emissive_factor = inactive_factor;
            material.emissive_strength = inactive_strength;
        }
    }

    if let Some(material) = registry_entry_by_name_mut(
        &mut world.resources.material_registry.registry,
        "TrafficGreen",
    ) {
        if phase.green > 0.5 {
            material.emissive_factor = [0.1, 1.5, 0.15];
            material.emissive_strength = 1.0;
        } else {
            material.emissive_factor = inactive_factor;
            material.emissive_strength = inactive_strength;
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

const NEON_BASE_INTENSITY: f32 = 3.0;

pub fn update_neon_lights(world: &mut World, time: f32) {
    let light_entities: Vec<Entity> = world.query_entities(LIGHT).collect();

    for entity in light_entities {
        let Some(light) = world.get_light(entity) else {
            continue;
        };
        if light.light_type != LightType::Point {
            continue;
        }

        let is_neon = (light.color.x > 0.8 || light.color.z > 0.8) && light.color.y < 0.4;
        if !is_neon {
            continue;
        }

        let phase = entity.id as f32 * std::f32::consts::PI;
        let flicker = (time * 3.0 + phase).sin();
        let multiplier = if flicker > 0.95 {
            0.3
        } else if flicker > 0.9 {
            0.7
        } else {
            1.0
        };

        if let Some(light) = world.get_light_mut(entity) {
            light.intensity = NEON_BASE_INTENSITY * multiplier;
        }
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
