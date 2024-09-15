#![doc = include_str!("../README.md")]

mod hex;
#[cfg(feature = "serde")]
pub mod stringify;

// We use `any` so we can add more database implementations.
#[cfg(any(feature = "with-sqlx-postgres"))]
mod sqlx;

mod fract_index;

#[cfg(feature = "serde")]
#[deprecated(
    since = "2.0.0",
    note = "Use FractionalIndex and fractional_index::stringify instead"
)]
pub mod lexico;

#[deprecated(since = "2.0.0", note = "Use FractionalIndex instead")]
pub mod zeno_index;

pub use fract_index::FractionalIndex;
#[allow(deprecated)]
pub use zeno_index::ZenoIndex;
