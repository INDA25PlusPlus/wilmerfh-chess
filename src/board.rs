use crate::piece::{Offset, Piece};
use std::ops::Add;

pub const BOARD_WIDTH: i8 = 8;
pub const BOARD_HEIGHT: i8 = 8;

#[derive(Clone)]
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

    fn get_piece_at_pos(&self, pos: &Position) -> Option<&Piece> {
        let Ok(index) = pos.try_to_index() else {
            return None;
        };
        match self.pieces.get(index) {
            Some(piece) => piece.as_ref(),
            None => None,
        }
    }

    pub fn is_move_valid(&self, from: &Position, to: &Position) -> bool {
        if !(from.is_on_board() && to.is_on_board()) {
            return false;
        };

        todo!()
    }

    pub fn get_valid_moves(&self, pos: &Position) -> Vec<Position> {
        let Some(targeted_piece) = self.get_piece_at_pos(pos) else {
            return Vec::new();
        };

        let possible_offsets = targeted_piece.piece_type.get_offsets();
        possible_offsets
            .into_iter()
            .map(|offset| (*pos).clone() + offset)
            .filter(|target_pos| self.is_move_valid(pos, target_pos))
            .collect()
    }

    pub fn execute_move(&mut self, from: &Position, to: &Position) -> Result<(), String> {
        if !self.is_move_valid(from, to) {
            return Err("Invalid move".to_string());
        };

        let from_index = from.try_to_index()?;
        let to_index = to.try_to_index()?;

        self.pieces[to_index] = self.pieces[from_index].clone();
        self.pieces[from_index] = None;
        Ok(())
    }
}
