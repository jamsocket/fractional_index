use crate::hex::{bytes_to_hex, hex_to_bytes};
use std::{
    error::Error,
    fmt::{self, Display},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub(crate) const TERMINATOR: u8 = 0b1000_0000; // =128

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FractionalIndex(Vec<u8>);

impl Default for FractionalIndex {
    fn default() -> Self {
        FractionalIndex(vec![TERMINATOR])
    }
}

fn new_before(bytes: &[u8]) -> Vec<u8> {
    for i in 0..bytes.len() {
        if bytes[i] > TERMINATOR {
            // If we encounter a byte greater than TERMINATOR, we can
            // create a byte string that comes lexicographically before
            // it (after appending the terminator to both strings) by
            // truncating the string just before this byte.
            return bytes[0..i].into();
        }
        if bytes[i] > u8::MIN {
            // If we encounter a byte greater than 0, we can create a
            // byte string that comes lexicographically before it by
            // decrementing that byte and truncating the string there.
            let mut bytes: Vec<u8> = bytes[0..=i].into();
            bytes[i] -= 1;
            return bytes;
        }
    }

    panic!("We should never reach the end of a properly-terminated fractional index without finding a byte greater than 0.")
}

fn new_after(bytes: &[u8]) -> Vec<u8> {
    for i in 0..bytes.len() {
        if bytes[i] < TERMINATOR {
            // If we encounter a byte less than TERMINATOR, we can
            // create a byte string that comes lexicographically after
            // it (after appending the terminator to both strings) by
            // truncating the string just before this byte.
            return bytes[0..i].into();
        }
        if bytes[i] < u8::MAX {
            // If we encounter a byte less than 255, we can create a
            // byte string that comes lexicographically after it by
            // incrementing that byte and truncating the string there.
            let mut bytes: Vec<u8> = bytes[0..=i].into();
            bytes[i] += 1;
            return bytes;
        }
    }

    panic!("We should never reach the end of a properly-terminated fractional index without finding a byte less than 255.")
}

#[derive(Debug)]
pub enum DecodeError {
    EmptyString,
    MissingTerminator,
    InvalidBase64,
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::EmptyString => write!(f, "Empty string"),
            DecodeError::MissingTerminator => write!(f, "Missing terminator"),
            DecodeError::InvalidBase64 => write!(f, "Invalid base64"),
        }
    }
}

impl Error for DecodeError {}

impl FractionalIndex {
    /// Constructs a FractionalIndex from a byte vec, which DOES NOT include
    /// the terminating byte.
    fn from_vec_unterminated(mut bytes: Vec<u8>) -> Self {
        bytes.push(TERMINATOR);
        FractionalIndex(bytes)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, DecodeError> {
        if bytes.last() != Some(&TERMINATOR) {
            return Err(DecodeError::MissingTerminator);
        }
        Ok(FractionalIndex(bytes))
    }

    /// Returns the byte representation of this FractionalIndex, which DOES
    /// INCLUDE the terminating byte.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        bytes_to_hex(&self.0)
    }

    pub fn from_hex(s: &str) -> Result<Self, DecodeError> {
        if s.is_empty() {
            return Err(DecodeError::EmptyString);
        }

        let bytes = hex_to_bytes(s).map_err(|_| DecodeError::InvalidBase64)?;

        if bytes.last() != Some(&TERMINATOR) {
            return Err(DecodeError::MissingTerminator);
        }

        FractionalIndex::from_bytes(bytes)
    }

    pub fn new_before(FractionalIndex(bytes): &FractionalIndex) -> FractionalIndex {
        FractionalIndex::from_vec_unterminated(new_before(bytes))
    }

    pub fn new_after(FractionalIndex(bytes): &FractionalIndex) -> FractionalIndex {
        FractionalIndex::from_vec_unterminated(new_after(bytes))
    }

