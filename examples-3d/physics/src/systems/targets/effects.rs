use nightshade::ecs::particles::components::{ColorGradient, EmitterShape, ParticleEmitter};
use nightshade::prelude::*;

pub(super) fn spawn_pop_effect(world: &mut World, position: Vec3, color: Vec3) -> Entity {
    let entity = world.spawn_entities(
        PARTICLE_EMITTER | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
        1,
    )[0];

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
    }

    let mut emitter = ParticleEmitter::firework_explosion(position, color, 30);
    emitter.shape = EmitterShape::Sphere { radius: 0.3 };
    emitter.one_shot = true;
    emitter.burst_count = 30;
    emitter.initial_velocity_min = 3.0;
    emitter.initial_velocity_max = 8.0;
    emitter.velocity_spread = 1.0;
    emitter.particle_lifetime_min = 0.4;
    emitter.particle_lifetime_max = 0.8;
    emitter.size_start = 0.15;
    emitter.size_end = 0.02;
    emitter.gravity = nalgebra_glm::vec3(0.0, -5.0, 0.0);
    emitter.drag = 0.3;
    emitter.emissive_strength = 8.0;
    emitter.color_gradient = ColorGradient {
        colors: vec![
            (0.0, nalgebra_glm::vec4(color.x, color.y, color.z, 1.0)),
            (
                0.3,
                nalgebra_glm::vec4(color.x * 1.2, color.y * 1.2, color.z * 1.2, 0.9),
            ),
            (
                0.7,
                nalgebra_glm::vec4(color.x * 0.5, color.y * 0.5, color.z * 0.5, 0.5),
            ),
            (1.0, nalgebra_glm::vec4(0.1, 0.1, 0.1, 0.0)),
        ],
    };

    world.core.set_particle_emitter(entity, emitter);

    entity
}
