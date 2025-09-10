//! WASM bindings for raito SPV verification
//! This crate provides WebAssembly bindings for SPV proof verification

use raito_spv_verify::{verify_proof, CompressedSpvProof, VerifierConfig};
use wasm_bindgen::prelude::*;

/// Verify an SPV proof from JSON data
#[wasm_bindgen]
pub async fn verify_proof_wasm(proof_data: &str) -> Result<bool, JsValue> {
    // Parse proof from JSON
    let proof: CompressedSpvProof = serde_json::from_str(proof_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse proof: {}", e)))?;

    // Use default configuration
    let config = VerifierConfig::default();

    // Verify the proof
    verify_proof(proof, &config, false)
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
    verify_proof(proof, &config, dev)
        .await
        .map_err(|e| JsValue::from_str(&format!("Verification failed: {}", e)))?;

    Ok(true)
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
