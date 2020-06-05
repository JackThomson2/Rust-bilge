use std::collections::HashSet;

pub const WIDTH: i16 = 6;
pub type MoveSet = HashSet<usize>;
pub type ColSet = HashSet<usize>;

pub type Pieces = u8;

pub const BluePentagon: u8 = 0;
pub const GreenSquare: u8 = 1;
pub const BlueCircle: u8 = 2;
pub const BreenOctagon: u8 = 3;
pub const DarkBlueSquare: u8 = 4;
pub const PaleCircle: u8 = 5;
pub const WavySquare: u8 = 6;
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
        BluePentagon => "A",
        GreenSquare => "B",
        BlueCircle => "C",
        BreenOctagon => "D",
        DarkBlueSquare => "E",
        PaleCircle => "F",
        WavySquare => "G",
        CRAB => "H",
        PUFFERFISH => "I",
        JELLYFISH => "J",
        _ => " ",
    }
}
