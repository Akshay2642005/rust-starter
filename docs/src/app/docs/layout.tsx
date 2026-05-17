import { source } from '@/lib/source';
import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { baseOptions } from '@/lib/layout.shared';
import { ThemeSwitch } from '@/layouts/shared/slots/theme-switch';

// TODO: Flip to "false" (or set NEXT_PUBLIC_DOCS_COMING_SOON=false)
// to restore the vanilla Fumadocs layout when you're ready to publish docs.
const DOCS_COMING_SOON = process.env.NEXT_PUBLIC_DOCS_COMING_SOON === 'true';

export default function Layout({ children }: LayoutProps<'/docs'>) {
    if (DOCS_COMING_SOON) {
        return (
            <main className="min-h-screen w-full bg-black text-white bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-zinc-800/40 via-black to-black">
                <div className="fixed right-6 top-6 z-50">
                    <ThemeSwitch />
                </div>
                <div className="flex min-h-screen flex-col items-center justify-center text-center">
                    <div className="relative mb-7 size-32 md:size-36">
                        <div className="absolute inset-[-56px] rounded-full bg-white/10 blur-3xl" />
                        <img
                            src="/rust-starter-mark.svg"
                            alt="Rust Starter"
                            className="relative size-full"
                        />
                    </div>
                    <p className="text-sm uppercase tracking-[0.55em] text-white/90">Coming soon</p>
                </div>
            </main>
        );
    }

    return (
        <DocsLayout tree={source.getPageTree()} {...baseOptions()}>
            {children}
        </DocsLayout>
    );
}
