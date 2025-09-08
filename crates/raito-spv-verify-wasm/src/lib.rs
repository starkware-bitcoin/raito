//! WASM bindings for raito SPV verification
//! This crate provides WebAssembly bindings for SPV proof verification

use wasm_bindgen::prelude::*;
use raito_spv_verify::{CompressedSpvProof, VerifierConfig, verify_proof};

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

/// Initialize panic hook for better error messages in WASM
#[wasm_bindgen]
pub fn init() -> () {
    console_error_panic_hook::set_once();
}
