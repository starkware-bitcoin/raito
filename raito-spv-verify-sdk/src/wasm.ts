let once: Promise<any> | null = null;

export async function importAndInit() {
  if (once) return once;

  const isBrowser = typeof window !== 'undefined';
  const isNode = !isBrowser && !!process?.versions?.node;
  const isEdge = !isBrowser && !isNode && process?.env?.NEXT_RUNTIME === 'edge';

  const load = async () => {
    try {
      if (isNode && !isEdge) {
        const mod = await import(
          /* webpackIgnore: true */ '../dist/node/index.js'
        );
        const initSync =
          (mod as any).initSync ?? (mod as any).__wbg_init ?? null;
        if (typeof initSync === 'function') initSync();
        return mod;
      } else {
        const mod = await import('../dist/bundler/index.js');
        const init =
          (mod as any).default ?? (mod as any).init ?? (mod as any).__wbg_init;

        if (typeof init === 'function') {
          const wasmBytes = await import('../dist/bundler/index_bg.wasm');

          if (
            wasmBytes &&
            typeof wasmBytes === 'object' &&
            wasmBytes !== null &&
            'default' in wasmBytes
          ) {
            const response = new Response(wasmBytes.default as any);
            await init(response);
          } else {
            await init();
          }
        }

        return mod;
      }
    } catch (err) {
      const e = err instanceof Error ? err : new Error(String(err));
      throw new Error('Failed to initialize WASM module', { cause: e });
    }
  };

  once = load();
  return once;
}
