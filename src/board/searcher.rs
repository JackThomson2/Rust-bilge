use crate::board::{GameState, Move};

pub fn find_best_move(board: &GameState) -> Move {
    println!("Finding best move");

    let possible_moves = board.get_moves();

    println!("there are {} moves", possible_moves.len());

    let mut best_score = std::i32::MIN;
    let mut best_move = *possible_moves.get(0).unwrap();

    for testing in possible_moves {
        let mut test_board = board.clone();

        let new_score = test_board.swap(&testing);

        if new_score > best_score {
            best_move = testing.clone();
            best_score = new_score;
        }
    }

    println!(
        "Best move found as {:#?} with score {}",
        best_move, best_score
    );

    best_move
}
