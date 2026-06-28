// COD site build — copies static assets + bundles TS with esbuild
import * as esbuild from 'esbuild';
import { copyFileSync, mkdirSync, readdirSync } from 'fs';
import { join, extname, dirname } from 'path';
import { fileURLToPath } from 'url';

const dir = dirname(fileURLToPath(import.meta.url));
const srcDir = join(dir, 'src');
const outDir = join(dir, 'dist');

mkdirSync(outDir, { recursive: true });

const isWatch = process.argv.includes('--watch');

const opts = {
  entryPoints: [join(srcDir, 'main.ts')],
  outfile: join(outDir, 'script.js'),
  bundle: true,
  minify: true,
  sourcemap: false,
  target: 'es2022',
  logLevel: 'info',
};

// Copy static files (html, css)
for (const f of readdirSync(srcDir)) {
  const ext = extname(f);
  if (ext === '.html' || ext === '.css') {
    copyFileSync(join(srcDir, f), join(outDir, f));
  }
}

if (isWatch) {
  const ctx = await esbuild.context(opts);
  await ctx.watch();
  console.log('Watching for changes...');
} else {
  await esbuild.build(opts);
  console.log('Build complete → dist/');
}
