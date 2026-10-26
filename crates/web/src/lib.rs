#![recursion_limit = "256"]
//! WebAssembly client entry point for the Leptos template.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() {
    spodeian_telemetry::init_default();

    #[cfg(feature = "hydrate")]
    {
        let has_ssr_content = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|doc| doc.query_selector(".app-root").ok().flatten())
            .is_some();

        if has_ssr_content {
            tracing::info!("Hydrating Serverless Leptos application from server-rendered HTML...");
            leptos::mount::hydrate_body(app::App);
        } else {
            tracing::info!(
                "Static HTML shell detected; mounting Serverless Leptos application via mount_to_body..."
            );
            leptos::mount::mount_to_body(app::App);
        }
    }

    #[cfg(not(feature = "hydrate"))]
    {
        tracing::info!("Mounting Serverless Leptos application in CSR fallback mode...");
        leptos::mount::mount_to_body(app::App);
    }
}
