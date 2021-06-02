//! Semantic Owl API
//!
//! This is a Rust implementation of owlapi which was originally written in Java.
//! However, Semantic Owl Api is not a direct one-to-one implementation of owlapi.
//! While ideas are borrowed, the implementation is not.

mod declarations;
mod loader;

pub use crate::declarations::*;
pub use crate::loader::*;
