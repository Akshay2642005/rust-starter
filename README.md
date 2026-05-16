# Rust Starter

Production-ready Axum backend template with Better Auth, SeaORM + Postgres, OpenAPI docs, and
built-in observability.

## What’s included

- **Axum 0.8** HTTP server with modular routing and middleware
- **Better Auth** (email/password, sessions, account management)
- **SeaORM** store + repository helpers for Postgres
- **Typed todo API** example (CRUD + OpenAPI schemas)
- **OpenAPI + Scalar UI** in development
- **Metrics** (`/metrics`), **health probes**, and **structured tracing**
- **Security middleware** (CORS, CSP, HSTS, request IDs, timeouts, body limits)
- **Rate limiting** for auth and global traffic
- **Config hot-reload** from file + environment
- **Docs site** (Next.js + Fumadocs)

## Repository layout

```
.
├─ crates/
│  ├─ configuration/   # config loading, validation, watcher
│  ├─ seaorm/          # SeaORM store + repository helpers
│  ├─ telemetry/       # tracing + OTLP
│  └─ macros/          # proc macros for routes + instrumentation
├─ docs/               # docs site (Next.js + Fumadocs)
├─ migrations/         # SQL migrations
├─ src/
│  ├─ auth/            # Better Auth wiring + Argon2 hasher
│  ├─ handlers/        # HTTP handlers
│  ├─ middleware/      # CORS, security headers, timeouts, rate limits
│  ├─ services/        # domain services (todo, system)
│  ├─ app.rs           # router, middleware, server builder
│  ├─ main.rs          # config + telemetry bootstrap
│  └─ openapi.rs       # Utoipa OpenAPI + Scalar UI
└─ tooling/xtask/       # migrations + entity generation tasks
```

## Requirements

- A Rust toolchain that supports **edition 2024**
- PostgreSQL
- (Optional) Docker + Docker Compose for local infra
- (Optional) Node.js + npm/pnpm/yarn for docs

## Quick start (local)

1. **Configure the app**

   Copy the example config and edit as needed:

   ```bash
   cp config.example.yml config.yml
   ```

2. **Start Postgres**

   - Use your own Postgres, or run the provided Docker Compose:

   ```bash
   docker compose up -d
   ```

   The default config expects:

   ```
   postgres://postgres:postgres@localhost:5432/rust_starter
   ```

3. **Run migrations**

   ```bash
   cargo xtask migrate up
   ```

4. **Run the API**

   ```bash
   cargo run
   ```

The server defaults to `http://localhost:8080` and API routes are mounted under
`/api/v1` (from `server.path_prefix`).

## Configuration

Configuration loads from YAML/TOML and supports hot reload.

- Default config files: `config.yml` or `config.toml`
- Environment-specific files: `config.<env>.yml` / `config.<env>.toml`
- Local overrides: `config.local.yml` / `config.local.toml`

Environment variables can override any field using the `APP__` prefix and `__` separators.
Example:

```bash
APP__STORE__URL=postgres://postgres:postgres@localhost:5432/rust_starter
APP__AUTH__SECRET=replace-with-a-real-secret
```

Runtime flags:

- `APP_ENV` (default: `development`)
- `CONFIG_DIR` (default: `.`)

## API endpoints

### System

- `GET /health`
- `GET /healthz`
- `GET /livez`
- `GET /readyz`
- `GET /status`
- `GET /metrics`

### Todo API (protected)

Mounted under `server.path_prefix` (default: `/api/v1`).

- `POST /todos`
- `GET /todos`
- `GET /todos/{id}`
- `PUT /todos/{id}`
- `DELETE /todos/{id}`

### Auth API

Mounted under `server.path_prefix` + `auth.path_prefix` (default: `/api/v1/auth`).
Provided by Better Auth (email/password, sessions, account management).

## OpenAPI

In `development`, the server exposes Scalar UI at:

- `http://localhost:8080/docs`

The auth OpenAPI spec is merged into the docs at:

- `http://localhost:8080/docs/auth-openapi.json`

## Migrations

SQL migrations live in `migrations/` and are applied via the `xtask` tool.

### Common commands

```bash
# Create a new migration
cargo xtask migrate generate --name create_widgets

# Apply pending migrations
cargo xtask migrate up

# List migration files
cargo xtask migrate list

# Show applied vs pending
cargo xtask migrate status
```

Migrations are tracked in the `_schema_migrations` table.

## SeaORM entity generation

Generate SeaORM entities from the database schema:

```bash
cargo xtask entity generate --database-url postgres://...
```

Apply migrations and regenerate entities in one step:

```bash
cargo xtask migrate-generate --database-url postgres://...
```

Entities are written to `crates/seaorm/src/store/entities`.

## Observability

- **Tracing**: structured logs via `tracing`
- **OTLP**: optional export via `telemetry.otlp`
- **Prometheus**: metrics at `/metrics`
- **Request IDs**: `x-request-id` header

You can run Jaeger locally with Docker Compose (see `compose.yaml`).

## Docs site

The documentation website is in `docs/` (Next.js + Fumadocs).

```bash
cd docs
npm install
npm run dev
```

Docs entrypoint: `docs/content/docs/index.mdx`.

## Docker Compose

`compose.yaml` provides:

- `server` (API)
- `db` (Postgres)
- `jaeger` (OTLP + UI)

It expects:

- `password.txt` for the database password
- `config/config.yml` for the API configuration

## Development notes

- The API uses **route registration macros** (`#[route]`) with an inventory-based registry.
- Protected routes are wrapped with `require_auth` middleware.
- The route prefix is skipped for system health endpoints.

## Contributing

See `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, and `SECURITY.md`.
