/**
 * Compressed SPV Proof Module
 * Handles fetching and verification of compressed SPV proofs
 */

export interface VerifierConfig {
  min_work: string;
  bootloader_hash: string;
  task_program_hash: string;
  task_output_size: number;
}

/**
 * Create verifier configuration with defaults
 */
export function createVerifierConfig(
  config?: Partial<VerifierConfig>
): VerifierConfig {
  return {
    min_work: config?.min_work || '1813388729421943762059264',
    bootloader_hash:
      config?.bootloader_hash ||
      '0x0001837d8b77b6368e0129ce3f65b5d63863cfab93c47865ee5cbe62922ab8f3',
    task_program_hash:
      config?.task_program_hash ||
      '0x00f0876bb47895e8c4a6e7043829d7886e3b135e3ef30544fb688ef4e25663ca',
    task_output_size: config?.task_output_size || 8,
  };
}
