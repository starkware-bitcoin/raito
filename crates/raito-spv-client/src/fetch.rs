use std::path::PathBuf;

use bitcoin::{block::Header as BlockHeader, consensus, MerkleBlock, Transaction, Txid};
use cairo_air::CairoProof;
use raito_spv_core::{bitcoin::BitcoinClient, block_mmr::BlockInclusionProof};
use serde::{Deserialize, Serialize};
use stwo_prover::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use tracing::info;

use crate::proof::{ChainState, CompressedSpvProof};

#[derive(Clone, Debug, clap::Args)]
pub struct FetchArgs {
    /// Transaction ID
    #[arg(long)]
    txid: Txid,
    /// Path to save the proof
    #[arg(long)]
    proof_path: PathBuf,
    /// Bridge node RPC URL
    #[arg(long, env = "RAITO_BRIGE_RPC", default_value = "https://api.raito.wtf")]
    bridge_rpc_url: String,
    /// Proof URL
    #[arg(
        long,
        env = "RAITO_PROOF_URL",
        default_value = "https://storage.googleapis.com/raito-proofs/proof_908383_00000000000000000000809d904a3a4cdb060ec54df9432a807ffad292374eb9.json"
    )]
    proof_url: String,
    /// Bitcoin RPC URL
    #[arg(long, env = "BITCOIN_RPC")]
    bitcoin_rpc_url: String,
    /// Bitcoin RPC user:password (optional)
    #[arg(long, env = "USERPWD")]
    bitcoin_rpc_userpwd: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ChainStateProof {
    #[serde(rename = "chainstate")]
    pub chain_state: ChainState,
    #[serde(rename = "proof")]
    pub chain_state_proof: CairoProof<Blake2sMerkleHasher>,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionInclusionProof {
    pub transaction: Transaction,
    pub transaction_proof: Vec<u8>,
    pub block_header: BlockHeader,
    pub block_height: u32,
}

pub async fn run(args: FetchArgs) -> Result<(), anyhow::Error> {
    let compressed_proof = fetch_compressed_proof(
        args.txid,
        args.bitcoin_rpc_url,
        args.bitcoin_rpc_userpwd,
        args.bridge_rpc_url,
        args.proof_url,
    )
    .await?;

    Ok(())
}

pub async fn fetch_compressed_proof(
    txid: Txid,
    bitcoin_rpc_url: String,
    bitcoin_rpc_userpwd: Option<String>,
    bridge_rpc_url: String,
    proof_url: String,
) -> Result<CompressedSpvProof, anyhow::Error> {
    let ChainStateProof {
        chain_state,
        chain_state_proof,
    } = fetch_chain_state_proof(proof_url)
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

    let block_header_proof = fetch_block_proof(
        block_height,
        chain_state.block_height as u32,
        bridge_rpc_url,
    )
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

pub async fn fetch_chain_state_proof(proof_url: String) -> Result<ChainStateProof, anyhow::Error> {
    info!("Fetching chain state proof from {}", proof_url);
    let response = reqwest::get(proof_url).await?;
    let proof: ChainStateProof = response.json().await?;
    Ok(proof)
}

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

pub async fn fetch_block_proof(
    block_height: u32,
    block_count: u32,
    bridge_rpc_url: String,
) -> Result<BlockInclusionProof, anyhow::Error> {
    info!("Fetching block proof for block height {}", block_height);
    let url = format!(
        "{}/proof/{}?block_count={}",
        bridge_rpc_url, block_height, block_count
    );
    let response = reqwest::get(url).await?;
    let proof: BlockInclusionProof = response.json().await?;
    Ok(proof)
}
