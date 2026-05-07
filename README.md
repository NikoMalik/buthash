# buthash

Fast, generic hash function for structs, slices, and integers in Rust.

## Features

- **Generic** -> works with any type implementing `bytemuck::NoUninit` (integers, packed structs, fixed-size arrays, etc.)
- **Seeded** -> same input with different seeds produces different hashes
- **Adaptive** -> optimized code paths for small (<=16 bytes), medium (<=64 bytes), and large inputs
- **Minimal dependencies** -> only [`bytemuck`](https://crates.io/crates/bytemuck)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
buthash = "0.1.0" 
```

## Usage

### Hashing primitives and structs

Any `Copy` + `'static` type with no padding/uninit bytes can be hashed directly:

```rust
use buthash::hash::hash_inline;

let h = hash_inline(&42u64, 0);
```

For custom structs, derive `bytemuck::NoUninit` and use `#[repr(C)]`:

```rust
use buthash::hash::hash_inline;
use bytemuck::NoUninit;

#[derive(Clone, Copy, NoUninit)]
#[repr(C)]
struct Key {
    id: u64,
    tag: u32,
    flags: u32,
}

let key = Key { id: 1, tag: 2, flags: 0 };
let h = hash_inline(&key, 0);       // seed = 0
let h2 = hash_inline(&key, 123);    // different seed -> different hash
```

### Hashing byte slices

```rust
use buthash::hash::{hash_inline, low_level_hash};

// via the Hashable trait
let h = hash_inline(b"hello world".as_slice(), 0);

// or call the low-level function directly
let h = low_level_hash(0, b"hello world");
```

### Hashing a single u64

An optimized fast path for `u64` values:

```rust
use buthash::hash::low_level_hash_u64;

let h = low_level_hash_u64(0, 42);
```

## API

### Trait `Hashable`

```rust
pub trait Hashable {
    fn hash_with_seed(&self, seed: u64) -> u64;
}
```

Blanket implementations:

- `impl<T: NoUninit> Hashable for T` -> any type safely castable to bytes
- `impl Hashable for [u8]` -> byte slices

### Functions

| Function | Description |
|---|---|
| `hash_inline(value, seed) -> u64` | Hash any `Hashable` type with the given seed |
| `low_level_hash(seed, input) -> u64` | Hash a byte slice directly |
| `low_level_hash_u64(seed, input) -> u64` | Optimized path for a single `u64` |

## Algorithm

The hash uses 128-bit widening multiplication with XOR folding as its core mixing step, combined with a set of fixed salt constants for domain separation. The input length is mixed into the final result.

Processing strategy by input size:

- **>64 bytes** -> dual-lane parallel processing in 64-byte blocks, then merge
- **17..=64 bytes** -> sequential 16-byte chunk processing
- **9..=16 bytes** -> overlapping u64 reads
- **4..=8 bytes** -> overlapping u32 reads
- **1..=3 bytes** -> packed into a single u64 via shifts

## License

Apache-2.0
