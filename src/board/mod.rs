mod defs;

use dashmap::DashMap;

use num_cpus;

use std::sync::{Arc, Mutex};

use rand::Rng;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::thread;

use fasthash::{FastHasher, MetroHasher};

pub type PeerMap = Arc<dashmap::DashMap<u64, HashEntry>>;

pub struct HashEntry {
    search_res: SearchResult,
    depth: i8,
}

#[derive(Clone)]
pub struct GameState {
    board: [defs::Pieces; 72],
}

#[derive(Debug, Copy, Clone)]
pub struct SearchResult {
    move_id: usize,
    score: i16,
}

impl GameState {
    pub fn draw(&self) {
        let mut cntr = 0;
        for i in self.board.iter() {
            print!("{}", defs::draw_piece(i));
            cntr += 1;
            if defs::WIDTH == cntr {
                println!();
                cntr = 0;
            }
        }
    }

    pub fn swap(&mut self, pos: usize) {
        if pos % 5 == 0 {
            return;
        }

        self.board.swap(pos, pos + 1);
    }

    pub fn get_moves(&self) -> Vec<usize> {
        let mut moves = Vec::new();

        for i in 0..self.board.len() - 1 {
            if i % 5 == 0 {
                continue;
            }

            if self.board[i] == defs::Pieces::CLEARED || self.board[i] == defs::Pieces::NULL {
                continue;
            }

            if self.board[i + 1] == defs::Pieces::CLEARED || self.board[i + 1] == defs::Pieces::NULL
            {
                continue;
            }

            if self.board[i] != self.board[i + 1] {
                moves.push(i)
            }
        }

        return moves;
    }

    fn hash_me(&self) -> u64 {
        let mut s = MetroHasher::default();
        self.board.hash(&mut s);
        s.finish()
    }

    fn shift_everything(&mut self) {
        for x in 0..6 {
            let mut last = 9999;
            for y in 0..12 {
                let piece = self.board[x + (y * 6)];

                if piece == defs::Pieces::CLEARED && last == 9999 {
                    last = y;
                }

                if last != 9999 && piece != defs::Pieces::CLEARED && piece != defs::Pieces::NULL {
                    self.board[x + (last * 6)] = piece;
                    self.board[x + (y * 6)] = defs::Pieces::CLEARED;
                    last += 1;
                }
            }
        }
    }

    pub fn clear_board(&mut self) -> i16 {
        let mut clears = 0;
        let mut new_board = self.board.clone();
        let mut broken = false;

        for i in 0..self.board.len() {
            let piece = self.board[i];
            if piece == defs::Pieces::CLEARED || piece == defs::Pieces::NULL {
                continue;
            }

            let x = i % 6;
            let y = i / 6 as usize;

            if x > 1 && piece == self.board[i - 1] && piece == self.board[i - 2] {
                clears += 1;
                new_board[i] = defs::Pieces::CLEARED;
                new_board[i - 1] = defs::Pieces::CLEARED;
                new_board[i - 2] = defs::Pieces::CLEARED;

                broken = true;
            }

            if x < 4 && piece == self.board[i + 1] && piece == self.board[i + 2] {
                clears += 1;
                new_board[i] = defs::Pieces::CLEARED;
                new_board[i + 1] = defs::Pieces::CLEARED;
                new_board[i + 2] = defs::Pieces::CLEARED;

                broken = true;
            }

            if y > 1 && piece == self.board[i - 6] && piece == self.board[i - 12] {
                clears += 1;
                new_board[i] = defs::Pieces::CLEARED;
                new_board[i - 6] = defs::Pieces::CLEARED;
                new_board[i - 12] = defs::Pieces::CLEARED;

                broken = true;
            }

            if y < 4 && piece == self.board[i + 6] && piece == self.board[i + 12] {
                clears += 1;
                new_board[i] = defs::Pieces::CLEARED;
                new_board[i + 6] = defs::Pieces::CLEARED;
                new_board[i + 12] = defs::Pieces::CLEARED;

                broken = true;
            }
        }

        self.board = new_board;

        if broken {
            self.shift_everything();
        }

        clears
    }
}

fn dfs(start: GameState, map: PeerMap, depth: i8) -> SearchResult {
    let hash = start.hash_me();

    if map.contains_key(&hash) {
        let found = map.get(&hash).unwrap();

        if found.depth > depth {
            return found.search_res;
        }
    }

    let moves = start.get_moves();

    let mut best_move_score = -9999;
    let mut best_move = 0;

    for i in moves.iter() {
        let mut clone = copy_board(&start);
        clone.swap(*i);

        let mut score = clone.clear_board();

        if depth > 1 {
            score += dfs(clone, map.clone(), depth - 1).score;
        }

        if score > best_move_score {
            best_move_score = score;
            best_move = *i;
        }
    }

    let res = SearchResult {
        move_id: best_move,
        score: best_move_score,
    };

    map.insert(
        hash,
        HashEntry {
            search_res: res,
            depth,
        },
    );

    res
}

pub fn search_board(start: GameState) -> usize {
    let map = PeerMap::new(DashMap::new());

    let moves = start.get_moves();
    let moves_len = moves.len();
    let mut move_values = moves.into_iter().peekable();

    let mut children = vec![];

    let mut best_move_score = -9999;
    let mut best_move = 0;

    let max = num_cpus::get();

    for c in 0..max {
        let copy = map.clone();
        let chunk: Vec<_> = move_values.by_ref().take(moves_len / max).collect();

        let board_copy = copy_board(&start);
        children.push(thread::spawn(move || {
            println!("Spinning up thread {}", c);

            let mut best_move_score_thread = -9999;
            let mut best_move_thread = 0;
            for i in chunk.iter() {
                let mut clone = copy_board(&board_copy);
                clone.swap(*i);
                let score = dfs(clone, copy.clone(), 4).score;
                if score > best_move_score_thread {
                    best_move_score_thread = score;
                    best_move_thread = *i;
                }
            }

            println!("Thread {} is done", c);

            SearchResult {
                move_id: best_move_thread,
                score: best_move_score_thread,
            }
        }));
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let res = child.join().unwrap();

        if res.score > best_move_score {
            best_move_score = res.score;
            best_move = res.move_id;
        }
    }

    println!("Final hash table size {}", map.len());

    println!("Best score was {}", best_move_score);

    best_move
}

pub fn generate_rand_board() -> GameState {
    let mut board = [defs::Pieces::CLEARED; 72];
    let mut rng = rand::thread_rng();

    for i in board.iter_mut() {
        *i = defs::piece_from_num(&rng.gen_range(1, 7))
    }

    GameState { board }
}

pub fn copy_board(copying: &GameState) -> GameState {
    copying.clone()
}

pub fn generate_game() -> GameState {
    GameState {
        board: [defs::Pieces::CLEARED; 72],
    }
}
