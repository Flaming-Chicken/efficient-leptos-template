#![recursion_limit = "256"]
#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]
//! Main Leptos UI application view controller, layout components, and state synchronization.
#![allow(clippy::unit_arg)]
#![allow(clippy::unused_unit)]

pub mod client;
pub mod components;
#[cfg(feature = "ssr")]
pub mod shell;
pub mod storage;
pub mod tauri_bridge;

#[cfg(feature = "ssr")]
pub use shell::shell;

use crate::components::theme::{
    apply_document_dyslexia, apply_document_theme, get_initial_dyslexia, get_initial_theme,
};
use crate::components::{
    AboutPage, AnalyticsPage, CommandPaletteModal, ExportModal, HelpModal, ImportModal, ItemList,
    Navbar, ResetModal, StorageModal, TelemetryModal,
};
use crate::storage::{
    query_storage_diagnostics, request_persistent_storage, save_state_to_storage,
    trigger_binary_download,
};
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title, provide_meta_context};
use leptos_router::components::{A, Route, Router, Routes};
use leptos_router::static_routes::StaticRoute;
use leptos_router::{SsrMode, path};
use shared::{AppPage, AppState, ItemCollection, ThemeMode, UndoStack, export_to_compressed_bson};
use tracing::info;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Sets the document title, description, canonical link, and OpenGraph tags for one page.
#[component]
fn PageMeta(page: AppPage) -> impl IntoView {
    let title = format!("{} | Leptos Serverless & Desktop Template", page.label());
    let desc = page.description();
    view! {
        <Title text=title.clone() />
        <Meta name="description" content=desc />
        <Meta property="og:type" content="website" />
        <Meta property="og:title" content=title />
        <Meta property="og:description" content=desc />
        <Link rel="canonical" href=page.path() />
    }
}

/// Shown for any URL that has no page.
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <Title text="Page not found | Leptos Serverless & Desktop Template" />
        <section class="not-found">
            <h1>"Page not found"</h1>
            <p>"There is no page at this address."</p>
            <A href=AppPage::Dashboard.path()>"Go to the dashboard"</A>
        </section>
    }
}

/// Recoverable alert surface displayed by Leptos [`ErrorBoundary`] and unit-tested in SSR mode.
#[component]
pub fn ErrorFallbackView(
    /// Formatted error summary and detail list.
    recovery: spodeian_leptos_ui::ErrorBoundaryRecovery,
    /// Optional reactive epoch incremented to remount the failed route subtree.
    #[prop(optional)]
    reset_epoch: Option<RwSignal<u64>>,
) -> impl IntoView {
    view! {
        <section
            class="card error-boundary-fallback"
            role="alert"
            aria-live="assertive"
            style="background: var(--bg-surface); border: 1px solid var(--danger); border-radius: 12px; padding: 24px; max-width: 680px; margin: 24px auto;"
        >
            <h2 style="margin-top: 0; color: var(--danger);">{recovery.title}</h2>
            <p style="color: var(--text-primary);">{recovery.summary}</p>
            {(!recovery.details.is_empty())
                .then(|| {
                    view! {
                        <ul
                            class="error-details"
                            style="color: var(--text-muted); font-family: var(--font-mono); font-size: 0.875rem;"
                        >
                            {recovery
                                .details
                                .into_iter()
                                .map(|detail| view! { <li>{detail}</li> })
                                .collect::<Vec<_>>()}
                        </ul>
                    }
                })}
            <div style="margin-top: 16px;">
                <button
                    class="btn btn-primary"
                    on:click=move |_| {
                        if let Some(epoch) = reset_epoch {
                            epoch.update(|v| *v = v.saturating_add(1));
                        }
                    }
                >
                    "Reset View"
                </button>
            </div>
        </section>
    }
}

fn target_is_editable_input(ev: &web_sys::KeyboardEvent) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(target) = ev.target()
            && let Some(el) = target.dyn_ref::<web_sys::Element>()
        {
            let tag = el.tag_name();
            return tag.eq_ignore_ascii_case("INPUT")
                || tag.eq_ignore_ascii_case("TEXTAREA")
                || tag.eq_ignore_ascii_case("SELECT");
        }
        false
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ev;
        false
    }
}

