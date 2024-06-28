//! A Rust implementation of the [XXHash][] algorithm.
//!
//! [XXHash]: https://github.com/Cyan4973/xxHash
//!
//! ## Hashing arbitrary data
//!
//! ```rust
//! use xx_renu::XxHash64;
//!
//! let seed = 1234;
//! let hash = XxHash64::oneshot(seed, b"some bytes");
//! assert_eq!(0xeab5_5659_a496_d78b, hash);
//! ```
//!
//! ## In a [`HashMap`](std::collections::HashMap)
//!
//! ### With a fixed seed
//!
//! ```rust
//! use std::{collections::HashMap, hash::BuildHasherDefault};
//! use xx_renu::XxHash64;
//!
//! let mut hash: HashMap<_, _, BuildHasherDefault<XxHash64>> = Default::default();
//! hash.insert(42, "the answer");
//! assert_eq!(hash.get(&42), Some(&"the answer"));
//! ```
//!
//! ### With a random seed
//!
//! ```rust
//! use std::collections::HashMap;
//! use xx_renu::RandomXxHash64Builder;
//!
//! let mut hash: HashMap<_, _, RandomXxHash64Builder> = Default::default();
//! hash.insert(42, "the answer");
//! assert_eq!(hash.get(&42), Some(&"the answer"));
//! ```

#![no_std]
#![deny(rust_2018_idioms)]
#![deny(missing_docs)]

#[cfg(any(doc, test))]
extern crate std;

#[cfg(feature = "xxhash32")]
mod xxhash32;

#[cfg(feature = "xxhash32")]
pub use xxhash32::*;

#[cfg(feature = "xxhash64")]
mod xxhash64;

#[cfg(feature = "xxhash64")]
pub use xxhash64::*;

trait IntoU32 {
    fn into_u32(self) -> u32;
}

impl IntoU32 for u8 {
    fn into_u32(self) -> u32 {
        self.into()
    }
}

trait IntoU64 {
    fn into_u64(self) -> u64;
}

impl IntoU64 for u8 {
    fn into_u64(self) -> u64 {
        self.into()
    }
}

impl IntoU64 for u32 {
    fn into_u64(self) -> u64 {
        self.into()
    }
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl IntoU64 for usize {
    fn into_u64(self) -> u64 {
        self as u64
    }
}
