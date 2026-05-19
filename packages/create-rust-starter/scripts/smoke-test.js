import { execFile } from 'node:child_process';
import { mkdtemp, readFile, rm, stat } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { promisify } from 'node:util';
import { fileURLToPath } from 'node:url';

const execFileAsync = promisify(execFile);
const packageRoot = path.resolve(fileURLToPath(import.meta.url), '..', '..');
const binPath = path.join(packageRoot, 'bin', 'create-rust-starter.js');

const tempRoot = await mkdtemp(path.join(os.tmpdir(), 'create-rust-starter-'));
const projectName = 'my-api-smoke';
const projectDir = path.join(tempRoot, projectName);

try {
  const { stdout, stderr } = await execFileAsync(process.execPath, [binPath, projectName], {
    cwd: tempRoot,
    env: {
      ...process.env,
      // Keep prompt libraries quiet and deterministic in CI logs.
      CI: '1',
      NO_COLOR: '1',
    },
  });

  await assertFile(path.join(projectDir, 'Cargo.toml'));
  await assertFile(path.join(projectDir, 'config.example.yml'));
  await assertFile(path.join(projectDir, 'compose.yaml'));
  await assertFile(path.join(projectDir, 'tooling', 'xtask', 'Cargo.toml'));

  const cargoToml = await readFile(path.join(projectDir, 'Cargo.toml'), 'utf8');
  assertIncludes(cargoToml, 'name = "my-api-smoke"', 'Cargo.toml should use the generated package name');

  const config = await readFile(path.join(projectDir, 'config.example.yml'), 'utf8');
  assertIncludes(config, 'name: my-api-smoke', 'config should use the generated service name');
  assertIncludes(config, 'service_name: my-api-smoke', 'telemetry service name should be replaced');
  assertIncludes(config, 'info,my_api_smoke=trace', 'tracing target should be snake_case');
  assertDoesNotInclude(
    config,
    'development-auth-secret-change-me-please',
    'auth secret should be randomized',
  );

  console.log(`Smoke test passed for ${projectName}`);
  if (stdout.trim()) console.log(stdout.trim());
  if (stderr.trim()) console.error(stderr.trim());
} finally {
  await rm(tempRoot, { recursive: true, force: true });
}

async function assertFile(filePath) {
  const info = await stat(filePath);
  if (!info.isFile()) {
    throw new Error(`Expected file: ${filePath}`);
  }
}

function assertIncludes(value, expected, message) {
  if (!value.includes(expected)) {
    throw new Error(`${message}. Missing: ${expected}`);
  }
}

function assertDoesNotInclude(value, unexpected, message) {
  if (value.includes(unexpected)) {
    throw new Error(`${message}. Found: ${unexpected}`);
  }
}
