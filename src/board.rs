use crate::piece::{Move, MoveShape, Offset, Piece, PieceColor, PieceType};
use std::ops::Add;

pub const BOARD_WIDTH: i8 = 8;
pub const BOARD_HEIGHT: i8 = 8;

#[derive(Copy, Clone, PartialEq)]
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
}

impl Add<Offset> for Position {
    type Output = Position;
    fn add(self, other: Offset) -> Self::Output {
        Position::new(self.file + other.file, self.rank + other.rank)
    }
}

enum MoveTurn {
    White,
    Black,
}

pub struct Board {
    pieces: [Option<Piece>; (BOARD_WIDTH * BOARD_HEIGHT) as usize],
    move_turn: MoveTurn,
}

impl Board {
    fn new() -> Self {
        Self {
            pieces: [const { None }; (BOARD_WIDTH * BOARD_HEIGHT) as usize],
            move_turn: MoveTurn::White,
        }
    }

    fn piece_at_pos(&self, pos: Position) -> Option<&Piece> {
        let Ok(index) = pos.to_index() else {
            return None;
        };
        match self.pieces.get(index) {
            Some(piece) => piece.as_ref(),
            None => None,
        }
    }

    fn path_clear(&self, move_: Move) -> bool {
        let Some(moving_piece) = self.piece_at_pos(move_.from()) else {
            return false;
        };
        let Some(shape) = move_.shape() else {
            return false;
        };
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

    pub fn pseudo_legal(&self, move_: Move) -> bool {
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

        self.path_clear(move_)
    }

    pub fn legal_moves(&self, pos: Position) -> Vec<Position> {
        todo!()
    }

    pub fn execute_move(&mut self, move_: Move) -> Result<(), String> {
        if !self.pseudo_legal(move_) {
            return Err("Invalid move".to_string());
        };

        let from_index = move_.from().to_index()?;
        let to_index = move_.to().to_index()?;

        self.pieces[to_index] = self.pieces[from_index].clone();
        self.pieces[from_index] = None;
        Ok(())
    }

    fn set(&mut self, pos: Position, piece: Piece) -> Result<(), String> {
        let index = pos.to_index()?;
        self.pieces[index] = Some(piece);
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
        let black_knight = Piece {
            type_: PieceType::Knight,
            color: PieceColor::Black,
        };
        let black_knight_position = Position::new(2, 4);
        let white_rook = Piece {
            type_: PieceType::Rook,
            color: PieceColor::White,
        };
        let white_rook_position = Position::new(1, 4);
        board.set(black_knight_position, black_knight).unwrap();
        board.set(white_rook_position, white_rook).unwrap();

        assert!(board.pseudo_legal(Move::new(white_rook_position, Position::new(1, 0))));
        assert!(board.pseudo_legal(Move::new(black_knight_position, Position::new(1, 2))));

        assert!(!board.pseudo_legal(Move::new(white_rook_position, Position::new(7, 4))));
        assert!(!board.pseudo_legal(Move::new(white_rook_position, Position::new(1, 8))));
        assert!(!board.pseudo_legal(Move::new(white_rook_position, Position::new(7, 7))));
        assert!(!board.pseudo_legal(Move::new(black_knight_position, Position::new(4, 4))));
    }
}
