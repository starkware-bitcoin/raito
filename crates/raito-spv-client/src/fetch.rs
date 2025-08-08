use std::path::PathBuf;

use bitcoin::{block::Header as BlockHeader, consensus, MerkleBlock, Transaction, Txid};
use cairo_air::CairoProof;
use raito_spv_core::{bitcoin::BitcoinClient, block_mmr::BlockInclusionProof};
use serde::{Deserialize, Serialize};
use stwo_prover::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use tracing::info;

use crate::proof::{ChainState, CompressedSpvProof};

/// CLI arguments for the `fetch` subcommand
#[derive(Clone, Debug, clap::Args)]
pub struct FetchArgs {
    /// Transaction ID
    #[arg(long)]
    txid: Txid,
    /// Path to save the proof
    #[arg(long)]
    proof_path: PathBuf,
    /// Raito node RPC URL
    #[arg(
        long,
        env = "RAITO_BRIDGE_RPC",
        default_value = "https://api.raito.wtf"
    )]
    raito_rpc_url: String,
    /// Bitcoin RPC URL
    #[arg(long, env = "BITCOIN_RPC")]
    bitcoin_rpc_url: String,
    /// Bitcoin RPC user:password (optional)
    #[arg(long, env = "USERPWD")]
    bitcoin_rpc_userpwd: Option<String>,
}

/// Chain state and its recursive proof produced by the Raito node
#[derive(Serialize, Deserialize)]
pub struct ChainStateProof {
    /// Canonical chain state snapshot
    #[serde(rename = "chainstate")]
    pub chain_state: ChainState,
    /// Recursive STARK proof attesting `chain_state` and block MMR root validity
    #[serde(rename = "proof")]
    pub chain_state_proof: CairoProof<Blake2sMerkleHasher>,
}

/// Bitcoin transaction inclusion data in a specific block
#[derive(Serialize, Deserialize)]
pub struct TransactionInclusionProof {
    /// The full Bitcoin transaction being proven
    pub transaction: Transaction,
    /// Encoded PartialMerkleTree containing the Merkle path for the transaction
    pub transaction_proof: Vec<u8>,
    /// Header of the block that includes the transaction
    pub block_header: BlockHeader,
    /// Height of the block that includes the transaction
    pub block_height: u32,
}

/// Run the `fetch` subcommand: build a compressed proof and write it to disk
///
/// Returns an error if any network request fails or the proof cannot be written
/// to the specified path.
pub async fn run(args: FetchArgs) -> Result<(), anyhow::Error> {
    // Construct compressed proof from different components
    let compressed_proof = fetch_compressed_proof(
        args.txid,
        args.bitcoin_rpc_url,
        args.bitcoin_rpc_userpwd,
        args.raito_rpc_url,
    )
    .await?;

    // Write proof to the file
    let proof_path = args.proof_path;
    let proof_dir = proof_path.parent().unwrap();
    std::fs::create_dir_all(proof_dir)?;

    let file = std::fs::File::create(&proof_path)?;
    let mut writer = std::io::BufWriter::new(file);
    serde_brief::to_writer(&compressed_proof, &mut writer)?;
    info!("Proof written to {}", proof_path.display());

    Ok(())
}

/// Fetch all components required to construct a `CompressedSpvProof`
///
/// - `txid`: Transaction id to prove
/// - `bitcoin_rpc_url`: URL of the Bitcoin node RPC
/// - `bitcoin_rpc_userpwd`: Optional `user:password` for basic auth
/// - `raito_rpc_url`: URL of the Raito bridge RPC
pub async fn fetch_compressed_proof(
    txid: Txid,
    bitcoin_rpc_url: String,
    bitcoin_rpc_userpwd: Option<String>,
    raito_rpc_url: String,
) -> Result<CompressedSpvProof, anyhow::Error> {
    let ChainStateProof {
        chain_state,
        chain_state_proof,
    } = fetch_chain_state_proof(raito_rpc_url.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch chain state proof: {:?}", e))?;

    let TransactionInclusionProof {
        transaction,
        transaction_proof,
        block_header,
        block_height,
    } = fetch_transaction_proof(txid, bitcoin_rpc_url, bitcoin_rpc_userpwd)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch transaction proof: {:?}", e))?;

    let block_header_proof =
        fetch_block_proof(block_height, chain_state.block_height as u32, raito_rpc_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch block proof: {:?}", e))?;

    Ok(CompressedSpvProof {
        chain_state,
        chain_state_proof,
        block_header,
        block_header_proof,
        transaction,
        transaction_proof,
    })
}

/// Fetch the latest chain state proof from the Raito bridge RPC
///
/// - `raito_rpc_url`: URL of the Raito bridge RPC endpoint
pub async fn fetch_chain_state_proof(
    raito_rpc_url: String,
) -> Result<ChainStateProof, anyhow::Error> {
    info!("Fetching chain state proof");
    let url = format!("{}/chainstate-proof/recent_proof", raito_rpc_url);
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Accept-Encoding", "gzip")
        .send()
        .await?;
    let proof: ChainStateProof = response.json().await?;
    Ok(proof)
}

/// Fetch the transaction inclusion data from a Bitcoin RPC
///
/// - `txid`: Transaction id to fetch
/// - `bitcoin_rpc_url`: URL of the Bitcoin node RPC
/// - `bitcoin_rpc_userpwd`: Optional `user:password` for basic auth
pub async fn fetch_transaction_proof(
    txid: Txid,
    bitcoin_rpc_url: String,
    bitcoin_rpc_userpwd: Option<String>,
) -> Result<TransactionInclusionProof, anyhow::Error> {
    info!("Fetching transaction proof for {}", txid);
    let bitcoin_client = BitcoinClient::new(bitcoin_rpc_url, bitcoin_rpc_userpwd)?;
    let MerkleBlock { header, txn } = bitcoin_client
        .get_transaction_inclusion_proof(&txid)
        .await?;

    let block_hash = header.block_hash();
    let transaction = bitcoin_client.get_transaction(&txid, &block_hash).await?;

    let block_header_ex = bitcoin_client.get_block_header_ex(&block_hash).await?;
    let block_height = block_header_ex.height;

    Ok(TransactionInclusionProof {
        transaction,
        transaction_proof: consensus::encode::serialize(&txn),
        block_header: header,
        block_height: block_height as u32,
    })
}

/// Fetch the block MMR inclusion proof from the Raito bridge RPC
///
/// - `block_height`: Height of the block to prove
/// - `block_count`: Current best height (chain head)
/// - `raito_rpc_url`: URL of the Raito bridge RPC endpoint
pub async fn fetch_block_proof(
    block_height: u32,
    block_count: u32,
    raito_rpc_url: String,
) -> Result<BlockInclusionProof, anyhow::Error> {
    info!("Fetching block proof for block height {}", block_height);
    if block_height > block_count {
        return Err(anyhow::anyhow!(
            "Block height {} cannot be greater than block count {}",
            block_height,
            block_count
        ));
    }
    let url = format!(
        "{}/block-inclusion-proof/{}?block_count={}",
        raito_rpc_url, block_height, block_count
    );
    let response = reqwest::get(url).await?;
    let proof: BlockInclusionProof = response.json().await?;
    Ok(proof)
}
