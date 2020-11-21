use std::collections::HashSet;

pub const WIDTH: i16 = 6;
pub type MoveSet = HashSet<usize>;
pub type ColSet = HashSet<usize>;

pub type Pieces = u8;

pub const BLUE_PENTAGON: u8 = 0;
pub const GREEN_SQUARE: u8 = 1;
pub const BLUE_CIRCLE: u8 = 2;
pub const BREEN_OCTAGON: u8 = 3;
pub const DARK_BLUE_SQUARE: u8 = 4;
pub const PALE_CIRCLE: u8 = 5;
pub const WAVY_SQUARE: u8 = 6;
pub const CRAB: u8 = 7;
pub const PUFFERFISH: u8 = 8;
pub const JELLYFISH: u8 = 9;
pub const CLEARED: u8 = 10;
pub const NULL: u8 = 255;

pub fn str_to_enum(input: &str) -> Vec<Pieces> {
    input
        .split("")
        .filter_map(|pce| pce.parse().ok())
        .map(dani_mapper)
        .collect()
}

pub fn dani_mapper(val: i16) -> Pieces {
    val as u8
}

pub fn piece_from_num(val: i16) -> Pieces {
    val as u8
}

pub fn draw_piece(piece: Pieces) -> &'static str {
    match piece {
        CLEARED => " ",
        BLUE_PENTAGON => "A",
        GREEN_SQUARE => "B",
        BLUE_CIRCLE => "C",
        BREEN_OCTAGON => "D",
        DARK_BLUE_SQUARE => "E",
        PALE_CIRCLE => "F",
        WAVY_SQUARE => "G",
        CRAB => "H",
        PUFFERFISH => "I",
        JELLYFISH => "J",
        _ => " ",
    }
}
