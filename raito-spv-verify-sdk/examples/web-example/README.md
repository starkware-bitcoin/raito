# Raito SPV SDK Web Example

A simple Vite application demonstrating how to use the Raito SPV SDK in a browser environment.

## Features

- 🚀 **Simple Setup**: Minimal Vite configuration for quick development
- 🌐 **Browser Demo**: Interactive web interface for testing SPV verification
- 📱 **Responsive Design**: Works on desktop and mobile devices
- ⚡ **Real-time Verification**: Fetch and verify Bitcoin transaction proofs
- 🎨 **Modern UI**: Clean, professional interface with loading states

## Quick Start

1. **Install dependencies**:
   ```bash
   npm install
   ```

2. **Start development server**:
   ```bash
   npm run dev
   ```

3. **Open your browser** to `http://localhost:3000`

4. **Test the SDK**:
   - The app comes pre-loaded with a sample transaction ID
   - Click "Verify Transaction" to fetch and verify the proof
   - Try different transaction IDs to test various scenarios

## What It Demonstrates

### SDK Initialization
```javascript
import { createRaitoSpvSdk } from '@raito-stark/spv-verify';

const sdk = createRaitoSpvSdk();
await sdk.init();
```

### Proof Fetching
```javascript
const proof = await sdk.fetchProof(txid);
```

### Proof Verification
```javascript
const result = await sdk.verifyProof(proof);
```

## Project Structure

```
web-example/
├── index.html          # Main HTML template
├── main.js            # Application logic and SDK usage
├── style.css          # Styling and responsive design
├── package.json       # Dependencies and scripts
├── vite.config.js     # Vite configuration
└── README.md          # This file
```

## Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run preview` - Preview production build

## Browser Compatibility

This example works in all modern browsers that support:
- ES6 modules
- WebAssembly
- Fetch API
- Async/await

## Troubleshooting

### SDK Initialization Fails
- Ensure the SDK is built and available in the parent directory
- Check browser console for detailed error messages
- Verify network connectivity for WASM module loading

### Proof Fetching Errors
- Check if the transaction ID is valid
- Verify network connectivity to the Raito API
- Some transactions may not have available proofs

### Performance Issues
- WASM module loading may take a few seconds on first load
- Large proofs may take longer to verify
- Consider implementing loading indicators for better UX

## Customization

You can easily customize this example by:

1. **Modifying the UI**: Edit `index.html` and `style.css`
2. **Adding features**: Extend `main.js` with additional functionality
3. **Changing configuration**: Update `vite.config.js` for different build settings
4. **Using different endpoints**: Modify the SDK initialization with custom RPC URLs

## Integration

This example shows the basic integration pattern you can use in your own applications:

1. Import the SDK
2. Initialize with `init()`
3. Use `fetchProof()` to get transaction proofs
4. Use `verifyProof()` to validate proofs
5. Handle errors appropriately
6. Provide user feedback during async operations
