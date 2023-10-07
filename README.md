# `fractional_index`

[![GitHub Repo stars](https://img.shields.io/github/stars/drifting-in-space/fractional_index?style=social)](https://github.com/drifting-in-space/fractional_index)
[![crates.io](https://img.shields.io/crates/v/fractional_index.svg)](https://crates.io/crates/fractional_index)
[![docs.rs](https://img.shields.io/badge/docs-release-brightgreen)](https://docs.rs/fractional_index/)
[![wokflow state](https://github.com/drifting-in-space/aper/workflows/build/badge.svg)](https://github.com/drifting-in-space/aper/actions/workflows/rust.yml)
[![dependency status](https://deps.rs/repo/github/drifting-in-space/fractional_index/status.svg)](https://deps.rs/repo/github/drifting-in-space/fractional_index)

This crate implements fractional indexing, a term coined by Figma in their blog post
[*Realtime Editing of Ordered Sequences*](https://www.figma.com/blog/realtime-editing-of-ordered-sequences/).

Specifically, this crate provides a type called `FractionalIndex`. A `FractionalIndex` acts as a
“black box” that is only useful for comparing to
another `FractionalIndex`. A `FractionalIndex` can only be constructed from a default constructor or by
reference to an existing `FractionalIndex`.

This is useful as a key in a `BTreeMap` when we want to be able to arbitrarily insert or
re-order elements in a collection, but don't actually care what the key is. It’s also useful for resloving conflicts when a list is modified concurrently by multiple users.

## Usage

The API of `FractionalIndex` is very simple:

- `FractionalIndex::default()` creates a new fractional index.
- `FractionalIndex::new_before(a)` creates a fractional index before another.
- `FractionalIndex::new_after(a)` creates a fractional index after another.
- `FractionalIndex::new_between(a, b)` creates a fractional index between two others.

```rust
use fractional_index::FractionalIndex;

fn main() {
  // Construct a fractional index.
  let index = FractionalIndex::default();

  // Construct another fractional index that comes before it.
  let before_index = FractionalIndex::new_before(&index);
  assert!(before_index < index);

  // Construct a third fractional index between the other two.
  let between_index = FractionalIndex::new_between(
    &before_index,
    &index
  ).unwrap();
  assert!(before_index < between_index);
  assert!(between_index < index);
}
```

### Stringification

`FractionalIndexes` are stored as byte strings, but they can be converted to and from strings.

```rust
use fractional_index::FractionalIndex;

fn main() {
  let a = FractionalIndex::default();
  let b = FractionalIndex::new_after(&a);
  let c = FractionalIndex::new_between(
    &a,
    &b
  ).unwrap();
  
  let c_str = c.to_string();
  assert_eq!("817f80", c_str);
  let c = FractionalIndex::new_between(&a, &b).unwrap();
}
```

The lexicographical order of two stringified `FractionalIndex` values is the same as the order of the unstringified version.

```rust
use fractional_index::FractionalIndex;

fn main() {
  let a = FractionalIndex::default();
  let b = FractionalIndex::new_after(&a);
  let c = FractionalIndex::new_between(&a, &b).unwrap();
  
  assert!(a.to_string() < c.to_string());
  assert!(c.to_string() < b.to_string());
}
```

This is mostly useful when constructing indexes that you need to be able to compare in a language with native string comparison but not bytestring comparison, like JavaScript.

### Serialization

With the `serde` feature (enabled by default), `FractionalIndexes` can be serialized.

```rust
use fractional_index::FractionalIndex;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct MyStruct {
  a: FractionalIndex,
  b: FractionalIndex,
  c: FractionalIndex,
}

fn main() {
  let a = FractionalIndex::default();
  let b = FractionalIndex::new_after(&a);
  let c = FractionalIndex::new_between(&a, &b).unwrap();

  let my_struct = MyStruct {
    a: a.clone(),
    b: b.clone(),
    c: c.clone(),
  };

  let json_string = serde_json::to_string(&my_struct).unwrap();
  let my_struct_de = serde_json::from_str(&json_string).unwrap();

  assert_eq!(my_struct, my_struct_de);
}
```

By default, `FractionalIndexes` are serialized as byte arrays, which can be serialized efficiently in formats like [bincode](https://github.com/bincode-org/bincode).

These byte arrays are less useful when using a format like JSON to send data to a non-Rust programming language.

For these use cases, we provide a stringifying serializer which can be enabled by annotating a field with `#[serde(with="fractional_index::stringify")]`.

```rust
use fractional_index::FractionalIndex;
use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct MyStruct {
  #[serde(with="fractional_index::stringify")]
  a: FractionalIndex,
  #[serde(with="fractional_index::stringify")]
  b: FractionalIndex,
  #[serde(with="fractional_index::stringify")]
  c: FractionalIndex,
}

fn main() {
  let a = FractionalIndex::default();
  let b = FractionalIndex::new_after(&a);
  let c = FractionalIndex::new_between(&a, &b).unwrap();

  let my_struct = MyStruct {
    a: a.clone(),
    b: b.clone(),
    c: c.clone(),
  };

  let json_value = serde_json::to_value(&my_struct).unwrap();

  let expected = json!({
    "a": "80",
    "b": "8180",
    "c": "817f80",
  });

  assert_eq!(expected, json_value);
}
```
