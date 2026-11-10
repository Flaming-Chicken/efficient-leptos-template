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
use sha2::{Digest, Sha256};
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

    if let Err(e) = finalise_site(Path::new(&site_root), &pages) {
        eprintln!("error: could not finalise the site: {e}");
        std::process::exit(1);
    }

    println!("Pre-rendered {} pages into {site_root}", pages.len());
}

/// Fills in the build-specific parts of `_headers` and `sw.js`.
fn finalise_site(site_root: &Path, pages: &[PathBuf]) -> std::io::Result<()> {
    // Content Security Policy: allow exactly the inline scripts the pages contain.
    let mut hashes = std::collections::BTreeSet::new();
    for page in pages {
        let html = std::fs::read_to_string(page)?;
        hashes.extend(inline_script_hashes(&html));
    }
    let headers = site_root.join("_headers");
    if headers.exists() {
        let text = std::fs::read_to_string(&headers)?;
        if text.contains(CSP_PLACEHOLDER) {
            let sources: Vec<String> = hashes.iter().map(|h| format!("'sha256-{h}'")).collect();
            std::fs::write(&headers, text.replace(CSP_PLACEHOLDER, &sources.join(" ")))?;
        }
    }

    // Service worker: precache every file, under a cache name derived from their content.
    let worker = site_root.join("sw.js");
    if worker.exists() {
        let mut urls = Vec::new();
        let mut digest = Sha256::new();
        for file in site_files(site_root) {
            let Some(url) = precache_url(site_root, &file) else {
                continue;
            };
            digest.update(url.as_bytes());
            digest.update(std::fs::read(&file)?);
            urls.push(url);
        }
        let build_id: String = format!("{:x}", digest.finalize())
            .chars()
            .take(12)
            .collect();
        let list = urls
            .iter()
            .map(|u| format!("'{u}'"))
            .collect::<Vec<_>>()
            .join(", ");
        let text = std::fs::read_to_string(&worker)?
            .replace("'__BUILD_ID__'", &format!("'{build_id}'"))
            .replace("/*__PRECACHE_URLS__*/ []", &format!("[{list}]"));
        std::fs::write(&worker, text)?;
    }
    Ok(())
}

const CSP_PLACEHOLDER: &str = "__INLINE_SCRIPT_HASHES__";

/// Base64 SHA-256 of each inline `<script>` body, as a CSP `'sha256-…'` source needs.
fn inline_script_hashes(html: &str) -> Vec<String> {
    use base64::Engine as _;
    let mut hashes = Vec::new();
    let mut rest = html;
    while let Some(open) = rest.find("<script") {
        let after_open = &rest[open..];
        let Some(tag_end) = after_open.find('>') else {
            break;
        };
        let tag = &after_open[..tag_end];
        let body_and_rest = &after_open[tag_end + 1..];
        let Some(close) = body_and_rest.find("</script>") else {
            break;
        };
        let body = &body_and_rest[..close];
        if !tag.contains(" src=") && !body.is_empty() {
            let digest = Sha256::digest(body.as_bytes());
            hashes.push(base64::engine::general_purpose::STANDARD.encode(digest));
        }
        rest = &body_and_rest[close..];
    }
    hashes
}

/// Every file under `dir`, sorted, so output is the same on every machine.
fn site_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(site_files(&path));
        } else {
            found.push(path);
        }
    }
    found.sort();
    found
}

/// The URL the service worker should precache for `file`, or `None` to skip it.
fn precache_url(site_root: &Path, file: &Path) -> Option<String> {
    let relative = file.strip_prefix(site_root).ok()?;
    let path = relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    // Not cacheable or not needed at runtime: the worker itself, host configuration,
    // TypeScript declarations, licence texts.
    if path == "sw.js" || path == "_headers" || path.ends_with(".ts") || path.ends_with(".txt") {
        return None;
    }
    Some(match path.as_str() {
        "index.html" => "/".to_string(),
        // Static hosts serve `about.html` at `/about`; 404.html is fetched by its own name.
        "404.html" => "/404.html".to_string(),
        other => match other.strip_suffix(".html") {
            Some(page) => format!("/{}", page.trim_end_matches("/index")),
            None => format!("/{other}"),
        },
    })
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
    fn hashes_only_inline_scripts() {
        let html = r#"<script src="/pwa.js" defer></script><script>alert(1)</script><script type="module">x()</script>"#;
        let hashes = super::inline_script_hashes(html);
        assert_eq!(hashes.len(), 2);
        // Known value: `echo -n 'alert(1)' | openssl dgst -sha256 -binary | base64`
        assert_eq!(hashes[0], "bhHHL3z2vDgxUt0W3dWQOrprscmda2Y5pLsLg4GF+pI=");
    }

    #[test]
    fn precache_urls_match_how_static_hosts_serve_files() {
        use std::path::Path;
        let root = Path::new("site");
        let url = |p: &str| super::precache_url(root, &root.join(p));
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
