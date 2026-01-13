use crate::ecs::{BOARD_SQUARE, BoardSquare, ChessWorld, SQUARE_POSITION, SquarePosition};
use nightshade::prelude::tracing;

pub fn spawn_board_squares(chess_world: &mut ChessWorld) {
    for file in 0..8 {
        for rank in 0..8 {
            let is_light = (file + rank) % 2 == 0;
            let entity = chess_world.spawn_entities(SQUARE_POSITION | BOARD_SQUARE, 1)[0];
            chess_world.set_square_position(entity, SquarePosition::new(file, rank));
            chess_world.set_board_square(
                entity,
                BoardSquare {
                    _is_light: is_light,
                },
            );
        }
    }
    tracing::info!("Spawned 64 board squares");
}
