import { cp, mkdir, rm } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const packageRoot = path.resolve(fileURLToPath(import.meta.url), '..', '..');
const repoRoot = path.resolve(packageRoot, '..', '..');
const source = path.join(repoRoot, 'templates', 'default');
const target = path.join(packageRoot, 'templates', 'default');

await rm(target, { recursive: true, force: true });
await mkdir(path.dirname(target), { recursive: true });
await cp(source, target, {
  recursive: true,
  filter: (sourcePath) => !sourcePath.includes(`${path.sep}target${path.sep}`),
});

console.log(`Bundled template from ${source}`);

