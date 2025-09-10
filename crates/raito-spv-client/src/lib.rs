//! Raito SPV Client Library
//!
//! This library provides functionality to fetch compressed SPV proofs
//! for Bitcoin transactions using the Raito bridge infrastructure.

pub mod fetch;
pub mod format;
pub mod proof;
pub mod verify;
pub mod work;

// Re-export only the main fetch function for library usage
pub use fetch::fetch_compressed_proof;
