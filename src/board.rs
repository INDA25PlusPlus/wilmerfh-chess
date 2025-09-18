use crate::piece::{Move, MoveShape, Offset, Piece, PieceColor, PieceType, ShapeData};
use std::ops::Add;

pub const BOARD_WIDTH: i8 = 8;
pub const BOARD_HEIGHT: i8 = 8;

#[derive(Clone, Copy)]
pub struct CastlingRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
}

impl CastlingRights {
    pub fn new() -> Self {
        Self {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }

    pub fn can_castle(&self, color: PieceColor, kingside: bool) -> bool {
        match (color, kingside) {
            (PieceColor::White, true) => self.white_kingside,
            (PieceColor::White, false) => self.white_queenside,
            (PieceColor::Black, true) => self.black_kingside,
            (PieceColor::Black, false) => self.black_queenside,
        }
    }

    pub fn disable_king_castling(&mut self, color: PieceColor) {
        match color {
            PieceColor::White => {
                self.white_kingside = false;
                self.white_queenside = false;
            }
            PieceColor::Black => {
                self.black_kingside = false;
                self.black_queenside = false;
            }
        }
    }

    pub fn disable_rook_castling(&mut self, color: PieceColor, kingside: bool) {
        match (color, kingside) {
            (PieceColor::White, true) => self.white_kingside = false,
            (PieceColor::White, false) => self.white_queenside = false,
            (PieceColor::Black, true) => self.black_kingside = false,
            (PieceColor::Black, false) => self.black_queenside = false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Position {
    pub file: i8,
    pub rank: i8,
}

impl Position {
    pub fn new(file: i8, rank: i8) -> Self {
        Self { file, rank }
    }

    pub fn is_on_board(&self) -> bool {
        if !((self.file >= 0) && (self.rank >= 0)) {
            return false;
        }
        (self.file < BOARD_WIDTH) && (self.rank < BOARD_HEIGHT)
    }

    fn to_index(&self) -> Result<usize, String> {
        if !self.is_on_board() {
            return Err("Position is not on board".to_string());
        }
        let index = (self.rank * BOARD_WIDTH + self.file) as usize;
        Ok(index)
    }

    fn from_index(index: usize) -> Self {
        let rank = (index as i8) / BOARD_WIDTH;
        let file = (index as i8) % BOARD_WIDTH;
        Position::new(file, rank)
    }
}

impl Add<Offset> for Position {
    type Output = Position;
    fn add(self, other: Offset) -> Self::Output {
        Position::new(self.file + other.file, self.rank + other.rank)
    }
}

#[derive(Clone, Copy)]
enum MoveTurn {
    White,
    Black,
}

#[derive(Clone, Copy, Debug)]
enum CastlingSide {
    Kingside,
    Queenside,
}

#[derive(Clone)]
pub struct Board {
    pieces: [Option<Piece>; (BOARD_WIDTH * BOARD_HEIGHT) as usize],
    move_turn: MoveTurn,
    castling_rights: CastlingRights,
}

impl Board {
    fn new() -> Self {
        Self {
            pieces: [const { None }; (BOARD_WIDTH * BOARD_HEIGHT) as usize],
            move_turn: MoveTurn::White,
            castling_rights: CastlingRights::new(),
        }
    }

    fn piece_at_pos(&self, pos: Position) -> Option<Piece> {
        let Ok(index) = pos.to_index() else {
            return None;
        };
        self.pieces[index]
    }

    fn cast_ray(&self, start_pos: Position, direction: Offset) -> Option<(Position, Piece)> {
        let mut current = start_pos + direction;
        while current.is_on_board() {
            if let Some(piece) = self.piece_at_pos(current) {
                return Some((current, piece));
            }
            current = current + direction;
        }
        None
    }

    fn is_pos_attacked(&self, square_pos: Position, attacking_color: PieceColor) -> bool {
        let knight_offsets = [
            Offset::new(2, 1),
            Offset::new(2, -1),
            Offset::new(-2, 1),
            Offset::new(-2, -1),
            Offset::new(1, 2),
            Offset::new(1, -2),
            Offset::new(-1, 2),
            Offset::new(-1, -2),
        ];
        let mut moves_and_pieces = Vec::<(Move, Piece)>::new();
        for offset in knight_offsets {
            let knight_pos = square_pos + offset;
            if let Some(piece) = self.piece_at_pos(knight_pos) {
                moves_and_pieces.push((Move::new(knight_pos, square_pos), piece));
            }
        }
        let ray_directions = [
            // Straight directions (rooks, queens)
            Offset::new(1, 0),  // right
            Offset::new(-1, 0), // left
            Offset::new(0, 1),  // up
            Offset::new(0, -1), // down
            // Diagonal directions (bishops, queens)
            Offset::new(1, 1),   // up-right
            Offset::new(1, -1),  // down-right
            Offset::new(-1, 1),  // up-left
            Offset::new(-1, -1), // down-left
        ];
        for direction in ray_directions {
            if let Some((piece_pos, piece)) = self.cast_ray(square_pos, direction) {
                moves_and_pieces.push((Move::new(piece_pos, square_pos), piece));
            }
        }
        // Filter by attacking color and move validity
        moves_and_pieces
            .into_iter()
            .filter(|(_, piece)| piece.color == attacking_color)
            .any(|(move_, _)| self.move_pseudo_legal(move_))
    }

    fn find_king(&self, color: PieceColor) -> Option<Position> {
        self.pieces
            .iter()
            .enumerate()
            .filter_map(|(index, piece_option)| piece_option.as_ref().map(|piece| (index, piece)))
            .find_map(|(index, piece)| {
                if piece.color == color && matches!(piece.type_, PieceType::King) {
                    Some(Position::from_index(index))
                } else {
                    None
                }
            })
    }

    pub fn is_in_check(&self, color: PieceColor) -> bool {
        let Some(king_pos) = self.find_king(color) else {
            return false;
        };

        let attacking_color = match color {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        };

        self.is_pos_attacked(king_pos, attacking_color)
    }

    fn path_clear(&self, move_: Move) -> bool {
        let Some(moving_piece) = self.piece_at_pos(move_.from()) else {
            return false;
        };
        let Some(shape) = move_.shape() else {
            return false;
        };
        // Ugly code
        match shape {
            MoveShape::Knight => {}
            _ => {
                let step = Offset {
                    file: (move_.to().file - move_.from().file).signum(),
                    rank: (move_.to().rank - move_.from().rank).signum(),
                };
                let mut current = move_.from() + step;
                while current != move_.to() {
                    if self.piece_at_pos(current).is_some() {
                        return false;
                    }
                    current = current + step;
                }
            }
        }
        // Check destination is valid (not capturing own piece)
        if let Some(target_piece) = self.piece_at_pos(move_.to()) {
            target_piece.color != moving_piece.color
        } else {
            true
        }
    }

    fn is_move_capture(&self, move_: Move) -> bool {
        let Some(moving_piece) = self.piece_at_pos(move_.from()) else {
            return false;
        };
        let Some(target_piece) = self.piece_at_pos(move_.to()) else {
            return false;
        };
        moving_piece.color != target_piece.color
    }

    pub fn move_pseudo_legal(&self, move_: Move) -> bool {
        if !move_.is_on_board() {
            return false;
        };
        let Some(moving_piece) = self.piece_at_pos(move_.from()) else {
            return false;
        };
        let Some(shape) = move_.shape() else {
            return false;
        };
        if !moving_piece.shape_allowed(shape) {
            return false;
        }

        // Special pawn movement rules
        if let PieceType::Pawn = moving_piece.type_ {
            if !moving_piece.validate_pawn_rules(move_, self.is_move_capture(move_)) {
                return false;
            }
        }

        // Special king movement rules (castling)
        if let PieceType::King = moving_piece.type_ {
            if self.get_castling(move_).is_some() {
                return self.validate_castling(move_);
            }
        }

        self.path_clear(move_)
    }

    fn get_castling(&self, move_: Move) -> Option<CastlingSide> {
        let Some(moving_piece) = self.piece_at_pos(move_.from()) else {
            return None;
        };

        if !matches!(moving_piece.type_, PieceType::King) {
            return None;
        }

        let Some(shape) = move_.shape() else {
            return None;
        };

        if matches!(shape, MoveShape::Straight(ShapeData { distance: 2, .. })) {
            if move_.to().file > move_.from().file {
                Some(CastlingSide::Kingside)
            } else {
                Some(CastlingSide::Queenside)
            }
        } else {
            None
        }
    }

    fn validate_castling(&self, move_: Move) -> bool {
        let Some(moving_piece) = self.piece_at_pos(move_.from()) else {
            return false;
        };

        let Some(castling_side) = self.get_castling(move_) else {
            return false;
        };

        // Check castling rights
        let is_kingside = matches!(castling_side, CastlingSide::Kingside);
        if !self
            .castling_rights
            .can_castle(moving_piece.color, is_kingside)
        {
            return false;
        }

        // Check path is clear by casting ray from rook to king
        let rook_file = if is_kingside { 7 } else { 0 };
        let rook_pos = Position::new(rook_file, move_.from().rank);

        // Cast ray from rook toward king
        let direction = if is_kingside {
            Offset::new(-1, 0)
        } else {
            Offset::new(1, 0)
        };

        if let Some((
            _,
            Piece {
                type_: PieceType::King,
                ..
            },
        )) = self.cast_ray(rook_pos, direction)
        {
            // Path is clear and king is reachable
        } else {
            return false;
        }

        // Check king doesn't travel through or end in check
        let attacking_color = match moving_piece.color {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        };

        let king_direction = match castling_side {
            CastlingSide::Kingside => Offset::new(1, 0),
            CastlingSide::Queenside => Offset::new(-1, 0),
        };

        let positions_to_check = [
            move_.from(),
            move_.from() + king_direction * 1,
            move_.from() + king_direction * 2,
        ];

        positions_to_check
            .into_iter()
            .all(|pos| !self.is_pos_attacked(pos, attacking_color))
    }

    pub fn move_legal(&self, move_: Move) -> bool {
        if !self.move_pseudo_legal(move_) {
            return false;
        }
        let mut test_board = self.clone();
        if let Err(_) = test_board.make_move(move_) {
            return false;
        }

        let moving_color = match self.move_turn {
            MoveTurn::White => PieceColor::White,
            MoveTurn::Black => PieceColor::Black,
        };
        !test_board.is_in_check(moving_color)
    }

    pub fn legal_moves(&self, pos: Position) -> Vec<Position> {
        todo!()
    }

    fn move_piece(&mut self, from: Position, to: Position) -> Result<(), String> {
        let piece = self.piece_at_pos(from);
        self.set(to, piece)?;
        self.set(from, None)?;
        Ok(())
    }

    pub fn make_move(&mut self, move_: Move) -> Result<(), String> {
        // Move the rook if castling
        if let Some(castling_side) = self.get_castling(move_) {
            let (rook_from_file, rook_to_file) = match castling_side {
                CastlingSide::Kingside => (7, 5),  // h->f
                CastlingSide::Queenside => (0, 3), // a->d
            };

            let rook_from = Position::new(rook_from_file, move_.from().rank);
            let rook_to = Position::new(rook_to_file, move_.from().rank);
            self.move_piece(rook_from, rook_to)?;
        }

        self.move_piece(move_.from(), move_.to())?;

        self.update_castling_rights_for_move(move_);
        self.move_turn = match self.move_turn {
            MoveTurn::White => MoveTurn::Black,
            MoveTurn::Black => MoveTurn::White,
        };

        Ok(())
    }

    fn update_castling_rights_for_move(&mut self, move_: Move) {
        // Handle pieces moving from critical squares
        match move_.from() {
            Position { file: 4, rank: 0 } => {
                self.castling_rights
                    .disable_king_castling(PieceColor::White);
            }
            Position { file: 4, rank: 7 } => {
                self.castling_rights
                    .disable_king_castling(PieceColor::Black);
            }
            Position { file: 0, rank: 0 } => {
                self.castling_rights
                    .disable_rook_castling(PieceColor::White, false);
            }
            Position { file: 7, rank: 0 } => {
                self.castling_rights
                    .disable_rook_castling(PieceColor::White, true);
            }
            Position { file: 0, rank: 7 } => {
                self.castling_rights
                    .disable_rook_castling(PieceColor::Black, false);
            }
            Position { file: 7, rank: 7 } => {
                self.castling_rights
                    .disable_rook_castling(PieceColor::Black, true);
            }
            _ => {}
        }

        // Handle captures on critical squares
        // If a rook is captured, the castling_rights are disabled
        match move_.to() {
            Position { file: 0, rank: 0 } => {
                self.castling_rights
                    .disable_rook_castling(PieceColor::White, false);
            }
            Position { file: 7, rank: 0 } => {
                self.castling_rights
                    .disable_rook_castling(PieceColor::White, true);
            }
            Position { file: 0, rank: 7 } => {
                self.castling_rights
                    .disable_rook_castling(PieceColor::Black, false);
            }
            Position { file: 7, rank: 7 } => {
                self.castling_rights
                    .disable_rook_castling(PieceColor::Black, true);
            }
            _ => {}
        }
    }

    fn set(&mut self, pos: Position, piece: Option<Piece>) -> Result<(), String> {
        let index = pos.to_index()?;
        self.pieces[index] = piece;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, Position},
        piece::{Move, Piece, PieceColor, PieceType},
    };

    #[test]
    fn test_is_pseudo_legal() {
        let mut board = Board::new();
        let black_knight = Some(Piece {
            type_: PieceType::Knight,
            color: PieceColor::Black,
        });
        let black_knight_position = Position::new(2, 4);
        let white_rook = Some(Piece {
            type_: PieceType::Rook,
            color: PieceColor::White,
        });
        let white_rook_position = Position::new(1, 4);
        board.set(black_knight_position, black_knight).unwrap();
        board.set(white_rook_position, white_rook).unwrap();

        assert!(board.move_pseudo_legal(Move::new(white_rook_position, Position::new(1, 0))));
        assert!(board.move_pseudo_legal(Move::new(black_knight_position, Position::new(1, 2))));

        assert!(!board.move_pseudo_legal(Move::new(white_rook_position, Position::new(7, 4))));
        assert!(!board.move_pseudo_legal(Move::new(white_rook_position, Position::new(1, 8))));
        assert!(!board.move_pseudo_legal(Move::new(white_rook_position, Position::new(7, 7))));
        assert!(!board.move_pseudo_legal(Move::new(black_knight_position, Position::new(4, 4))));
    }

    #[test]
    fn test_is_in_check() {
        let mut board = Board::new();

        // Set up white king and black rook attacking it
        let white_king = Some(Piece {
            type_: PieceType::King,
            color: PieceColor::White,
        });
        let white_king_position = Position::new(4, 0);

        let black_rook = Some(Piece {
            type_: PieceType::Rook,
            color: PieceColor::Black,
        });
        let black_rook_position = Position::new(4, 7);

        board.set(white_king_position, white_king).unwrap();
        board.set(black_rook_position, black_rook).unwrap();

        // White king should be in check from black rook
        assert!(board.is_in_check(PieceColor::White));

        // Black should not be in check (no black king on board)
        assert!(!board.is_in_check(PieceColor::Black));

        // Add black king in safe position
        let black_king = Some(Piece {
            type_: PieceType::King,
            color: PieceColor::Black,
        });
        let black_king_position = Position::new(0, 0);
        board.set(black_king_position, black_king).unwrap();

        // Now black king should not be in check
        assert!(!board.is_in_check(PieceColor::Black));

        // Test knight check
        let mut board2 = Board::new();
        let black_king2 = Some(Piece {
            type_: PieceType::King,
            color: PieceColor::Black,
        });
        let black_king_position2 = Position::new(3, 3);

        let white_knight = Some(Piece {
            type_: PieceType::Knight,
            color: PieceColor::White,
        });
        let white_knight_position = Position::new(1, 2); // Knight move away from king

        board2.set(black_king_position2, black_king2).unwrap();
        board2.set(white_knight_position, white_knight).unwrap();

        // Black king should be in check from white knight
        assert!(board2.is_in_check(PieceColor::Black));
    }

    #[test]
    fn test_pinned_piece() {
        let mut board = Board::new();

        // White king on e1, white rook on e4, black rook on e8
        let white_king = Some(Piece {
            type_: PieceType::King,
            color: PieceColor::White,
        });
        let white_rook = Some(Piece {
            type_: PieceType::Rook,
            color: PieceColor::White,
        });
        let black_rook = Some(Piece {
            type_: PieceType::Rook,
            color: PieceColor::Black,
        });

        board.set(Position::new(4, 0), white_king).unwrap(); // e1
        board.set(Position::new(4, 3), white_rook).unwrap(); // e4
        board.set(Position::new(4, 7), black_rook).unwrap(); // e8

        // Rook cannot move horizontally (pinned)
        let horizontal_move = Move::new(Position::new(4, 3), Position::new(7, 3));
        assert!(board.move_pseudo_legal(horizontal_move));
        assert!(!board.move_legal(horizontal_move));

        // Rook can move along the pin
        let vertical_move = Move::new(Position::new(4, 3), Position::new(4, 5));
        assert!(board.move_pseudo_legal(vertical_move));
        assert!(board.move_legal(vertical_move));
    }

    #[test]
    fn test_castling() {
        let mut board = Board::new();

        board.set(Position::new(5, 0), None).unwrap();
        board.set(Position::new(6, 0), None).unwrap();

        board
            .set(
                Position::new(4, 2),
                Some(Piece {
                    type_: PieceType::Knight,
                    color: PieceColor::Black,
                }),
            )
            .unwrap();

        let kingside_castle = Move::new(Position::new(4, 0), Position::new(6, 0));
        assert!(!board.move_legal(kingside_castle));

        let mut board2 = Board::new();

        board2
            .set(
                Position::new(4, 7),
                Some(Piece {
                    type_: PieceType::King,
                    color: PieceColor::Black,
                }),
            )
            .unwrap();
        board2
            .set(
                Position::new(0, 7),
                Some(Piece {
                    type_: PieceType::Rook,
                    color: PieceColor::Black,
                }),
            )
            .unwrap();

        board2
            .set(
                Position::new(1, 4),
                Some(Piece {
                    type_: PieceType::Rook,
                    color: PieceColor::White,
                }),
            )
            .unwrap();

        let queenside_castle = Move::new(Position::new(4, 7), Position::new(2, 7));
        assert!(board2.move_legal(queenside_castle));

        board2.make_move(queenside_castle).unwrap();

        let king_at_c8 = board2.piece_at_pos(Position::new(2, 7));
        assert!(matches!(
            king_at_c8,
            Some(Piece {
                type_: PieceType::King,
                color: PieceColor::Black
            })
        ));

        let rook_at_d8 = board2.piece_at_pos(Position::new(3, 7));
        assert!(matches!(
            rook_at_d8,
            Some(Piece {
                type_: PieceType::Rook,
                color: PieceColor::Black
            })
        ));
    }
}
