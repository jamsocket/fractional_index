use crate::FractionalIndex;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(index: &FractionalIndex, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = index.to_hex();
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<FractionalIndex, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    FractionalIndex::from_hex(&s).map_err(serde::de::Error::custom)
}
