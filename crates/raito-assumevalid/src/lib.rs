//! Raito Prove - Generate assumevalid arguments and prove Cairo programs
//!
//! This library provides functionality to:
//! 1. Generate assumevalid arguments from bridge node data
//! 2. Prove assumevalid arguments using Cairo programs and STARK proofs

pub mod generate_args;
pub mod prove;

// Re-export main public API for convenience
pub use generate_args::{
    AssumeValidParams, GenerateArgsResult, ProveClient, ProveConfig,
    generate_assumevalid_args, generate_and_save_args, save_cairo_args_to_file,
};
pub use prove::{
    ProveBatchParams, ProveBatchResult, StepMetrics, ProveParams,
    prove_batch, prove, auto_detect_start_height, find_proof_file,
};