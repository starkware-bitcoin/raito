use std::collections::HashMap;
use std::ops::DerefMut;
use std::path::Path;

use accumulators::store::{sqlite::SQLiteStore, Store as AccumulatorsStore, StoreError};
use async_trait::async_trait;
use bitcoin::block::Header as BlockHeader;
use bitcoin::consensus::{Decodable, Encodable};
use bitcoin::BlockHash;
use raito_spv_mmr::block_mmr::BlockMMRStore;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
    SqliteTransactionManager,
};
use sqlx::{Row, TransactionManager};
use tokio::fs;

/// SQLite busy timeout in milliseconds
const SQLITE_BUSY_TIMEOUT: &str = "5000";

/// Maximum number of concurrent readers (size of the connection pool)
const SQLITE_MAX_CONCURRENT_READERS: u32 = 10;

/// SQLite-backed store with single-writer and multi-reader pools.
/// - WAL mode for concurrent readers during writes
/// - Single writer (max_connections = 1)
/// - Optional active write transaction encapsulated in the store
#[derive(Debug)]
pub struct AppStore(SQLiteStore);

impl AppStore {
    /// Create a store for a single atomic writer
    pub async fn single_atomic_writer<P: AsRef<Path>>(
        path: P,
        id: Option<String>,
    ) -> Result<Self, sqlx::Error> {
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).await?;
        }

        let options = SqliteConnectOptions::new()
            .filename(path.as_ref())
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .pragma("busy_timeout", SQLITE_BUSY_TIMEOUT);

        // Writer pool: single connection ensures single-writer semantics
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        let store = Self(SQLiteStore::with_pool(pool, id));
        store.init().await?;

        Ok(store)
    }

    /// Create a store for multiple concurrent readers
    pub async fn multiple_concurrent_readers<P: AsRef<Path>>(
        path: P,
        id: Option<String>,
    ) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::new()
            .filename(path.as_ref())
            .read_only(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(SQLITE_MAX_CONCURRENT_READERS)
            .connect_with(options)
            .await?;

        Ok(Self(SQLiteStore::with_pool(pool, id)))
    }

    /// Initialize the store by creating the tables if missing
    async fn init(&self) -> Result<(), sqlx::Error> {
        // Create a key-value store table for MMR accumulator state
        self.0.init().await?;
        // Create a table for encoded block headers
        let mut conn = self.0.acquire_connection().await?;
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS block_headers (
                height INTEGER PRIMARY KEY,
                hash TEXT NOT NULL,
                header BLOB NOT NULL
            );"#,
        )
        .execute(conn.deref_mut())
        .await?;
        // Add index on block hash column
        sqlx::query(
            r#"CREATE INDEX IF NOT EXISTS idx_block_headers_hash ON block_headers (hash);"#,
        )
        .execute(conn.deref_mut())
        .await?;
        Ok(())
    }

    /// Begin a new transaction.
    /// NOTE that this function does not check if there is already a transaction in progress.
    pub async fn begin(&self) -> Result<(), StoreError> {
        let mut conn = self.0.acquire_connection().await?;
        SqliteTransactionManager::begin(&mut conn, None)
            .await
            .map_err(StoreError::SQLite)
    }

    /// Commit the current transaction.
    /// NOTE that this function does not check if there is a transaction in progress.
    pub async fn commit(&self) -> Result<(), StoreError> {
        let mut conn = self.0.acquire_connection().await?;
        SqliteTransactionManager::commit(&mut conn)
            .await
            .map_err(StoreError::SQLite)
    }
}

#[async_trait]
impl BlockMMRStore for AppStore {
    /// Add a new block header to the store
    async fn add_block_header(
        &self,
        height: u32,
        block_header: &BlockHeader,
    ) -> Result<(), StoreError> {
        let mut block_header_data = Vec::new();
        let mut conn = self.0.acquire_connection().await?;
        block_header
            .consensus_encode(&mut block_header_data)
            .map_err(|e| StoreError::Custom(Box::new(e)))?;
        sqlx::query("INSERT INTO block_headers (height, hash, header) VALUES (?, ?, ?)")
            .bind(height)
            .bind(block_header.block_hash().to_string())
            .bind(block_header_data)
            .execute(conn.deref_mut())
            .await?;
        Ok(())
    }

    /// Get a range of block headers from the store
    async fn get_block_headers(
        &self,
        start_height: u32,
        num_blocks: u32,
    ) -> Result<Vec<BlockHeader>, StoreError> {
        let mut conn = self.0.acquire_connection().await?;
        let rows = sqlx::query("SELECT header FROM block_headers WHERE height >= ? AND height < ?")
            .bind(start_height)
            .bind(start_height + num_blocks)
            .fetch_all(conn.deref_mut())
            .await?;
        rows.iter()
            .map(|row| {
                let header: Vec<u8> = row.get("header");
                BlockHeader::consensus_decode(&mut header.as_slice())
                    .map_err(|e| StoreError::Custom(Box::new(e)))
            })
            .collect()
    }

    /// Get the height of a block by its hash
    async fn get_block_height(&self, block_hash: &BlockHash) -> Result<u32, StoreError> {
        let mut conn = self.0.acquire_connection().await?;
        let row = sqlx::query("SELECT height FROM block_headers WHERE hash = ?")
            .bind(block_hash.to_string())
            .fetch_optional(conn.deref_mut())
            .await?;
        row.map(|row| row.get("height")).ok_or(StoreError::GetError)
    }
}

#[async_trait]
impl AccumulatorsStore for AppStore {
    fn id(&self) -> String {
        self.0.id()
    }
    async fn get(&self, key: &str) -> Result<Option<String>, StoreError> {
        self.0.get(key).await
    }
    async fn get_many(&self, keys: Vec<&str>) -> Result<HashMap<String, String>, StoreError> {
        self.0.get_many(keys).await
    }
    async fn set(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.0.set(key, value).await
    }
    async fn set_many(&self, entries: HashMap<String, String>) -> Result<(), StoreError> {
        self.0.set_many(entries).await
    }
    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.0.delete(key).await
    }
    async fn delete_many(&self, keys: Vec<&str>) -> Result<(), StoreError> {
        self.0.delete_many(keys).await
    }
}
