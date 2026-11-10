//! The HTML page shell for the pre-rendered web release (`ssr` builds only).
//!
//! The desktop release uses crates/web/index.html (Trunk) instead. Keep the head of both
//! in step: icons, manifest, scripts and the theme pre-paint script.

use crate::App;
use leptos::prelude::*;
use leptos_meta::{HashedStylesheet, MetaTags};

/// Applies the saved theme before first paint, so the page never flashes the wrong theme.
/// Inline on purpose; the CSP allows it by hash. Identical to the copy in index.html.
pub const THEME_PREPAINT_SCRIPT: &str = r#"(function () {
            try {
                var t = localStorage.getItem('serverless_leptos_theme');
                if (t === 'light' || t === 'high-contrast-dark' || t === 'high-contrast-light') {
                    document.documentElement.setAttribute('data-theme', t);
                } else {
                    document.documentElement.setAttribute('data-theme', 'dark');
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
                <meta name="theme-color" content="#0d1117" />
                <script inner_html=THEME_PREPAINT_SCRIPT></script>
                <link rel="manifest" href="/manifest.json" />
                <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
                <link rel="alternate icon" href="/favicon.ico" />
                <script src="/pwa.js" defer></script>
                <script src="/storage-bridge.js" defer></script>
                <HashedStylesheet options=options.clone() id="leptos" />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}
