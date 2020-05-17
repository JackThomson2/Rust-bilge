use std::collections::HashSet;

pub const WIDTH: i16 = 6;
pub type MoveSet = HashSet<usize>;
pub type ColSet = HashSet<usize>;

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
pub enum Pieces {
    CLEARED = 0,
    BluePentagon = 1,
    GreenSquare = 2,
    BlueCircle = 3,
    BreenOctagon = 4,
    DarkBlueSquare = 5,
    PaleCircle = 6,
    WavySquare = 7,
    CRAB = 8,
    PUFFERFISH = 9,
    JELLYFISH = 10,
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
    piece_from_num(val + 1)
}

pub fn piece_from_num(val: i16) -> Pieces {
    match val {
        0 => Pieces::CLEARED,
        1 => Pieces::BluePentagon,
        2 => Pieces::GreenSquare,
        3 => Pieces::BlueCircle,
        4 => Pieces::BreenOctagon,
        5 => Pieces::DarkBlueSquare,
        6 => Pieces::PaleCircle,
        7 => Pieces::WavySquare,
        8 => Pieces::CRAB,
        9 => Pieces::PUFFERFISH,
        10 => Pieces::JELLYFISH,
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
