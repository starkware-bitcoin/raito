//! WASM bindings for raito SPV verification
//! This crate provides WebAssembly bindings for SPV proof verification

use bitcoin::block::Header;
use raito_spv_mmr::block_mmr::BlockInclusionProof;
use raito_spv_verify::ChainState;
use raito_spv_verify::{
    verify::ChainStateProof, CompressedSpvProof, TransactionInclusionProof, VerifierConfig,
};
use wasm_bindgen::prelude::*;

/// Verify an SPV proof from JSON data
#[wasm_bindgen]
pub async fn verify_proof(proof_data: &str) -> Result<bool, JsValue> {
    // Parse proof from JSON
    let proof: CompressedSpvProof = serde_json::from_str(proof_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse proof: {}", e)))?;

    // Use default configuration
    let config = VerifierConfig::default();

    // Verify the proof
    raito_spv_verify::verify_proof(proof, &config, false)
        .await
        .map_err(|e| JsValue::from_str(&format!("Verification failed: {}", e)))?;

    Ok(true)
}

/// Verify an SPV proof with custom configuration
#[wasm_bindgen]
pub async fn verify_proof_with_config(
    proof_data: &str,
    config_data: &str,
    dev: bool,
) -> Result<bool, JsValue> {
    // Parse proof from JSON
    let proof: CompressedSpvProof = serde_json::from_str(proof_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse proof: {}", e)))?;

    // Parse config from JSON
    let config: VerifierConfig = serde_json::from_str(config_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse config: {}", e)))?;

    // Verify the proof
    raito_spv_verify::verify_proof(proof, &config, dev)
        .await
        .map_err(|e| JsValue::from_str(&format!("Verification failed: {}", e)))?;

    Ok(true)
}

/// Verify that a transaction is included in a block header using a Merkle proof
#[wasm_bindgen]
pub fn verify_transaction(
    transaction_proof_data: &str, // contains serialized TransactionInclusionProof
) -> Result<bool, JsValue> {
    // Parse transaction from JSON
    let TransactionInclusionProof {
        transaction,
        transaction_proof,
        block_header,
        block_height: _,
    } = serde_json::from_str(transaction_proof_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse transaction proof: {}", e)))?;

    // Verify the transaction
    raito_spv_verify::verify_transaction(&transaction, &block_header, transaction_proof)
        .map_err(|e| JsValue::from_str(&format!("Transaction verification failed: {}", e)))?;

    Ok(true)
}

/// Verify that a block header is included in the block MMR using an inclusion proof
#[wasm_bindgen]
pub async fn verify_block_header(
    block_header_data: &str,
    block_header_proof_data: &str,
) -> Result<String, JsValue> {
    // Parse block header from JSON
    let block_header: Header = serde_json::from_str(block_header_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse block header: {}", e)))?;

    // Parse block header proof from JSON
    let block_header_proof: BlockInclusionProof = serde_json::from_str(block_header_proof_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse block header proof: {}", e)))?;

    // Verify the block header and get the MMR root
    let mmr_root = raito_spv_verify::verify_block_header(&block_header, block_header_proof)
        .await
        .map_err(|e| JsValue::from_str(&format!("Block header verification failed: {}", e)))?;

    Ok(mmr_root)
}

/// Verify the Cairo recursive proof and consistency of the bootloader output with chain state
#[wasm_bindgen]
pub fn verify_chain_state(
    chain_state_proof_data: &str,
    config_data: &str,
) -> Result<String, JsValue> {
    // Parse chain state proof from JSON
    let chain_state_proof: ChainStateProof = serde_json::from_str(chain_state_proof_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse chain state proof: {}", e)))?;

    // Parse config from JSON
    let config: VerifierConfig = serde_json::from_str(config_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse config: {}", e)))?;

    // Verify the chain state and get the MMR hash
    let mmr_hash = raito_spv_verify::verify_chain_state(
        &chain_state_proof.chain_state,
        chain_state_proof.chain_state_proof,
        &config,
    )
    .map_err(|e| JsValue::from_str(&format!("Chain state verification failed: {}", e)))?;

    Ok(mmr_hash)
}

/// Create a default verifier configuration
#[wasm_bindgen]
pub fn create_default_config() -> JsValue {
    let config = VerifierConfig::default();
    serde_wasm_bindgen::to_value(&config).unwrap_or(JsValue::NULL)
}

/// Create a custom verifier configuration
#[wasm_bindgen]
pub fn create_custom_config(
    min_work: &str,
    bootloader_hash: &str,
    task_program_hash: &str,
    task_output_size: u32,
) -> JsValue {
    let config = VerifierConfig {
        min_work: min_work.to_string(),
        bootloader_hash: bootloader_hash.to_string(),
        task_program_hash: task_program_hash.to_string(),
        task_output_size,
    };
    serde_wasm_bindgen::to_value(&config).unwrap_or(JsValue::NULL)
}

/// Verify that there is enough work added on top of the target block
#[wasm_bindgen]
pub fn verify_subchain_work(
    block_height: u32,
    chain_state_data: &str,
    config_data: &str,
) -> Result<bool, JsValue> {
    // Parse chain state from JSON
    let chain_state: ChainState = serde_json::from_str(chain_state_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse chain state: {}", e)))?;

    // Parse config from JSON
    let config: VerifierConfig = serde_json::from_str(config_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse config: {}", e)))?;

    // Verify the subchain work
    raito_spv_verify::verify_subchain_work(block_height, &chain_state, &config)
        .map_err(|e| JsValue::from_str(&format!("Subchain work verification failed: {}", e)))?;

    Ok(true)
}

/// Initialize panic hook for better error messages in WASM
#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Get the version of the WASM module
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
