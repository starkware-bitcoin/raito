use std::str::FromStr;
use std::sync::Arc;

use accumulators::store::StoreError;
use async_trait::async_trait;
use bitcoin::block::BlockHash;
use bitcoin::block::Header as BlockHeader;
use bitcoin::Target;
use bitcoin::Work;
use raito_spv_verify::ChainState;

const BLOCKS_PER_EPOCH: u32 = 2016;

#[async_trait]
pub trait ChainStateStore: Send + Sync {
    async fn add_block_header(
        &self,
        height: u32,
        block_header: &BlockHeader,
    ) -> Result<(), StoreError>;
    async fn get_block_headers(
        &self,
        start_height: u32,
        num_blocks: u32,
    ) -> Result<Vec<BlockHeader>, StoreError>;
    async fn get_block_height(&self, block_hash: &BlockHash) -> Result<u32, StoreError>;
    async fn add_chain_state(
        &self,
        height: u32,
        chain_state: &ChainState,
    ) -> Result<(), StoreError>;
    async fn get_chain_state(&self, height: u32) -> Result<ChainState, StoreError>;
}

pub struct ChainStateManager {
    current_state: ChainState,
    store: Arc<dyn ChainStateStore>,
}

impl ChainStateManager {
    pub async fn restore(
        store: Arc<dyn ChainStateStore>,
        height: u32,
    ) -> Result<Self, anyhow::Error> {
        let current_state = if height == 0 {
            Self::genesis_state()
        } else {
            store.get_chain_state(height - 1).await?
        };
        Ok(Self {
            current_state,
            store,
        })
    }

    pub async fn update(
        &mut self,
        block_height: u32,
        block_header: &BlockHeader,
    ) -> Result<(), anyhow::Error> {
        let new_state = if block_height == 0 {
            self.current_state.clone()
        } else {
            let mut prev_timestamps = self.current_state.prev_timestamps.clone();
            prev_timestamps.push(block_header.time);
            if prev_timestamps.len() > 11 {
                prev_timestamps.remove(0);
            }

            let epoch_start_time = if block_height % BLOCKS_PER_EPOCH == 0 {
                block_header.time
            } else {
                self.current_state.epoch_start_time
            };

            ChainState {
                block_height,
                total_work: self.current_state.total_work + block_header.work(),
                best_block_hash: block_header.block_hash(),
                current_target: block_header.target(),
                epoch_start_time,
                prev_timestamps,
            }
        };

        self.store.add_chain_state(block_height, &new_state).await?;
        self.store
            .add_block_header(block_height, block_header)
            .await?;
        self.current_state = new_state;

        Ok(())
    }

    pub fn genesis_state() -> ChainState {
        ChainState {
            block_height: 0,
            total_work: Work::from_hex("0x100010001").unwrap(),
            best_block_hash: BlockHash::from_str(
                "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
            )
            .unwrap(),
            current_target: Target::from_hex(
                "0xffff0000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            epoch_start_time: 1231006505,
            prev_timestamps: vec![1231006505],
        }
    }
}
