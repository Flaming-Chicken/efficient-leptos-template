//! Whether the app is running interactively in a browser.
//!
//! The web release is pre-rendered to HTML on a build machine, then hydrated in the
//! browser. Hydration only works if the browser's first render matches the pre-rendered
//! HTML exactly, so anything that reads browser state (storage, persisted settings,
//! diagnostics) must wait until [`client_ready`] is true. It becomes true right after the
//! first render, in every rendering mode, and never during server rendering.

use leptos::prelude::*;

/// Context flag set once the app is running interactively in a browser.
#[derive(Debug, Clone, Copy)]
pub struct ClientReady(pub ReadSignal<bool>);

/// Reactive: true once the app is running in a browser after its first render.
pub fn client_ready() -> bool {
    use_context::<ClientReady>()
        .map(|ready| ready.0.get())
        .unwrap_or(false)
}
