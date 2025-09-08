//! Raito SPV verification library
//! 
//! This crate provides verification routines for compressed SPV proofs, including transaction, 
//! block MMR, Cairo recursive proof, and subchain work checks.

pub mod proof;
pub mod verify;
pub mod work;

pub use proof::{CompressedSpvProof, ChainState, BootloaderOutput, TaskResult};
pub use verify::{VerifierConfig, verify_proof};
pub use work::verify_subchain_work; 