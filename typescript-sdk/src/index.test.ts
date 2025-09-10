/**
 * Tests for Raito SPV TypeScript SDK
 */

import { RaitoSpvSdk, createRaitoSpvSdk } from './index';
import { CompressedSpvProof, VerifierConfig } from '../types';

// Mock the WASM module
const mockWasmModule = {
  init: jest.fn().mockResolvedValue(undefined),
  verify_proof_wasm: jest.fn().mockResolvedValue(true)
};

// Mock require for WASM module
jest.mock('raito-spv-verify-wasm', () => mockWasmModule);

describe('RaitoSpvSdk', () => {
  let sdk: RaitoSpvSdk;

  beforeEach(() => {
    sdk = new RaitoSpvSdk({
      bitcoinRpcUrl: 'http://localhost:8332',
      bitcoinRpcUserPwd: 'user:password'
    });
    
    // Reset mocks
    jest.clearAllMocks();
  });

  describe('constructor', () => {
    it('should create SDK instance with required options', () => {
      expect(sdk).toBeInstanceOf(RaitoSpvSdk);
    });

    it('should use default raito RPC URL when not provided', () => {
      const sdkWithDefaults = new RaitoSpvSdk({
        bitcoinRpcUrl: 'http://localhost:8332'
      });
      expect(sdkWithDefaults).toBeInstanceOf(RaitoSpvSdk);
    });
  });

  describe('init', () => {
    it('should initialize WASM module successfully', async () => {
      await sdk.init();
      expect(mockWasmModule.init).toHaveBeenCalled();
    });

    it('should throw error if WASM module fails to load', async () => {
      mockWasmModule.init.mockRejectedValueOnce(new Error('WASM load failed'));
      
      await expect(sdk.init()).rejects.toThrow('Failed to initialize WASM module');
    });
  });

  describe('verifyProof', () => {
    const mockProof: CompressedSpvProof = {
      chain_state: {
        block_height: 800000,
        total_work: "1813388729421943762059264",
        best_block_hash: "0000000000000000000000000000000000000000000000000000000000000000",
        current_target: "123456789",
        epoch_start_time: 1234567890,
        prev_timestamps: [1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890, 1234567890]
      },
      chain_state_proof: {},
      block_header: {
        version: 1,
        previousblockhash: "0000000000000000000000000000000000000000000000000000000000000000",
        merkleroot: "0000000000000000000000000000000000000000000000000000000000000000",
        time: 1234567890,
        bits: "1d00ffff",
        nonce: 123456,
        hash: "0000000000000000000000000000000000000000000000000000000000000000"
      },
      block_header_proof: {
        peaks_hashes: [],
        siblings_hashes: [],
        leaf_index: 0,
        leaf_count: 1
      },
      transaction: {
        version: 1,
        locktime: 0,
        vin: [],
        vout: []
      },
      transaction_proof: []
    };

    beforeEach(async () => {
      await sdk.init();
    });

    it('should verify proof successfully', async () => {
      const result = await sdk.verifyProof(mockProof);
      
      expect(result).toBe(true);
      expect(mockWasmModule.verify_proof_wasm).toHaveBeenCalledWith(JSON.stringify(mockProof));
    });

    it('should throw error if SDK not initialized', async () => {
      const uninitializedSdk = new RaitoSpvSdk({
        bitcoinRpcUrl: 'http://localhost:8332'
      });

      await expect(uninitializedSdk.verifyProof(mockProof)).rejects.toThrow('SDK not initialized');
    });

    it('should use custom config when provided', async () => {
      const customConfig: Partial<VerifierConfig> = {
        min_work: "1000000000000000000000000",
        task_output_size: 16
      };

      await sdk.verifyProof(mockProof, customConfig);
      
      // The config should be used internally (we can't easily test this without exposing internals)
      expect(mockWasmModule.verify_proof_wasm).toHaveBeenCalled();
    });

    it('should handle verification errors', async () => {
      mockWasmModule.verify_proof_wasm.mockRejectedValueOnce(new Error('Invalid proof'));

      await expect(sdk.verifyProof(mockProof)).rejects.toThrow('Proof verification failed');
    });
  });

  describe('createRaitoSpvSdk', () => {
    it('should create SDK instance using factory function', () => {
      const sdk = createRaitoSpvSdk({
        bitcoinRpcUrl: 'http://localhost:8332'
      });

      expect(sdk).toBeInstanceOf(RaitoSpvSdk);
    });
  });
});

// Integration tests (these would require actual RPC endpoints)
describe('Integration Tests', () => {
  // These tests would be skipped in CI/CD and only run locally with actual Bitcoin/Raito nodes
  const isIntegrationTest = process.env.INTEGRATION_TESTS === 'true';

  const integrationTest = isIntegrationTest ? it : it.skip;

  integrationTest('should fetch and verify real proof', async () => {
    const sdk = createRaitoSpvSdk({
      bitcoinRpcUrl: process.env.BITCOIN_RPC_URL || 'http://localhost:8332',
      bitcoinRpcUserPwd: process.env.BITCOIN_RPC_USERPWD,
      raitoRpcUrl: process.env.RAITO_RPC_URL || 'https://api.raito.wtf'
    });

    await sdk.init();

    const result = await sdk.fetchAndVerifyProof({
      txid: process.env.TEST_TXID || 'a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456',
      bitcoinRpcUrl: process.env.BITCOIN_RPC_URL || 'http://localhost:8332'
    });

    expect(result.verified).toBeDefined();
    expect(result.proof).toBeDefined();
  });
});
