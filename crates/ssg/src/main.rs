#![recursion_limit = "512"]
//! Pre-renders the web release: writes one HTML file per route, generates `sitemap.xml` and
//! `robots.txt`, stamps service worker precache URLs, and computes strict CSP inline script hashes.
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
use shared::AppPage;
use spodeian_web_utils::pwa::{finalise_static_site, html_files, strip_nonce};
#[cfg(test)]
use spodeian_web_utils::pwa::{inline_script_hashes, precache_url, remove_nonce};
use std::path::Path;

/// Must match `output-name` and `site-pkg-dir` in `[[workspace.metadata.leptos]]`.
const OUTPUT_NAME: &str = "web";
const SITE_PKG_DIR: &str = "pkg";
const DEFAULT_BASE_URL: &str = "https://example.spodeian.trade";

/// Builds a deterministic XML sitemap from the application's static route list.
pub fn generate_sitemap_xml(base_url: &str, pages: &[AppPage]) -> String {
    let base = base_url.trim_end_matches('/');
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for &page in pages {
        let priority = if page == AppPage::Dashboard {
            "1.0"
        } else {
            "0.8"
        };
        xml.push_str(&format!(
            "  <url><loc>{base}{}</loc><priority>{priority}</priority></url>\n",
            page.path()
        ));
    }
    xml.push_str("</urlset>\n");
    xml
}

/// Builds a `robots.txt` file pointing crawlers to the generated `sitemap.xml`.
pub fn generate_robots_txt(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    format!("User-agent: *\nAllow: /\nSitemap: {base}/sitemap.xml\n")
}

#[tokio::main]
async fn main() {
    let site_root = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/site".to_string());
    let base_url = std::env::var("SITE_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

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

    let site_dir = Path::new(&site_root);
    if let Err(e) = std::fs::write(
        site_dir.join("sitemap.xml"),
        generate_sitemap_xml(&base_url, &AppPage::ALL),
    ) {
        eprintln!("error: could not write sitemap.xml: {e}");
        std::process::exit(1);
    }
    if let Err(e) = std::fs::write(site_dir.join("robots.txt"), generate_robots_txt(&base_url)) {
        eprintln!("error: could not write robots.txt: {e}");
        std::process::exit(1);
    }

    if let Err(e) = finalise_static_site(site_dir, &pages) {
        eprintln!("error: could not finalise the site: {e}");
        std::process::exit(1);
    }

    println!(
        "Pre-rendered {} pages (+ sitemap.xml & robots.txt) into {site_root}",
        pages.len()
    );
}

#[cfg(test)]
mod tests {
    use super::{
        generate_robots_txt, generate_sitemap_xml, inline_script_hashes, precache_url, remove_nonce,
    };
    use shared::AppPage;

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
        assert_eq!(url("analytics.html").as_deref(), Some("/analytics"));
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

    #[test]
    fn generates_sitemap_and_robots_for_all_routes() {
        let xml = generate_sitemap_xml("https://app.spodeian.trade/", &AppPage::ALL);
        assert!(xml.contains("<loc>https://app.spodeian.trade/</loc>"));
        assert!(xml.contains("<loc>https://app.spodeian.trade/analytics</loc>"));
        assert!(xml.contains("<loc>https://app.spodeian.trade/about</loc>"));

        let robots = generate_robots_txt("https://app.spodeian.trade/");
        assert!(robots.contains("Allow: /"));
        assert!(robots.contains("Sitemap: https://app.spodeian.trade/sitemap.xml"));
    }
}
