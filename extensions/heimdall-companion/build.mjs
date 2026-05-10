import { build } from 'esbuild';
import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { join, dirname } from 'node:path';

const target = process.argv[2] ?? 'chrome';
const out = `dist/${target}`;
await mkdir(out, { recursive: true });

const common = {
  bundle: true,
  format: 'esm',
  target: 'es2022',
  sourcemap: true,
  logLevel: 'info',
};

await build({
  ...common,
  entryPoints: ['src/background.ts', 'src/options/options.ts', 'src/popup/popup.ts', 'src/content/observer.ts'],
  outdir: out,
  entryNames: '[name]',
});

// Merge manifests for Firefox.
const base = JSON.parse(await readFile('manifest.json', 'utf8'));
let manifest = base;
if (target === 'firefox') {
  const fx = JSON.parse(await readFile('manifest.firefox.json', 'utf8'));
  manifest = deepMerge(base, fx);
  delete manifest.background.service_worker;
}
await writeFile(join(out, 'manifest.json'), JSON.stringify(manifest, null, 2));

for (const file of ['src/options/options.html', 'src/popup/popup.html']) {
  const dst = join(out, file.split('/').pop());
  await mkdir(dirname(dst), { recursive: true });
  await writeFile(dst, await readFile(file, 'utf8'));
}

function deepMerge(a, b) {
  if (a === null || typeof a !== 'object') return b;
  if (b === null || typeof b !== 'object') return a;
  const out = Array.isArray(a) ? [...a] : { ...a };
  for (const k of Object.keys(b)) out[k] = deepMerge(a[k], b[k]);
  return out;
}
