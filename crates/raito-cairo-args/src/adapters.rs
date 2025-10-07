use crate::error::CairoSerializeError;
use crate::serializer::to_runner_args_hex;
use serde::Serialize;

#[cfg(feature = "raito-spv-verify")]
pub mod assumevalid_args {
    use super::*;
    #[cfg(feature = "bitcoin")]
    use bitcoin::hashes::Hash;
    #[cfg(feature = "raito-spv-mmr")]
    use raito_spv_mmr::sparse_roots::SparseRoots;
    use raito_spv_verify::ChainState;

    /// View for assumevalid Args struct that matches Cairo's structure
    #[derive(Serialize)]
    struct AssumeValidArgsView<'a> {
        /// Current (initial) chain state
        chain_state: ChainState,
        /// Batch of blocks that have to be applied to the current chain state
        blocks: &'a [BlockView],
        /// Merkle Mountain Range of the block hashes
        block_mmr: &'a SparseRoots,
        /// Proof of the previous chain state transition (pre-serialized Cairo proof)
        chain_state_proof: Option<&'a [u8]>,
    }

    /// View for a single block matching Cairo's Block structure
    #[derive(Serialize)]
    struct BlockView {
        /// Block header
        header: HeaderView,
        /// Transaction data: merkle root (light client mode)
        data: TransactionDataView,
    }

    /// View for transaction data - only MerkleRoot variant for light client mode
    #[derive(Serialize)]
    struct TransactionDataView {
        /// Merkle root of all transactions in the block (light client mode)
        merkle_root: String, // 64-hex digest
    }

    /// Reuse HeaderView from header module
    #[derive(Serialize)]
    struct HeaderView {
        pub version: u32,
        pub time: u32,
        pub bits: u32,
        pub nonce: u32,
    }

    /// Main adapter function for assumevalid Args
    pub fn to_runner_args_hex(
        chain_state: ChainState,
        headers: &[bitcoin::block::Header],
        block_mmr: &SparseRoots,
        chain_state_proof: Option<&[u8]>,
    ) -> std::result::Result<Vec<String>, CairoSerializeError> {
        // Convert headers to BlockView (merkle_root is already in the header)
        let blocks: Vec<BlockView> = headers
            .iter()
            .map(|header| BlockView {
                header: HeaderView {
                    version: header.version.to_consensus() as u32,
                    time: header.time,
                    bits: header.bits.to_consensus(),
                    nonce: header.nonce,
                },
                data: TransactionDataView {
                    merkle_root: hex::encode(header.merkle_root.as_byte_array()),
                },
            })
            .collect();

        let args_view = AssumeValidArgsView {
            chain_state: chain_state,
            blocks: &blocks,
            block_mmr,
            chain_state_proof,
        };

        super::to_runner_args_hex(&args_view)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use bitcoin::block::Header as BlockHeader;
        use bitcoin::block::Version;
        use bitcoin::blockdata::block::BlockHash;
        use bitcoin::hashes::Hash;
        use bitcoin::CompactTarget;
        use bitcoin::Target;
        use bitcoin::TxMerkleNode;
        use bitcoin::Work;
        use num_bigint::BigUint;
        use serde_json;
        use std::str::FromStr;

        #[test]
        fn test_assumevalid_args_to_runner_args_hex_with_batch_100() {
            // Load test data from batch_100.json
            let test_data = include_str!("../../../packages/assumevalid/tests/data/batch_100.json");
            let json_data: serde_json::Value =
                serde_json::from_str(test_data).expect("Failed to parse JSON");

            // Parse chain state
            let chain_state = parse_chain_state(&json_data["chain_state"]);

            // Parse block headers
            let headers = parse_block_headers(&json_data["blocks"]);

            // Parse block MMR
            let block_mmr = parse_block_mmr(&json_data["block_mmr"]);

            // Parse chain state proof (optional)
            let chain_state_proof = if json_data["chain_state_proof"].is_null() {
                None
            } else {
                // For now, we'll handle None case since the test data has null
                None
            };

            // Call the function under test
            let result =
                to_runner_args_hex(chain_state.clone(), &headers, &block_mmr, chain_state_proof);

            // Verify the result
            match result {
                Ok(args_hex) => {
                    // Basic assertions
                    assert!(!args_hex.is_empty(), "Result should not be empty");
                    assert!(
                        args_hex.len() > 100,
                        "Result should have substantial data for 100 blocks"
                    );

                    // All elements should be valid hex strings
                    for arg in &args_hex {
                        assert!(arg.starts_with("0x"), "All arguments should start with 0x");
                        assert!(arg.len() > 2, "All arguments should have content after 0x");
                    }

                    // Test specific expectations based on the data structure
                    // The first few arguments should represent the chain state
                    assert_eq!(
                        args_hex[0], "0x0",
                        "First argument should be block_height (0)"
                    );
                    assert_eq!(
                        args_hex[1], "0x10001",
                        "Second argument should be total_work"
                    );

                    // Test that we have data for all 100 blocks
                    // Each block contributes multiple felts, so we expect a substantial number
                    assert!(
                        args_hex.len() > 1000,
                        "Should have substantial data for 100 blocks"
                    );

                    println!("Successfully generated {} hex arguments", args_hex.len());
                    println!(
                        "First few arguments: {:?}",
                        &args_hex[..std::cmp::min(10, args_hex.len())]
                    );

                    // Test that the function can be called multiple times with the same data
                    let result2 = to_runner_args_hex(
                        chain_state.clone(),
                        &headers,
                        &block_mmr,
                        chain_state_proof,
                    );
                    assert!(
                        result2.is_ok(),
                        "Function should be callable multiple times"
                    );
                    assert_eq!(args_hex, result2.unwrap(), "Results should be identical");
                }
                Err(e) => {
                    panic!("Failed to generate runner args: {:?}", e);
                }
            }
        }

        #[test]
        fn test_assumevalid_args_to_runner_args_hex_error_handling() {
            // Test with empty data to verify error handling
            let empty_headers: Vec<BlockHeader> = vec![];
            let empty_mmr = SparseRoots {
                block_height: 0,
                roots: vec![],
            };

            // Create a minimal chain state
            let chain_state = ChainState {
                block_height: 0,
                total_work: Work::from_be_bytes([0u8; 32]),
                best_block_hash: BlockHash::all_zeros(),
                current_target: Target::from_be_bytes([0u8; 32]),
                epoch_start_time: 0,
                prev_timestamps: vec![],
            };

            // This should still work with empty data
            let result = to_runner_args_hex(chain_state, &empty_headers, &empty_mmr, None);
            assert!(
                result.is_ok(),
                "Function should handle empty data gracefully"
            );

            let args_hex = result.unwrap();
            assert!(
                !args_hex.is_empty(),
                "Should still generate some arguments even with empty data"
            );
        }

        fn parse_chain_state(chain_state_json: &serde_json::Value) -> ChainState {
            ChainState {
                block_height: chain_state_json["block_height"].as_u64().unwrap() as u32,
                total_work: {
                    let work_bytes =
                        BigUint::from_str(chain_state_json["total_work"].as_str().unwrap())
                            .unwrap()
                            .to_bytes_be();
                    let mut work_array = [0u8; 32];
                    let start_idx = 32 - work_bytes.len();
                    work_array[start_idx..].copy_from_slice(&work_bytes);
                    Work::from_be_bytes(work_array)
                },
                best_block_hash: BlockHash::from_str(
                    chain_state_json["best_block_hash"].as_str().unwrap(),
                )
                .expect("Failed to parse best_block_hash"),
                current_target: {
                    let target_bytes =
                        BigUint::from_str(chain_state_json["current_target"].as_str().unwrap())
                            .unwrap()
                            .to_bytes_be();
                    let mut target_array = [0u8; 32];
                    let start_idx = 32 - target_bytes.len();
                    target_array[start_idx..].copy_from_slice(&target_bytes);
                    Target::from_be_bytes(target_array)
                },
                epoch_start_time: chain_state_json["epoch_start_time"].as_u64().unwrap() as u32,
                prev_timestamps: chain_state_json["prev_timestamps"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as u32)
                    .collect(),
            }
        }

        fn parse_block_headers(blocks_json: &serde_json::Value) -> Vec<BlockHeader> {
            blocks_json
                .as_array()
                .unwrap()
                .iter()
                .map(|block| {
                    let header_json = &block["header"];
                    let data_json = &block["data"];

                    // Create a minimal block header
                    // Note: We need to construct the full header with proper hash
                    let version =
                        Version::from_consensus(header_json["version"].as_u64().unwrap() as i32);
                    let time = header_json["time"].as_u64().unwrap() as u32;
                    let bits =
                        CompactTarget::from_consensus(header_json["bits"].as_u64().unwrap() as u32);
                    let nonce = header_json["nonce"].as_u64().unwrap() as u32;

                    // Parse merkle root
                    let merkle_root_str = data_json["merkle_root"].as_str().unwrap();
                    let merkle_root_bytes =
                        hex::decode(merkle_root_str).expect("Failed to decode merkle root");
                    let merkle_root =
                        TxMerkleNode::from_slice(&merkle_root_bytes).expect("Invalid merkle root");

                    // Create block header
                    BlockHeader {
                        version,
                        prev_blockhash: BlockHash::all_zeros(), // We'll use zeros for prev_blockhash in test
                        merkle_root,
                        time,
                        bits,
                        nonce,
                    }
                })
                .collect()
        }

        fn parse_block_mmr(block_mmr_json: &serde_json::Value) -> SparseRoots {
            let roots_array = block_mmr_json["roots"].as_array().unwrap();
            let mut roots = Vec::new();

            for root_obj in roots_array {
                let hi = root_obj["hi"]
                    .as_str()
                    .and_then(|s| BigUint::from_str(s).ok())
                    .or_else(|| root_obj["hi"].as_u64().map(BigUint::from))
                    .unwrap_or_default();
                let lo = root_obj["lo"]
                    .as_str()
                    .and_then(|s| BigUint::from_str(s).ok())
                    .or_else(|| root_obj["lo"].as_u64().map(BigUint::from))
                    .unwrap_or_default();

                // Convert to 64-character hex string
                let hi_hex = format!("{:032x}", hi);
                let lo_hex = format!("{:032x}", lo);
                let full_hex = format!("0x{}{}", hi_hex, lo_hex);

                roots.push(full_hex);
            }

            // Create SparseRoots with proper block height
            // The block height should be calculated from the number of blocks
            SparseRoots {
                block_height: 100, // Based on the test data having 100 blocks
                roots,
            }
        }
    }
}
