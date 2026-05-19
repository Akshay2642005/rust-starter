# create-rust-starter

Create a production-ready Axum backend from the Rust Starter template.

```bash
npx create-rust-starter@latest my-api
cd my-api
cp config.example.yml config.yml
docker compose up -d
cargo xtask migrate up
cargo run
```

The generated service includes Axum, Better Auth, SeaORM, Postgres, OpenAPI,
telemetry, Docker, and migration tooling.

## Requirements

- Node.js 18 or newer
- A Rust toolchain that supports edition 2024
- PostgreSQL, or Docker + Docker Compose for local development

## Usage

```bash
npx create-rust-starter@latest [project-name]
```

If `project-name` is omitted, the CLI prompts for one.

## Generated project

The starter gives you:

- Axum 0.8 routing and middleware
- Better Auth email/password flows and sessions
- SeaORM entities and repository helpers for Postgres
- Utoipa OpenAPI with Scalar UI in development
- Prometheus metrics, health probes, structured tracing, and optional OTLP
- Docker Compose for Postgres and Jaeger
- `cargo xtask` commands for migrations and entity generation

## Local package checks

From this package directory:

```bash
npm run typecheck
npm run smoke
```