    pub fn new_between(
        FractionalIndex(left): &FractionalIndex,
        FractionalIndex(right): &FractionalIndex,
    ) -> Option<FractionalIndex> {
        let shorter_len = std::cmp::min(left.len(), right.len()) - 1;
        for i in 0..shorter_len {
            if left[i] < right[i] - 1 {
                let mut bytes: Vec<u8> = left[0..=i].into();
                bytes[i] += (right[i] - left[i]) / 2;
                return Some(FractionalIndex::from_vec_unterminated(bytes));
            }

            if left[i] == right[i] - 1 {
                let (prefix, suffix) = left.split_at(i + 1);
                let mut bytes = Vec::with_capacity(suffix.len() + prefix.len() + 1);
                bytes.extend_from_slice(&prefix);
                bytes.extend_from_slice(&new_after(&suffix));
                return Some(FractionalIndex::from_vec_unterminated(bytes));
            }

            if left[i] > right[i] {
                // We return None if right is greater than left.
                return None;
            }
        }

        if left.len() < right.len() {
            let (prefix, suffix) = right.split_at(shorter_len + 1);
            let new_suffix = new_before(&suffix);
            let mut bytes = Vec::with_capacity(new_suffix.len() + prefix.len() + 1);
            bytes.extend_from_slice(&prefix);
            bytes.extend_from_slice(&new_suffix);
            return Some(FractionalIndex::from_vec_unterminated(bytes));
        } else if left.len() > right.len() {
            let (prefix, suffix) = left.split_at(shorter_len + 1);
            let new_suffix = new_after(&suffix);
            let mut bytes = Vec::with_capacity(new_suffix.len() + prefix.len() + 1);
            bytes.extend_from_slice(&prefix);
            bytes.extend_from_slice(&new_suffix);
            return Some(FractionalIndex::from_vec_unterminated(bytes));
        } else {
            // They are equal.
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_before_simple() {
        let mut i = FractionalIndex::default();
        assert_eq!(i.as_bytes(), &[128]);

        i = FractionalIndex::new_before(&i);
        assert_eq!(i.as_bytes(), &[127, 128]);

        let i = FractionalIndex::new_before(&i);
        assert_eq!(i.as_bytes(), &[126, 128]);
    }

    #[test]
    fn new_after_simple() {
        let mut i = FractionalIndex::default();
        assert_eq!(i.as_bytes(), &[128]);

        i = FractionalIndex::new_after(&i);
        assert_eq!(i.as_bytes(), &[129, 128]);

        let i = FractionalIndex::new_after(&i);
        assert_eq!(i.as_bytes(), &[130, 128]);
    }

    #[test]
    fn new_before_longer() {
        let mut i = FractionalIndex::from_vec_unterminated(vec![100, 100, 3]);
        assert_eq!(i.as_bytes(), &[100, 100, 3, 128]);

        i = FractionalIndex::new_before(&i);
        assert_eq!(i.as_bytes(), &[99, 128]);

        i = FractionalIndex::new_before(&i);
        assert_eq!(i.as_bytes(), &[98, 128]);
    }

    #[test]
    fn new_after_longer() {
        let mut i = FractionalIndex::from_vec_unterminated(vec![240, 240, 3]);
        assert_eq!(i.as_bytes(), &[240, 240, 3, 128]);

        i = FractionalIndex::new_after(&i);
        assert_eq!(i.as_bytes(), &[241, 128]);

        i = FractionalIndex::new_after(&i);
        assert_eq!(i.as_bytes(), &[242, 128]);
    }

    #[test]
    fn new_before_zeros() {
        let mut i = FractionalIndex::from_vec_unterminated(vec![0, 0]);
        assert_eq!(i.as_bytes(), &[0, 0, 128]);

        i = FractionalIndex::new_before(&i);
        assert_eq!(i.as_bytes(), &[0, 0, 127, 128]);

        i = FractionalIndex::new_before(&i);
        assert_eq!(i.as_bytes(), &[0, 0, 126, 128]);
    }

    #[test]
    fn new_after_max() {
        let mut i = FractionalIndex::from_vec_unterminated(vec![255, 255]);
        assert_eq!(i.as_bytes(), &[255, 255, 128]);

        i = FractionalIndex::new_after(&i);
        assert_eq!(i.as_bytes(), &[255, 255, 129, 128]);

        i = FractionalIndex::new_after(&i);
        assert_eq!(i.as_bytes(), &[255, 255, 130, 128]);
    }

    #[test]
    fn new_before_wrap() {
        let mut i = FractionalIndex::from_vec_unterminated(vec![0]);
        assert_eq!(i.as_bytes(), &[0, 128]);

        i = FractionalIndex::new_before(&i);
        assert_eq!(i.as_bytes(), &[0, 127, 128]);
    }

    #[test]
    fn new_after_wrap() {
        let mut i = FractionalIndex::from_vec_unterminated(vec![255]);
        assert_eq!(i.as_bytes(), &[255, 128]);

        i = FractionalIndex::new_after(&i);
        assert_eq!(i.as_bytes(), &[255, 129, 128]);
    }

    #[test]
    fn new_between_simple() {
        {
            let left = FractionalIndex::from_vec_unterminated(vec![100]);
            let right = FractionalIndex::from_vec_unterminated(vec![119]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[109, 128]);
        }

        {
            let left = FractionalIndex::from_vec_unterminated(vec![100, 100]);
            let right = FractionalIndex::from_vec_unterminated(vec![100, 104]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[100, 102, 128]);
        }

        {
            let left = FractionalIndex::from_vec_unterminated(vec![100, 100]);
            let right = FractionalIndex::from_vec_unterminated(vec![100, 103]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[100, 101, 128]);
        }

        {
            let left = FractionalIndex::from_vec_unterminated(vec![100, 100]);
            let right = FractionalIndex::from_vec_unterminated(vec![100, 102]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[100, 101, 128]);
        }

        {
            let left = FractionalIndex::from_vec_unterminated(vec![108]);
            let right = FractionalIndex::from_vec_unterminated(vec![109]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[108, 129, 128]);
        }

        {
            let left = FractionalIndex::from_vec_unterminated(vec![127, 128]);
            let right = FractionalIndex::from_vec_unterminated(vec![128]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[127, 129, 128]);
        }

        {
            let left = FractionalIndex::from_vec_unterminated(vec![127, 129]);
            let right = FractionalIndex::from_vec_unterminated(vec![]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[127, 130, 128]);
        }

        {
            let left = FractionalIndex::from_vec_unterminated(vec![127]);
            let right = FractionalIndex::from_vec_unterminated(vec![]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[127, 129, 128]);
        }
    }

    #[test]
    fn new_between_extend() {
        {
            let left = FractionalIndex::from_vec_unterminated(vec![100]);
            let right = FractionalIndex::from_vec_unterminated(vec![101]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[100, 129, 128]);
        }
    }

    #[test]
    fn new_between_prefix() {
        {
            let left = FractionalIndex::from_vec_unterminated(vec![100]);
            let right = FractionalIndex::from_vec_unterminated(vec![100, 144]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[100, 144, 127, 128]);
        }

        {
            let left = FractionalIndex::from_vec_unterminated(vec![100, 122]);
            let right = FractionalIndex::from_vec_unterminated(vec![100]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[100, 122, 129, 128]);
        }

        {
            let left = FractionalIndex::from_vec_unterminated(vec![100, 122]);
            let right = FractionalIndex::from_vec_unterminated(vec![100, 128]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[100, 125, 128]);
        }

        {
            let left = FractionalIndex::from_vec_unterminated(vec![]);
            let right = FractionalIndex::from_vec_unterminated(vec![128, 192]);
            let mid = FractionalIndex::new_between(&left, &right).unwrap();
            assert_eq!(mid.as_bytes(), &[128, 128]);
        }
    }

    #[test]
    fn test_fractional_index() {
        let mut indices: Vec<FractionalIndex> = Vec::new();

        let c = FractionalIndex::default();

        {
            let mut m = c.clone();
            let mut low = Vec::new();
            for _ in 0..20 {
                m = FractionalIndex::new_before(&m);
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
                m = FractionalIndex::new_after(&m);
                high.push(m.clone())
            }

            indices.append(&mut high)
        }

        for i in 0..(indices.len() - 1) {
            assert!(indices[i] < indices[i + 1])
        }

        for _ in 0..12 {
            let mut new_indices: Vec<FractionalIndex> = Vec::new();
            for i in 0..(indices.len() - 1) {
                let cb = FractionalIndex::new_between(&indices[i], &indices[i + 1]).unwrap();
                assert!(&indices[i] < &cb);
                assert!(&cb < &indices[i + 1]);

                let st = cb.to_hex();
                assert!(FractionalIndex::from_hex(&st).unwrap() == cb);
                println!("{:?} {:?}", cb, indices[i]);
                println!("{} {}", st, indices[i].to_hex());
                assert!(st < indices[i + 1].to_hex());

                new_indices.push(cb);
                new_indices.push(indices[i + 1].clone());
            }

            indices = new_indices;
        }
    }
}
