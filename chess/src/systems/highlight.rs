use crate::ecs::{ChessWorld, SquarePosition};
use nightshade::ecs::asset_id::MaterialId;
use nightshade::ecs::material::components::Material;
use nightshade::prelude::*;
use std::collections::HashSet;

pub fn spawn_move_indicators(
    chess_world: &mut ChessWorld,
    world: &mut World,
    valid_moves: &HashSet<SquarePosition>,
    square_size: f32,
) {
    let sphere_radius = square_size * 0.25;

    for square in valid_moves {
        let world_pos = square.to_world_position(square_size);
        let sphere_pos = nalgebra_glm::vec3(world_pos.x, sphere_radius, world_pos.z);

        let entity = spawn_mesh_at(
            world,
            "Sphere",
            sphere_pos,
            nalgebra_glm::vec3(sphere_radius, sphere_radius, sphere_radius),
        );

        let material_name = format!("move_indicator_{}", entity.id);
        let (index, generation) = world.resources.material_registry.registry.insert(
            material_name.clone(),
            Material {
                base_color: [0.2, 0.8, 0.2, 1.0],
                emissive_factor: [0.1, 0.3, 0.1],
                roughness: 0.3,
                metallic: 0.0,
                ..Default::default()
            },
        );
        let material_id = MaterialId { index, generation };
        world.set_material_ref(
            entity,
            nightshade::ecs::material::components::MaterialRef::with_id(material_name, material_id),
        );

        chess_world.resources.move_indicator_entities.push(entity);
    }
}

pub fn despawn_move_indicators(chess_world: &mut ChessWorld, world: &mut World) {
    for entity in chess_world.resources.move_indicator_entities.drain(..) {
        world.queue_command(WorldCommand::DespawnRecursive { entity });
    }
}

pub fn highlight_system(_chess_world: &ChessWorld, _world: &mut World) {}
