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
pub enum MoveTurn {
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
    en_passant_target: Option<Position>,
}

impl Board {
    pub fn new(
        pieces: [Option<Piece>; (BOARD_WIDTH * BOARD_HEIGHT) as usize],
        move_turn: MoveTurn,
        castling_rights: CastlingRights,
        en_passant_target: Option<Position>,
    ) -> Self {
        Self {
            pieces,
            move_turn,
            castling_rights,
            en_passant_target,
        }
    }

    pub fn starting_position() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn empty() -> Self {
        Self::new(
            [const { None }; (BOARD_WIDTH * BOARD_HEIGHT) as usize],
            MoveTurn::White,
            CastlingRights::new(),
            None,
        )
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            return Err("FEN string must have 6 parts".to_string());
        }

        let piece_placement = parts[0];
        let active_color = parts[1];
        let castling_rights_str = parts[2];
        let en_passant_square = parts[3];

        if piece_placement.split('/').count() != 8 {
            return Err("FEN piece placement must have 8 ranks".to_string());
        }

        // Expands digits like "8" -> "........"
        let expand_digit = |c: char| -> Vec<char> {
            if let Some(digit) = c.to_digit(10) {
                vec!['.'; digit as usize]
            } else {
                vec![c]
            }
        };

        let char_to_piece = |ch: char| -> Result<Option<Piece>, String> {
            if ch == '.' {
                return Ok(None);
            }
            let (piece_type, color) = match ch {
                'P' => (PieceType::Pawn, PieceColor::White),
                'N' => (PieceType::Knight, PieceColor::White),
                'B' => (PieceType::Bishop, PieceColor::White),
                'R' => (PieceType::Rook, PieceColor::White),
                'Q' => (PieceType::Queen, PieceColor::White),
                'K' => (PieceType::King, PieceColor::White),
                'p' => (PieceType::Pawn, PieceColor::Black),
                'n' => (PieceType::Knight, PieceColor::Black),
                'b' => (PieceType::Bishop, PieceColor::Black),
                'r' => (PieceType::Rook, PieceColor::Black),
                'q' => (PieceType::Queen, PieceColor::Black),
                'k' => (PieceType::King, PieceColor::Black),
                _ => return Err(format!("Invalid piece character: {}", ch)),
            };
            Ok(Some(Piece {
                type_: piece_type,
                color,
            }))
        };

        let ranks: Vec<&str> = piece_placement.split('/').collect();

        // Reverse the order of ranks and expand digits
        let chars: Vec<char> = ranks
            .iter()
            .rev()
            .flat_map(|rank| rank.chars())
            .flat_map(expand_digit)
            .collect();

        // Convert chars to pieces
        let pieces: Vec<Option<Piece>> = chars
            .into_iter()
            .map(char_to_piece)
            .collect::<Result<Vec<Option<Piece>>, String>>()?;

        let pieces: [Option<Piece>; (BOARD_WIDTH * BOARD_HEIGHT) as usize] = pieces
            .try_into()
            .map_err(|_| "Invalid number of pieces in FEN")?;

        // Parse active color
        let move_turn = match active_color {
            "w" => MoveTurn::White,
            "b" => MoveTurn::Black,
            _ => return Err("Invalid active color".to_string()),
        };

        // Parse castling rights
        let castling_rights = CastlingRights {
            white_kingside: castling_rights_str.contains('K'),
            white_queenside: castling_rights_str.contains('Q'),
            black_kingside: castling_rights_str.contains('k'),
            black_queenside: castling_rights_str.contains('q'),
        };

        // Parse en passant target square
        let en_passant_target = match en_passant_square {
            "-" => None,
            square => {
                if square.len() != 2 {
                    return Err("Invalid en passant square".to_string());
                }
                let file_char = square.chars().nth(0).unwrap();
                let rank_char = square.chars().nth(1).unwrap();

                let file = (file_char as i8) - 'a' as i8;
                let rank = (rank_char as i8) - '1' as i8;

                let pos = Position::new(file, rank);
                if pos.is_on_board() {
                    Some(pos)
                } else {
                    return Err("En passant square out of bounds".to_string());
                }
            }
        };

