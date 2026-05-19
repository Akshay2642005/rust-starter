# Rust Starter Docs

This is the documentation website for Rust Starter, a production-ready Axum
backend starter published through the `create-rust-starter` npm CLI.

The current CLI release is `0.1.0`:

```bash
npx create-rust-starter@latest my-api
```

## Local development

Run the docs development server:

```bash
npm run dev
```

Open http://localhost:3000 with your browser to see the result.

## Content

Main docs content lives in `content/docs`:

- `content/docs/index.mdx`: getting started guide.
- `content/docs/reference/api-routes.mdx`: CLI and generated API reference.

Useful app files:

- `src/app/(home)/page.tsx`: landing page.
- `src/app/docs`: documentation layout and pages.
- `src/app/api/search/route.ts`: search route handler.
- `src/lib/source.ts`: Fumadocs content source adapter.
- `src/lib/layout.shared.tsx`: shared layout options.

## Checks

```bash
npm run types:check
npm run lint
```
