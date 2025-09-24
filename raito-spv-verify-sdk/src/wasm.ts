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
        const wasmUrl = new URL(
          '../dist/bundler/index_bg.wasm',
          import.meta.url
        );
        const init = (mod as any).default ?? (mod as any).init;
        if (typeof init !== 'function')
          throw new Error('Browser/Edge initializer not found');
        await init(wasmUrl);
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