        Ok(Board::new(
            pieces,
            move_turn,
            castling_rights,
            en_passant_target,
        ))
    }

    fn piece_at_pos(&self, pos: Position) -> Option<Piece> {
        let Ok(index) = pos.to_index() else {
            return None;
        };
        self.pieces[index]
    }

    fn cast_ray(
        &self,
        start_pos: Position,
        direction: Offset,
    ) -> Result<(Position, Option<Piece>), String> {
        let mut current = start_pos + direction;
        let mut last_valid = None;

        while current.is_on_board() {
            last_valid = Some(current);
            if let Some(piece) = self.piece_at_pos(current) {
                return Ok((current, Some(piece)));
            }
            current = current + direction;
        }

        match last_valid {
            Some(pos) => Ok((pos, None)),
            None => Err("No valid positions in this direction".to_string()),
        }
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
            if let Ok((piece_pos, Some(piece))) = self.cast_ray(square_pos, direction) {
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
        if self.is_move_en_passant(move_) {
            return true;
        }
        let Some(moving_piece) = self.piece_at_pos(move_.from()) else {
            return false;
        };
        let Some(target_piece) = self.piece_at_pos(move_.to()) else {
            return false;
        };
        moving_piece.color != target_piece.color
    }

    fn is_move_en_passant(&self, move_: Move) -> bool {
        let Some(en_passant_target) = self.en_passant_target else {
            return false;
        };
        if move_.to() != en_passant_target {
            return false;
        }
        let Some(moving_piece) = self.piece_at_pos(move_.from()) else {
            return false;
        };
        if !matches!(moving_piece.type_, PieceType::Pawn) {
            return false;
        }
        match (move_.shape(), moving_piece.color) {
            (
                Some(MoveShape::Diagonal(ShapeData {
                    forward_only: true,
                    distance: 1,
                    ..
                })),
                PieceColor::White,
            ) => true,
            (
                Some(MoveShape::Diagonal(ShapeData {
                    backward_only: true,
                    distance: 1,
                    ..
                })),
                PieceColor::Black,
            ) => true,
            _ => false,
        }
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

        if let Ok((
            _,
            Some(Piece {
                type_: PieceType::King,
                ..
            }),
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
        let Some(_piece) = self.piece_at_pos(pos) else {
            return Vec::new();
        };

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

        let ray_directions = [
            Offset::new(1, 0),
            Offset::new(-1, 0),
            Offset::new(0, 1),
            Offset::new(0, -1),
            Offset::new(1, 1),
            Offset::new(1, -1),
            Offset::new(-1, 1),
            Offset::new(-1, -1),
        ];

        let knight_moves = knight_offsets
            .into_iter()
            .map(|offset| pos + offset)
            .map(|to_pos| Move::new(pos, to_pos));

        let sliding_moves = ray_directions
            .into_iter()
            .filter_map(|dir| self.cast_ray(pos, dir).ok())
            .map(|(hit_pos, _piece)| Move::new(pos, hit_pos))
            .filter_map(|move_| move_.path().ok())
            .flatten()
            .map(|target_pos| Move::new(pos, target_pos));

        knight_moves
            .chain(sliding_moves)
            .filter(|&move_| self.move_legal(move_))
            .map(|move_| move_.to())
            .collect()
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

        if self.is_move_en_passant(move_) {
            let captured_pawn_pos = Position::new(move_.to().file, move_.from().rank);
            self.set(captured_pawn_pos, None)?;
        }

        self.move_piece(move_.from(), move_.to())?;

        self.update_castling_rights_for_move(move_);
        self.update_en_passant_target(move_);
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

    fn update_en_passant_target(&mut self, move_: Move) {
        if !matches!(
            move_.shape(),
            Some(MoveShape::Straight(ShapeData { distance: 2, .. }))
        ) {
            self.en_passant_target = None;
            return;
        }
        if !matches!(
            self.piece_at_pos(move_.to()),
            Some(Piece {
                type_: PieceType::Pawn,
                ..
            })
        ) {
            self.en_passant_target = None;
            return;
        }

        let target_rank = (move_.from().rank + move_.to().rank) / 2;
        self.en_passant_target = Some(Position::new(move_.to().file, target_rank));
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
        // Black knight on c5, white rook on b5
        let board = Board::from_fen("8/8/8/1Rn5/8/8/8/8 w - - 0 1").unwrap();
        let black_knight_position = Position::new(2, 4);
        let white_rook_position = Position::new(1, 4);

        assert!(board.move_pseudo_legal(Move::new(white_rook_position, Position::new(1, 0))));
        assert!(board.move_pseudo_legal(Move::new(black_knight_position, Position::new(1, 2))));

        assert!(!board.move_pseudo_legal(Move::new(white_rook_position, Position::new(7, 4))));
        assert!(!board.move_pseudo_legal(Move::new(white_rook_position, Position::new(1, 8))));
        assert!(!board.move_pseudo_legal(Move::new(white_rook_position, Position::new(7, 7))));
        assert!(!board.move_pseudo_legal(Move::new(black_knight_position, Position::new(4, 4))));
    }

    #[test]
    fn test_is_in_check() {
        // White king on e1, black rook on e8 attacking it
        let board = Board::from_fen("4r3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        // White king should be in check from black rook
        assert!(board.is_in_check(PieceColor::White));

        // Black should not be in check (no black king on board)
        assert!(!board.is_in_check(PieceColor::Black));

        // White king on e1, black rook on e8, but white rook on e4 blocks the check
        let board2 = Board::from_fen("4r3/8/8/8/4R3/8/8/4K3 w - - 0 1").unwrap();
        assert!(!board2.is_in_check(PieceColor::White));
    }

    #[test]
    fn test_pinned_piece() {
        // White king on e1, white rook on e4, black rook on e8, white rook pinned
        let board = Board::from_fen("4r3/8/8/8/4R3/8/8/4K3 w - - 0 1").unwrap();

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
        // White king and rook in starting positions, but black knight attacks king's path
        let board = Board::from_fen("r3k2r/8/8/8/8/4n3/8/R3K2R w KQkq - 0 1").unwrap();
        let kingside_castle = Move::new(Position::new(4, 0), Position::new(6, 0));
        assert!(!board.move_legal(kingside_castle));

        // Black king and queenside rook, white rook on b5, castling should be legal
        let mut board2 = Board::from_fen("r3k3/8/8/8/1R6/8/8/8 b q - 0 1").unwrap();

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

        // Test castling after rook capture - white king and rook, black knight captures rook
        let mut board3 = Board::from_fen("8/8/8/8/8/8/6n1/R3K3 w Q - 0 1").unwrap();
        board3
            .make_move(Move::new(Position::new(6, 1), Position::new(0, 0)))
            .unwrap();

        let queenside_castle = Move::new(Position::new(4, 0), Position::new(2, 0));
        assert!(!board3.move_legal(queenside_castle));
    }

    #[test]
    fn test_en_passant() {
        // White pawn on e5, black pawn on f7
        let mut board = Board::from_fen("8/5p2/8/4P3/8/8/8/8 w - - 0 1").unwrap();

        board
            .make_move(Move::new(Position::new(5, 6), Position::new(5, 4)))
            .unwrap();

        let en_passant_move = Move::new(Position::new(4, 4), Position::new(5, 5));
        assert!(board.is_move_en_passant(en_passant_move));

        let mut board2 = Board::from_fen("8/8/8/8/8/8/8/R7 w - - 0 1").unwrap();
        board2
            .make_move(Move::new(Position::new(0, 0), Position::new(0, 1)))
            .unwrap();

        assert!(!board2.is_move_en_passant(en_passant_move));
    }
}
