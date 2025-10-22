use anyhow::Result;
use bytes::Bytes;
use cairo_air::CairoProof;
use flate2::write::GzEncoder;
use flate2::Compression;
use google_cloud_storage::client::Storage;
use google_cloud_storage::model_ext::ReadRange;
use raito_spv_verify::ChainState;
use serde::{Deserialize, Serialize};
use std::io::Write;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use tracing::{debug, info};

/// Create a Google Cloud Storage client
async fn create_gcs_client() -> Result<Storage> {
    Ok(Storage::builder().build().await?)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecentProvenHeight {
    pub block_height: u32,
}

/// Download the latest proven height JSON from a GCS bucket object `recent_proven_height`
pub async fn download_recent_proof_height_from(bucket_name: &str) -> Result<u32> {
    info!(
        "Downloading latest proof height from GCS bucket: {} (object: recent_proven_height)",
        bucket_name
    );

    let client = create_gcs_client().await?;
    let bucket_path = format!("projects/_/buckets/{}", bucket_name);

    let mut reader = client
        .read_object(&bucket_path, "recent_proven_height")
        .set_read_range(ReadRange::offset(0))
        .send()
        .await?;

    let mut contents = Vec::new();
    while let Some(chunk) = reader.next().await.transpose()? {
        contents.extend_from_slice(&chunk);
    }

    let body = String::from_utf8(contents)?;
    let response: RecentProvenHeight = serde_json::from_str(&body)?;
    let height = response.block_height;
    debug!(
        "Successfully downloaded latest GCS proof height: {}",
        height
    );
    Ok(height)
}

#[derive(Serialize, Deserialize)]
pub struct RecentProof {
    pub timestamp: String,
    pub chainstate: ChainState,
    pub proof: CairoProof<Blake2sMerkleHasher>,
}

/// Download complete `recent_proof` using only reqwest, bypassing the GCS crate
/// Google Cloud Storage client library requires a content-length header when reading objects
pub async fn download_recent_proof_via_reqwest(bucket_name: &str) -> Result<RecentProof> {
    debug!(
        "Downloading proof data from GCS bucket: {} (object: recent_proof)",
        bucket_name
    );

    // Build the media URL
    let url = format!(
        "https://storage.googleapis.com/storage/v1/b/{}/o/recent_proof?alt=media",
        bucket_name
    );

    let client = reqwest::Client::new();
    // Fetch access token via ADC (uses GOOGLE_APPLICATION_CREDENTIALS if set)
    let scopes = &["https://www.googleapis.com/auth/devstorage.read_only"];
    let manager = gcp_auth::AuthenticationManager::new().await?;
    let token = manager.get_token(scopes).await?;

    let resp = client.get(url).bearer_auth(token.as_str()).send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("GCS request failed: {} - {}", status, body);
    }

    let body = resp.text().await?;

    debug!("Downloaded {} bytes of GCS proof data", body.len());

    Ok(serde_json::from_str(&body)?)
}

/// Download complete `recent_proof` object from a GCS bucket, does not work with gzipped objects
pub async fn download_recent_proof(bucket_name: &str) -> Result<RecentProof> {
    debug!(
        "Downloading proof data from GCS bucket: {} (object: recent_proof)",
        bucket_name
    );

    let client = create_gcs_client().await?;
    let bucket_path = format!("projects/_/buckets/{}", bucket_name);

    let mut reader = client
        .read_object(&bucket_path, "recent_proof")
        .set_read_range(ReadRange::offset(0))
        .send()
        .await?;

    let mut contents = Vec::new();
    while let Some(chunk) = reader.next().await.transpose()? {
        contents.extend_from_slice(&chunk);
    }

    let body = String::from_utf8(contents)?;

    debug!("Downloaded {} bytes of GCS proof data", body.len());

    Ok(serde_json::from_str(&body)?)
}

/// Upload recent proof to Google Cloud Storage in gzipped JSON format
pub async fn upload_recent_proof(recent_proof: &RecentProof, bucket_name: &str) -> Result<()> {
    debug!(
        "Uploading recent proof to Google Cloud Storage bucket: {}",
        bucket_name
    );

    let client = create_gcs_client().await?;
    let bucket_path = format!("projects/_/buckets/{}", bucket_name);

    // Serialize the proof to JSON
    let json_data = serde_json::to_string_pretty(recent_proof)?;

    // Compress the JSON data using gzip
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(json_data.as_bytes())?;
    let compressed_data = encoder.finish()?;

    debug!("Compressed data size: {} bytes", compressed_data.len());

    // Upload the compressed data as recent_proof with proper content type and encoding
    let _ = client
        .write_object(&bucket_path, "recent_proof", Bytes::from(compressed_data))
        .set_content_type("application/json")
        .set_content_encoding("gzip")
        .send_buffered()
        .await?;

    debug!("Successfully uploaded compressed proof to GCS as recent_proof");

    // Upload recent_proven_height file with block height information
    let proven_height_data = RecentProvenHeight {
        block_height: recent_proof.chainstate.block_height,
    };

    let height_json = serde_json::to_string_pretty(&proven_height_data)?;

    let _ = client
        .write_object(
            &bucket_path,
            "recent_proven_height",
            Bytes::from(height_json),
        )
        .set_content_type("application/json")
        .send_unbuffered()
        .await?;

    debug!("Successfully uploaded block_height to GCS as recent_proven_height");

    Ok(())
}
