//! Raito SPV verification library
//!
//! This crate provides verification routines for compressed SPV proofs, including transaction,
//! block MMR, Cairo recursive proof, and subchain work checks.

pub mod proof;
pub mod verify;
pub mod work;

pub use proof::{
    BootloaderOutput, ChainState, CompressedSpvProof, TaskResult, TransactionInclusionProof,
};
pub use verify::{
    verify_block_header, verify_chain_state, verify_proof, verify_transaction, VerifierConfig,
};
pub use work::verify_subchain_work;
