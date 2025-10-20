//! Raito Prove - Generate assumevalid arguments and prove Cairo programs
//!
//! This library provides functionality to:
//! 1. Generate assumevalid arguments from bridge node data
//! 2. Prove assumevalid arguments using Cairo programs and STARK proofs

pub mod adapters;
pub mod gcs;
pub mod generate_args;
pub mod prove;

pub use prove::{prove, ProveParams};
