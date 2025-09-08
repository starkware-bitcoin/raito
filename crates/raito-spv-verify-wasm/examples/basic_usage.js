// Basic usage example for raito-spv-verify-wasm
// This example demonstrates how to use the WASM bindings in a Node.js environment

const { 
    verify_proof_async, 
    verify_proof_sync, 
    create_default_config, 
    create_custom_config 
} = require('../pkg/raito_spv_verify_wasm');

async function main() {
    console.log('Raito SPV Verify WASM - Basic Usage Example');
    console.log('===========================================');

    // Create a default configuration
    console.log('\n1. Creating default configuration...');
    const defaultConfig = create_default_config();
    console.log('Default config:', defaultConfig);

    // Create a custom configuration
    console.log('\n2. Creating custom configuration...');
    const customConfig = create_custom_config(
        "1813388729421943762059264",
        "0x0001837d8b77b6368e0129ce3f65b5d63863cfab93c47865ee5cbe62922ab8f3",
        "0x00f0876bb47895e8c4a6e7043829d7886e3b135e3ef30544fb688ef4e25663ca",
        8
    );
    console.log('Custom config:', customConfig);

    // Example proof data (this would normally come from your application)
    const exampleProof = {
        chain_state: {
            block_height: 123456,
            total_work: "1813388729421943762059264",
            best_block_hash: "0000000000000000000000000000000000000000000000000000000000000000",
            current_target: "123456789",
            epoch_start_time: 1234567890,
            prev_timestamps: [1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890]
        },
        chain_state_proof: {
            // This would be actual Cairo proof data
            // For demonstration purposes, we'll use placeholder data
        },
        block_header: {
            // This would be actual Bitcoin block header data
            // For demonstration purposes, we'll use placeholder data
        },
        block_header_proof: {
            // This would be actual MMR proof data
            // For demonstration purposes, we'll use placeholder data
        },
        transaction: {
            // This would be actual Bitcoin transaction data
            // For demonstration purposes, we'll use placeholder data
        },
        transaction_proof: []
    };

    console.log('\n3. Example proof structure created');
    console.log('Note: This is placeholder data for demonstration');

    // Demonstrate async verification (recommended for production)
    console.log('\n4. Demonstrating async verification...');
    try {
        // Note: This will fail with the placeholder data, but shows the API usage
        const asyncResult = await verify_proof_async(exampleProof, defaultConfig, true);
        console.log('Async verification result:', asyncResult);
    } catch (error) {
        console.log('Async verification error (expected with placeholder data):', error.message);
    }

    // Demonstrate sync verification (not recommended for production)
    console.log('\n5. Demonstrating sync verification...');
    try {
        // Note: This will fail with the placeholder data, but shows the API usage
        const syncResult = verify_proof_sync(exampleProof, defaultConfig, true);
        console.log('Sync verification result:', syncResult);
    } catch (error) {
        console.log('Sync verification error (expected with placeholder data):', error.message);
    }

    console.log('\n6. Example completed successfully!');
    console.log('\nTo use with real data:');
    console.log('- Replace the placeholder proof data with actual SPV proof data');
    console.log('- Use verify_proof_async() for production applications');
    console.log('- Handle errors appropriately in your application');
}

// Run the example
main().catch(console.error); 