use crate::ecs::ChessWorld;
use crate::pieces::{PiecePrefabs, spawn_piece};
use crate::selection::get_piece_at_square;
use nightshade::prelude::*;

pub fn spawn_piece_system(chess_world: &mut ChessWorld, world: &mut World, prefabs: &PiecePrefabs) {
    if let Some(hovered_square) = chess_world.resources.hovered_square
        && get_piece_at_square(chess_world, hovered_square).is_none()
    {
        let piece_type = chess_world.resources.selected_piece_type;
        let color = chess_world.resources.selected_piece_color;
        spawn_piece(
            chess_world,
            world,
            prefabs,
            piece_type,
            color,
            hovered_square,
        );
        tracing::info!(
            "Spawned {:?} {:?} at {:?}",
            color,
            piece_type,
            hovered_square
        );
    }
}
