import fs from 'fs';

const paths = [
  'dist/bundler/index_bg.wasm.d.ts',
  'dist/node/index_bg.wasm.d.ts',
];
for (const p of paths) {
  let s = fs.readFileSync(p, 'utf8');
  s = s
    .split('\n')
    .filter(l => !l.includes('LIBBZ2_RS_SYS_v0.1.x_'))
    .join('\n');
  fs.writeFileSync(p, s);
}
