use nightshade::prelude::*;
use std::collections::HashSet;

pub use freecs::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PieceColor {
    #[default]
    White,
    Black,
}

impl PieceColor {
    pub fn opposite(self) -> PieceColor {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PieceType {
    #[default]
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl PieceType {
    pub fn next(self) -> Self {
        match self {
            PieceType::Pawn => PieceType::Rook,
            PieceType::Rook => PieceType::Knight,
            PieceType::Knight => PieceType::Bishop,
            PieceType::Bishop => PieceType::Queen,
            PieceType::Queen => PieceType::King,
            PieceType::King => PieceType::Pawn,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            PieceType::Pawn => PieceType::King,
            PieceType::Rook => PieceType::Pawn,
            PieceType::Knight => PieceType::Rook,
            PieceType::Bishop => PieceType::Knight,
            PieceType::Queen => PieceType::Bishop,
            PieceType::King => PieceType::Queen,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SquarePosition {
    pub file: i32,
    pub rank: i32,
}

impl SquarePosition {
    pub fn new(file: i32, rank: i32) -> Self {
        Self { file, rank }
    }

    pub fn is_valid(&self) -> bool {
        self.file >= 0 && self.file < 8 && self.rank >= 0 && self.rank < 8
    }

    pub fn to_world_position(self, square_size: f32) -> Vec3 {
        nalgebra_glm::vec3(
            self.file as f32 * square_size,
            0.0,
            self.rank as f32 * square_size,
        )
    }

    pub fn from_world_position(world_pos: Vec3, square_size: f32) -> Self {
        Self {
            file: (world_pos.x / square_size).round() as i32,
            rank: (world_pos.z / square_size).round() as i32,
        }
    }
}

freecs::ecs! {
    ChessWorld {
        engine_entity: EngineEntity => ENGINE_ENTITY,
        world_position: WorldPosition => WORLD_POSITION,
        square_position: SquarePosition => SQUARE_POSITION,
        piece: Piece => PIECE,
        dragging: Dragging => DRAGGING,
        board_square: BoardSquare => BOARD_SQUARE,
    }
    Tags {
        selected => SELECTED,
    }
    ChessResources {
        square_size: f32,
        hovered_square: Option<SquarePosition>,
        hovered_engine_entity: Option<nightshade::prelude::Entity>,
        dragged_engine_entity: Option<nightshade::prelude::Entity>,
        selected_piece_type: PieceType,
        selected_piece_color: PieceColor,
        drag_offset: Vec3,
        drag_start_pos: Vec3,
        drag_start_square: SquarePosition,
        dragged_piece_type: Option<PieceType>,
        dragged_piece_color: Option<PieceColor>,
        is_dragging: bool,
        valid_moves: HashSet<SquarePosition>,
        move_indicator_entities: Vec<nightshade::prelude::Entity>,
        closest_valid_square: Option<SquarePosition>,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EngineEntity(pub nightshade::prelude::Entity);

#[derive(Debug, Clone, Copy, Default)]
pub struct WorldPosition {
    pub _position: Vec3,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Piece {
    pub _piece_type: PieceType,
    pub _color: PieceColor,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Dragging {
    pub _start_position: Vec3,
    pub _start_square: SquarePosition,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BoardSquare {
    pub _is_light: bool,
}

use std::collections::HashMap;

pub fn calculate_valid_moves(
    piece_type: PieceType,
    color: PieceColor,
    from: SquarePosition,
    occupied_squares: &HashMap<SquarePosition, PieceColor>,
) -> HashSet<SquarePosition> {
    let mut valid_moves = HashSet::new();
    let enemy_color = color.opposite();

    match piece_type {
        PieceType::Pawn => {
            let direction = match color {
                PieceColor::White => 1,
                PieceColor::Black => -1,
            };
            let start_rank = match color {
                PieceColor::White => 1,
                PieceColor::Black => 6,
            };

            let forward = SquarePosition::new(from.file, from.rank + direction);
            if forward.is_valid() && !occupied_squares.contains_key(&forward) {
                valid_moves.insert(forward);

                if from.rank == start_rank {
                    let double_forward = SquarePosition::new(from.file, from.rank + 2 * direction);
                    if double_forward.is_valid() && !occupied_squares.contains_key(&double_forward)
                    {
                        valid_moves.insert(double_forward);
                    }
                }
            }

            for file_offset in [-1, 1] {
                let capture = SquarePosition::new(from.file + file_offset, from.rank + direction);
                if capture.is_valid()
                    && let Some(&target_color) = occupied_squares.get(&capture)
                    && target_color == enemy_color
                {
                    valid_moves.insert(capture);
                }
            }
        }
        PieceType::Rook => {
            add_line_moves(&mut valid_moves, from, 1, 0, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, -1, 0, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, 0, 1, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, 0, -1, color, occupied_squares);
        }
        PieceType::Knight => {
            let offsets = [
                (1, 2),
                (2, 1),
                (2, -1),
                (1, -2),
                (-1, -2),
                (-2, -1),
                (-2, 1),
                (-1, 2),
            ];
            for (file_offset, rank_offset) in offsets {
                let target = SquarePosition::new(from.file + file_offset, from.rank + rank_offset);
                if target.is_valid() {
                    match occupied_squares.get(&target) {
                        Some(&target_color) if target_color == color => {}
                        _ => {
                            valid_moves.insert(target);
                        }
                    }
                }
            }
        }
        PieceType::Bishop => {
            add_line_moves(&mut valid_moves, from, 1, 1, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, 1, -1, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, -1, 1, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, -1, -1, color, occupied_squares);
        }
        PieceType::Queen => {
            add_line_moves(&mut valid_moves, from, 1, 0, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, -1, 0, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, 0, 1, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, 0, -1, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, 1, 1, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, 1, -1, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, -1, 1, color, occupied_squares);
            add_line_moves(&mut valid_moves, from, -1, -1, color, occupied_squares);
        }
        PieceType::King => {
            for file_offset in -1..=1 {
                for rank_offset in -1..=1 {
                    if file_offset == 0 && rank_offset == 0 {
                        continue;
                    }
                    let target =
                        SquarePosition::new(from.file + file_offset, from.rank + rank_offset);
                    if target.is_valid() {
                        match occupied_squares.get(&target) {
                            Some(&target_color) if target_color == color => {}
                            _ => {
                                valid_moves.insert(target);
                            }
                        }
                    }
                }
            }
        }
    }

    valid_moves
}

fn add_line_moves(
    valid_moves: &mut HashSet<SquarePosition>,
    from: SquarePosition,
    file_dir: i32,
    rank_dir: i32,
    moving_color: PieceColor,
    occupied_squares: &HashMap<SquarePosition, PieceColor>,
) {
    let mut current = from;
    loop {
        current = SquarePosition::new(current.file + file_dir, current.rank + rank_dir);
        if !current.is_valid() {
            break;
        }
        match occupied_squares.get(&current) {
            Some(&target_color) => {
                if target_color != moving_color {
                    valid_moves.insert(current);
                }
                break;
            }
            None => {
                valid_moves.insert(current);
            }
        }
    }
}
