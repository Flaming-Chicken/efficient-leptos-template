#![recursion_limit = "512"]
//! Pre-renders the web release: writes one HTML file per route into the site root.
//!
//! Run after `cargo leptos build --release`, from the workspace root:
//!
//! ```text
//! target/release/ssg [SITE_ROOT]
//! ```
//!
//! `SITE_ROOT` defaults to `target/site`, where cargo-leptos puts the hydration bundle,
//! stylesheet and static assets. The binary must stay next to the `hash.txt` that
//! cargo-leptos writes, because the pages reference the content-hashed bundle names.

use leptos::prelude::LeptosOptions;
use spodeian_web_utils::pwa::{finalise_static_site, html_files, strip_nonce};
#[cfg(test)]
use spodeian_web_utils::pwa::{inline_script_hashes, precache_url, remove_nonce};
use std::path::Path;

/// Must match `output-name` and `site-pkg-dir` in `[[workspace.metadata.leptos]]`.
const OUTPUT_NAME: &str = "web";
const SITE_PKG_DIR: &str = "pkg";

#[tokio::main]
async fn main() {
    let site_root = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/site".to_string());

    let options = LeptosOptions::builder()
        .output_name(OUTPUT_NAME)
        .site_root(site_root.as_str())
        .site_pkg_dir(SITE_PKG_DIR)
        .hash_files(true)
        .build();

    let shell_options = options.clone();
    let (_routes, generator) =
        leptos_axum::generate_route_list_with_ssg(move || app::shell(shell_options.clone()));
    generator.generate(&options).await;

    let pages = html_files(Path::new(&site_root));
    if pages.is_empty() {
        eprintln!("error: no pages were generated in {site_root}");
        std::process::exit(1);
    }
    for page in &pages {
        if let Err(e) = strip_nonce(page) {
            eprintln!("error: could not finalise {}: {e}", page.display());
            std::process::exit(1);
        }
    }

    if let Err(e) = finalise_static_site(Path::new(&site_root), &pages) {
        eprintln!("error: could not finalise the site: {e}");
        std::process::exit(1);
    }

    println!("Pre-rendered {} pages into {site_root}", pages.len());
}

#[cfg(test)]
mod tests {
    use super::{inline_script_hashes, precache_url, remove_nonce};

    #[test]
    fn removes_every_nonce_and_fixes_crossorigin() {
        let html = r#"<link rel="modulepreload" href="/pkg/a.js" crossorigin="AbC123"><script type="module" nonce="AbC123">x()</script><script nonce="AbC123">y()</script>"#;
        assert_eq!(
            remove_nonce(html),
            r#"<link rel="modulepreload" href="/pkg/a.js" crossorigin="anonymous"><script type="module">x()</script><script>y()</script>"#
        );
    }

    #[test]
    fn hashes_only_inline_scripts() {
        let html = r#"<script src="/pwa.js" defer></script><script>alert(1)</script><script type="module">x()</script>"#;
        let hashes = inline_script_hashes(html);
        assert_eq!(hashes.len(), 2);
        // Known value: `echo -n 'alert(1)' | openssl dgst -sha256 -binary | base64`
        assert_eq!(hashes[0], "bhHHL3z2vDgxUt0W3dWQOrprscmda2Y5pLsLg4GF+pI=");
    }

    #[test]
    fn precache_urls_match_how_static_hosts_serve_files() {
        use std::path::Path;
        let root = Path::new("site");
        let url = |p: &str| precache_url(root, &root.join(p));
        assert_eq!(url("index.html").as_deref(), Some("/"));
        assert_eq!(url("about.html").as_deref(), Some("/about"));
        assert_eq!(url("404.html").as_deref(), Some("/404.html"));
        assert_eq!(
            url("pkg/web.abc.wasm").as_deref(),
            Some("/pkg/web.abc.wasm")
        );
        assert_eq!(url("sw.js"), None);
        assert_eq!(url("_headers"), None);
        assert_eq!(url("pkg/web.d.abc.ts"), None);
        assert_eq!(url("fonts/OFL.txt"), None);
    }

    #[test]
    fn leaves_pages_without_a_nonce_unchanged() {
        let html = "<html><body>plain</body></html>";
        assert_eq!(remove_nonce(html), html);
    }
}
