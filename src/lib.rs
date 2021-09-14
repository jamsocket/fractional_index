use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// The largest value less than the magic byte.
const MAGIC_FLOOR: u8 = 0b0111_1111; // =127

/// The smallest value greater than the magic byte.
const MAGIC_CEIL: u8 = 0b1000_0000; // =128

/// The byte we append to a byte string in order to generate a new byte
/// string that compares as lower. Any value less than or equal to
/// MAGIC_FLOOR is valid, but picking from the middle of the range
/// is optimal under the assumption of random inserts.
const MID_LOW: u8 = 0b0100_0000; // =64

/// The byte we append to a byte string in order to generate a new byte
/// string that compares as greater.
const MID_HIGH: u8 = 0b1100_0000; // =192

/// A [FractionByte] is the logical representation of a digit
/// of a [ZenoIndex]. A [ZenoIndex] represents a finite number
/// of [FractionByte::Byte] digits followed by an infinite number
/// of [FractionByte::Magic] digits. Since we only need to store
/// the “regular” bytes, the underlying representation stores just the
/// raw `u8` values of the regular bytes. Conversion to [FractionByte]
/// instances happens when individual digits of a [ZenoIndex] are
/// accessed by calling `digit`.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum FractionByte {
    /// A special “byte” which compares as if it were equal to 127.5.
    /// I.e., Byte(x) < Magic if x <= 127, otherwise Byte(x) > Magic.
    /// Byte(x) is never equal to Magic, but Magic == Magic.
    ///
    /// Th value 127.5 comes from the fact that the infinite sum
    /// of 127.5 * (1/256)^i over i=1..infinity equals 0.5, which is
    /// our desired default value. So a sequence of zero “regular”
    /// bytes followed by infinite “magic” bytes represents the
    /// fraction 0.5.
    Magic,

    /// A not-very-special byte.
    Byte(u8),
}

impl Default for FractionByte {
    fn default() -> Self {
        FractionByte::Magic
    }
}

