# `fractional_index`

This crate implements fractional indexing, an idea coined by Figma in their blog post
[*Realtime Editing of Ordered Sequences*](https://www.figma.com/blog/realtime-editing-of-ordered-sequences/).

Specifically, this crate provides an arbitrary-precision representation of numbers between 
zero and one (exclusive). For an ordered sequence data structure built atop this 
implementation see the [List](https://aper.dev/doc/aper/data_structures/struct.List.html)
implementation of [Aper](https://aper.dev/).

## Introduction

When implementing data structures that represent ordered sequences, it becomes inadequate
to identify elements by their array index because the array index may differ between
systems due to local changes.

One solution to this is to assign a unique floating-point index to each element, such that
iterating over each element in ascending order of the index produces the desired order. When an element is added to the sequence, the two indexes adjacent to the desired location
are averaged to generate the new element's index.

A naive implementation of this approach using standard 32- or 64-bit floats suffers from
numerical precision issues: it's not always possible to find a float between two other
floats, even if they are different. (If you squint, this is also like the [line-numbering problem](https://en.wikipedia.org/wiki/Line_number#Line_numbers_and_style) that plagued BASIC developers.)

Figma's post sketches the approach they use, which is based on a string representation
of fractional numbers, although some of the implementation details are left up to the reader. This crate attempts to formalize the math behind the approach and provide a clean interface that abstracts the implementation details away.

## Zeno Index

To differentiate between the *concept* of fractional indexing and the mathematical implementation used here, I have called the mathematical implementation I used a **Zeno index** after the philosopher Zeno of Elea, whose [dichotomy paradox](https://plato.stanford.edu/entries/paradox-zeno/#ParMot) has fun parallels to the implementation.

This crate exposes the `ZenoIndex` struct. The things you can do with a `ZenoIndex` are, by design,
very limited:

- Construct a default `ZenoIndex` (`Default` implementation).
- Given any `ZenoIndex`, construct another `ZenoIndex` before or after it.
- Given any two `ZenoIndex`es, construct a `ZenoIndex` between them.
- Compare two `ZenoIndex`es for order and equality.
- Serialize and deserialize using serde.

Notably, `ZenoIndex`es are opaque: even though they represent a number, they don't
provide an interface for accessing it directly. Additionally, they
don't provide guarantees about representation beyond what is exposed by the interface
(for example, there are infinite valid possible `ZenoIndex` values between any other
two non-equal `ZenoIndex`es), which gives the implementation room to optimize for space.

## Implementation

One of the goals of this crate is to provide an opaque interface that you can use
without needing to study the implementation, but if you're interested in making changes
to the implementation or just curious, what follows is a description of the implementation.

### Representation

Each `ZenoIndex` is backed by a (private) `Vec<u8>`, i.e. a sequence of bytes. 
Mathematically, the numeric value represented by this sequence of bytes is:

![](expr.png)

Where *n* is the number of bytes and *v<sub>i</sub>* is the value of the *i<sup>th</sup>* byte (zero-indexed).

The right term alone would be sufficient as an arbitrary-precision fraction representation, but the left term serves several purposes:

- It makes it impossible to represent zero. This is necessary because we always need to be able to represent a value *between* a `ZenoIndex` and zero.
- It ensures that no two differing sequences of bytes represent the same number (without it, trailing zeros could be added without changing the represented numeric value).
- It removes the “floor bias” that would bias the representation towards zero. In particular, it means that an empty sequence of bytes represents ½ instead of 0.

### Comparisons

To compare two numbers using this representation, we iterate through them byte-wise. If they differ at a given index, we can simply compare those values to determine the order.

Things get more complicated when one string of bytes is a prefix of the other. Without the first term, our representation would be [lexicographic](https://en.wikipedia.org/wiki/Lexicographic_order) and we could say the shorter one comes before the longer one. Due to the first term, though, the longer one could either be before or after. So we check the following byte to see if it is at most 127 (in which case the longer string comes *before* its prefix) or not (in which case the longer string comes *after*).

To simpilify the code, we take advantage of some properties of infinite series. The representation above is equivalent to

![](expr2.png)

which we can rewrite as

![](expr3.png)

by defining *v'<sub>i<sub>* as the *i<sup>th</sup>* byte when *i* < *n* or 127.5 if *i* ≥ *n*.

Since it's impossible to represent a fractional byte value, we convert bytes into an `enum` representation where they can take either a standard `u8` byte value, or the “magic” value of 127.5 (the word *magic* here refers to the [wizard sense](https://en.wikipedia.org/wiki/Places_in_Harry_Potter#Platform_Nine_and_Three-Quarters) rather than the [computational one](https://en.wikipedia.org/wiki/Magic_number_(programming))). The comparison operator is implemented on this enum such that the magic value compares as greater than 127 but less than 128.

### Construction

There are three ways to construct a `ZenoIndex`:

- From nothing: `ZenoIndex` implements `Default`. Under the hood, this constructs a Zeno Index backed by an empty byte string -- i.e., equivalent to 0.5.
- In relation to one other `ZenoIndex` (either before or after). We hop to the end of the reference `ZenoIndex`'s byte string, and see if we can increment or decrement the final byte to get the order desired. If we can't, we add another byte to the end to get a new `ZenoIndex` with the desired order.
- In between two other `ZenoIndex`es. We find the first byte index at which they differ. If the values of the index at which they differ fall on different sides of 127.5, we use the prefix that both share as the representation of the newly constructed `ZenoIndex`. Otherwise, we look for a byte value between the two, and extend the representation by a byte if that isn't possible.

These are not the only way to satisfy the public interface of `ZenoIndex`, and represent certain design considerations. In particular, the decision to increment or decrement the last byte by 1 instead of averaging with 0 or 255 comes from the fact that we expect a new item to often come directly after the last new item. In the limit case of a data structure that is only ever appended to, this allows us to grow the size of the underlying representation by a byte only once every 64 new items, instead of every 8 if we averaged.

Likewise, the decision to check whether two differing bytes straddle 127.5 is not strictly necessary, but allows us to find opportunities to use a smaller underlying representation in cases where items have been removed from a data structure. If we instead were to just average the two values, it would be likely that a collection of elements would grow in representation size over successive additions and deletions, even if the number of elements stayed constant.