/// Root reactive Leptos application component shared across static SSR, hydration, and Tauri CSR.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Every build renders the same defaults first, so pre-rendered HTML and the browser's
    // first (hydrating) render match. Saved settings and data load right after, in the
    // browser only.
    let theme = RwSignal::new(ThemeMode::default());
    let dyslexia = RwSignal::new(false);
    let state = RwSignal::new(AppState::new());
    let undo_stack = RwSignal::new(UndoStack::<ItemCollection>::new(64));
    let (ready, set_ready) = signal(false);
    provide_context(client::ClientReady(ready));
    provide_context(state);
    provide_context(undo_stack);
    let storage_status = RwSignal::new(crate::storage::StorageStatus::default());

    // Effects never run during server rendering; in the browser this runs once after the
    // first render.
    Effect::new(move |_| {
        theme.set(get_initial_theme());
        apply_document_theme(theme.get_untracked());
        dyslexia.set(get_initial_dyslexia());
        apply_document_dyslexia(dyslexia.get_untracked());
        // Storage is asynchronous (IndexedDB). Saving is only possible once this load
        // has finished, so the sample data can never overwrite saved data.
        leptos::task::spawn_local(async move {
            match crate::storage::init(storage_status).await {
                Some(saved) => state.set(saved),
                None => info!("No saved state; keeping the sample data"),
            }
            set_ready.set(true);
        });
    });

    // Modal state signals
    let show_reset_modal = RwSignal::new(false);
    let show_help_modal = RwSignal::new(false);
    let show_import_modal = RwSignal::new(false);
    let show_export_modal = RwSignal::new(false);
    let show_storage_modal = RwSignal::new(false);
    let show_telemetry_modal = RwSignal::new(false);
    let show_command_palette = RwSignal::new(false);
    let mobile_menu_open = RwSignal::new(false);
    let route_reset_epoch = RwSignal::new(0u64);

    // Warning banner dismissal signals
    let dismissed_ephemeral = RwSignal::new(false);
    let dismissed_quota = RwSignal::new(false);
    let dismissed_combined = RwSignal::new(false);

    // Screen reader live announcements (WCAG 4.1.3)
    let announcement = RwSignal::new(String::new());

    let diag = Memo::new(move |_| {
        if !ready.get() {
            return crate::storage::StorageDiagnostics::default();
        }
        query_storage_diagnostics()
    });

    // Global keyboard listener for Escape, Ctrl/Cmd+K Command Palette, and Ctrl/Cmd+Z/Y Undo/Redo
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        let ctrl_or_meta = ev.ctrl_key() || ev.meta_key();

        if ctrl_or_meta && key.eq_ignore_ascii_case("k") {
            ev.prevent_default();
            show_command_palette.update(|open| *open = !*open);
            return;
        }

        if key == "Escape" {
            show_reset_modal.set(false);
            show_help_modal.set(false);
            show_import_modal.set(false);
            show_export_modal.set(false);
            show_storage_modal.set(false);
            show_telemetry_modal.set(false);
            show_command_palette.set(false);
            mobile_menu_open.set(false);
            return;
        }

        if target_is_editable_input(&ev) || show_command_palette.get_untracked() {
            return;
        }

        if ctrl_or_meta && !ev.shift_key() && key.eq_ignore_ascii_case("z") {
            ev.prevent_default();
            let current = state.get_untracked().collection;
            let mut restored = None;
            undo_stack.update(|s| restored = s.undo(current));
            if let Some(prev) = restored {
                state.update(|st| {
                    st.collection = prev;
                    let _ = save_state_to_storage(st);
                });
                announcement.set("Undid last collection change".to_string());
            }
        } else if (ctrl_or_meta && key.eq_ignore_ascii_case("y"))
            || (ctrl_or_meta && ev.shift_key() && key.eq_ignore_ascii_case("z"))
        {
            ev.prevent_default();
            let current = state.get_untracked().collection;
            let mut restored = None;
            undo_stack.update(|s| restored = s.redo(current));
            if let Some(next) = restored {
                state.update(|st| {
                    st.collection = next;
                    let _ = save_state_to_storage(st);
                });
                announcement.set("Redid collection change".to_string());
            }
        }
    };

    let on_backup_bson = move |_| {
        let current_state = state.get();
        if let Ok(bytes) = export_to_compressed_bson(&current_state) {
            trigger_binary_download("data_backup.bson", &bytes, "application/octet-stream");
        }
    };

    view! {
        <Router>
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
                    show_command_palette=show_command_palette
                    mobile_menu_open=mobile_menu_open
                    announcement=announcement
                />

                <main id="main-content" class="main-wrapper" tabindex="-1">
                    // A failed save or load is always shown: never imply data is safe when
                    // the browser refused the write.
                    {move || {
                        diag.get()
                            .last_error
                            .map(|message| {
                                view! {
                                    <div class="save-error" role="alert">
                                        <strong>"Not saved. "</strong>
                                        {message}
                                    </div>
                                }
                            })
                    }}
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
                                        <button
                                            class="btn btn-sm btn-primary"
                                            on:click=on_backup_bson
                                        >
                                            "Save .bson Backup"
                                        </button>
                                        <button
                                            class="btn btn-sm btn-outline"
                                            on:click=move |_| request_persistent_storage()
                                        >
                                            "Request Persistence"
                                        </button>
                                        <button
                                            class="btn btn-sm btn-secondary"
                                            on:click=on_dismiss
                                        >
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

                    <ErrorBoundary fallback=move |errors| {
                        let messages: Vec<String> = errors
                            .get()
                            .into_iter()
                            .map(|(_, err)| err.to_string())
                            .collect();
                        let recovery = spodeian_leptos_ui::ErrorBoundaryRecovery::from_errors(
                            messages,
                        );
                        view! {
                            <ErrorFallbackView recovery=recovery reset_epoch=route_reset_epoch />
                        }
                    }>
                        {move || {
                            let _epoch = route_reset_epoch.get();
                            view! {
                                <Routes fallback=NotFound>
                                    <Route
                                        path=path!("/")
                                        ssr=SsrMode::Static(StaticRoute::new())
                                        view=move || {
                                            view! {
                                                <PageMeta page=AppPage::Dashboard />
                                                <ItemList state=state announcement=announcement />
                                            }
                                        }
                                    />
                                    <Route
                                        path=path!("/analytics")
                                        ssr=SsrMode::Static(StaticRoute::new())
                                        view=move || {
                                            view! {
                                                <PageMeta page=AppPage::Analytics />
                                                <AnalyticsPage
                                                    state=state
                                                    show_export_modal=show_export_modal
                                                />
                                            }
                                        }
                                    />
                                    <Route
                                        path=path!("/about")
                                        ssr=SsrMode::Static(StaticRoute::new())
                                        view=move || {
                                            view! {
                                                <PageMeta page=AppPage::About />
                                                <AboutPage state=state diag=diag />
                                            }
                                        }
                                    />
                                    // Pre-rendered as 404.html, which static hosts serve for unknown paths.
                                    <Route
                                        path=path!("/404")
                                        ssr=SsrMode::Static(StaticRoute::new())
                                        view=NotFound
                                    />
                                </Routes>
                            }
                        }}
                    </ErrorBoundary>
                </main>

                <footer class="app-footer">
                    <div class="footer-container">
                        <p>
                            "Built with " <strong>"Rust & Leptos 0.8"</strong>
                            " • Serverless WASM (Cloudflare Pages) & Native Desktop (Tauri v2)"
                        </p>
                        <div class="footer-links">
                            <A href=AppPage::Dashboard.path() attr:class="footer-nav-link">
                                "Dashboard"
                            </A>
                            <span class="footer-dot">"•"</span>
                            <A href=AppPage::Analytics.path() attr:class="footer-nav-link">
                                "Analytics"
                            </A>
                            <span class="footer-dot">"•"</span>
                            <A href=AppPage::About.path() attr:class="footer-nav-link">
                                "About"
                            </A>
                            <span class="footer-dot">"•"</span>
                            <a href="https://leptos.dev" target="_blank" rel="noopener noreferrer">
                                "Leptos Docs"
                            </a>
                            <span class="footer-dot">"•"</span>
                            <a
                                href="https://v2.tauri.app"
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                "Tauri v2 Docs"
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
                <TelemetryModal is_open=show_telemetry_modal announcement=announcement />
                <CommandPaletteModal
                    is_open=show_command_palette
                    state=state
                    theme=theme
                    dyslexia=dyslexia
                    show_import_modal=show_import_modal
                    show_export_modal=show_export_modal
                    show_storage_modal=show_storage_modal
                    show_telemetry_modal=show_telemetry_modal
                    show_help_modal=show_help_modal
                    show_reset_modal=show_reset_modal
                />
            </div>
        </Router>
    }
}
