//! Block time validation helpers.
//!
//! Read more: https://learnmeabitcoin.com/technical/block/time/

/// Computes the Median Time Past (MTP) from the previous timestamps.
pub fn compute_median_time_past(prev_timestamps: Span<u32>) -> u32 {
    // Sort the last 11 timestamps
    // adapted from :
    // https://github.com/keep-starknet-strange/alexandria/blob/main/packages/sorting/src/bubble_sort.cairo
    let mut idx1 = 0;
    let mut idx2 = 1;
    let mut sorted_iteration = true;
    let mut prev_timestamps: Span<u32> = prev_timestamps.clone();
    let mut sorted_prev_timestamps: Array<u32> = array![];

    loop {
        if idx2 == prev_timestamps.len() {
            sorted_prev_timestamps.append(*prev_timestamps[idx1]);
            if sorted_iteration {
                break;
            }
            prev_timestamps = sorted_prev_timestamps.span();
            sorted_prev_timestamps = array![];
            idx1 = 0;
            idx2 = 1;
            sorted_iteration = true;
        } else if *prev_timestamps[idx1] <= *prev_timestamps[idx2] {
            sorted_prev_timestamps.append(*prev_timestamps[idx1]);
            idx1 = idx2;
            idx2 += 1;
        } else {
            sorted_prev_timestamps.append(*prev_timestamps[idx2]);
            idx2 += 1;
            sorted_iteration = false;
        }
    }

    (*sorted_prev_timestamps.at(sorted_prev_timestamps.len() / 2))
}

/// Checks that the block time is greater than the Median Time Past (MTP).
pub fn validate_timestamp(median_time_past: u32, block_time: u32) -> Result<(), ByteArray> {
    if block_time > median_time_past {
        Result::Ok(())
    } else {
        Result::Err(
            format!("Median time past: {} >= block's timestamp: {}", median_time_past, block_time),
        )
    }
}

/// Updates the list of the recent timestamps, removing the oldest and appending the most recent
/// one.
pub fn next_prev_timestamps(prev_timestamps: Span<u32>, block_time: u32) -> Span<u32> {
    let mut timestamps: Array<u32> = prev_timestamps.into();
    if timestamps.len() == 11 {
        timestamps.pop_front().unwrap(); // remove the oldest timestamp (not necessarily the min)
    }
    timestamps.append(block_time); //  append the most recent timestamp (not necessarily the max)

    timestamps.span()
}

#[cfg(test)]
mod tests {
    use super::{compute_median_time_past, next_prev_timestamps, validate_timestamp};

    #[test]
    fn test_compute_median_time_past() {
        let prev_timestamps = array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11].span();
        let mtp = compute_median_time_past(prev_timestamps);
        assert_eq!(mtp, 6, "Expected MTP to be 6");

        let unsorted_timestamps = array![1, 3, 2, 5, 4, 6, 8, 7, 9, 10, 11].span();
        let mtp = compute_median_time_past(unsorted_timestamps);
        assert_eq!(mtp, 6, "Expected MTP to be 6 for unsorted");
    }

    #[test]
    fn test_validate_timestamp() {
        let mtp = 6_u32;
        let mut block_time = 7_u32;

        // New timestamp is greater than MTP
        let result = validate_timestamp(mtp, block_time);
        assert(result.is_ok(), 'Expected timestamp to be valid');

        // New timestamp is equal to MTP
        block_time = 6;
        let result = validate_timestamp(mtp, block_time);
        assert!(result.is_err(), "MTP is greater than or equal to block's timestamp");

        // New timestamp is less than MTP
        block_time = 5;
        let result = validate_timestamp(mtp, block_time);
        assert!(result.is_err(), "MTP is greater than block's timestamp");
    }

    #[test]
    fn test_next_prev_timestamps() {
        let prev_timestamps = array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11].span();
        let mut block_time = 12_u32;

        let next_prev_timestamps = next_prev_timestamps(prev_timestamps, block_time);
        assert_eq!(next_prev_timestamps.len(), 11);
        assert_eq!(next_prev_timestamps, array![2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12].span());
    }

    #[test]
    fn test_validate_timestamp_with_unsorted_input() {
        let prev_timestamps = array![1, 3, 2, 5, 4, 6, 8, 7, 9, 10, 11].span();
        let mtp = compute_median_time_past(prev_timestamps);
        let mut block_time = 12_u32;

        // New timestamp is greater than MTP
        let result = validate_timestamp(mtp, block_time);
        assert(result.is_ok(), 'Expected timestamp to be valid');

        // New timestamp is equal to MTP
        block_time = 6;
        let result = validate_timestamp(mtp, block_time);
        assert!(result.is_err(), "MTP is greater than or equal to block's timestamp");

        // New timestamp is less than MTP
        block_time = 5;
        let result = validate_timestamp(mtp, block_time);
        assert!(result.is_err(), "MTP is greater than block's timestamp");
    }

    #[test]
    fn test_few_prev_timestamps() {
        assert_eq!(1, compute_median_time_past(array![1].span()));
        assert_eq!(2, compute_median_time_past(array![1, 2].span()));
        assert_eq!(2, compute_median_time_past(array![1, 2, 3].span()));
        assert_eq!(3, compute_median_time_past(array![1, 2, 3, 4].span()));
        assert_eq!(3, compute_median_time_past(array![1, 2, 3, 4, 5].span()));
    }
}