impl PartialOrd for FractionByte {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FractionByte {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (FractionByte::Magic, FractionByte::Magic) => Ordering::Equal,
            (FractionByte::Byte(lhs), FractionByte::Magic) => {
                if *lhs <= MAGIC_FLOOR {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            (FractionByte::Magic, FractionByte::Byte(rhs)) => {
                if *rhs <= MAGIC_FLOOR {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (FractionByte::Byte(lhs), FractionByte::Byte(rhs)) => lhs.cmp(rhs),
        }
    }
}

/// A [ZenoIndex] is a binary representation of a fraction between 0 and 1,
/// *exclusive*, with arbitrary precision. The only operations it supports are:
///
/// - Construction of a [ZenoIndex] representing one half.
/// - Comparison of two [ZenoIndex] values.
/// - Returning an arbitrary [ZenoIndex] less or greater than another
///   given [ZenoIndex].
/// - Returning an arbitrary [ZenoIndex] strictly between two other [ZenoIndex]es.
///
/// Note that as a result of these restrictions:
/// - It's possible to arrive at a value infinitely close, but not equal to,
///   zero or one ([hence the name](https://plato.stanford.edu/entries/paradox-zeno/)).
/// - We only ever care about the  _relative_ value of two [ZenoIndex]es; not
///   their actual value. In fact, the only reason to think about them as fractions
///   at all is because it makes them easier to reason about.
///
/// The use of fractional indexes for real-time editing of lists is described in
/// [this post](https://www.figma.com/blog/realtime-editing-of-ordered-sequences/).
/// The specifics of the encoding used in that post differ from the one we use.
///
/// The underlying data structure used by a ZenoIndex is a vector of bytes. The
/// fraction represented by a given vector of N bytes, where z<sub>i</sub> is the
/// i<sup>th</sup> byte (1-based indexing):
///
/// (128/256)^N + sum<sub>i=1..N</sub> (z_i / 256^i)
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct ZenoIndex(Vec<u8>);

fn new_before(bytes: &[u8]) -> Vec<u8> {
    for i in 0..bytes.len() {
        if bytes[i] > MAGIC_FLOOR {
            let bytes: Vec<u8> = bytes[0..i].into();
            return bytes;
        }
        if bytes[i] > u8::MIN {
            let mut bytes: Vec<u8> = bytes[0..=i].into();
            bytes[i] -= 1;
            return bytes;
        }
    }

    // We have a sequence like [0, 0, 0], so we can't
    // decrement an existing digit; instead we append a
    // digit that falls below the magic byte.
    let mut bytes = bytes.to_vec();
    bytes.push(MID_LOW);
    bytes
}

fn new_after(bytes: &[u8]) -> Vec<u8> {
    for i in 0..bytes.len() {
        if bytes[i] < MAGIC_CEIL {
            let bytes: Vec<u8> = bytes[0..i].into();
            return bytes;
        }
        if bytes[i] < u8::MAX {
            let mut bytes: Vec<u8> = bytes[0..=i].into();
            bytes[i] += 1;
            return bytes;
        }
    }

    // We have a sequence like [255, 255, 255], so we can't
    // decrement an existing digit; instead we append a
    // digit that falls below the magic byte.
    let mut bytes = bytes.to_vec();
    bytes.push(MID_HIGH);
    bytes
}

fn new_between(left: &[u8], right: &[u8]) -> Option<Vec<u8>> {
    // The shortest of the two representations.
    let shortest_length = left.len().min(right.len());

    for i in 0..shortest_length {
        // Check whether the two differ at the byte at index i.

        match left[i].cmp(&right[i]) {
            Ordering::Less => {
                if (left[i]..right[i]).contains(&MAGIC_FLOOR) {
                    // They straddle the magic number, so we can just use the
                    // common prefix.
                    let prefix = left[0..i].to_vec();
                    return Some(prefix);
                } else if left[i] < right[i] - 1 {
                    // They differ by more than 1, so there is a byte between them.
                    let mid_value = ((right[i] - left[i]) / 2) + left[i];
                    let mut bytes: Vec<u8> = left[0..i].to_vec();
                    bytes.push(mid_value);
                    return Some(bytes);
                }

                // They differ by exactly 1; pick the shorter of the two and
                // find a value before or after the portion that comes after the
                // prefix.
                if left.len() <= right.len() {
                    let (prefix, suffix) = left.split_at(i + 1);
                    let mut bytes = prefix.to_vec();
                    bytes.extend_from_slice(&new_after(suffix));
                    return Some(bytes);
                }

                let (prefix, suffix) = right.split_at(i + 1);
                let mut bytes = prefix.to_vec();
                bytes.extend_from_slice(&new_before(suffix));
                return Some(bytes);
            }
            Ordering::Greater => {
                // If left > right, we don't attempt to find a value between them.
                return None;
            }
            Ordering::Equal => (),
        }
    }

    // If we reach this point, one must be a prefix of the other.
    match left.len().cmp(&right.len()) {
        Ordering::Less => {
            match right[shortest_length].cmp(&MAGIC_CEIL) {
                Ordering::Greater => {
                    let mut bytes = right[0..=shortest_length].to_vec();
                    bytes[shortest_length] -= 1;
                    Some(bytes)
                }
                Ordering::Equal => {
                    let (prefix, suffix) = right.split_at(shortest_length + 1);
                    let mut bytes = prefix.to_vec();
                    bytes.extend_from_slice(&new_before(suffix));
                    Some(bytes)
                }
                Ordering::Less => {
                    None
                }
            }
        }
        Ordering::Greater => {
            match left[shortest_length].cmp(&MAGIC_FLOOR) {
                Ordering::Less => {
                    let mut bytes = left[0..=shortest_length].to_vec();
                    bytes[shortest_length] += 1;
                    Some(bytes)
                }
                Ordering::Equal => {
                    let (prefix, suffix) = left.split_at(shortest_length + 1);
                    let mut bytes = prefix.to_vec();
                    bytes.extend_from_slice(&new_after(suffix));
                    Some(bytes)
                }
                Ordering::Greater => {
                    None
                }
            }
        }
        Ordering::Equal => None,
    }
}

impl ZenoIndex {
    fn digit(&self, i: usize) -> FractionByte {
        self.0
            .get(i)
            .copied()
            .map(FractionByte::Byte)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn new_before(fs: &ZenoIndex) -> ZenoIndex {
        ZenoIndex(new_before(&fs.0))
    }

    #[must_use]
    pub fn new_after(fs: &ZenoIndex) -> ZenoIndex {
        ZenoIndex(new_after(&fs.0))
    }

    #[must_use]
    pub fn new_between(left: &ZenoIndex, right: &ZenoIndex) -> Option<ZenoIndex> {
        new_between(&left.0, &right.0).map(ZenoIndex)
    }
}

impl PartialOrd for ZenoIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ZenoIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in 0..=self.0.len() {
            let sd = self.digit(i);
            let od = other.digit(i);
            #[allow(clippy::comparison_chain)]
            if sd < od {
                return Ordering::Less;
            } else if sd > od {
                return Ordering::Greater;
            }
        }
        Ordering::Equal
    }
}

impl Default for ZenoIndex {
    fn default() -> Self {
        ZenoIndex(Vec::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{*, FractionByte::{Magic, Byte}};

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
            assert!(indices[i] < indices[i + 1])
        }

        for _ in 0..12 {
            let mut new_indices: Vec<ZenoIndex> = Vec::new();
            for i in 0..(indices.len() - 1) {
                let cb = ZenoIndex::new_between(&indices[i], &indices[i + 1]).unwrap();
                assert!(&indices[i] < &cb);
                assert!(&cb < &indices[i + 1]);
                new_indices.push(cb);
                new_indices.push(indices[i + 1].clone());
            }

            indices = new_indices;
        }
    }

    #[test]
    fn test_fraction_byte_comparisons() {
        assert!(Byte(0) < Magic);
        assert!(Byte(255) > Magic);
        assert!(Byte(127) < Magic);
        assert!(Byte(128) > Magic);
        assert_eq!(Magic, Magic);
        assert_eq!(Byte(128), Byte(128));
        assert!(Byte(8) < Byte(9));
    }
}
