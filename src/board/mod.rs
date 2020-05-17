#[macro_use]
pub mod helpers;

pub mod defs;
pub mod generator;
pub mod searcher;
pub mod structure;
pub mod transforms;

pub use generator::*;
pub use helpers::*;
pub use structure::*;
pub use transforms::*;

use seahash;

use colored::*;
use defs::Pieces::*;
use helpers::can_move;

use arrayvec::ArrayVec;
