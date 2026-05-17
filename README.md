# Rust Starter

Rust Starter is becoming a generator for production-ready Rust backend services.

The generated template includes Axum, Better Auth, SeaORM + Postgres, OpenAPI docs,
configuration hot reload, metrics, tracing, Docker, and migration tooling.

## Repository layout

```text
.
├─ docs/                         # Documentation website
├─ packages/create-rust-starter/ # npm CLI generator
└─ templates/default/            # Rust backend app template
```

## CLI goal

The first generator flow is:

```bash
npx create-rust-starter my-api
cd my-api
cp config.example.yml config.yml
docker compose up -d
cargo xtask migrate up
cargo run
```

During local development of this repository, run the CLI directly:

```bash
node packages/create-rust-starter/bin/create-rust-starter.js my-api
```

