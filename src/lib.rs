#![doc = include_str!("../README.md")]

#[cfg(feature = "serde")]
pub mod lexico;
pub mod zeno;
pub mod fract_index;

pub use zeno::ZenoIndex;
