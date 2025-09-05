#[derive(Clone)]

pub struct Offset {
    pub file: i8,
    pub rank: i8,
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

impl PieceType {
    pub fn get_offsets(&self) -> Vec<Offset> {
        match self {
            Self::Pawn => vec![Offset { file: 0, rank: 1 }],
            Self::Knight => vec![
                Offset { file: 2, rank: 1 },
                Offset { file: 2, rank: -1 },
                Offset { file: -2, rank: 1 },
                Offset { file: -2, rank: -1 },
                Offset { file: 1, rank: 2 },
                Offset { file: 1, rank: -2 },
                Offset { file: -1, rank: 2 },
                Offset { file: -1, rank: -2 },
            ],
            Self::Bishop => {
                let mut offsets = Vec::new();
                for i in 1..8 {
                    offsets.push(Offset { file: i, rank: i });
                    offsets.push(Offset { file: i, rank: -i });
                    offsets.push(Offset { file: -i, rank: i });
                    offsets.push(Offset { file: -i, rank: -i });
                }
                offsets
            }
            Self::Rook => {
                let mut offsets = Vec::new();
                for i in 1..8 {
                    offsets.push(Offset { file: i, rank: 0 });
                    offsets.push(Offset { file: -i, rank: 0 });
                    offsets.push(Offset { file: 0, rank: i });
                    offsets.push(Offset { file: 0, rank: -i });
                }
                offsets
            }
            Self::Queen => {
                let mut offsets = Vec::new();
                for i in 1..8 {
                    offsets.push(Offset { file: i, rank: i });
                    offsets.push(Offset { file: i, rank: -i });
                    offsets.push(Offset { file: -i, rank: i });
                    offsets.push(Offset { file: -i, rank: -i });
                    offsets.push(Offset { file: i, rank: 0 });
                    offsets.push(Offset { file: -i, rank: 0 });
                    offsets.push(Offset { file: 0, rank: i });
                    offsets.push(Offset { file: 0, rank: -i });
                }
                offsets
            }
            Self::King => vec![
                Offset { file: 1, rank: 0 },
                Offset { file: -1, rank: 0 },
                Offset { file: 0, rank: 1 },
                Offset { file: 0, rank: -1 },
                Offset { file: 1, rank: 1 },
                Offset { file: 1, rank: -1 },
                Offset { file: -1, rank: 1 },
                Offset { file: -1, rank: -1 },
            ],
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Clone)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: PieceColor,
}
