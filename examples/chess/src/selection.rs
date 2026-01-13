use crate::ecs::{ChessWorld, Entity, PIECE, SQUARE_POSITION, SquarePosition};

pub fn get_piece_at_square(chess_world: &ChessWorld, square: SquarePosition) -> Option<Entity> {
    chess_world
        .query_entities(SQUARE_POSITION | PIECE)
        .find(|&entity| {
            chess_world
                .get_square_position(entity)
                .map(|pos| *pos == square)
                .unwrap_or(false)
        })
}

pub fn clear_selection(chess_world: &mut ChessWorld) {
    let selected: Vec<_> = chess_world.query_selected().collect();
    for entity in selected {
        chess_world.remove_selected(entity);
    }
}
