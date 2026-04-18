use nightshade::tui::prelude::*;

const BOARD_SIZE: usize = 8;
const CELL_WIDTH: usize = 3;

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 300;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 10000;

const AI_DEPTH: i32 = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PieceColor {
    White,
    Black,
}

impl PieceColor {
    fn opponent(self) -> Self {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Piece {
    kind: PieceKind,
    color: PieceColor,
}

impl Piece {
    fn character(self) -> char {
        let base = match self.kind {
            PieceKind::Pawn => 'P',
            PieceKind::Knight => 'N',
            PieceKind::Bishop => 'B',
            PieceKind::Rook => 'R',
            PieceKind::Queen => 'Q',
            PieceKind::King => 'K',
        };
        match self.color {
            PieceColor::White => base,
            PieceColor::Black => base.to_ascii_lowercase(),
        }
    }

    fn value(self) -> i32 {
        match self.kind {
            PieceKind::Pawn => PAWN_VALUE,
            PieceKind::Knight => KNIGHT_VALUE,
            PieceKind::Bishop => BISHOP_VALUE,
            PieceKind::Rook => ROOK_VALUE,
            PieceKind::Queen => QUEEN_VALUE,
            PieceKind::King => KING_VALUE,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ChessMove {
    from_column: usize,
    from_row: usize,
    to_column: usize,
    to_row: usize,
    promotion: Option<PieceKind>,
}

#[derive(Clone)]
struct Board {
    squares: [[Option<Piece>; BOARD_SIZE]; BOARD_SIZE],
    white_king_moved: bool,
    black_king_moved: bool,
    white_rook_a_moved: bool,
    white_rook_h_moved: bool,
    black_rook_a_moved: bool,
    black_rook_h_moved: bool,
    en_passant_column: Option<usize>,
    en_passant_row: Option<usize>,
}

impl Board {
    fn new() -> Self {
        let mut squares = [[None; BOARD_SIZE]; BOARD_SIZE];

        squares[0][0] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        squares[0][1] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::Black,
        });
        squares[0][2] = Some(Piece {
            kind: PieceKind::Bishop,
            color: PieceColor::Black,
        });
        squares[0][3] = Some(Piece {
            kind: PieceKind::Queen,
            color: PieceColor::Black,
        });
        squares[0][4] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::Black,
        });
        squares[0][5] = Some(Piece {
            kind: PieceKind::Bishop,
            color: PieceColor::Black,
        });
        squares[0][6] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::Black,
        });
        squares[0][7] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });

        for slot in squares[1].iter_mut() {
            *slot = Some(Piece {
                kind: PieceKind::Pawn,
                color: PieceColor::Black,
            });
        }

        for slot in squares[6].iter_mut() {
            *slot = Some(Piece {
                kind: PieceKind::Pawn,
                color: PieceColor::White,
            });
        }

        squares[7][0] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        squares[7][1] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        squares[7][2] = Some(Piece {
            kind: PieceKind::Bishop,
            color: PieceColor::White,
        });
        squares[7][3] = Some(Piece {
            kind: PieceKind::Queen,
            color: PieceColor::White,
        });
        squares[7][4] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        squares[7][5] = Some(Piece {
            kind: PieceKind::Bishop,
            color: PieceColor::White,
        });
        squares[7][6] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        squares[7][7] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });

        Self {
            squares,
            white_king_moved: false,
            black_king_moved: false,
            white_rook_a_moved: false,
            white_rook_h_moved: false,
            black_rook_a_moved: false,
            black_rook_h_moved: false,
            en_passant_column: None,
            en_passant_row: None,
        }
    }

    fn get(&self, column: usize, row: usize) -> Option<Piece> {
        self.squares[row][column]
    }

    fn find_king(&self, color: PieceColor) -> Option<(usize, usize)> {
        for row in 0..BOARD_SIZE {
            for column in 0..BOARD_SIZE {
                if let Some(piece) = self.squares[row][column]
                    && piece.kind == PieceKind::King
                    && piece.color == color
                {
                    return Some((column, row));
                }
            }
        }
        None
    }

    fn is_square_attacked_by(
        &self,
        target_column: usize,
        target_row: usize,
        attacker_color: PieceColor,
    ) -> bool {
        for row in 0..BOARD_SIZE {
            for column in 0..BOARD_SIZE {
                if let Some(piece) = self.squares[row][column] {
                    if piece.color != attacker_color {
                        continue;
                    }
                    if self.can_piece_attack(column, row, target_column, target_row, piece) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn can_piece_attack(
        &self,
        from_column: usize,
        from_row: usize,
        to_column: usize,
        to_row: usize,
        piece: Piece,
    ) -> bool {
        let delta_column = to_column as i32 - from_column as i32;
        let delta_row = to_row as i32 - from_row as i32;

        match piece.kind {
            PieceKind::Pawn => {
                let direction: i32 = match piece.color {
                    PieceColor::White => -1,
                    PieceColor::Black => 1,
                };
                delta_row == direction && (delta_column == 1 || delta_column == -1)
            }
            PieceKind::Knight => {
                let abs_column = delta_column.abs();
                let abs_row = delta_row.abs();
                (abs_column == 2 && abs_row == 1) || (abs_column == 1 && abs_row == 2)
            }
            PieceKind::Bishop => {
                if delta_column.abs() != delta_row.abs() || delta_column == 0 {
                    return false;
                }
                self.is_path_clear(from_column, from_row, to_column, to_row)
            }
            PieceKind::Rook => {
                if delta_column != 0 && delta_row != 0 {
                    return false;
                }
                if delta_column == 0 && delta_row == 0 {
                    return false;
                }
                self.is_path_clear(from_column, from_row, to_column, to_row)
            }
            PieceKind::Queen => {
                let is_diagonal = delta_column.abs() == delta_row.abs() && delta_column != 0;
                let is_straight = (delta_column == 0) != (delta_row == 0);
                if !is_diagonal && !is_straight {
                    return false;
                }
                self.is_path_clear(from_column, from_row, to_column, to_row)
            }
            PieceKind::King => {
                delta_column.abs() <= 1
                    && delta_row.abs() <= 1
                    && (delta_column != 0 || delta_row != 0)
            }
        }
    }

    fn is_path_clear(
        &self,
        from_column: usize,
        from_row: usize,
        to_column: usize,
        to_row: usize,
    ) -> bool {
        let step_column = (to_column as i32 - from_column as i32).signum();
        let step_row = (to_row as i32 - from_row as i32).signum();

        let mut current_column = from_column as i32 + step_column;
        let mut current_row = from_row as i32 + step_row;

        while current_column != to_column as i32 || current_row != to_row as i32 {
            if self.squares[current_row as usize][current_column as usize].is_some() {
                return false;
            }
            current_column += step_column;
            current_row += step_row;
        }
        true
    }

    fn is_in_check(&self, color: PieceColor) -> bool {
        if let Some((king_column, king_row)) = self.find_king(color) {
            self.is_square_attacked_by(king_column, king_row, color.opponent())
        } else {
            false
        }
    }

    fn apply_move(&self, chess_move: ChessMove) -> Board {
        let mut new_board = self.clone();
        let piece = new_board.squares[chess_move.from_row][chess_move.from_column];

        new_board.squares[chess_move.to_row][chess_move.to_column] = piece;
        new_board.squares[chess_move.from_row][chess_move.from_column] = None;

        new_board.en_passant_column = None;
        new_board.en_passant_row = None;

        if let Some(moved_piece) = piece {
            match moved_piece.kind {
                PieceKind::Pawn => {
                    let delta_row = chess_move.to_row as i32 - chess_move.from_row as i32;

                    if delta_row.abs() == 2 {
                        new_board.en_passant_column = Some(chess_move.from_column);
                        new_board.en_passant_row = Some(
                            (chess_move.from_row as i32 + chess_move.to_row as i32) as usize / 2,
                        );
                    }

                    if let (Some(en_passant_column), Some(en_passant_row)) =
                        (self.en_passant_column, self.en_passant_row)
                        && chess_move.to_column == en_passant_column
                        && chess_move.to_row == en_passant_row
                    {
                        new_board.squares[chess_move.from_row][chess_move.to_column] = None;
                    }

                    if let Some(promotion_kind) = chess_move.promotion {
                        new_board.squares[chess_move.to_row][chess_move.to_column] = Some(Piece {
                            kind: promotion_kind,
                            color: moved_piece.color,
                        });
                    }
                }
                PieceKind::King => {
                    match moved_piece.color {
                        PieceColor::White => new_board.white_king_moved = true,
                        PieceColor::Black => new_board.black_king_moved = true,
                    }

                    let delta_column = chess_move.to_column as i32 - chess_move.from_column as i32;
                    if delta_column.abs() == 2 {
                        if delta_column > 0 {
                            let rook = new_board.squares[chess_move.from_row][7];
                            new_board.squares[chess_move.from_row][7] = None;
                            new_board.squares[chess_move.from_row][5] = rook;
                        } else {
                            let rook = new_board.squares[chess_move.from_row][0];
                            new_board.squares[chess_move.from_row][0] = None;
                            new_board.squares[chess_move.from_row][3] = rook;
                        }
                    }
                }
                PieceKind::Rook => {
                    if chess_move.from_row == 7 && chess_move.from_column == 0 {
                        new_board.white_rook_a_moved = true;
                    }
                    if chess_move.from_row == 7 && chess_move.from_column == 7 {
                        new_board.white_rook_h_moved = true;
                    }
                    if chess_move.from_row == 0 && chess_move.from_column == 0 {
                        new_board.black_rook_a_moved = true;
                    }
                    if chess_move.from_row == 0 && chess_move.from_column == 7 {
                        new_board.black_rook_h_moved = true;
                    }
                }
                _ => {}
            }
        }

        new_board
    }

    fn generate_pseudo_legal_moves(&self, color: PieceColor) -> Vec<ChessMove> {
        let mut moves = Vec::new();

        for row in 0..BOARD_SIZE {
            for column in 0..BOARD_SIZE {
                if let Some(piece) = self.squares[row][column] {
                    if piece.color != color {
                        continue;
                    }
                    self.generate_piece_moves(column, row, piece, &mut moves);
                }
            }
        }

        moves
    }

    fn generate_piece_moves(
        &self,
        from_column: usize,
        from_row: usize,
        piece: Piece,
        moves: &mut Vec<ChessMove>,
    ) {
        match piece.kind {
            PieceKind::Pawn => self.generate_pawn_moves(from_column, from_row, piece.color, moves),
            PieceKind::Knight => {
                self.generate_knight_moves(from_column, from_row, piece.color, moves)
            }
            PieceKind::Bishop => self.generate_sliding_moves(
                from_column,
                from_row,
                piece.color,
                &[(-1, -1), (-1, 1), (1, -1), (1, 1)],
                moves,
            ),
            PieceKind::Rook => self.generate_sliding_moves(
                from_column,
                from_row,
                piece.color,
                &[(-1, 0), (1, 0), (0, -1), (0, 1)],
                moves,
            ),
            PieceKind::Queen => self.generate_sliding_moves(
                from_column,
                from_row,
                piece.color,
                &[
                    (-1, -1),
                    (-1, 1),
                    (1, -1),
                    (1, 1),
                    (-1, 0),
                    (1, 0),
                    (0, -1),
                    (0, 1),
                ],
                moves,
            ),
            PieceKind::King => self.generate_king_moves(from_column, from_row, piece.color, moves),
        }
    }

    fn generate_pawn_moves(
        &self,
        from_column: usize,
        from_row: usize,
        color: PieceColor,
        moves: &mut Vec<ChessMove>,
    ) {
        let direction: i32 = match color {
            PieceColor::White => -1,
            PieceColor::Black => 1,
        };
        let start_row = match color {
            PieceColor::White => 6,
            PieceColor::Black => 1,
        };
        let promotion_row = match color {
            PieceColor::White => 0,
            PieceColor::Black => 7,
        };

        let forward_row = from_row as i32 + direction;
        if forward_row >= 0 && forward_row < BOARD_SIZE as i32 {
            let forward_row_usize = forward_row as usize;
            if self.squares[forward_row_usize][from_column].is_none() {
                if forward_row_usize == promotion_row {
                    for promotion_kind in [
                        PieceKind::Queen,
                        PieceKind::Rook,
                        PieceKind::Bishop,
                        PieceKind::Knight,
                    ] {
                        moves.push(ChessMove {
                            from_column,
                            from_row,
                            to_column: from_column,
                            to_row: forward_row_usize,
                            promotion: Some(promotion_kind),
                        });
                    }
                } else {
                    moves.push(ChessMove {
                        from_column,
                        from_row,
                        to_column: from_column,
                        to_row: forward_row_usize,
                        promotion: None,
                    });

                    if from_row == start_row {
                        let double_row = (from_row as i32 + direction * 2) as usize;
                        if self.squares[double_row][from_column].is_none() {
                            moves.push(ChessMove {
                                from_column,
                                from_row,
                                to_column: from_column,
                                to_row: double_row,
                                promotion: None,
                            });
                        }
                    }
                }
            }

            for capture_delta in [-1i32, 1] {
                let capture_column = from_column as i32 + capture_delta;
                if capture_column < 0 || capture_column >= BOARD_SIZE as i32 {
                    continue;
                }
                let capture_column_usize = capture_column as usize;

                let has_enemy = self.squares[forward_row_usize][capture_column_usize]
                    .is_some_and(|target| target.color != color);

                let is_en_passant = self.en_passant_column == Some(capture_column_usize)
                    && self.en_passant_row == Some(forward_row_usize);

                if has_enemy || is_en_passant {
                    if forward_row_usize == promotion_row {
                        for promotion_kind in [
                            PieceKind::Queen,
                            PieceKind::Rook,
                            PieceKind::Bishop,
                            PieceKind::Knight,
                        ] {
                            moves.push(ChessMove {
                                from_column,
                                from_row,
                                to_column: capture_column_usize,
                                to_row: forward_row_usize,
                                promotion: Some(promotion_kind),
                            });
                        }
                    } else {
                        moves.push(ChessMove {
                            from_column,
                            from_row,
                            to_column: capture_column_usize,
                            to_row: forward_row_usize,
                            promotion: None,
                        });
                    }
                }
            }
        }
    }

    fn generate_knight_moves(
        &self,
        from_column: usize,
        from_row: usize,
        color: PieceColor,
        moves: &mut Vec<ChessMove>,
    ) {
        let offsets = [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ];

        for (delta_column, delta_row) in offsets {
            let to_column = from_column as i32 + delta_column;
            let to_row = from_row as i32 + delta_row;

            if to_column < 0
                || to_column >= BOARD_SIZE as i32
                || to_row < 0
                || to_row >= BOARD_SIZE as i32
            {
                continue;
            }

            let to_column_usize = to_column as usize;
            let to_row_usize = to_row as usize;

            if self.squares[to_row_usize][to_column_usize]
                .is_some_and(|target| target.color == color)
            {
                continue;
            }

            moves.push(ChessMove {
                from_column,
                from_row,
                to_column: to_column_usize,
                to_row: to_row_usize,
                promotion: None,
            });
        }
    }

    fn generate_sliding_moves(
        &self,
        from_column: usize,
        from_row: usize,
        color: PieceColor,
        directions: &[(i32, i32)],
        moves: &mut Vec<ChessMove>,
    ) {
        for &(step_column, step_row) in directions {
            let mut current_column = from_column as i32 + step_column;
            let mut current_row = from_row as i32 + step_row;

            while current_column >= 0
                && current_column < BOARD_SIZE as i32
                && current_row >= 0
                && current_row < BOARD_SIZE as i32
            {
                let target_column = current_column as usize;
                let target_row = current_row as usize;

                if let Some(target) = self.squares[target_row][target_column] {
                    if target.color != color {
                        moves.push(ChessMove {
                            from_column,
                            from_row,
                            to_column: target_column,
                            to_row: target_row,
                            promotion: None,
                        });
                    }
                    break;
                }

                moves.push(ChessMove {
                    from_column,
                    from_row,
                    to_column: target_column,
                    to_row: target_row,
                    promotion: None,
                });

                current_column += step_column;
                current_row += step_row;
            }
        }
    }

    fn generate_king_moves(
        &self,
        from_column: usize,
        from_row: usize,
        color: PieceColor,
        moves: &mut Vec<ChessMove>,
    ) {
        for delta_row in -1i32..=1 {
            for delta_column in -1i32..=1 {
                if delta_row == 0 && delta_column == 0 {
                    continue;
                }

                let to_column = from_column as i32 + delta_column;
                let to_row = from_row as i32 + delta_row;

                if to_column < 0
                    || to_column >= BOARD_SIZE as i32
                    || to_row < 0
                    || to_row >= BOARD_SIZE as i32
                {
                    continue;
                }

                let to_column_usize = to_column as usize;
                let to_row_usize = to_row as usize;

                if self.squares[to_row_usize][to_column_usize]
                    .is_some_and(|target| target.color == color)
                {
                    continue;
                }

                moves.push(ChessMove {
                    from_column,
                    from_row,
                    to_column: to_column_usize,
                    to_row: to_row_usize,
                    promotion: None,
                });
            }
        }

        let (king_moved, rook_a_moved, rook_h_moved, king_row) = match color {
            PieceColor::White => (
                self.white_king_moved,
                self.white_rook_a_moved,
                self.white_rook_h_moved,
                7,
            ),
            PieceColor::Black => (
                self.black_king_moved,
                self.black_rook_a_moved,
                self.black_rook_h_moved,
                0,
            ),
        };

        if !king_moved && from_row == king_row && from_column == 4 {
            if !rook_h_moved
                && self.squares[king_row][5].is_none()
                && self.squares[king_row][6].is_none()
                && self.squares[king_row][7]
                    .is_some_and(|piece| piece.kind == PieceKind::Rook && piece.color == color)
                && !self.is_square_attacked_by(4, king_row, color.opponent())
                && !self.is_square_attacked_by(5, king_row, color.opponent())
                && !self.is_square_attacked_by(6, king_row, color.opponent())
            {
                moves.push(ChessMove {
                    from_column: 4,
                    from_row: king_row,
                    to_column: 6,
                    to_row: king_row,
                    promotion: None,
                });
            }

            if !rook_a_moved
                && self.squares[king_row][3].is_none()
                && self.squares[king_row][2].is_none()
                && self.squares[king_row][1].is_none()
                && self.squares[king_row][0]
                    .is_some_and(|piece| piece.kind == PieceKind::Rook && piece.color == color)
                && !self.is_square_attacked_by(4, king_row, color.opponent())
                && !self.is_square_attacked_by(3, king_row, color.opponent())
                && !self.is_square_attacked_by(2, king_row, color.opponent())
            {
                moves.push(ChessMove {
                    from_column: 4,
                    from_row: king_row,
                    to_column: 2,
                    to_row: king_row,
                    promotion: None,
                });
            }
        }
    }

    fn generate_legal_moves(&self, color: PieceColor) -> Vec<ChessMove> {
        let pseudo_legal = self.generate_pseudo_legal_moves(color);
        let mut legal = Vec::new();

        for chess_move in pseudo_legal {
            let new_board = self.apply_move(chess_move);
            if !new_board.is_in_check(color) {
                legal.push(chess_move);
            }
        }

        legal
    }

    fn is_checkmate(&self, color: PieceColor) -> bool {
        self.is_in_check(color) && self.generate_legal_moves(color).is_empty()
    }

    fn is_stalemate(&self, color: PieceColor) -> bool {
        !self.is_in_check(color) && self.generate_legal_moves(color).is_empty()
    }
}

fn position_bonus(piece: Piece, column: usize, row: usize) -> i32 {
    let center_column_distance = ((column as i32) - 3).abs().min(((column as i32) - 4).abs());
    let center_row_distance = ((row as i32) - 3).abs().min(((row as i32) - 4).abs());
    let center_bonus = (3 - center_column_distance.max(center_row_distance)) * 5;

    match piece.kind {
        PieceKind::Pawn => {
            let advance_bonus = match piece.color {
                PieceColor::White => (6 - row as i32) * 8,
                PieceColor::Black => (row as i32 - 1) * 8,
            };
            center_bonus + advance_bonus
        }
        PieceKind::Knight => center_bonus * 3,
        PieceKind::Bishop => center_bonus * 2,
        PieceKind::Rook => 0,
        PieceKind::Queen => center_bonus,
        PieceKind::King => match piece.color {
            PieceColor::White => {
                if row == 7 {
                    10
                } else {
                    -20
                }
            }
            PieceColor::Black => {
                if row == 0 {
                    10
                } else {
                    -20
                }
            }
        },
    }
}

fn evaluate_board(board: &Board) -> i32 {
    let mut score = 0;

    for row in 0..BOARD_SIZE {
        for column in 0..BOARD_SIZE {
            if let Some(piece) = board.squares[row][column] {
                let material = piece.value();
                let positional = position_bonus(piece, column, row);
                let total = material + positional;
                match piece.color {
                    PieceColor::White => score -= total,
                    PieceColor::Black => score += total,
                }
            }
        }
    }

    score
}

fn minimax(board: &Board, depth: i32, mut alpha: i32, mut beta: i32, maximizing: bool) -> i32 {
    let color = if maximizing {
        PieceColor::Black
    } else {
        PieceColor::White
    };

    if depth == 0 {
        return evaluate_board(board);
    }

    let legal_moves = board.generate_legal_moves(color);

    if legal_moves.is_empty() {
        if board.is_in_check(color) {
            return if maximizing {
                -100000 + (AI_DEPTH - depth)
            } else {
                100000 - (AI_DEPTH - depth)
            };
        }
        return 0;
    }

    if maximizing {
        let mut max_eval = i32::MIN;
        for chess_move in legal_moves {
            let new_board = board.apply_move(chess_move);
            let eval = minimax(&new_board, depth - 1, alpha, beta, false);
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for chess_move in legal_moves {
            let new_board = board.apply_move(chess_move);
            let eval = minimax(&new_board, depth - 1, alpha, beta, true);
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        min_eval
    }
}

fn find_best_move(board: &Board) -> Option<ChessMove> {
    let legal_moves = board.generate_legal_moves(PieceColor::Black);
    if legal_moves.is_empty() {
        return None;
    }

    let mut best_move = None;
    let mut best_eval = i32::MIN;

    for chess_move in legal_moves {
        let new_board = board.apply_move(chess_move);
        let eval = minimax(&new_board, AI_DEPTH - 1, i32::MIN, i32::MAX, false);
        if eval > best_eval {
            best_eval = eval;
            best_move = Some(chess_move);
        }
    }

    best_move
}

#[derive(PartialEq, Eq)]
enum GamePhase {
    PlayerTurn,
    AiThinking,
    Checkmate,
    Stalemate,
}

fn light_square_background() -> TermColor {
    TermColor::Rgb {
        r: 180,
        g: 140,
        b: 100,
    }
}

fn dark_square_background() -> TermColor {
    TermColor::Rgb {
        r: 120,
        g: 80,
        b: 50,
    }
}

fn selected_background() -> TermColor {
    TermColor::Rgb {
        r: 100,
        g: 160,
        b: 220,
    }
}

fn legal_move_background() -> TermColor {
    TermColor::Rgb {
        r: 80,
        g: 180,
        b: 80,
    }
}

fn cursor_background() -> TermColor {
    TermColor::Rgb {
        r: 200,
        g: 200,
        b: 80,
    }
}

fn last_move_background() -> TermColor {
    TermColor::Rgb {
        r: 200,
        g: 200,
        b: 120,
    }
}

fn square_background(column: usize, row: usize) -> TermColor {
    if (column + row).is_multiple_of(2) {
        light_square_background()
    } else {
        dark_square_background()
    }
}

fn piece_foreground(color: PieceColor) -> TermColor {
    match color {
        PieceColor::White => TermColor::White,
        PieceColor::Black => TermColor::Black,
    }
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Chess - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "CHESS";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - title.len() as f64 / 2.0,
                row: center_row - 5.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: title.to_string(),
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let piece_display = " K Q R B N P ";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - piece_display.len() as f64 / 2.0,
                row: center_row - 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: piece_display.to_string(),
                foreground: TermColor::White,
                background: TermColor::Rgb {
                    r: 120,
                    g: 80,
                    b: 50,
                },
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let subtitle = "White vs Black AI (Minimax depth 3)";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - subtitle.len() as f64 / 2.0,
                row: center_row - 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: subtitle.to_string(),
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let controls_1 = "Mouse click or Arrow keys + Enter";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - controls_1.len() as f64 / 2.0,
                row: center_row + 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: controls_1.to_string(),
                foreground: TermColor::Rgb {
                    r: 150,
                    g: 150,
                    b: 200,
                },
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let prompt = "Press ENTER to start";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - prompt.len() as f64 / 2.0,
                row: center_row + 4.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: prompt.to_string(),
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let quit = "ESC to quit";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - quit.len() as f64 / 2.0,
                row: center_row + 6.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: quit.to_string(),
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Enter => self.start_game = true,
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.start_game {
            self.entities.despawn_all(world);
            return Some(Box::new(GameplayState::new()));
        }
        None
    }
}

struct GameplayState {
    board: Board,
    phase: GamePhase,
    tilemap_entity: Entity,
    board_entities: EntityGroup,
    hud_entities: EntityGroup,
    offset_column: i32,
    offset_row: i32,
    cursor_column: usize,
    cursor_row: usize,
    selected_column: Option<usize>,
    selected_row: Option<usize>,
    legal_moves_for_selected: Vec<ChessMove>,
    last_move: Option<ChessMove>,
    status_text: String,
    ai_pending: bool,
}

impl GameplayState {
    fn new() -> Self {
        Self {
            board: Board::new(),
            phase: GamePhase::PlayerTurn,
            tilemap_entity: Entity::default(),
            board_entities: EntityGroup::new(),
            hud_entities: EntityGroup::new(),
            offset_column: 0,
            offset_row: 0,
            cursor_column: 4,
            cursor_row: 6,
            selected_column: None,
            selected_row: None,
            legal_moves_for_selected: Vec::new(),
            last_move: None,
            status_text: "Your turn (White)".to_string(),
            ai_pending: false,
        }
    }

    fn board_pixel_width(&self) -> i32 {
        (BOARD_SIZE * CELL_WIDTH) as i32
    }

    fn board_pixel_height(&self) -> i32 {
        BOARD_SIZE as i32
    }

    fn build_board_tilemap(&mut self, world: &mut World) {
        let tilemap_width = BOARD_SIZE * CELL_WIDTH;
        let tilemap_height = BOARD_SIZE;
        let mut tilemap = Tilemap::new(tilemap_width, tilemap_height);

        for row in 0..BOARD_SIZE {
            for column in 0..BOARD_SIZE {
                let background = self.cell_background(column, row);
                let tile_start_column = column * CELL_WIDTH;

                let (left_char, center_char, right_char) = match self.board.get(column, row) {
                    Some(piece) => (' ', piece.character(), ' '),
                    None => (' ', ' ', ' '),
                };

                let foreground = match self.board.get(column, row) {
                    Some(piece) => piece_foreground(piece.color),
                    None => TermColor::White,
                };

                tilemap.set(
                    tile_start_column,
                    row,
                    TilemapCell {
                        character: left_char,
                        foreground,
                        background,
                    },
                );
                tilemap.set(
                    tile_start_column + 1,
                    row,
                    TilemapCell {
                        character: center_char,
                        foreground,
                        background,
                    },
                );
                tilemap.set(
                    tile_start_column + 2,
                    row,
                    TilemapCell {
                        character: right_char,
                        foreground,
                        background,
                    },
                );
            }
        }

        if self.tilemap_entity != Entity::default()
            && let Some(existing_tilemap) = world.get_tilemap_mut(self.tilemap_entity)
        {
            *existing_tilemap = tilemap;
            return;
        }

        self.tilemap_entity = EntityBuilder::new()
            .position(Position {
                column: self.offset_column as f64,
                row: self.offset_row as f64,
            })
            .tilemap(tilemap)
            .z_index(ZIndex(0))
            .spawn(world);
    }

    fn cell_background(&self, column: usize, row: usize) -> TermColor {
        if self.cursor_column == column && self.cursor_row == row {
            return cursor_background();
        }

        if let (Some(selected_column), Some(selected_row)) =
            (self.selected_column, self.selected_row)
            && selected_column == column
            && selected_row == row
        {
            return selected_background();
        }

        if self
            .legal_moves_for_selected
            .iter()
            .any(|chess_move| chess_move.to_column == column && chess_move.to_row == row)
        {
            return legal_move_background();
        }

        if let Some(last) = &self.last_move
            && ((last.from_column == column && last.from_row == row)
                || (last.to_column == column && last.to_row == row))
        {
            return last_move_background();
        }

        square_background(column, row)
    }

    fn update_hud(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);

        let hud_column = self.offset_column as f64;
        let hud_row = self.offset_row as f64 + self.board_pixel_height() as f64 + 1.0;

        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: hud_column,
                row: hud_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: self.status_text.clone(),
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(15));

        let check_indicator = if self.board.is_in_check(PieceColor::White) {
            "CHECK!"
        } else {
            ""
        };

        if !check_indicator.is_empty() {
            let entity = self
                .hud_entities
                .spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: hud_column,
                    row: hud_row + 1.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: check_indicator.to_string(),
                    foreground: TermColor::Red,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(15));
        }

        let file_labels = " a  b  c  d  e  f  g  h ";
        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: self.offset_column as f64,
                row: self.offset_row as f64 - 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: file_labels.to_string(),
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(15));

        for rank in 0..BOARD_SIZE {
            let rank_label = format!("{}", 8 - rank);
            let entity = self
                .hud_entities
                .spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: self.offset_column as f64 - 2.0,
                    row: self.offset_row as f64 + rank as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: rank_label,
                    foreground: TermColor::Grey,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(15));
        }

        let side_column = self.offset_column as f64 + self.board_pixel_width() as f64 + 3.0;

        let controls = [
            "Controls:",
            "Arrows: Move cursor",
            "Enter:  Select/Move",
            "Esc:    Deselect/Quit",
            "",
            "Captured:",
        ];

        for (line_index, line) in controls.iter().enumerate() {
            let entity = self
                .hud_entities
                .spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: side_column,
                    row: self.offset_row as f64 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: line.to_string(),
                    foreground: TermColor::DarkGrey,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(15));
        }

        let mut white_captured = String::new();
        let mut black_captured = String::new();
        self.compute_captured_pieces(&mut white_captured, &mut black_captured);

        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: side_column,
                row: self.offset_row as f64 + 6.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: format!("W lost: {}", white_captured),
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(15));

        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: side_column,
                row: self.offset_row as f64 + 7.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: format!("B lost: {}", black_captured),
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(15));
    }

    fn compute_captured_pieces(&self, white_captured: &mut String, black_captured: &mut String) {
        let initial_white = [
            (PieceKind::Pawn, 8),
            (PieceKind::Knight, 2),
            (PieceKind::Bishop, 2),
            (PieceKind::Rook, 2),
            (PieceKind::Queen, 1),
        ];
        let initial_black = initial_white;

        let mut current_white = [0i32; 5];
        let mut current_black = [0i32; 5];

        for row in 0..BOARD_SIZE {
            for column in 0..BOARD_SIZE {
                if let Some(piece) = self.board.squares[row][column] {
                    let kind_index = match piece.kind {
                        PieceKind::Pawn => 0,
                        PieceKind::Knight => 1,
                        PieceKind::Bishop => 2,
                        PieceKind::Rook => 3,
                        PieceKind::Queen => 4,
                        PieceKind::King => continue,
                    };
                    match piece.color {
                        PieceColor::White => current_white[kind_index] += 1,
                        PieceColor::Black => current_black[kind_index] += 1,
                    }
                }
            }
        }

        let piece_chars = ['P', 'N', 'B', 'R', 'Q'];
        for (kind_index, (_, initial_count)) in initial_white.iter().enumerate() {
            let lost = *initial_count - current_white[kind_index];
            for _ in 0..lost.max(0) {
                white_captured.push(piece_chars[kind_index]);
            }
        }

        for (kind_index, (_, initial_count)) in initial_black.iter().enumerate() {
            let lost = *initial_count - current_black[kind_index];
            for _ in 0..lost.max(0) {
                black_captured.push(piece_chars[kind_index].to_ascii_lowercase());
            }
        }
    }

    fn select_piece(&mut self, column: usize, row: usize) {
        if let Some(piece) = self.board.get(column, row)
            && piece.color == PieceColor::White
        {
            self.selected_column = Some(column);
            self.selected_row = Some(row);
            self.legal_moves_for_selected = self
                .board
                .generate_legal_moves(PieceColor::White)
                .into_iter()
                .filter(|chess_move| chess_move.from_column == column && chess_move.from_row == row)
                .collect();
        }
    }

    fn deselect(&mut self) {
        self.selected_column = None;
        self.selected_row = None;
        self.legal_moves_for_selected.clear();
    }

    fn try_move_to(&mut self, to_column: usize, to_row: usize) -> bool {
        if let Some(chess_move) = self
            .legal_moves_for_selected
            .iter()
            .find(|chess_move| chess_move.to_column == to_column && chess_move.to_row == to_row)
        {
            let mut final_move = *chess_move;

            if final_move.promotion.is_none()
                && let Some(piece) = self.board.get(final_move.from_column, final_move.from_row)
                && piece.kind == PieceKind::Pawn
            {
                let promotion_row = match piece.color {
                    PieceColor::White => 0,
                    PieceColor::Black => 7,
                };
                if to_row == promotion_row {
                    final_move.promotion = Some(PieceKind::Queen);
                }
            }

            self.board = self.board.apply_move(final_move);
            self.last_move = Some(final_move);
            self.deselect();

            if self.board.is_checkmate(PieceColor::Black) {
                self.phase = GamePhase::Checkmate;
                self.status_text = "Checkmate! White wins!".to_string();
            } else if self.board.is_stalemate(PieceColor::Black) {
                self.phase = GamePhase::Stalemate;
                self.status_text = "Stalemate! Draw.".to_string();
            } else {
                self.phase = GamePhase::AiThinking;
                self.status_text = "AI is thinking...".to_string();
                self.ai_pending = true;
            }
            return true;
        }
        false
    }

    fn handle_click(&mut self, column: usize, row: usize) {
        if self.phase != GamePhase::PlayerTurn {
            return;
        }

        if self.selected_column.is_some() {
            if self
                .legal_moves_for_selected
                .iter()
                .any(|chess_move| chess_move.to_column == column && chess_move.to_row == row)
            {
                self.try_move_to(column, row);
                return;
            }

            if let Some(piece) = self.board.get(column, row)
                && piece.color == PieceColor::White
            {
                self.select_piece(column, row);
                return;
            }

            self.deselect();
        } else {
            self.select_piece(column, row);
        }
    }

    fn execute_ai_move(&mut self) {
        if let Some(best_move) = find_best_move(&self.board) {
            self.board = self.board.apply_move(best_move);
            self.last_move = Some(best_move);

            if self.board.is_checkmate(PieceColor::White) {
                self.phase = GamePhase::Checkmate;
                self.status_text = "Checkmate! Black wins!".to_string();
            } else if self.board.is_stalemate(PieceColor::White) {
                self.phase = GamePhase::Stalemate;
                self.status_text = "Stalemate! Draw.".to_string();
            } else {
                self.phase = GamePhase::PlayerTurn;
                self.status_text = "Your turn (White)".to_string();
            }
        }
        self.ai_pending = false;
    }

    fn screen_to_board(&self, screen_column: u16, screen_row: u16) -> Option<(usize, usize)> {
        let board_pixel_column = screen_column as i32 - self.offset_column;
        let board_pixel_row = screen_row as i32 - self.offset_row;

        if board_pixel_column < 0
            || board_pixel_column >= self.board_pixel_width()
            || board_pixel_row < 0
            || board_pixel_row >= self.board_pixel_height()
        {
            return None;
        }

        let board_column = board_pixel_column as usize / CELL_WIDTH;
        let board_row = board_pixel_row as usize;

        if board_column < BOARD_SIZE && board_row < BOARD_SIZE {
            Some((board_column, board_row))
        } else {
            None
        }
    }

    fn clear_all(&mut self, world: &mut World) {
        self.board_entities.despawn_all(world);
        self.hud_entities.despawn_all(world);
        if self.tilemap_entity != Entity::default() {
            world.despawn_entities(&[self.tilemap_entity]);
            self.tilemap_entity = Entity::default();
        }
    }
}

