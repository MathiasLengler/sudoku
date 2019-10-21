use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::mem::replace;
use std::num::NonZeroU8;

use fixedbitset::FixedBitSet;
use num::{cast, ToPrimitive};

pub use compact::Cell;

use crate::cell::view::CellView;

pub mod compact;
pub mod view;

// TODO: decide if useful
pub trait SudokuCell {}
