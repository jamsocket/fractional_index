use crate::{ZenoIndex, MAGIC_CEIL};
use serde::{Deserialize, Deserializer, Serializer};
use std::{error::Error, fmt::Display};

const HEX_CHARS: &[u8] = b"0123456789abcdef";

fn byte_to_hex(byte: u8) -> String {
    let mut s = String::new();
    s.push(HEX_CHARS[(byte >> 4) as usize] as char);
    s.push(HEX_CHARS[(byte & 0xf) as usize] as char);
    s
}

#[derive(Debug)]
struct InvalidChar(char);

impl Display for InvalidChar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid hex character: {}", self.0)
    }
}

impl Error for InvalidChar {
    fn description(&self) -> &str {
        "Invalid hex character"
    }
}

fn hex_to_byte(hex: &str) -> Result<u8, InvalidChar> {
    let mut byte = 0;
    for c in hex.chars() {
        byte <<= 4;
        match c {
            '0'..='9' => byte += c as u8 - b'0',
            'a'..='f' => byte += c as u8 - b'a' + 10,
            _ => return Err(InvalidChar(c)),
        }
    }
    Ok(byte)
}

pub fn serialize<S>(z: &ZenoIndex, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let bytes = z.as_bytes();
    let mut s = String::with_capacity(bytes.len() * 2 + 2);
    for byte in bytes {
        s.push_str(&byte_to_hex(*byte));
    }
    s.push_str(&byte_to_hex(MAGIC_CEIL));

    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<ZenoIndex, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let mut bytes = Vec::with_capacity(s.len() / 2);
    for i in 0..s.len() / 2 {
        bytes.push(hex_to_byte(&s[i * 2..i * 2 + 2]).map_err(serde::de::Error::custom)?);
    }

    if bytes.pop() != Some(MAGIC_CEIL) {
        return Err(serde::de::Error::custom("Expected trailing byte 128."));
    }

    Ok(ZenoIndex::from_bytes(bytes))
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Serialize, Deserialize};
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
        let TestStruct(result) = serde_json::from_str::<TestStruct>(&format!(r#""{}""#, s)).unwrap();
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
            assert!(zeno_index_to_string(indices[i].clone()) < zeno_index_to_string(indices[i + 1].clone()));
            assert_eq!(
                string_to_zeno_index(&zeno_index_to_string(indices[i].clone())),
                indices[i]
            );
        }

        for _ in 0..12 {
            let mut new_indices: Vec<ZenoIndex> = Vec::new();
            for i in 0..(indices.len() - 1) {
                let cb = ZenoIndex::new_between(&indices[i], &indices[i + 1]).unwrap();

                assert!(zeno_index_to_string(indices[i].clone()) < zeno_index_to_string(cb.clone()));
                assert_eq!(
                    string_to_zeno_index(&zeno_index_to_string(cb.clone())),
                    cb
                );

                new_indices.push(cb);
                new_indices.push(indices[i + 1].clone());
            }

            indices = new_indices;
        }
    }
}
