//! Transaction formatting utilities for terminal display.
//!
//! Provides ASCII art visualization of Bitcoin transactions similar to block explorers.

use bitcoin::absolute::LockTime;
use bitcoin::block::Header as BlockHeader;
use bitcoin::{Address, Amount, Network, Transaction, TxIn, TxOut};
use chrono::DateTime;

/// Format a Bitcoin transaction for terminal display
pub fn format_transaction(
    tx: &Transaction,
    network: Network,
    block_header: &BlockHeader,
    block_height: u32,
    chain_height: u32,
) -> String {
    let mut output = String::new();

    output.push_str("\n");

    // Header - make even wider to accommodate full TXID and longer addresses
    output.push_str("┌─ Bitcoin Transaction ───────────────────────────────────────────────────────────────────────────────────────────────────────────────┐\n");
    output.push_str(&format!(
        "│ \x1b[33mTXID:\x1b[0m {:<125} │\n",
        tx.compute_txid()
    ));
    output.push_str("├─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤\n");

    // Two-column layout: inputs on left, outputs on right
    let inputs_section = format_inputs(&tx.input);
    let outputs_section = format_outputs(&tx.output, network);

    // Split sections into lines for side-by-side display
    let input_lines: Vec<&str> = inputs_section.lines().collect();
    let output_lines: Vec<&str> = outputs_section.lines().collect();
    let max_lines = input_lines.len().max(output_lines.len());

    for i in 0..max_lines {
        let left = input_lines.get(i).unwrap_or(&"");
        let right = output_lines.get(i).unwrap_or(&"");

        // Handle line formatting with proper truncation and padding - make left column wider for full TXID
        let left_formatted = format_column_content(left, 64);
        let right_formatted = format_column_content(right, 64);

        output.push_str(&format!("│ {} │ {} │\n", left_formatted, right_formatted));
    }

    output.push_str("├─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤\n");

    // Details section - one column
    let details = format_transaction_details(tx, block_header, block_height, chain_height);

    for line in details.lines() {
        let line_formatted = format_column_content(line, 131);
        output.push_str(&format!("│ {} │\n", line_formatted));
    }

    output.push_str("└─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘\n");

    output
}

/// Format transaction inputs
fn format_inputs(inputs: &[TxIn]) -> String {
    let mut output = String::new();
    output.push_str("\x1b[33mINPUTS:\x1b[0m\n");

    for input in inputs.iter() {
        let address = format_input_address(input);
        output.push_str(&format!("{}\n\n", address));
    }

    if inputs.is_empty() {
        output.push_str("  (no inputs)\n");
    }

    output
}

/// Format transaction outputs
fn format_outputs(outputs: &[TxOut], network: Network) -> String {
    let mut output = String::new();
    output.push_str("\x1b[33mOUTPUTS:\x1b[0m\n");

    for txout in outputs.iter() {
        let address = format_output_address(txout, network);
        let amount_btc = Amount::from_sat(txout.value.to_sat()).to_btc();

        output.push_str(&format!("{}        {:.8} BTC\n", address, amount_btc));

        // Add script with each opcode on separate line
        let script_asm = txout.script_pubkey.to_asm_string();
        if !script_asm.is_empty() {
            let opcodes: Vec<&str> = script_asm.split_whitespace().collect();
            for opcode in opcodes {
                output.push_str(&format!("\x1b[90m  {}\x1b[0m\n", opcode));
            }
            // Add padding between outputs
            output.push_str("\n");
        }
    }

    if outputs.is_empty() {
        output.push_str("  (no outputs)\n");
    }

    output
}

/// Format transaction details card
fn format_transaction_details(
    tx: &Transaction,
    block_header: &BlockHeader,
    block_height: u32,
    chain_height: u32,
) -> String {
    let mut output = String::new();
    output.push_str("\x1b[33mDETAILS:\x1b[0m\n");

    output.push_str(&format!("Transaction size: {} bytes\n", tx.total_size()));

    output.push_str(&format!("Block hash: {}\n", block_header.block_hash()));
    output.push_str(&format!("Block height: {}\n", block_height));

    let timestamp = format_unix_timestamp(block_header.time);
    output.push_str(&format!("Block timestamp: {}\n", timestamp));

    // Calculate confirmations if both block_height and chain_height are available
    let confirmations = chain_height.saturating_sub(block_height);
    output.push_str(&format!("Confirmations: {}\n", confirmations));

    // Format locktime if set
    if tx.lock_time != LockTime::ZERO {
        let locktime_desc = match tx.lock_time {
            LockTime::Blocks(height) => format!("block {}", height),
            LockTime::Seconds(timestamp) => {
                // Convert Unix timestamp to readable format
                format!(
                    "timestamp {}",
                    format_unix_timestamp(timestamp.to_consensus_u32())
                )
            }
        };
        output.push_str(&format!("Locktime: {}\n", locktime_desc));
    }

    output
}

/// Get address string for a transaction input
fn format_input_address(input: &TxIn) -> String {
    // For inputs, we can try to extract address from script_sig, but it's not always possible
    // In many cases, we'd need the previous transaction output to know the address
    if input.previous_output.is_null() {
        "Coinbase".to_string()
    } else {
        // Show the TXID on one line and output index on the next line
        format!(
            "{}\nvout = {}",
            input.previous_output.txid, input.previous_output.vout
        )
    }
}

/// Get address string for a transaction output
fn format_output_address(output: &TxOut, network: Network) -> String {
    // Try to derive address from script_pubkey
    match Address::from_script(&output.script_pubkey, network) {
        Ok(address) => address.to_string(),
        Err(_) => {
            // If we can't parse as a standard address, show script type or raw script
            if output.script_pubkey.is_p2pk() {
                "P2PK".to_string()
            } else if output.script_pubkey.is_p2pkh() {
                "P2PKH".to_string()
            } else if output.script_pubkey.is_p2sh() {
                "P2SH".to_string()
            } else if output.script_pubkey.is_p2wpkh() {
                "P2WPKH".to_string()
            } else if output.script_pubkey.is_p2wsh() {
                "P2WSH".to_string()
            } else if output.script_pubkey.is_p2tr() {
                "P2TR".to_string()
            } else if output.script_pubkey.is_op_return() {
                "OP_RETURN".to_string()
            } else {
                "Unknown".to_string()
            }
        }
    }
}

/// Format content for a column with proper padding and truncation
fn format_column_content(content: &str, width: usize) -> String {
    // Remove ANSI color codes for length calculation
    let visible_content = strip_ansi_codes(content);
    let visible_len = visible_content.len();

    if visible_len <= width {
        // Content fits, pad with spaces
        let padding = width - visible_len;
        format!("{}{}", content, " ".repeat(padding))
    } else {
        // Return content as-is without truncation
        content.to_string()
    }
}

/// Remove ANSI color codes from a string for length calculation
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip the ANSI escape sequence
            while let Some(next_c) = chars.next() {
                if next_c == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Format Unix timestamp to human-readable string
fn format_unix_timestamp(timestamp: u32) -> String {
    let dt = DateTime::from_timestamp(timestamp as i64, 0).expect("Invalid timestamp");
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
