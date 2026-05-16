# Contributing to Rust Starter

Thanks for your interest in contributing! This document outlines the workflow and expectations for
submitting changes.

## Quick start

1. **Fork and clone** the repository.
2. **Copy configuration**:
   - Duplicate `config.example.yml` to `config.yml` and update values as needed.
3. **Run the API**:
   - `cargo run`
4. **Run tests**:
   - `cargo test`

## Development workflow

- **Formatting**: `cargo fmt`
- **Linting**: `cargo clippy`
- **Tests**: `cargo test`

Please keep commits focused and follow the existing project structure.

## Docs development

The docs site lives in `docs/`.

- Install dependencies: `npm install` (or `pnpm install` / `yarn`)
- Run the dev server: `npm run dev`

## Pull request checklist

- [ ] Clear description of what changed and why
- [ ] Relevant tests added or updated
- [ ] Formatting and linting passed
- [ ] Docs updated if behavior or API changed

## Reporting issues

If you find a bug, include:
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version)

## Security

For security concerns, see `SECURITY.md`.
