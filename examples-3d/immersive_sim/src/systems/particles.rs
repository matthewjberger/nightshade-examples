use nightshade::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ParticleType {
    Fire,
    Ice,
    Lightning,
    Heal,
    Explosion,
    Magic,
    Blood,
    Sparkle,
}

#[derive(Default)]
pub struct ParticleSystem {
    pub spawn_queue: Vec<ParticleSpawnRequest>,
    pub active_emitters: Vec<Entity>,
}

pub struct ParticleSpawnRequest {
    pub position: Vec3,
    pub particle_type: ParticleType,
    pub count: usize,
}

impl ParticleSystem {
    pub fn spawn_burst(&mut self, position: Vec3, particle_type: ParticleType, count: usize) {
        let capped_count = count.min(50);

        self.spawn_queue.push(ParticleSpawnRequest {
            position,
            particle_type,
            count: capped_count,
        });
    }
}

fn get_particle_color(particle_type: ParticleType) -> Vec3 {
    match particle_type {
        ParticleType::Fire => Vec3::new(1.0, 0.5, 0.1),
        ParticleType::Ice => Vec3::new(0.5, 0.8, 1.0),
        ParticleType::Lightning => Vec3::new(0.9, 0.9, 1.0),
        ParticleType::Heal => Vec3::new(0.3, 1.0, 0.4),
        ParticleType::Explosion => Vec3::new(1.0, 0.6, 0.2),
        ParticleType::Magic => Vec3::new(0.8, 0.3, 1.0),
        ParticleType::Blood => Vec3::new(0.8, 0.1, 0.1),
        ParticleType::Sparkle => Vec3::new(1.0, 1.0, 0.8),
    }
}

fn create_emitter_for_type(
    position: Vec3,
    particle_type: ParticleType,
    count: u32,
) -> ParticleEmitter {
    let color = get_particle_color(particle_type);

    match particle_type {
        ParticleType::Fire => {
            let mut emitter = ParticleEmitter::firework_explosion(position, color, count);
            emitter.initial_velocity_min = 2.0;
            emitter.initial_velocity_max = 4.0;
            emitter.particle_lifetime_min = 0.3;
            emitter.particle_lifetime_max = 0.6;
            emitter.size_start = 0.15;
            emitter.size_end = 0.02;
            emitter.gravity = Vec3::new(0.0, 2.0, 0.0);
            emitter.drag = 0.5;
            emitter.emissive_strength = 8.0;
            emitter
        }
        ParticleType::Ice => {
            let mut emitter = ParticleEmitter::firework_explosion(position, color, count);
            emitter.initial_velocity_min = 1.5;
            emitter.initial_velocity_max = 3.0;
            emitter.particle_lifetime_min = 0.4;
            emitter.particle_lifetime_max = 0.7;
            emitter.size_start = 0.12;
            emitter.size_end = 0.01;
            emitter.gravity = Vec3::new(0.0, -2.0, 0.0);
            emitter.drag = 0.3;
            emitter.emissive_strength = 6.0;
            emitter
        }
        ParticleType::Lightning => {
            let mut emitter = ParticleEmitter::firework_explosion(position, color, count);
            emitter.initial_velocity_min = 6.0;
            emitter.initial_velocity_max = 10.0;
            emitter.particle_lifetime_min = 0.1;
            emitter.particle_lifetime_max = 0.25;
            emitter.size_start = 0.08;
            emitter.size_end = 0.01;
            emitter.gravity = Vec3::new(0.0, 0.0, 0.0);
            emitter.drag = 0.1;
            emitter.emissive_strength = 15.0;
            emitter
        }
        ParticleType::Heal => {
            let mut emitter = ParticleEmitter::firework_explosion(position, color, count);
            emitter.initial_velocity_min = 1.0;
            emitter.initial_velocity_max = 2.0;
            emitter.particle_lifetime_min = 0.5;
            emitter.particle_lifetime_max = 1.0;
            emitter.size_start = 0.1;
            emitter.size_end = 0.02;
            emitter.gravity = Vec3::new(0.0, 1.0, 0.0);
            emitter.drag = 0.4;
            emitter.emissive_strength = 5.0;
            emitter
        }
        ParticleType::Explosion => {
            let mut emitter = ParticleEmitter::firework_explosion(position, color, count);
            emitter.initial_velocity_min = 6.0;
            emitter.initial_velocity_max = 12.0;
            emitter.particle_lifetime_min = 0.3;
            emitter.particle_lifetime_max = 0.6;
            emitter.size_start = 0.25;
            emitter.size_end = 0.05;
            emitter.gravity = Vec3::new(0.0, -5.0, 0.0);
            emitter.drag = 0.2;
            emitter.emissive_strength = 10.0;
            emitter
        }
        ParticleType::Magic => {
            let mut emitter = ParticleEmitter::firework_explosion(position, color, count);
            emitter.initial_velocity_min = 2.0;
            emitter.initial_velocity_max = 4.0;
            emitter.particle_lifetime_min = 0.4;
            emitter.particle_lifetime_max = 0.8;
            emitter.size_start = 0.1;
            emitter.size_end = 0.02;
            emitter.gravity = Vec3::new(0.0, 0.5, 0.0);
            emitter.drag = 0.3;
            emitter.emissive_strength = 8.0;
            emitter
        }
        ParticleType::Blood => {
            let mut emitter = ParticleEmitter::firework_explosion(position, color, count);
            emitter.initial_velocity_min = 3.0;
            emitter.initial_velocity_max = 6.0;
            emitter.particle_lifetime_min = 0.2;
            emitter.particle_lifetime_max = 0.4;
            emitter.size_start = 0.08;
            emitter.size_end = 0.01;
            emitter.gravity = Vec3::new(0.0, -10.0, 0.0);
            emitter.drag = 0.2;
            emitter.emissive_strength = 3.0;
            emitter
        }
        ParticleType::Sparkle => {
            let mut emitter = ParticleEmitter::firework_glitter(position, count);
            emitter.color_gradient = ColorGradient::firework_explosion(color);
            emitter.initial_velocity_min = 0.5;
            emitter.initial_velocity_max = 1.5;
            emitter.particle_lifetime_min = 0.6;
            emitter.particle_lifetime_max = 1.2;
            emitter.size_start = 0.06;
            emitter.size_end = 0.01;
            emitter.gravity = Vec3::new(0.0, 0.3, 0.0);
            emitter.drag = 0.5;
            emitter.emissive_strength = 6.0;
            emitter
        }
    }
}

