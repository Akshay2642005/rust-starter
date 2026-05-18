#!/usr/bin/env node

import { run } from '../dist/cli.js';

run(process.argv).catch((error) => {
  console.error(`create-rust-starter: ${error.message}`);
  process.exitCode = 1;
});
