use std::collections::HashSet;

pub const WIDTH: i16 = 6;
pub type MoveSet = HashSet<usize>;
pub type ColSet = HashSet<usize>;

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
pub enum Pieces {
    BluePentagon = 0,
    GreenSquare = 1,
    BlueCircle = 2,
    BreenOctagon = 3,
    DarkBlueSquare = 4,
    PaleCircle = 5,
    WavySquare = 6,
    CRAB = 7,
    PUFFERFISH = 8,
    JELLYFISH = 9,
    CLEARED = 10,
    NULL = 255,
}

pub fn str_to_enum(input: &str) -> Vec<Pieces> {
    input
        .split("")
        .filter_map(|pce| pce.parse().ok())
        .map(dani_mapper)
        .collect()
}

pub fn dani_mapper(val: i16) -> Pieces {
    piece_from_num(val)
}

pub fn piece_from_num(val: i16) -> Pieces {
    match val {
        0 => Pieces::BluePentagon,
        1 => Pieces::GreenSquare,
        2 => Pieces::BlueCircle,
        3 => Pieces::BreenOctagon,
        4 => Pieces::DarkBlueSquare,
        5 => Pieces::PaleCircle,
        6 => Pieces::WavySquare,
        7 => Pieces::CRAB,
        8 => Pieces::PUFFERFISH,
        9 => Pieces::JELLYFISH,
        10 => Pieces::CLEARED,
        _ => Pieces::NULL,
    }
}

pub fn draw_piece(piece: &Pieces) -> &str {
    match piece {
        Pieces::CLEARED => " ",
        Pieces::BluePentagon => "A",
        Pieces::GreenSquare => "B",
        Pieces::BlueCircle => "C",
        Pieces::BreenOctagon => "D",
        Pieces::DarkBlueSquare => "E",
        Pieces::PaleCircle => "F",
        Pieces::WavySquare => "G",
        Pieces::CRAB => "H",
        Pieces::PUFFERFISH => "I",
        Pieces::JELLYFISH => "J",
        Pieces::NULL => " ",
    }
}
