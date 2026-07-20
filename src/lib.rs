//! Fast and ergonomic byte-oriented input/output utilities for Rust.

pub mod output;
pub mod scanner;

pub use output::{OutputBuffer, Writable};
pub use scanner::Scanner;
