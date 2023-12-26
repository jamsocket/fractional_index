//! Implements a string-based serde serializer and deserializer for FractionalIndex.
//!
//! You can use this with serde's `with` attribute:
//!
//! ```rust
//! use fractional_index::FractionalIndex;
//! use serde::{Serialize, Deserialize};
//! use serde_json::json;
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct MyStruct {
//!   #[serde(with="fractional_index::stringify")]
//!   a: FractionalIndex,
//!   #[serde(with="fractional_index::stringify")]
//!   b: FractionalIndex,
//!   #[serde(with="fractional_index::stringify")]
//!   c: FractionalIndex,
//! }
//!
//! fn main() {
//!   let a = FractionalIndex::default();
//!   let b = FractionalIndex::new_after(&a);
//!   let c = FractionalIndex::new_between(&a, &b).unwrap();
//!
//!   let my_struct = MyStruct {
//!     a: a.clone(),
//!     b: b.clone(),
//!     c: c.clone(),
//!   };
//!
//!   let json_value = serde_json::to_value(&my_struct).unwrap();
//!
//!   let expected = json!({
//!     "a": "80",
//!     "b": "8180",
//!     "c": "817f80",
//!   });
//!
//!   assert_eq!(expected, json_value);
//! }
//! ```
use crate::FractionalIndex;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(index: &FractionalIndex, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = index.to_string();
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<FractionalIndex, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    FractionalIndex::from_string(&s).map_err(serde::de::Error::custom)
}
