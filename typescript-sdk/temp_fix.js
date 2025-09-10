// Read the current index.ts file
const fs = require('fs');
const content = fs.readFileSync('src/index.ts', 'utf8');

// Add a helper function to convert hex string to number
const helperFunction = `
  /**
   * Convert hex string to number for Bitcoin block header fields
   */
  private convertHexToNumber(hexString: string): number {
    return parseInt(hexString, 16);
  }

  /**
   * Normalize proof data for WASM compatibility
   */
  private normalizeProofForWasm(proof: CompressedSpvProof): CompressedSpvProof {
    const normalizedProof = { ...proof };
    
    // Convert bits field from hex string to number
    if (normalizedProof.block_header && typeof normalizedProof.block_header.bits === 'string') {
      normalizedProof.block_header = {
        ...normalizedProof.block_header,
        bits: this.convertHexToNumber(normalizedProof.block_header.bits)
      };
    }
    
    return normalizedProof;
  }
`;

// Find the position to insert the helper functions (after the constructor)
const constructorEnd = content.indexOf('  }', content.indexOf('constructor(')) + 3;
const beforeInit = content.indexOf('  /**\n   * Initialize the SDK with WASM module\n   */');

// Insert the helper functions
const newContent = content.slice(0, beforeInit) + helperFunction + '\n' + content.slice(beforeInit);

// Modify the verifyProof method to use the normalization
const verifyProofStart = newContent.indexOf('  async verifyProof(');
const verifyProofEnd = newContent.indexOf('    } catch (error) {', verifyProofStart);
const verifyProofMethod = newContent.slice(verifyProofStart, verifyProofEnd);

// Replace the proof serialization line
const newVerifyProofMethod = verifyProofMethod.replace(
  '      const proofJson = JSON.stringify(proof);',
  '      const normalizedProof = this.normalizeProofForWasm(proof);\n      const proofJson = JSON.stringify(normalizedProof);'
);

// Replace the method in the content
const finalContent = newContent.replace(verifyProofMethod, newVerifyProofMethod);

// Write the modified content back
fs.writeFileSync('src/index.ts', finalContent);

console.log('✅ Modified src/index.ts to fix bits field conversion');
