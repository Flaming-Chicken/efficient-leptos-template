//! The HTML page shell for the pre-rendered web release (`ssr` builds only).
//!
//! The desktop release uses crates/web/index.html (Trunk) instead. Keep the head of both
//! in step: icons, manifest, scripts and the theme + dyslexia pre-paint script.

use crate::App;
use leptos::prelude::*;
use leptos_meta::{HashedStylesheet, MetaTags};

/// Applies the saved theme and dyslexia reading attribute before first paint, so the page never
/// flashes the wrong theme or typography. Inline on purpose; the CSP allows it by hash.
/// Identical to the copy in `crates/web/index.html`.
pub const THEME_PREPAINT_SCRIPT: &str = r#"(function () {
            try {
                var t = localStorage.getItem('serverless_leptos_theme');
                if (t === 'light' || t === 'high-contrast-dark' || t === 'high-contrast-light') {
                    document.documentElement.setAttribute('data-theme', t);
                } else {
                    document.documentElement.setAttribute('data-theme', 'dark');
                }
                if (localStorage.getItem('serverless_leptos_dyslexia') === 'true') {
                    document.documentElement.setAttribute('data-dyslexia', 'true');
                }
            } catch (e) {}
        })();"#;

/// The full HTML document for one pre-rendered page.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" data-theme="dark">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <meta name="color-scheme" content="dark light" />
                <meta name="theme-color" content="#0d1117" />
                <script inner_html=THEME_PREPAINT_SCRIPT></script>
                <link rel="manifest" href="/manifest.json" />
                <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
                <link rel="alternate icon" href="/favicon.ico" />
                <script src="/pwa.js" defer></script>
                <HashedStylesheet options=options.clone() id="leptos" />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <noscript>
                    <div
                        class="noscript-notice"
                        role="note"
                        style="padding: 0.75rem 1.25rem; text-align: center; border-bottom: 1px solid var(--border-color); background: var(--bg-surface); color: var(--text-primary); font-size: 0.9rem;"
                    >
                        "JavaScript and WebAssembly are disabled. You are viewing the static pre-rendered content; interactive editing and browser persistence require JavaScript."
                    </div>
                </noscript>
                <App />
            </body>
        </html>
    }
}
