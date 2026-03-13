use crate::ecs::{GameWorld, MovementState};
use nightshade::ecs::particles::components::{
    ColorGradient, EmitterShape, EmitterType, ParticleEmitter,
};
use nightshade::prelude::*;

pub fn spawn_snow_blizzard(world: &mut World) {
    let snow_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

    let snow_gradient = ColorGradient {
        colors: vec![
            (0.0, Vec4::new(1.0, 1.0, 1.0, 0.0)),
            (0.1, Vec4::new(1.0, 1.0, 1.0, 0.9)),
            (0.3, Vec4::new(0.95, 0.97, 1.0, 0.95)),
            (0.7, Vec4::new(0.9, 0.95, 1.0, 0.85)),
            (0.9, Vec4::new(0.85, 0.9, 0.98, 0.5)),
            (1.0, Vec4::new(0.8, 0.85, 0.95, 0.0)),
        ],
    };

    let snow_emitter = ParticleEmitter {
        emitter_type: EmitterType::Smoke,
        shape: EmitterShape::Box {
            half_extents: Vec3::new(60.0, 3.0, 60.0),
        },
        position: Vec3::new(0.0, 35.0, 0.0),
        direction: Vec3::new(0.3, -1.0, 0.2).normalize(),
        spawn_rate: 12000.0,
        burst_count: 0,
        particle_lifetime_min: 6.0,
        particle_lifetime_max: 12.0,
        initial_velocity_min: 2.0,
        initial_velocity_max: 4.5,
        velocity_spread: 0.5,
        gravity: Vec3::new(0.0, -1.5, 0.0),
        drag: 0.08,
        size_start: 0.12,
        size_end: 0.08,
        color_gradient: snow_gradient,
        emissive_strength: 1.2,
        enabled: true,
        accumulated_spawn: 0.0,
        one_shot: false,
        has_fired: false,
        turbulence_strength: 1.8,
        turbulence_frequency: 0.5,

        ..Default::default()
    };

    world.core.set_particle_emitter(snow_entity, snow_emitter);

    let snow_entity_2 = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

    let sparkle_gradient = ColorGradient {
        colors: vec![
            (0.0, Vec4::new(1.0, 1.0, 1.0, 0.0)),
            (0.15, Vec4::new(1.0, 1.0, 1.0, 1.0)),
            (0.5, Vec4::new(0.9, 0.95, 1.0, 0.8)),
            (0.85, Vec4::new(0.85, 0.9, 1.0, 0.4)),
            (1.0, Vec4::new(0.8, 0.85, 0.95, 0.0)),
        ],
    };

    let sparkle_emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Box {
            half_extents: Vec3::new(60.0, 3.0, 60.0),
        },
        position: Vec3::new(0.0, 32.0, 0.0),
        direction: Vec3::new(-0.2, -1.0, 0.1).normalize(),
        spawn_rate: 3000.0,
        burst_count: 0,
        particle_lifetime_min: 5.0,
        particle_lifetime_max: 10.0,
        initial_velocity_min: 1.5,
        initial_velocity_max: 3.5,
        velocity_spread: 0.6,
        gravity: Vec3::new(0.0, -1.8, 0.0),
        drag: 0.08,
        size_start: 0.15,
        size_end: 0.06,
        color_gradient: sparkle_gradient,
        emissive_strength: 5.0,
        enabled: true,
        accumulated_spawn: 0.0,
        one_shot: false,
        has_fired: false,
        turbulence_strength: 2.5,
        turbulence_frequency: 0.8,

        ..Default::default()
    };

    world.core.set_particle_emitter(snow_entity_2, sparkle_emitter);
}

pub fn spawn_footprint_emitter(game_world: &mut GameWorld, world: &mut World) {
    let footprint_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

    let footprint_gradient = ColorGradient {
        colors: vec![
            (0.0, Vec4::new(0.7, 0.75, 0.85, 0.0)),
            (0.1, Vec4::new(0.6, 0.65, 0.75, 0.6)),
            (0.3, Vec4::new(0.55, 0.6, 0.7, 0.5)),
            (0.7, Vec4::new(0.5, 0.55, 0.65, 0.3)),
            (1.0, Vec4::new(0.5, 0.55, 0.65, 0.0)),
        ],
    };

    let footprint_emitter = ParticleEmitter {
        emitter_type: EmitterType::Smoke,
        shape: EmitterShape::Sphere { radius: 0.15 },
        position: Vec3::new(0.0, 0.0, 0.0),
        direction: Vec3::new(0.0, -1.0, 0.0),
        spawn_rate: 8.0,
        burst_count: 0,
        particle_lifetime_min: 4.0,
        particle_lifetime_max: 6.0,
        initial_velocity_min: 0.0,
        initial_velocity_max: 0.02,
        velocity_spread: 0.0,
        gravity: Vec3::new(0.0, -0.5, 0.0),
        drag: 10.0,
        size_start: 0.12,
        size_end: 0.15,
        color_gradient: footprint_gradient,
        emissive_strength: 0.0,
        enabled: false,
        accumulated_spawn: 0.0,
        one_shot: false,
        has_fired: false,
        turbulence_strength: 0.0,
        turbulence_frequency: 0.0,

        ..Default::default()
    };

    world.core.set_particle_emitter(footprint_entity, footprint_emitter);

    game_world.resources.footprint_emitter = Some(freecs::Entity {
        id: footprint_entity.id,
        generation: footprint_entity.generation,
    });
}

pub fn update_footprint_emitter(game_world: &GameWorld, world: &mut World) {
    let Some(footprint_entity) = game_world.resources.footprint_emitter else {
        return;
    };
    let Some(controller_entity) = game_world.resources.controller_entity else {
        return;
    };

    let engine_footprint = nightshade::prelude::Entity {
        id: footprint_entity.id,
        generation: footprint_entity.generation,
    };
    let engine_controller = nightshade::prelude::Entity {
        id: controller_entity.id,
        generation: controller_entity.generation,
    };

    let controller_pos = world
        .core.get_local_transform(engine_controller)
        .map(|t| t.translation)
        .unwrap_or(Vec3::zeros());

    let is_moving = game_world.resources.movement_state != MovementState::Idle;

    if let Some(emitter) = world.core.get_particle_emitter_mut(engine_footprint) {
        emitter.position = Vec3::new(controller_pos.x, controller_pos.y - 0.75, controller_pos.z);
        emitter.enabled = is_moving;
    }
}
