#![recursion_limit = "256"]
//! Browser entry points for the Leptos template.
//!
//! - `hydrate` (web release, built by cargo-leptos): the page arrives pre-rendered by the
//!   `ssg` binary; cargo-leptos's generated script calls [`hydrate`] to make it interactive.
//! - `csr` (desktop release, built by Trunk for Tauri): the page arrives empty and
//!   [`start`] renders the app into it.

use wasm_bindgen::prelude::*;

/// Makes a pre-rendered page interactive. Called by cargo-leptos's hydration script.
#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    spodeian_telemetry::init_default();
    tracing::info!("Hydrating pre-rendered page");
    leptos::mount::hydrate_body(app::App);
}

/// Renders the app into an empty page. Runs automatically when the module loads.
#[cfg(feature = "csr")]
#[wasm_bindgen(start)]
pub fn start() {
    spodeian_telemetry::init_default();
    tracing::info!("Rendering app in the browser (client-side rendering)");
    leptos::mount::mount_to_body(app::App);
}
