//! Main Leptos UI application view controller, layout components, and state synchronization.
#![allow(clippy::unit_arg)]
#![allow(clippy::unused_unit)]

pub mod components;
pub mod storage;

use crate::components::theme::{
    apply_document_dyslexia, apply_document_theme, get_initial_dyslexia, get_initial_theme,
};
use crate::components::{
    ExportModal, HelpModal, ImportModal, ItemList, Navbar, ResetModal, StorageModal,
    TelemetryModal,
};
use crate::storage::{
    load_state_from_storage, query_storage_diagnostics, request_persistent_storage,
    trigger_binary_download,
};
use leptos::prelude::*;
use shared::{AppState, export_to_compressed_bson};
use tracing::info;

#[component]
pub fn App() -> impl IntoView {
    // Initialize Theme & Typography
    let theme = RwSignal::new(get_initial_theme());
    apply_document_theme(theme.get());

    let dyslexia = RwSignal::new(get_initial_dyslexia());
    apply_document_dyslexia(dyslexia.get());

    // Initialize State from local storage or sample data
    let initial_state = load_state_from_storage().unwrap_or_else(|| {
        info!("Initializing new default AppState");
        AppState::new()
    });
    let state = RwSignal::new(initial_state);

    // Modal state signals
    let show_reset_modal = RwSignal::new(false);
    let show_help_modal = RwSignal::new(false);
    let show_import_modal = RwSignal::new(false);
    let show_export_modal = RwSignal::new(false);
    let show_storage_modal = RwSignal::new(false);
    let show_telemetry_modal = RwSignal::new(false);
    let mobile_menu_open = RwSignal::new(false);

    // Warning banner dismissal signals
    let dismissed_ephemeral = RwSignal::new(false);
    let dismissed_quota = RwSignal::new(false);
    let dismissed_combined = RwSignal::new(false);

    // Screen reader live announcements (WCAG 4.1.3)
    let announcement = RwSignal::new(String::new());

    let diag = Memo::new(move |_| query_storage_diagnostics());

    // Global keyboard listener for Escape key to close modals
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            show_reset_modal.set(false);
            show_help_modal.set(false);
            show_import_modal.set(false);
            show_export_modal.set(false);
            show_storage_modal.set(false);
            show_telemetry_modal.set(false);
            mobile_menu_open.set(false);
        }
    };

    let on_backup_bson = move |_| {
        let current_state = state.get();
        if let Ok(bytes) = export_to_compressed_bson(&current_state) {
            trigger_binary_download("data_backup.bson", &bytes, "application/octet-stream");
        }
    };

    view! {
        <div class="app-root" on:keydown=on_keydown tabindex="0">
            // Accessible Skip Link for Keyboard Navigation (WCAG 2.4.1)
            <a href="#main-content" class="skip-to-content">
                "Skip to main content"
            </a>

            // ARIA Live Region for Screen Reader Announcements (WCAG 4.1.3)
            <div role="status" aria-live="polite" aria-atomic="true" class="sr-only">
                {move || announcement.get()}
            </div>

            <Navbar
                theme=theme
                dyslexia=dyslexia
                show_reset_modal=show_reset_modal
                show_help_modal=show_help_modal
                show_import_modal=show_import_modal
                show_export_modal=show_export_modal
                show_storage_modal=show_storage_modal
                show_telemetry_modal=show_telemetry_modal
                mobile_menu_open=mobile_menu_open
                announcement=announcement
            />

            <main id="main-content" class="main-wrapper" tabindex="-1">
                // Diagnostics / Warning Banners
                {move || {
                    let d = diag.get();
                    let is_ephemeral = d.is_persisted == Some(false);
                    let is_quota = d.quota_exceeded;
                    let show_combined = is_ephemeral && is_quota && !dismissed_combined.get();
                    let show_ephemeral = is_ephemeral && !dismissed_ephemeral.get();
                    let show_quota = is_quota && !dismissed_quota.get();
                    if show_combined || show_ephemeral || show_quota {
                        let (alert_class, bg_style, border_style, text_msg) = if show_combined {
                            (
                                "banner-alert banner-danger",
                                "background: rgba(35, 20, 20, 0.95);",
                                "border: 1px solid var(--danger);",

                                view! {
                                    <span style="color: rgb(255, 120, 120);">
                                        <strong>"Storage Warning: "</strong>
                                        "Running in ephemeral storage and quota is full."
                                    </span>
                                }
                                    .into_any(),
                            )
                        } else if show_ephemeral {
                            (
                                "banner-alert banner-warning",
                                "background: rgba(35, 28, 15, 0.95);",
                                "border: 1px solid var(--warning);",
                                view! {
                                    <span style="color: rgb(255, 200, 80);">
                                        <strong>"Ephemeral Storage: "</strong>
                                        "Browser may clear local data under storage pressure."
                                    </span>
                                }
                                    .into_any(),
                            )
                        } else {
                            (
                                "banner-alert banner-warning",
                                "background: rgba(35, 28, 15, 0.95);",
                                "border: 1px solid var(--warning);",
                                view! {
                                    <span style="color: rgb(255, 200, 80);">
                                        <strong>"Quota Exceeded: "</strong>
                                        "Data migrated to IndexedDB fallback tier."
                                    </span>
                                }
                                    .into_any(),
                            )
                        };
                        let on_dismiss = move |_| {
                            if show_combined {
                                dismissed_combined.set(true);
                            } else if show_ephemeral {
                                dismissed_ephemeral.set(true);
                            } else {
                                dismissed_quota.set(true);
                            }
                        };

                        view! {
                            <div
                                class=alert_class
                                style=format!(
                                    "position: fixed; bottom: 20px; left: 50%; transform: translateX(-50%); z-index: 1000; padding: 12px 18px; {} {} border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.3); display: flex; flex-direction: column; align-items: center; gap: 8px; max-width: 90vw; width: max-content;",
                                    bg_style,
                                    border_style,
                                )
                            >
                                <div>{text_msg}</div>
                                <div style="display: flex; justify-content: center; align-items: center; gap: 8px; flex-wrap: wrap;">
                                    <button class="btn btn-sm btn-primary" on:click=on_backup_bson>
                                        "Save .bson Backup"
                                    </button>
                                    <button
                                        class="btn btn-sm btn-outline"
                                        on:click=move |_| request_persistent_storage()
                                    >
                                        "Request Persistence"
                                    </button>
                                    <button class="btn btn-sm btn-secondary" on:click=on_dismiss>
                                        "Dismiss"
                                    </button>
                                </div>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}

                <ItemList state=state announcement=announcement />
            </main>

            <footer class="app-footer">
                <div class="footer-container">
                    <p>
                        "Built with " <strong>"Rust & Leptos 0.8"</strong>
                        " • Serverless WASM (Cloudflare Pages) & Native Desktop (Tauri v2)"
                    </p>
                    <div class="footer-links">
                        <a
                            href="https://github.com/Spodeian/Revisited-IPIP-NEO"
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            "Inspired by Revisited IPIP-NEO"
                        </a>
                        <span class="footer-dot">"•"</span>
                        <a href="https://leptos.dev" target="_blank" rel="noopener noreferrer">
                            "Leptos Docs"
                        </a>
                    </div>
                </div>
            </footer>

            // Modals
            <ResetModal is_open=show_reset_modal state=state />
            <HelpModal is_open=show_help_modal />
            <ImportModal is_open=show_import_modal state=state />
            <ExportModal is_open=show_export_modal state=state />
            <StorageModal
                is_open=show_storage_modal
                show_import_modal=show_import_modal
                state=state
            />
            <TelemetryModal
                is_open=show_telemetry_modal
                announcement=announcement
            />
        </div>
    }
}
