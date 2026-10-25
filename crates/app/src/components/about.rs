//! About & Architecture view page component.

use crate::storage::StorageDiagnostics;
use leptos::prelude::*;
use shared::AppState;

#[component]
pub fn AboutPage(state: RwSignal<AppState>, diag: Memo<StorageDiagnostics>) -> impl IntoView {
    view! {
        <div
            class="about-page-container"
            style="max-width: 900px; margin: 0 auto; padding: 24px 16px;"
        >
            // Hero / Title Section
            <div class="about-hero" style="text-align: center; margin-bottom: 32px;">
                <h1 style="font-size: 2rem; font-weight: 700; margin-bottom: 8px;">
                    "Leptos Serverless & Desktop Template"
                </h1>
                <p style="color: var(--text-muted); font-size: 1.1rem; max-width: 650px; margin: 0 auto 12px;">
                    "A modular, high-performance foundation for reactive Rust WebAssembly and Native Desktop apps"
                </p>
                <span
                    class="badge"
                    style="background: var(--bg-surface-elevated); padding: 4px 12px; border-radius: 9999px; font-family: var(--font-mono); font-size: 0.85rem; border: 1px solid var(--border-color);"
                >
                    "Version 0.1.0 • Rust 2024 • Leptos 0.8 • Tauri v2"
                </span>
            </div>

            // Card 1: Four-Tier Architecture
            <div
                class="card"
                style="background: var(--bg-surface); border: 1px solid var(--border-color); border-radius: 12px; padding: 24px; margin-bottom: 24px; box-shadow: var(--shadow-sm);"
            >
                <h2 style="font-size: 1.3rem; font-weight: 600; margin-bottom: 12px; display: flex; align-items: center; gap: 8px;">
                    "🏛️ Four-Tier Architecture"
                </h2>
                <p style="color: var(--text-muted); margin-bottom: 16px;">
                    "The workspace is cleanly decoupled into four single-responsibility crates:"
                </p>

                <div class="arch-list" style="display: flex; flex-direction: column; gap: 12px;">
                    <div style="display: flex; gap: 16px; align-items: baseline; padding: 8px 12px; background: var(--bg-surface-elevated); border-radius: 8px;">
                        <code style="font-weight: 700; color: var(--accent); min-width: 140px;">
                            "crates/shared"
                        </code>
                        <span style="color: var(--text-primary);">
                            "Domain entities, AppState, AppConfig, and JSON/CSV/BSON serialization engines."
                        </span>
                    </div>
                    <div style="display: flex; gap: 16px; align-items: baseline; padding: 8px 12px; background: var(--bg-surface-elevated); border-radius: 8px;">
                        <code style="font-weight: 700; color: var(--accent); min-width: 140px;">
                            "crates/app"
                        </code>
                        <span style="color: var(--text-primary);">
                            "Universal Leptos reactive UI view-controllers, navbar, item lists, modals, and persistence manager."
                        </span>
                    </div>
                    <div style="display: flex; gap: 16px; align-items: baseline; padding: 8px 12px; background: var(--bg-surface-elevated); border-radius: 8px;">
                        <code style="font-weight: 700; color: var(--accent); min-width: 140px;">
                            "crates/desktop"
                        </code>
                        <span style="color: var(--text-primary);">
                            "Native desktop application bundle powered by Tauri v2 with strict Content Security Policy."
                        </span>
                    </div>
                    <div style="display: flex; gap: 16px; align-items: baseline; padding: 8px 12px; background: var(--bg-surface-elevated); border-radius: 8px;">
                        <code style="font-weight: 700; color: var(--accent); min-width: 140px;">
                            "crates/web"
                        </code>
                        <span style="color: var(--text-primary);">
                            "WebAssembly client entrypoint, Trunk asset pipeline, PWA Service Worker, and Cloudflare Pages headers."
                        </span>
                    </div>
                </div>
            </div>

            // Card 2: Live Diagnostics & Storage
            <div
                class="card"
                style="background: var(--bg-surface); border: 1px solid var(--border-color); border-radius: 12px; padding: 24px; margin-bottom: 24px; box-shadow: var(--shadow-sm);"
            >
                <h2 style="font-size: 1.3rem; font-weight: 600; margin-bottom: 12px; display: flex; align-items: center; gap: 8px;">
                    "📊 Live System Diagnostics & Storage"
                </h2>

                <div
                    class="diag-grid"
                    style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px;"
                >
                    <div style="padding: 12px; background: var(--bg-surface-elevated); border-radius: 8px;">
                        <div style="font-size: 0.8rem; color: var(--text-muted); text-transform: uppercase;">
                            "Storage Tier"
                        </div>
                        <div style="font-weight: 600; margin-top: 4px;">
                            {move || diag.get().backend.label()}
                        </div>
                    </div>
                    <div style="padding: 12px; background: var(--bg-surface-elevated); border-radius: 8px;">
                        <div style="font-size: 0.8rem; color: var(--text-muted); text-transform: uppercase;">
                            "Persistence Mode"
                        </div>
                        <div style="font-weight: 600; margin-top: 4px;">
                            {move || match diag.get().is_persisted {
                                Some(true) => "Persistent (Guaranteed)",
                                Some(false) => "Ephemeral (Browser Managed)",
                                None => "Native / Unknown",
                            }}
                        </div>
                    </div>
                    <div style="padding: 12px; background: var(--bg-surface-elevated); border-radius: 8px;">
                        <div style="font-size: 0.8rem; color: var(--text-muted); text-transform: uppercase;">
                            "Total Managed Tasks"
                        </div>
                        <div style="font-weight: 600; margin-top: 4px;">
                            {move || {
                                format!(
                                    "{} items ({} completed)",
                                    state.get().collection.total_count(),
                                    state.get().collection.completed_count(),
                                )
                            }}
                        </div>
                    </div>
                    <div style="padding: 12px; background: var(--bg-surface-elevated); border-radius: 8px;">
                        <div style="font-size: 0.8rem; color: var(--text-muted); text-transform: uppercase;">
                            "Active Theme"
                        </div>
                        <div style="font-weight: 600; margin-top: 4px;">
                            {move || {
                                format!(
                                    "{} {}",
                                    state.get().config.theme.icon(),
                                    state.get().config.theme.label(),
                                )
                            }}
                        </div>
                    </div>
                </div>
            </div>

            // Card 3: Shortcuts & Guidance
            <div
                class="card"
                style="background: var(--bg-surface); border: 1px solid var(--border-color); border-radius: 12px; padding: 24px; box-shadow: var(--shadow-sm);"
            >
                <h2 style="font-size: 1.3rem; font-weight: 600; margin-bottom: 12px; display: flex; align-items: center; gap: 8px;">
                    "⌨️ Keyboard Shortcuts & Features"
                </h2>
                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 12px;">
                    <div style="display: flex; gap: 12px; align-items: center;">
                        <kbd style="padding: 4px 8px; background: var(--bg-surface-elevated); border: 1px solid var(--border-color); border-radius: 4px; font-family: var(--font-mono); font-size: 0.85rem;">
                            "Enter"
                        </kbd>
                        <span>"Submit new item from the input form"</span>
                    </div>
                    <div style="display: flex; gap: 12px; align-items: center;">
                        <kbd style="padding: 4px 8px; background: var(--bg-surface-elevated); border: 1px solid var(--border-color); border-radius: 4px; font-family: var(--font-mono); font-size: 0.85rem;">
                            "Escape"
                        </kbd>
                        <span>"Close active modal dialogs"</span>
                    </div>
                    <div style="display: flex; gap: 12px; align-items: center;">
                        <span style="font-weight: 600;">"Data Export:"</span>
                        <span>"JSON, RFC 4180 CSV, Compressed BSON"</span>
                    </div>
                    <div style="display: flex; gap: 12px; align-items: center;">
                        <span style="font-weight: 600;">"Offline PWA:"</span>
                        <span>"Local icons, service worker cache-first"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
