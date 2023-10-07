//! A module for serializing and deserializing a [ZenoIndex] to
//! lexicographically comparable strings.
//! 
//! Deprecated along with [ZenoIndex]. The equivalent for
//! [crate::FractionalIndex] is [crate::stringify].

#![allow(deprecated)]

use crate::{
    hex::{byte_to_hex, bytes_to_hex, hex_to_bytes},
    zeno_index::{MAGIC_CEIL, ZenoIndex},
};
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(z: &ZenoIndex, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut s = bytes_to_hex(z.as_bytes());
    s.push_str(&byte_to_hex(MAGIC_CEIL));

    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<ZenoIndex, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let mut bytes = hex_to_bytes(&s).map_err(serde::de::Error::custom)?;

    if bytes.pop() != Some(MAGIC_CEIL) {
        return Err(serde::de::Error::custom("Expected trailing byte 128."));
    }

    Ok(ZenoIndex::from_bytes(bytes))
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    struct TestStruct(#[serde(with = "super")] ZenoIndex);

    fn zeno_index_to_string(z: ZenoIndex) -> String {
        let result = serde_json::to_value(TestStruct(z)).unwrap();
        let Value::String(s) = result else {
            panic!("Expected string")
        };

        s
    }

    fn string_to_zeno_index(s: &str) -> ZenoIndex {
        let TestStruct(result) =
            serde_json::from_str::<TestStruct>(&format!(r#""{}""#, s)).unwrap();
        result.clone()
    }

    #[test]
    fn test_zeno_index() {
        let mut indices: Vec<ZenoIndex> = Vec::new();

        let c = ZenoIndex::default();

        {
            let mut m = c.clone();
            let mut low = Vec::new();
            for _ in 0..20 {
                m = ZenoIndex::new_before(&m);
                low.push(m.clone())
            }

            low.reverse();
            indices.append(&mut low)
        }

        indices.push(c.clone());

        {
            let mut m = c.clone();
            let mut high = Vec::new();
            for _ in 0..20 {
                m = ZenoIndex::new_after(&m);
                high.push(m.clone())
            }

            indices.append(&mut high)
        }

        for i in 0..(indices.len() - 1) {
            assert!(
                zeno_index_to_string(indices[i].clone())
                    < zeno_index_to_string(indices[i + 1].clone())
            );
            assert_eq!(
                string_to_zeno_index(&zeno_index_to_string(indices[i].clone())),
                indices[i]
            );
        }

        for _ in 0..12 {
            let mut new_indices: Vec<ZenoIndex> = Vec::new();
            for i in 0..(indices.len() - 1) {
                let cb = ZenoIndex::new_between(&indices[i], &indices[i + 1]).unwrap();

                assert!(
                    zeno_index_to_string(indices[i].clone()) < zeno_index_to_string(cb.clone())
                );
                assert_eq!(string_to_zeno_index(&zeno_index_to_string(cb.clone())), cb);

                new_indices.push(cb);
                new_indices.push(indices[i + 1].clone());
            }

            indices = new_indices;
        }
    }
}
