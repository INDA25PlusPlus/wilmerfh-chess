use crate::board::Position;
use std::ops::Mul;

#[derive(Copy, Clone)]
pub struct Offset {
    pub file: i8,
    pub rank: i8,
}

impl Offset {
    pub fn new(file: i8, rank: i8) -> Self {
        Self { file, rank }
    }
}

impl Mul<i8> for Offset {
    type Output = Offset;

    fn mul(self, scalar: i8) -> Self::Output {
        Offset {
            file: self.file * scalar,
            rank: self.rank * scalar,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub struct ShapeData {
    pub forward_only: bool,
    pub backward_only: bool,
    pub distance: i8,
}

#[derive(PartialEq, Clone, Copy)]
pub enum MoveShape {
    Straight(ShapeData),
    Diagonal(ShapeData),
    Knight,
}

impl MoveShape {
    pub fn from_positions(from: Position, to: Position) -> Result<Self, String> {
        if from == to {
            return Err("From positon can't be same as to position".to_string());
        }
        let delta_file = (to.file - from.file).abs();
        let delta_rank = (to.rank - from.rank).abs();
        let distance = delta_file.max(delta_rank);

        if (delta_file == 2 && delta_rank == 1) || (delta_file == 1 && delta_rank == 2) {
            return Ok(MoveShape::Knight);
        }
        let moving_forward = to.rank > from.rank;
        let moving_backward = to.rank < from.rank;
        if delta_file * delta_rank == 0 {
            // Straight move
            return Ok(MoveShape::Straight(ShapeData {
                forward_only: moving_forward,
                backward_only: moving_backward,
                distance,
            }));
        }
        if delta_file == delta_rank {
            // Diagonal move
            return Ok(MoveShape::Diagonal(ShapeData {
                forward_only: moving_forward,
                backward_only: moving_backward,
                distance,
            }));
        }
        Err("Invalid move shape".to_string())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PieceType {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Copy, Clone, Debug)]
pub struct Piece {
    pub type_: PieceType,
    pub color: PieceColor,
}

impl Piece {
    pub fn shape_allowed(&self, shape: MoveShape) -> bool {
        match (&self.type_, shape) {
            (PieceType::Rook, MoveShape::Straight(_)) => true,
            (PieceType::Bishop, MoveShape::Diagonal(_)) => true,
            (PieceType::Queen, MoveShape::Straight(_)) => true,
            (PieceType::Queen, MoveShape::Diagonal(_)) => true,
            (PieceType::Knight, MoveShape::Knight) => true,
            (PieceType::King, MoveShape::Straight(data)) => {
                data.distance == 1 || data.distance == 2
            }
            (PieceType::King, MoveShape::Diagonal(data)) => data.distance == 1,
            (PieceType::Pawn, MoveShape::Straight(data)) => match self.color {
                PieceColor::White => {
                    data.forward_only && (data.distance == 1 || data.distance == 2)
                }
                PieceColor::Black => {
                    data.backward_only && (data.distance == 1 || data.distance == 2)
                }
            },
            (PieceType::Pawn, MoveShape::Diagonal(data)) => match self.color {
                PieceColor::White => data.forward_only && !data.backward_only && data.distance == 1,
                PieceColor::Black => !data.forward_only && data.backward_only && data.distance == 1,
            },
            _ => false,
        }
    }

    pub fn validate_pawn_rules(&self, move_: Move, is_capture: bool) -> bool {
        let Some(shape) = move_.shape() else {
            return false;
        };

        match (shape, is_capture) {
            (MoveShape::Straight(data), false) => {
                if data.distance == 2 {
                    let starting_rank = match self.color {
                        PieceColor::White => 1,
                        PieceColor::Black => 6,
                    };
                    move_.from().rank == starting_rank
                } else {
                    data.distance == 1
                }
            }
            (MoveShape::Diagonal(data), true) => data.distance == 1,
            _ => false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Move {
    from: Position,
    to: Position,
}

impl Move {
    pub fn new(from: Position, to: Position) -> Self {
        Self { from, to }
    }

    pub fn from(&self) -> Position {
        self.from
    }

    pub fn to(&self) -> Position {
        self.to
    }

    pub fn shape(&self) -> Option<MoveShape> {
        MoveShape::from_positions(self.from, self.to).ok()
    }

    pub fn is_on_board(&self) -> bool {
        self.from.is_on_board() && self.to.is_on_board()
    }

    pub fn path(&self) -> Result<Vec<Position>, String> {
        if self.from == self.to {
            return Ok(Vec::new());
        }

        let Some(shape) = self.shape() else {
            return Err("Invalid move shape".to_string());
        };

        match shape {
            MoveShape::Knight => {
                // Knight moves directly to destination, only includes the destination
                Ok(vec![self.to])
            }
            MoveShape::Straight(_) | MoveShape::Diagonal(_) => {
                let delta_file = self.to.file - self.from.file;
                let delta_rank = self.to.rank - self.from.rank;
                let step = Offset::new(delta_file.signum(), delta_rank.signum());

                let mut positions = Vec::new();
                let mut current = self.from + step;

                while current.is_on_board() {
                    positions.push(current);
                    if current == self.to {
                        break;
                    }
                    current = current + step;
                }

                Ok(positions)
            }
        }
    }
}
