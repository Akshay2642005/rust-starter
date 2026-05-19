#!/usr/bin/env node

import { run } from '../dist/cli.js';

run(process.argv).catch((error) => {
  console.error(`@akshay2642005/rust-starter: ${error.message}`);
  process.exitCode = 1;
});
