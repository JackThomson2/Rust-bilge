pub const WIDTH: i16 = 6;

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum Pieces {
    CLEARED = 0,
    BLUE_PENTAGON = 1,
    GREEN_SQUARE = 2,
    BLUE_CIRCLE = 3,
    BREEN_OCTAGON = 4,
    DARK_BLUE_SQUARE = 5,
    PALE_CIRCLE = 6,
    WAVY_SQUARE = 7,
    CRAB = 8,
    PUFFERFISH = 9,
    JELLYFISH = 10,
    NULL = 999,
}

pub fn piece_from_num(val: &i16) -> Pieces {
    match val {
        0 => Pieces::CLEARED,
        1 => Pieces::BLUE_PENTAGON,
        2 => Pieces::GREEN_SQUARE,
        3 => Pieces::BLUE_CIRCLE,
        4 => Pieces::BREEN_OCTAGON,
        5 => Pieces::DARK_BLUE_SQUARE,
        6 => Pieces::PALE_CIRCLE,
        7 => Pieces::WAVY_SQUARE,
        8 => Pieces::CRAB,
        9 => Pieces::PUFFERFISH,
        10 => Pieces::JELLYFISH,
        _ => Pieces::NULL,
    }
}

pub fn draw_piece(piece: &Pieces) -> &str {
    match piece {
        Pieces::CLEARED => " ",
        Pieces::BLUE_PENTAGON => "A",
        Pieces::GREEN_SQUARE => "B",
        Pieces::BLUE_CIRCLE => "C",
        Pieces::BREEN_OCTAGON => "D",
        Pieces::DARK_BLUE_SQUARE => "E",
        Pieces::PALE_CIRCLE => "F",
        Pieces::WAVY_SQUARE => "G",
        Pieces::CRAB => "H",
        Pieces::PUFFERFISH => "I",
        Pieces::JELLYFISH => "J",
        Pieces::NULL => " ",
        _ => " ",
    }
}