pub fn update_particles(particle_system: &mut ParticleSystem, world: &mut World, _delta_time: f32) {
    for request in particle_system.spawn_queue.drain(..) {
        let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let emitter = create_emitter_for_type(
            request.position,
            request.particle_type,
            request.count as u32,
        );
        world.core.set_particle_emitter(entity, emitter);
        particle_system.active_emitters.push(entity);
    }

    particle_system.active_emitters.retain(|&entity| {
        if let Some(emitter) = world.core.get_particle_emitter(entity) {
            emitter.enabled
        } else {
            false
        }
    });

    update_particle_emitters(world, _delta_time);
}

pub fn spawn_fireball_effect(particle_system: &mut ParticleSystem, position: Vec3) {
    particle_system.spawn_burst(position, ParticleType::Fire, 20);
}

pub fn spawn_ice_effect(particle_system: &mut ParticleSystem, position: Vec3) {
    particle_system.spawn_burst(position, ParticleType::Ice, 15);
}

pub fn spawn_lightning_effect(particle_system: &mut ParticleSystem, position: Vec3) {
    particle_system.spawn_burst(position, ParticleType::Lightning, 25);
}

pub fn spawn_heal_effect(particle_system: &mut ParticleSystem, position: Vec3) {
    particle_system.spawn_burst(position, ParticleType::Heal, 20);
}

pub fn spawn_explosion_effect(particle_system: &mut ParticleSystem, position: Vec3) {
    particle_system.spawn_burst(position, ParticleType::Explosion, 30);
    particle_system.spawn_burst(position, ParticleType::Fire, 15);
}

pub fn spawn_damage_effect(particle_system: &mut ParticleSystem, position: Vec3) {
    particle_system.spawn_burst(position, ParticleType::Blood, 10);
}

pub fn spawn_magic_effect(particle_system: &mut ParticleSystem, position: Vec3) {
    particle_system.spawn_burst(position, ParticleType::Magic, 20);
}

pub fn spawn_pickup_effect(particle_system: &mut ParticleSystem, position: Vec3) {
    particle_system.spawn_burst(position, ParticleType::Sparkle, 15);
}

pub fn spawn_level_up_effect(particle_system: &mut ParticleSystem, position: Vec3) {
    particle_system.spawn_burst(position, ParticleType::Magic, 30);
    particle_system.spawn_burst(position, ParticleType::Sparkle, 20);
}
