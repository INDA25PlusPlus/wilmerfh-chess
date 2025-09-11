use crate::board::Position;

#[derive(Copy, Clone)]
pub struct Offset {
    pub file: i8,
    pub rank: i8,
}

#[derive(PartialEq, Clone, Copy)]
pub struct StraightData {
    pub forward_only: bool,
    pub backward_only: bool,
    pub distance: i8,
}

#[derive(PartialEq, Clone, Copy)]
pub struct DiagonalData {
    pub forward_only: bool,
    pub backward_only: bool,
    pub distance: i8,
}

#[derive(PartialEq, Clone, Copy)]
pub enum StraightDirection {
    North,
    East,
    South,
    West,
}

#[derive(PartialEq, Clone, Copy)]
pub enum DiagonalDirection {
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
}

#[derive(PartialEq, Clone, Copy)]
pub enum MoveShape {
    Straight(StraightData),
    Diagonal(DiagonalData),
    Knight,
}

impl MoveShape {
    pub fn straight_forward(distance: i8) -> Self {
        Self::Straight(StraightData {
            forward_only: true,
            backward_only: false,
            distance,
        })
    }

    pub fn straight_backward(distance: i8) -> Self {
        Self::Straight(StraightData {
            forward_only: false,
            backward_only: true,
            distance,
        })
    }

    pub fn straight_any_direction(distance: i8) -> Self {
        Self::Straight(StraightData {
            forward_only: false,
            backward_only: false,
            distance,
        })
    }

    pub fn straight_sliding() -> Self {
        Self::Straight(StraightData {
            forward_only: false,
            backward_only: false,
            distance: 0,
        })
    }

    pub fn diagonal_forward(distance: i8) -> Self {
        Self::Diagonal(DiagonalData {
            forward_only: true,
            backward_only: false,
            distance,
        })
    }

    pub fn diagonal_backward(distance: i8) -> Self {
        Self::Diagonal(DiagonalData {
            forward_only: false,
            backward_only: true,
            distance,
        })
    }

    pub fn diagonal_any_direction(distance: i8) -> Self {
        Self::Diagonal(DiagonalData {
            forward_only: false,
            backward_only: false,
            distance,
        })
    }

    pub fn diagonal_sliding() -> Self {
        Self::Diagonal(DiagonalData {
            forward_only: false,
            backward_only: false,
            distance: 0,
        })
    }

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
            return Ok(MoveShape::Straight(StraightData {
                forward_only: moving_forward,
                backward_only: moving_backward,
                distance,
            }));
        }
        if delta_file == delta_rank {
            // Diagonal move
            return Ok(MoveShape::Diagonal(DiagonalData {
                forward_only: moving_forward,
                backward_only: moving_backward,
                distance,
            }));
        }
        Err("Invalid move shape".to_string())
    }
}

#[derive(Clone)]
pub enum PieceType {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}


#[derive(Clone, PartialEq)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Clone)]
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
            (PieceType::King, MoveShape::Straight(data)) => data.distance == 1,
            (PieceType::King, MoveShape::Diagonal(data)) => data.distance == 1,
            (PieceType::Pawn, MoveShape::Straight(data)) => {
                match self.color {
                    PieceColor::White => data.forward_only && !data.backward_only && (data.distance == 1 || data.distance == 2),
                    PieceColor::Black => !data.forward_only && data.backward_only && (data.distance == 1 || data.distance == 2),
                }
            },
            (PieceType::Pawn, MoveShape::Diagonal(data)) => {
                match self.color {
                    PieceColor::White => data.forward_only && !data.backward_only && data.distance == 1,
                    PieceColor::Black => !data.forward_only && data.backward_only && data.distance == 1,
                }
            },
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
    
}
