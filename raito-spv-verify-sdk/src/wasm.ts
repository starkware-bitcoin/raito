export async function importAndInit(): Promise<any> {
  // Environment detection
  const isNode =
    typeof window === 'undefined' &&
    typeof process !== 'undefined' &&
    process.versions &&
    process.versions.node;
  const isBrowser = typeof window !== 'undefined';

  try {
    // Load WASM module based on environment

    let wasm: any;
    if (isNode) {
      // Node.js environment - use dynamic import for ES modules
      wasm = await import('../dist/node/index.js');
    } else if (isBrowser) {
      // Browser environment - use web version for direct browser usage
      wasm = await import('../dist/web/index.js');
      const start = wasm.default ?? wasm.__wbg_init;
      if (typeof start !== 'function') {
        throw new Error('WASM initializer not found on module');
      }
      await start();
    } else {
      throw new Error(
        'Unsupported environment: neither Node.js nor browser detected'
      );
    }
    await wasm.init();
    return wasm;
  } catch (error) {
    throw new Error(`Failed to initialize WASM module: ${error}`);
  }
}
