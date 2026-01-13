use crate::ecs::{
    ChessWorld, ENGINE_ENTITY, EngineEntity, PIECE, Piece, PieceColor, PieceType, SQUARE_POSITION,
    SquarePosition, WORLD_POSITION, WorldPosition,
};
use nightshade::ecs::prefab::{
    GltfLoadResult, Prefab, PrefabNode, resources::mesh_cache_insert, spawn_prefab,
};
use nightshade::prelude::*;

#[derive(Default)]
pub struct PiecePrefabs {
    pub full_scene: Option<Prefab>,
    pub individual_pieces: Vec<(String, Prefab)>,
}

fn extract_node_as_prefab(node: &PrefabNode, name: &str) -> Prefab {
    Prefab {
        name: name.to_string(),
        root_nodes: vec![node.clone()],
    }
}

fn collect_piece_nodes(nodes: &[PrefabNode]) -> Vec<(String, PrefabNode)> {
    let mut pieces = Vec::new();

    for node in nodes {
        if let Some(node_name) = &node.components.name {
            let name = &node_name.0;
            if name.starts_with("Circle.") && !name.contains('_') {
                pieces.push((name.clone(), node.clone()));
            }
            if name == "Circle" {
                pieces.push((name.clone(), node.clone()));
            }
        }
        let child_pieces = collect_piece_nodes(&node.children);
        pieces.extend(child_pieces);
    }

    pieces
}

pub fn load_piece_prefabs(world: &mut World, result: &GltfLoadResult) -> Option<PiecePrefabs> {
    for (name, (rgba_data, width, height)) in &result.textures {
        tracing::info!("Loading texture '{}': {}x{}", name, width, height);
        world.queue_command(WorldCommand::LoadTexture {
            name: name.clone(),
            rgba_data: rgba_data.clone(),
            width: *width,
            height: *height,
        });
    }

    for (name, mesh) in &result.meshes {
        mesh_cache_insert(&mut world.resources.mesh_cache, name.clone(), mesh.clone());
    }

    let mut prefabs = PiecePrefabs::default();

    if let Some(scene_prefab) = result.prefabs.first() {
        prefabs.full_scene = Some(scene_prefab.clone());

        let piece_nodes = collect_piece_nodes(&scene_prefab.root_nodes);
        for (name, node) in piece_nodes {
            prefabs
                .individual_pieces
                .push((name.clone(), extract_node_as_prefab(&node, &name)));
        }
        tracing::info!(
            "Found {} individual piece prefabs",
            prefabs.individual_pieces.len()
        );
    }

    Some(prefabs)
}

pub fn spawn_full_scene(world: &mut World, prefabs: &PiecePrefabs, scale: f32) -> Option<Entity> {
    let prefab = prefabs.full_scene.as_ref()?;
    let entity = spawn_prefab(world, prefab, nalgebra_glm::vec3(0.0, 0.0, 0.0));

    if let Some(transform) = world.get_local_transform_mut(entity) {
        transform.scale = nalgebra_glm::vec3(scale, scale, scale);
    }
    world.mark_local_transform_dirty(entity);

    Some(entity)
}

pub fn get_prefab_for_piece(
    prefabs: &PiecePrefabs,
    piece_type: PieceType,
    color: PieceColor,
) -> Option<&Prefab> {
    let index = match (color, piece_type) {
        (PieceColor::White, PieceType::Pawn) => 0,
        (PieceColor::White, PieceType::Rook) => 1,
        (PieceColor::White, PieceType::Knight) => 2,
        (PieceColor::White, PieceType::Bishop) => 3,
        (PieceColor::White, PieceType::Queen) => 4,
        (PieceColor::White, PieceType::King) => 5,
        (PieceColor::Black, PieceType::Pawn) => 8,
        (PieceColor::Black, PieceType::Rook) => 9,
        (PieceColor::Black, PieceType::Knight) => 10,
        (PieceColor::Black, PieceType::Bishop) => 11,
        (PieceColor::Black, PieceType::Queen) => 12,
        (PieceColor::Black, PieceType::King) => 13,
    };

    prefabs.individual_pieces.get(index).map(|(_, p)| p)
}

pub fn spawn_piece(
    chess_world: &mut ChessWorld,
    world: &mut World,
    prefabs: &PiecePrefabs,
    piece_type: PieceType,
    color: PieceColor,
    position: SquarePosition,
) -> Option<freecs::Entity> {
    let prefab = get_prefab_for_piece(prefabs, piece_type, color)?;
    let square_size = chess_world.resources.square_size;
    let world_pos = position.to_world_position(square_size);

    let render_entity = spawn_prefab(world, prefab, world_pos);

    let game_entity =
        chess_world.spawn_entities(ENGINE_ENTITY | WORLD_POSITION | SQUARE_POSITION | PIECE, 1)[0];

    chess_world.set_engine_entity(game_entity, EngineEntity(render_entity));
    chess_world.set_world_position(
        game_entity,
        WorldPosition {
            _position: world_pos,
        },
    );
    chess_world.set_square_position(game_entity, position);
    chess_world.set_piece(
        game_entity,
        Piece {
            _piece_type: piece_type,
            _color: color,
        },
    );

    Some(game_entity)
}

pub fn spawn_all_pieces(
    _chess_world: &mut ChessWorld,
    _world: &mut World,
    _prefabs: &PiecePrefabs,
) {
}

pub fn despawn_piece(chess_world: &mut ChessWorld, world: &mut World, entity: freecs::Entity) {
    if let Some(engine_entity) = chess_world.get_engine_entity(entity) {
        world.queue_command(WorldCommand::DespawnRecursive {
            entity: engine_entity.0,
        });
    }
    chess_world.despawn_entities(&[entity]);
}
