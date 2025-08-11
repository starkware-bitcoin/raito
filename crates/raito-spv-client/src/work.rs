//! Work verification utilities for ensuring sufficient confirmations on top of a target block.

use num_bigint::BigUint;
use std::cmp::{max, min};
use std::str::FromStr;
use tracing::info;

use crate::{proof::ChainState, verify::VerifierConfig};

/// Verify that there is enough work added on top of the target block.
pub fn verify_subchain_work(
    block_height: u32,
    chain_state: &ChainState,
    config: &VerifierConfig,
) -> anyhow::Result<()> {
    // Difficulty target is readjusted every 2016 blocks
    // The maximum difficulty re-adjustment step is 4x.
    // We are rewinding the chain state down to the target block height, assuming worst case scenario
    // where the difficulty is reducing (target is increasing) by 4x every 2016 blocks
    let start_epoch = chain_state.block_height / 2016;
    let end_epoch = block_height / 2016;
    let mut subchain_work = BigUint::ZERO;
    let mut target = BigUint::from_str(&chain_state.current_target).unwrap();

    for epoch in (end_epoch..=start_epoch).rev() {
        let start_block = min(2016 * (epoch + 1), chain_state.block_height);
        let end_block = max(2016 * epoch, block_height);
        let block_span = BigUint::from(start_block - end_block);
        let block_work = compute_work_from_target(target.clone());
        subchain_work += block_work * block_span;
        target *= BigUint::from(4_u32);
    }

    let min_work = BigUint::from_str(&config.min_work).unwrap();
    if subchain_work < min_work {
        anyhow::bail!(
            "Subchain work is less than the minimum work: {} < {}",
            subchain_work,
            min_work
        );
    }

    info!(
        "Subchain work is sufficient: 0x{:x} >= 0x{:x}",
        subchain_work, min_work
    );
    Ok(())
}

/// Compute the expected work for a single block given the target difficulty.
fn compute_work_from_target(target: BigUint) -> BigUint {
    // 2^256
    let max_work = BigUint::from_str(
        "115792089237316195423570985008687907853269984665640564039457584007913129639936",
    )
    .unwrap();
    max_work / (target + BigUint::from(1_u32))
}
