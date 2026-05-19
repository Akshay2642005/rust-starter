import Link from 'next/link';
import { ThemeSwitch } from '@/layouts/shared/slots/theme-switch';
import { gitConfig } from '@/lib/shared';

export default function HomePage() {
    const githubUrl = `https://github.com/${gitConfig.user}/${gitConfig.repo}`;



    return (

        /* FIXED BACKGROUND OPACITIES:
          - Light mode changed from 0.06 (6%) to rgba(0,0,0,0.15) (15%) for clear visibility.
          - Dark mode changed from 0.06 (6%) to rgba(255,255,255,0.18) (18%) to make it crisp on deep backgrounds.
        */
        <div className="w-full min-h-screen bg-fd-background text-fd-foreground bg-[radial-gradient(rgba(0,0,0,0.18)_1.2px,transparent_1.2px)] dark:bg-[radial-gradient(rgba(255,255,255,0.12)_1.2px,transparent_1.2px)] [background-size:24px_24px] pb-12">
            {/* Absolute positioning wrapper for theme switch button */}
            <div className="fixed right-6 top-6 z-50">
                <ThemeSwitch />
            </div>

            {/* --- HERO SECTION --- */}
            <section className="w-full pt-32 pb-20 px-6 text-center">
                <div className="max-w-5xl mx-auto">
                    {/* Refined version badge matched with reference layout design */}
                    <span className="inline-flex items-center gap-2 px-2.5 py-1 text-[11px] font-bold uppercase tracking-wider text-orange-600 bg-orange-50/60 dark:bg-orange-950/30 border border-orange-100 dark:border-orange-900/60 rounded mb-8">
                        v0.1.0
                    </span>

                    <h1 className="text-4xl md:text-6xl font-extrabold tracking-tight mb-6 leading-[1.15]">
                        Production-ready Rust Backend Starter
                    </h1>

                    <p className="text-lg md:text-xl text-fd-muted-foreground max-w-2xl mx-auto mb-10 leading-relaxed">
                        Create a production-ready Axum backend in seconds with the published v0.1.0 npx CLI.
                    </p>

                    <div className="mx-auto mb-8 flex max-w-fit items-center rounded-lg border border-fd-border bg-fd-card px-4 py-3 font-mono text-sm text-fd-foreground shadow-sm">
                        npx @akshay2642005/rust-starter@latest my-api
                    </div>

                    <div className="flex justify-center gap-4">
                        <Link href="/docs" className="bg-orange-600 text-white px-6 py-2.5 rounded-lg font-semibold text-sm shadow-sm hover:bg-orange-700 transition">
                            Get Started
                        </Link>
                        <Link href="/docs" className="border border-fd-border bg-fd-card text-fd-foreground px-6 py-2.5 rounded-lg font-semibold text-sm shadow-sm hover:bg-fd-accent transition">
                            View Docs
                        </Link>
                        <Link href={githubUrl}
                            target="_blank"
                            rel="noreferrer"
                            className="border border-fd-border bg-fd-card text-fd-foreground px-6 py-2.5 rounded-lg font-semibold text-sm shadow-sm hover:bg-fd-accent transition">
                            Github
                        </Link>
                    </div>
                </div>
            </section>


            {/* --- 3-COLUMN BRIEF FEATURES --- */}
            <section className="w-full px-6 py-12">
                <div className="max-w-6xl mx-auto grid md:grid-cols-3 gap-8 text-left">
                    <div className="p-4 transition-colors duration-200 rounded-xl hover:bg-fd-accent/20">
                        <div className="text-orange-600 text-xl mb-3">⚙️</div>
                        <h3 className="text-lg font-semibold mb-2">Configuration + Telemetry</h3>
                        <p className="text-sm text-fd-muted-foreground">File + env configuration with hot reload, structured tracing, and optional OTLP export.</p>
                    </div>
                    <div className="p-4 transition-colors duration-200 rounded-xl hover:bg-fd-accent/20">
                        <div className="text-orange-600 text-xl mb-3">🔒</div>
                        <h3 className="text-lg font-semibold mb-2">Auth + Security</h3>
                        <p className="text-sm text-fd-muted-foreground">Better Auth email/password flows, Argon2 hashing, sessions, rate limits, and secure headers.</p>
                    </div>
                    <div className="p-4 transition-colors duration-200 rounded-xl hover:bg-fd-accent/20">
                        <div className="text-orange-600 text-xl mb-3">🗄️</div>
                        <h3 className="text-lg font-semibold mb-2">Database + APIs</h3>
                        <p className="text-sm text-fd-muted-foreground">SeaORM on PostgreSQL, a typed todo example, Utoipa OpenAPI + Scalar UI, plus Prometheus metrics.</p>
                    </div>
                </div>
            </section>

            {/* --- DETAILED FEATURES GRID --- */}
            <section className="max-w-6xl mx-auto px-6 py-24 text-center">
                <h2 className="text-3xl font-bold mb-4">Powerful primitives, seamlessly integrated</h2>
                <p className="text-fd-muted-foreground max-w-2xl mx-auto mb-16">
                    Built on industry standards for high-performance systems. Enterprise-grade reliability and security come standard.
                </p>

                <div className="grid md:grid-cols-3 gap-12 text-left">
                    {/* Auth & Security */}
                    <div>
                        <h4 className="text-xs font-bold uppercase tracking-wider text-fd-muted-foreground mb-6">Auth & Security</h4>
                        <div className="space-y-6">
                            <FeatureItem title="Better Auth" desc="Production-ready authentication framework" />
                            <FeatureItem title="Argon2" desc="State-of-the-art password hashing" />
                            <FeatureItem title="Rate Limits" desc="Prevent abuse with flexible throttling" />
                        </div>
                    </div>
                    {/* Database & APIs */}
                    <div>
                        <h4 className="text-xs font-bold uppercase tracking-wider text-fd-muted-foreground mb-6">Database & APIs</h4>
                        <div className="space-y-6">
                            <FeatureItem title="SeaORM" desc="Async ORM for Rust with full type safety" />
                            <FeatureItem title="OpenAPI + Scalar" desc="Auto-generated docs and beautiful UI" />
                            <FeatureItem title="PostgreSQL" desc="Optimized connection pooling built-in" />
                        </div>
                    </div>
                    {/* Ops & Observability */}
                    <div>
                        <h4 className="text-xs font-bold uppercase tracking-wider text-fd-muted-foreground mb-6">Ops & Observability</h4>
                        <div className="space-y-6">
                            <FeatureItem title="Hot Reload" desc="Runtime configuration updates" />
                            <FeatureItem title="OTLP Export" desc="Export traces to Honeycomb or Jaeger" />
                            <FeatureItem title="Prometheus" desc="Native metrics for system health" />
                        </div>
                    </div>
                </div>
            </section>
        </div>
    );
}

function FeatureItem({ title, desc }: { title: string; desc: string }) {
    return (
        <div className="flex gap-3 items-start">
            <div className="text-orange-600 mt-1 text-[10px]">◆</div>
            <div>
                <h5 className="font-semibold text-fd-foreground text-sm mb-0.5">{title}</h5>
                <p className="text-xs text-fd-muted-foreground leading-relaxed">{desc}</p>
            </div>
        </div>
    );
}
