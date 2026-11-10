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
use std::path::{Path, PathBuf};

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

    println!("Pre-rendered {} pages into {site_root}", pages.len());
}

/// Every `.html` file under `dir`, sorted.
fn html_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(html_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "html") {
            found.push(path);
        }
    }
    found.sort();
    found
}

/// Removes the per-render nonce `leptos_axum` adds to script and preload tags.
///
/// A nonce only works when a server sends a matching `Content-Security-Policy` header
/// with each response. In a static file it does nothing, and its random value would make
/// two builds of the same commit differ. The CSP in `_headers` allows the inline scripts
/// by hash instead.
fn strip_nonce(page: &Path) -> std::io::Result<()> {
    let html = std::fs::read_to_string(page)?;
    std::fs::write(page, remove_nonce(&html))
}

fn remove_nonce(html: &str) -> String {
    const MARKER: &str = " nonce=\"";
    let Some(start) = html.find(MARKER) else {
        return html.to_string();
    };
    let value_start = start + MARKER.len();
    let Some(len) = html[value_start..].find('"') else {
        return html.to_string();
    };
    let nonce = &html[value_start..value_start + len];
    html.replace(&format!(" nonce=\"{nonce}\""), "")
        // Leptos also (incorrectly) uses the nonce as the preload `crossorigin` value.
        .replace(
            &format!(" crossorigin=\"{nonce}\""),
            " crossorigin=\"anonymous\"",
        )
}

#[cfg(test)]
mod tests {
    use super::remove_nonce;

    #[test]
    fn removes_every_nonce_and_fixes_crossorigin() {
        let html = r#"<link rel="modulepreload" href="/pkg/a.js" crossorigin="AbC123"><script type="module" nonce="AbC123">x()</script><script nonce="AbC123">y()</script>"#;
        assert_eq!(
            remove_nonce(html),
            r#"<link rel="modulepreload" href="/pkg/a.js" crossorigin="anonymous"><script type="module">x()</script><script>y()</script>"#
        );
    }

    #[test]
    fn leaves_pages_without_a_nonce_unchanged() {
        let html = "<html><body>plain</body></html>";
        assert_eq!(remove_nonce(html), html);
    }
}
