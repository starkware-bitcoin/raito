//! Core SPV (Simplified Payment Verification) functionality for Raito
//!
//! This crate provides shared functionality for both the bridge node and client,
//! including Bitcoin RPC client, MMR (Merkle Mountain Range) accumulator, and
//! sparse roots representation.

pub mod bitcoin;
pub mod block_mmr;
pub mod sparse_roots;
