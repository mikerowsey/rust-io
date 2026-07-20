//! Fast and ergonomic input/output utilities for Rust.

pub mod output;
pub mod scanner;

pub use output::{Output, Writable};
pub use scanner::Scanner;
