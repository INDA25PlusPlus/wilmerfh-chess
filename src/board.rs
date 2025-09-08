use crate::piece::{Offset, Piece, PieceType};
use std::ops::Add;

pub const BOARD_WIDTH: i8 = 8;
pub const BOARD_HEIGHT: i8 = 8;

#[derive(Copy, Clone, PartialEq)]
pub struct Position {
    file: i8,
    rank: i8,
}

impl Position {
    fn is_on_board(&self) -> bool {
        if !((self.file >= 0) && (self.rank >= 0)) {
            return false;
        }
        (self.file < BOARD_WIDTH) && (self.rank < BOARD_HEIGHT)
    }

    fn try_to_index(&self) -> Result<usize, String> {
        if !self.is_on_board() {
            return Err("Position is not on board".to_string());
        }
        let index = ((self.rank - 1) * BOARD_WIDTH + self.file - 1) as usize;
        Ok(index)
    }
}

impl Add<Offset> for Position {
    type Output = Position;
    fn add(self, other: Offset) -> Self::Output {
        Position {
            file: self.file + other.file,
            rank: self.rank + other.rank,
        }
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

    fn get_piece_at_pos(&self, pos: Position) -> Option<&Piece> {
        let Ok(index) = pos.try_to_index() else {
            return None;
        };
        match self.pieces.get(index) {
            Some(piece) => piece.as_ref(),
            None => None,
        }
    }

    fn get_path(&self, from: Position, to: Position) -> Result<Vec<Position>, String> {
        if from == to {
            return Err("From positon can't be the same as to position".to_string());
        }
        let is_on_same_file = from.file == to.file;
        let is_on_same_rank = from.rank == to.rank;
        let is_on_same_diagonal = {
            let file_diff = (from.file - to.file).abs();
            let rank_diff = (from.rank - to.rank).abs();
            file_diff == rank_diff
        };

        if !(is_on_same_file || is_on_same_rank || is_on_same_diagonal) {
            return Err("Positions are not on the same file, rank or diagonal".to_string());
        }

        let step_offset = Offset {
            file: (to.file - from.file).signum(),
            rank: (to.rank - from.rank).signum(),
        };

        let mut positions = Vec::new();
        let mut current = from + step_offset;

        while current != to {
            positions.push(current);
            current = current + step_offset;
        }

        Ok(positions)
    }

    pub fn is_move_valid(&self, from: Position, to: Position) -> bool {
        if !(from.is_on_board() && to.is_on_board()) {
            return false;
        };
        let Some(piece) = self.get_piece_at_pos(from) else {
            return false;
        };
        // Check if "to" is in the piece type's valid offsets
        if !piece
            .piece_type
            .get_offsets()
            .into_iter()
            .map(|o| from + o)
            .any(|p| p == to)
        {
            return false;
        }
        // Check if target square has a piece of the same color
        if let Some(target_piece) = self.get_piece_at_pos(to) {
            if piece.color == target_piece.color {
                return false;
            }
        }
        match piece.piece_type {
            PieceType::Knight => true,
            _ => {
                let Ok(path) = self.get_path(from, to) else {
                    return false;
                };

                path.into_iter().all(|p| self.get_piece_at_pos(p).is_none())
            }
        }
    }

    pub fn get_valid_moves(&self, pos: Position) -> Vec<Position> {
        let Some(targeted_piece) = self.get_piece_at_pos(pos) else {
            return Vec::new();
        };

        let possible_offsets = targeted_piece.piece_type.get_offsets();
        possible_offsets
            .into_iter()
            .map(|offset| pos + offset)
            .filter(|target_pos| self.is_move_valid(pos, *target_pos))
            .collect()
    }

    pub fn execute_move(&mut self, from: Position, to: Position) -> Result<(), String> {
        if !self.is_move_valid(from, to) {
            return Err("Invalid move".to_string());
        };

        let from_index = from.try_to_index()?;
        let to_index = to.try_to_index()?;

        self.pieces[to_index] = self.pieces[from_index].clone();
        self.pieces[from_index] = None;
        Ok(())
    }

    fn set(&mut self, pos: Position, piece: Piece) -> Result<(), String> {
        let index = pos.try_to_index()?;
        self.pieces[index] = Some(piece);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{self, Board, Position},
        piece::{Piece, PieceColor, PieceType},
    };

    #[test]
    fn test_is_valid_move() {
        let mut board = Board::new();
        let black_knight = Piece {
            piece_type: PieceType::Knight,
            color: PieceColor::Black,
        };
        let black_knight_position = Position {file: 2, rank: 4};
        let white_rook = Piece {
            piece_type: PieceType::Rook,
            color: PieceColor::White,
        };
        let white_rook_position = Position {file: 1, rank: 4};
        board
            .set(black_knight_position, black_knight)
            .unwrap();
        board
            .set(white_rook_position, white_rook)
            .unwrap();

        assert!(board.is_move_valid(white_rook_position, Position { file: 1, rank: 1 }));
        assert!(board.is_move_valid(black_knight_position, Position { file: 1, rank: 2}));

        assert!(!board.is_move_valid(white_rook_position, Position { file: 7, rank: 4}));
        assert!(!board.is_move_valid(white_rook_position, Position { file: 1, rank: 8}));
        assert!(!board.is_move_valid(white_rook_position, Position { file: 7, rank: 7}));
        assert!(!board.is_move_valid(black_knight_position, Position { file: 4, rank: 4}));
    }

    #[test]
    fn test_get_valid_moves() {
        let mut board = Board::new();
        let black_knight = Piece {
            piece_type: PieceType::Knight,
            color: PieceColor::Black,
        };
        let black_knight_position = Position {file: 2, rank: 4};
        let white_rook = Piece {
            piece_type: PieceType::Rook,
            color: PieceColor::White,
        };
        let white_rook_position = Position {file: 1, rank: 4};
        board
            .set(black_knight_position, black_knight)
            .unwrap();
        board
            .set(white_rook_position, white_rook)
            .unwrap();

        assert_eq!(board.get_valid_moves(black_knight_position).len(), 8);
        assert_eq!(board.get_valid_moves(white_rook_position).len(), 9);
    }
}
