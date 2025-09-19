mod board;
mod piece;

pub use board::{Board, Position};

#[cfg(test)]
mod tests {
    use super::*;

    fn perft(board: &Board, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let legal_moves = board.all_legal_moves();

        if depth == 1 {
            return legal_moves.len() as u64;
        }

        legal_moves
            .into_iter()
            .map(|move_| {
                let mut new_board = board.clone();
                new_board.make_move(move_.from(), move_.to()).unwrap();
                perft(&new_board, depth - 1)
            })
            .sum()
    }

    #[test]
    fn test_perft_positions() {
        let board = Board::starting_position();
        assert_eq!(perft(&board, 1), 20);
        assert_eq!(perft(&board, 2), 400);
        assert_eq!(perft(&board, 3), 8902);

        let board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ").unwrap();
        assert_eq!(perft(&board, 1), 14);
        assert_eq!(perft(&board, 2), 191);
        assert_eq!(perft(&board, 3), 2812);
        assert_eq!(perft(&board, 4), 43238);
        assert_eq!(perft(&board, 5), 674624);
    }
}
