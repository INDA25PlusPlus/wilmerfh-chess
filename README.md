# lachess

## Usage

### Creating a board

```rust
use lachess::{Board, Position, MoveResult, PieceType};

// New game
let mut board = Board::starting_position();

// Empty board
let mut board = Board::empty();

// From FEN
let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
```

### Making moves

```rust

// Positions use file (0-7) and rank (0-7)
let from = Position::new(4, 1); // e2
let to = Position::new(4, 3);   // e4

match board.make_move(from, to) {
    MoveResult::Normal => println!("Move made"),
    MoveResult::Promotion => {
        println!("Pawn promotion!");
        // Choose promotion piece
        board.resolve_promotion(PieceType::Queen).unwrap();
        // Or cancel the promotion
        // board.cancel_promotion();
    },
    MoveResult::Illegal => println!("Illegal move"),
}
```

### Getting legal moves

```rust
// Legal moves from a position
let from = Position::new(4, 1); // e2
let legal_destinations = board.legal_moves(from);

// All legal moves in current position
let all_moves = board.all_legal_moves();
```

### Checking game state

```rust
// Check if current player is in check
if board.is_in_check() {
    println!("Current player is in check");
}

// Check for game end
if board.is_checkmate() {
    println!("Checkmate");
} else if board.is_stalemate() {
    println!("Stalemate");
}
```

### Position coordinates

- Files: 0-7 (a-h)
- Ranks: 0-7 (1-8)
- Bottom-left (a1) is (0, 0)
- Top-right (h8) is (7, 7)