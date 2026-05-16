import Link from 'next/link';
import { ThemeSwitch } from '@/layouts/shared/slots/theme-switch';
import { gitConfig } from '@/lib/shared';

export default function HomePage() {
    const githubUrl = `https://github.com/${gitConfig.user}/${gitConfig.repo}`;

    return (
        <div className="flex flex-col justify-center text-center flex-1 px-4">
            <div className="fixed right-6 top-6 z-50">
                <ThemeSwitch />
            </div>
            <h1 className="text-4xl font-bold mb-4">
                Rust <span className="opacity-50">Starter</span>
            </h1>
            <p className="text-lg text-fd-muted-foreground mb-8 max-w-2xl mx-auto">
                Production-ready Axum backend starter with Better Auth, SeaORM + Postgres,
                OpenAPI docs, and built-in observability.
            </p>
            <div className="flex gap-4 justify-center mb-12">
                <Link
                    href="/docs"
                    className="px-6 py-3 rounded-lg bg-fd-primary text-fd-primary-foreground font-medium hover:opacity-90 transition-opacity"
                >
                    Get Started
                </Link>
                <Link
                    href="/docs/reference/api-routes"
                    className="px-6 py-3 rounded-lg border border-fd-border font-medium hover:bg-fd-accent transition-colors"
                >
                    API Reference
                </Link>
                <Link
                    href={githubUrl}
                    target="_blank"
                    rel="noreferrer"
                    className="px-6 py-3 rounded-lg border border-fd-border font-medium hover:bg-fd-accent transition-colors inline-flex items-center gap-2"
                >
                    <svg
                        role="img"
                        aria-label="GitHub"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                        className="h-4 w-4"
                    >
                        <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
                    </svg>
                    GitHub
                </Link>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-4xl mx-auto text-left">
                <div className="p-6 rounded-lg border border-fd-border">
                    <h3 className="font-semibold mb-2">Configuration + Telemetry</h3>
                    <p className="text-sm text-fd-muted-foreground">
                        File + env configuration with hot reload, structured tracing,
                        and optional OTLP export.
                    </p>
                </div>
                <div className="p-6 rounded-lg border border-fd-border">
                    <h3 className="font-semibold mb-2">Auth + Security</h3>
                    <p className="text-sm text-fd-muted-foreground">
                        Better Auth email/password flows, Argon2 hashing, sessions,
                        rate limits, and secure headers.
                    </p>
                </div>
                <div className="p-6 rounded-lg border border-fd-border">
                    <h3 className="font-semibold mb-2">Database + APIs</h3>
                    <p className="text-sm text-fd-muted-foreground">
                        SeaORM on PostgreSQL, a typed todo example, Utoipa OpenAPI + Scalar UI,
                        plus Prometheus metrics and health probes.
                    </p>
                </div>
            </div>
        </div>
    );
}
