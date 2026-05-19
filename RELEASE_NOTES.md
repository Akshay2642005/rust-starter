# Release Notes

## v0.1.0

Rust Starter `v0.1.0` is the first npm CLI release for generating a
production-ready Rust backend service from the default template.

### Install

```bash
npx @akshay2642005/rust-starter@latest my-api
```

### npm package

- `@akshay2642005/rust-starter@0.1.0`

### Included stack

- Axum
- Better Auth
- SeaORM
- Postgres
- OpenAPI
- Telemetry
- Docker
- Migrations

### Quick start

```bash
npx @akshay2642005/rust-starter@latest my-api
cd my-api
cp config.example.yml config.yml
docker compose up -d
cargo xtask migrate up
cargo run
```

The generated service starts at `http://localhost:8080` by default.

## v0.1.1 Plan

- Make `--no-install` either perform a real install flow or hide it until the feature is ready.
- Add more CLI smoke tests around invalid names, existing target directories, and published package contents.
- Fix the docs lint/Biome setup so `npm run lint` can run cleanly in CI.
- Add a `--template` option later if Rust Starter grows multiple starter templates.