impl State for GameplayState {
    fn title(&self) -> &str {
        "Chess - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let board_width = self.board_pixel_width();
        let side_panel_width = 25;
        let total_width = board_width + 3 + side_panel_width;
        self.offset_column = ((terminal.columns as i32 - total_width) / 2).max(3);
        self.offset_row = ((terminal.rows as i32 - self.board_pixel_height() - 4) / 2).max(2);

        self.build_board_tilemap(world);
        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }

        if self.phase == GamePhase::Checkmate || self.phase == GamePhase::Stalemate {
            match key {
                KeyCode::Enter | KeyCode::Char('r') => {
                    self.clear_all(world);
                    *self = GameplayState::new();
                    self.initialize(world);
                }
                KeyCode::Escape => world.resources.should_exit = true,
                _ => {}
            }
            return;
        }

        if self.phase != GamePhase::PlayerTurn {
            return;
        }

        match key {
            KeyCode::Up if self.cursor_row > 0 => {
                self.cursor_row -= 1;
            }
            KeyCode::Down if self.cursor_row < BOARD_SIZE - 1 => {
                self.cursor_row += 1;
            }
            KeyCode::Left if self.cursor_column > 0 => {
                self.cursor_column -= 1;
            }
            KeyCode::Right if self.cursor_column < BOARD_SIZE - 1 => {
                self.cursor_column += 1;
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.handle_click(self.cursor_column, self.cursor_row);
            }
            KeyCode::Escape => {
                if self.selected_column.is_some() {
                    self.deselect();
                } else {
                    world.resources.should_exit = true;
                }
            }
            _ => {}
        }
    }

    fn on_mouse_input(
        &mut self,
        _world: &mut World,
        button: MouseButton,
        column: u16,
        row: u16,
        pressed: bool,
    ) {
        if !pressed || button != MouseButton::Left {
            return;
        }

        if self.phase != GamePhase::PlayerTurn {
            return;
        }

        if let Some((board_column, board_row)) = self.screen_to_board(column, row) {
            self.cursor_column = board_column;
            self.cursor_row = board_row;
            self.handle_click(board_column, board_row);
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.ai_pending && self.phase == GamePhase::AiThinking {
            self.build_board_tilemap(world);
            self.update_hud(world);
            self.execute_ai_move();
        }

        self.build_board_tilemap(world);
        self.update_hud(world);
    }

    fn next_state(&mut self, _world: &mut World) -> Option<Box<dyn State>> {
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(TitleScreenState {
        entities: EntityGroup::new(),
        start_game: false,
    }))
}
