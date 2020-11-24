pub const WIDTH: i16 = 6;

pub type Pieces = u8;

pub const BLUE_PENTAGON: u8    = 0b0000_0000;
pub const GREEN_SQUARE: u8     = 0b0000_0001;
pub const BLUE_CIRCLE: u8      = 0b0000_0010;
pub const BREEN_OCTAGON: u8    = 0b0000_0011;
pub const DARK_BLUE_SQUARE: u8 = 0b0000_0100;
pub const PALE_CIRCLE: u8      = 0b0000_0101;
pub const WAVY_SQUARE: u8      = 0b0000_0110;
pub const CRAB: u8             = 0b0000_0111;
pub const PUFFERFISH: u8       = 0b0000_1000;
pub const JELLYFISH: u8        = 0b0000_1001;
pub const CLEARED: u8          = 0b0000_1010;
pub const NULL: u8             = 0b1111_1111;

pub const TRI_TOP:u8           = 0b0001_0000;
pub const TRI_BOT:u8           = 0b1001_0000;
pub const SQR_TOP:u8           = 0b0010_0000;
pub const SQR_BOT:u8           = 0b1010_0000;
pub const ARR_TOP:u8           = 0b0100_0000;
pub const ARR_BOT:u8           = 0b1100_0000;
pub const CRO_TOP:u8           = 0b0101_0000;
pub const CRO_BOT:u8           = 0b1101_0000;
pub const TRA_TOP:u8           = 0b0110_0000;
pub const TRA_BOT:u8           = 0b1110_0000;

pub const UPPER_BIT_MASK:u8    = 0b1111_0000;
pub const LOWER_BIT_MASK:u8    = 0b0000_1111;
pub const BOT_BIT_MASK:u8      = 0b1000_0000;

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
