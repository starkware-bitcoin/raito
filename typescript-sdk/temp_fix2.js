// Read the current index.ts file
const fs = require('fs');
const content = fs.readFileSync('src/index.ts', 'utf8');

// Add a WASM-compatible proof type
const wasmProofType = `
// WASM-compatible proof type with numeric bits field
interface WasmCompatibleProof {
  chain_state: ChainState;
  chain_state_proof: ChainStateProof;
  block_header: {
    version: number;
    previousblockhash: string;
    merkleroot: string;
    time: number;
    bits: number; // Converted from hex string to number
    nonce: number;
    hash: string;
  };
  block_header_proof: BlockInclusionProof;
  transaction: BitcoinTransaction;
  transaction_proof: number[];
}
`;

// Find the position to insert the type (after the imports)
const afterImports = content.indexOf('declare const require: any;') + 'declare const require: any;'.length;
const beforeClass = content.indexOf('export class RaitoSpvSdk');

// Insert the type definition
const newContent = content.slice(0, beforeClass) + wasmProofType + '\n' + content.slice(beforeClass);

// Update the normalizeProofForWasm method to return the correct type
const normalizeMethod = newContent.indexOf('  private normalizeProofForWasm(proof: CompressedSpvProof): CompressedSpvProof {');
const normalizeMethodEnd = newContent.indexOf('  }', normalizeMethod) + 3;

const newNormalizeMethod = `  private normalizeProofForWasm(proof: CompressedSpvProof): WasmCompatibleProof {
    const normalizedProof = { ...proof } as any;
    
    // Convert bits field from hex string to number
    if (normalizedProof.block_header && typeof normalizedProof.block_header.bits === 'string') {
      normalizedProof.block_header = {
        ...normalizedProof.block_header,
        bits: this.convertHexToNumber(normalizedProof.block_header.bits)
      };
    }
    
    return normalizedProof;
  }`;

const finalContent = newContent.replace(
  newContent.slice(normalizeMethod, normalizeMethodEnd),
  newNormalizeMethod
);

// Write the modified content back
fs.writeFileSync('src/index.ts', finalContent);

console.log('✅ Modified src/index.ts with WASM-compatible proof type');
