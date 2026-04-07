use nightshade::prelude::*;

pub(in crate::systems::exhibits) fn spawn_visual_cube(
    world: &mut World,
    position: Vec3,
    scale: Vec3,
    material: nightshade::ecs::material::components::Material,
    name: String,
) {
    crate::systems::spawn::spawn_visual_entity_with_shadow(world, position, scale, "Cube", material, name);
}
