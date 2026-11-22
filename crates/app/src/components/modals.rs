//! Modal dialogs for State Reset, Help & Architecture, Data Import, Data Export, Storage Diagnostics, and Command Palette.

use crate::components::theme::{apply_document_dyslexia, apply_document_theme};
use crate::storage::{
    query_storage_diagnostics, request_persistent_storage, save_state_to_storage,
    trigger_binary_download, trigger_file_download, trigger_pwa_install,
};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use shared::{
    AppPage, AppState, ItemCollection, ThemeMode, UndoStack, decode_shareable_url_hash,
    encode_shareable_url_hash, export_to_compressed_bson, export_to_csv, export_to_json,
    export_to_printable_html, export_to_svg, import_from_compressed_bson, import_from_csv,
    import_from_json,
};
use tracing::info;

/// Traps `Tab` / `Shift+Tab` keyboard focus within the dialog element `modal_id` (WCAG 2.4.3).
#[allow(unused_variables)]
pub fn handle_modal_tab_trap(ev: &web_sys::KeyboardEvent, modal_id: &str) {
    if ev.key() == "Tab" && spodeian_leptos_ui::trap_tab_within_element(modal_id, ev.shift_key()) {
        ev.prevent_default();
    }
}

fn push_undo_if_present(state: RwSignal<AppState>) {
    if let Some(undo_sig) = use_context::<RwSignal<UndoStack<ItemCollection>>>() {
        let current = state.get_untracked().collection;
        undo_sig.update(|stack| stack.push(current));
    }
}

/// Confirmation dialog to reset application state to default sample items.
#[component]
pub fn ResetModal(
    /// Controls modal visibility.
    is_open: RwSignal<bool>,
    /// Root application state signal.
    state: RwSignal<AppState>,
) -> impl IntoView {
    let confirm_reset = move |_| {
        push_undo_if_present(state);
        let mut next = state.get_untracked();
        next.reset_data();
        next.collection = ItemCollection::with_sample_data();
        state.set(next.clone());
        if let Err(e) = save_state_to_storage(&next) {
            tracing::error!("Failed to persist state on reset: {e}");
        }
        is_open.set(false);
        info!("Application state reset to default sample dataset");
    };

    let close_modal = move |_| {
        is_open.set(false);
    };

    let cancel_btn_ref = NodeRef::<leptos::html::Button>::new();

    Effect::new(move |_| {
        if let (true, Some(btn)) = (is_open.get(), cancel_btn_ref.get()) {
            let _ = btn.focus();
        }
    });

    let on_dialog_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            ev.stop_propagation();
            is_open.set(false);
        } else {
            handle_modal_tab_trap(&ev, "reset-modal-dialog");
        }
    };

    view! {
        {move || {
            if is_open.get() {
                view! {
                    <div class="modal-backdrop" on:click=close_modal>
                        <div
                            id="reset-modal-dialog"
                            class="modal-card"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="reset-modal-title"
                            tabindex="-1"
                            on:click=move |ev| ev.stop_propagation()
                            on:keydown=on_dialog_keydown
                        >
                            <div class="modal-header">
                                <h3 id="reset-modal-title" class="modal-title">
                                    "Reset Application State"
                                </h3>
                                <button
                                    class="modal-close-btn"
                                    on:click=close_modal
                                    aria-label="Close dialog"
                                >
                                    "✕"
                                </button>
                            </div>
                            <div class="modal-body">
                                <p>
                                    "Are you sure you want to reset all tasks to the default sample dataset?"
                                </p>
                                <p class="text-warning">
                                    "Any custom items will be replaced in local browser storage (use Ctrl+Z to undo)."
                                </p>
                            </div>
                            <div class="modal-footer">
                                <button
                                    node_ref=cancel_btn_ref
                                    class="btn btn-secondary"
                                    on:click=close_modal
                                >
                                    "Cancel"
                                </button>
                                <button class="btn btn-danger" on:click=confirm_reset>
                                    "Yes, Reset Everything"
                                </button>
                            </div>
                        </div>
                    </div>
                }
                    .into_any()
            } else {
                view! {}.into_any()
            }
        }}
    }
}

/// Architecture overview and keyboard shortcuts dialog.
#[component]
pub fn HelpModal(
    /// Controls modal visibility.
    is_open: RwSignal<bool>,
) -> impl IntoView {
    let close_modal = move |_| {
        is_open.set(false);
    };

    let close_btn_ref = NodeRef::<leptos::html::Button>::new();

    Effect::new(move |_| {
        if let (true, Some(btn)) = (is_open.get(), close_btn_ref.get()) {
            let _ = btn.focus();
        }
    });

    let on_dialog_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            ev.stop_propagation();
            is_open.set(false);
        } else {
            handle_modal_tab_trap(&ev, "help-modal-dialog");
        }
    };

    view! {
        {move || {
            if is_open.get() {
                view! {
                    <div class="modal-backdrop" on:click=close_modal>
                        <div
                            id="help-modal-dialog"
                            class="modal-card modal-large"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="help-modal-title"
                            tabindex="-1"
                            on:click=move |ev| ev.stop_propagation()
                            on:keydown=on_dialog_keydown
                        >
                            <div class="modal-header">
                                <h3 id="help-modal-title" class="modal-title">
                                    "Architecture & Help Guide • "
                                    {concat!("v", env!("CARGO_PKG_VERSION"))}
                                </h3>
                                <button
                                    node_ref=close_btn_ref
                                    class="modal-close-btn"
                                    on:click=close_modal
                                    aria-label="Close dialog"
                                >
                                    "✕"
                                </button>
                            </div>
                            <div class="modal-body help-content">
                                <section class="help-section">
                                    <h4>"Leptos Serverless & Desktop Architecture"</h4>
                                    <p>
                                        "This template provides a modular Rust architecture using "
                                        <strong>"Leptos 0.8"</strong>
                                        " compiled to pre-rendered static HTML + WASM hydration for static hosts and Client-Side WASM for Native Desktop (Tauri v2)."
                                    </p>
                                </section>

                                <section class="help-section">
                                    <h4>"Crate Decomposition"</h4>
                                    <ul>
                                        <li>
                                            <strong>"crates/shared:"</strong>
                                            " Domain models ("
                                            <code>"Item"</code>
                                            ", "
                                            <code>"AppState"</code>
                                            ", "
                                            <code>"UndoStack"</code>
                                            ") and JSON / CSV / BSON / SVG / HTML export engines."
                                        </li>
                                        <li>
                                            <strong>"crates/app:"</strong>
                                            " Universal Leptos UI components, reactive signals, theme engine, analytics, and multi-tier persistence."
                                        </li>
                                        <li>
                                            <strong>"crates/web & crates/ssg:"</strong>
                                            " WASM browser entrypoints, static route pre-rendering, auto-generated sitemap/robots, and hash-stamped PWA Service Worker."
                                        </li>
                                        <li>
                                            <strong>"crates/desktop:"</strong>
                                            " Tauri v2 native desktop runner with strict CSP and mimalloc."
                                        </li>
                                    </ul>
                                </section>

                                <section class="help-section">
                                    <h4>"Keyboard Shortcuts"</h4>
                                    <ul>
                                        {spodeian_leptos_ui::default_web_shortcuts()
                                            .into_iter()
                                            .map(|entry| {
                                                view! {
                                                    <li>
                                                        <kbd>{entry.keys}</kbd>
                                                        {format!(" — {}", entry.description)}
                                                    </li>
                                                }
                                            })
                                            .collect::<Vec<_>>()} <li>
                                            <kbd>"Ctrl+K / Cmd+K"</kbd>
                                            " — Open searchable Command Palette"
                                        </li> <li>
                                            <kbd>"Ctrl+Z / Ctrl+Y"</kbd>
                                            " — Undo / Redo item collection changes"
                                        </li>
                                    </ul>
                                </section>
                            </div>
                            <div class="modal-footer">
                                <button class="btn btn-primary" on:click=close_modal>
                                    "Got it"
                                </button>
                            </div>
                        </div>
                    </div>
                }
                    .into_any()
            } else {
                view! {}.into_any()
            }
        }}
    }
}

/// Supported import formats in [`ImportModal`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ImportFormat {
    /// JSON application state or item collection.
    #[default]
    Json,
    /// RFC 4180 CSV spreadsheet.
    Csv,
    /// Base64-encoded compressed BSON backup.
    Bson,
    /// Compact `#state=...` shareable URL fragment.
    ShareLink,
}

/// Modal dialog for importing JSON, CSV, Base64 BSON, or `#state=...` shareable links.
#[component]
pub fn ImportModal(
    /// Controls modal visibility.
    is_open: RwSignal<bool>,
    /// Root application state signal.
    state: RwSignal<AppState>,
) -> impl IntoView {
    let import_format = RwSignal::new(ImportFormat::Json);
    let raw_text = RwSignal::new(String::new());
    let status_message = RwSignal::new(Option::<Result<String, String>>::None);

    let close_modal = move |_| {
        is_open.set(false);
        raw_text.set(String::new());
        status_message.set(None);
    };

    let close_btn_ref = NodeRef::<leptos::html::Button>::new();

    Effect::new(move |_| {
        if let (true, Some(btn)) = (is_open.get(), close_btn_ref.get()) {
            let _ = btn.focus();
        }
    });

    let on_dialog_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            ev.stop_propagation();
            is_open.set(false);
        } else {
            handle_modal_tab_trap(&ev, "import-modal-dialog");
        }
    };

    let do_import = move |_| {
        let content = raw_text.get();
        if content.trim().is_empty() {
            status_message.set(Some(Err("Please paste data to import.".to_string())));
            return;
        }

        match import_format.get() {
            ImportFormat::Json => match import_from_json(&content) {
                Ok(new_state) => {
                    push_undo_if_present(state);
                    let count = new_state.collection.items.len();
                    state.set(new_state.clone());
                    if let Err(e) = save_state_to_storage(&new_state) {
                        tracing::error!("Failed to persist state on JSON import: {e}");
                    }
                    status_message.set(Some(Ok(format!(
                        "Successfully imported {} items from JSON!",
                        count
                    ))));
                    info!("Successfully imported {} items via JSON", count);
                }
                Err(e) => {
                    status_message.set(Some(Err(format!("JSON Parse Error: {}", e))));
                }
            },
            ImportFormat::Csv => match import_from_csv(&content) {
                Ok(new_collection) => {
                    push_undo_if_present(state);
                    let count = new_collection.items.len();
                    state.update(|s| {
                        s.collection = new_collection;
                        if let Err(e) = save_state_to_storage(s) {
                            tracing::error!("Failed to persist state on CSV import: {e}");
                        }
                    });
                    status_message.set(Some(Ok(format!(
                        "Successfully imported {} items from CSV!",
                        count
                    ))));
                    info!("Successfully imported {} items via CSV", count);
                }
                Err(e) => {
                    status_message.set(Some(Err(format!("CSV Parse Error: {}", e))));
                }
            },
            ImportFormat::Bson => {
                use base64::Engine;
                match base64::engine::general_purpose::STANDARD.decode(content.trim()) {
                    Ok(decoded_bytes) => match import_from_compressed_bson(&decoded_bytes) {
                        Ok(new_state) => {
                            push_undo_if_present(state);
                            let count = new_state.collection.items.len();
                            state.set(new_state.clone());
                            if let Err(e) = save_state_to_storage(&new_state) {
                                tracing::error!("Failed to persist state on BSON import: {e}");
                            }
                            status_message.set(Some(Ok(format!(
                                "Successfully imported {} items from compressed BSON!",
                                count
                            ))));
                            info!("Successfully imported state via BSON");
                        }
                        Err(e) => {
                            status_message
                                .set(Some(Err(format!("BSON Deserialization Error: {}", e))));
                        }
                    },
                    Err(e) => {
                        status_message.set(Some(Err(format!("Base64 Decode Error: {}", e))));
                    }
                }
            }
            ImportFormat::ShareLink => match decode_shareable_url_hash(&content) {
                Ok(new_state) => {
                    push_undo_if_present(state);
                    let count = new_state.collection.items.len();
                    state.set(new_state.clone());
                    if let Err(e) = save_state_to_storage(&new_state) {
                        tracing::error!("Failed to persist state on share link import: {e}");
                    }
                    status_message.set(Some(Ok(format!(
                        "Successfully imported {} items from share link!",
                        count
                    ))));
                    info!("Successfully imported {} items via share link", count);
                }
                Err(e) => {
                    status_message.set(Some(Err(format!("Share Link Error: {}", e))));
                }
            },
        }
    };

    view! {
        {move || {
            if is_open.get() {
                view! {
                    <div class="modal-backdrop" on:click=close_modal>
                        <div
                            id="import-modal-dialog"
                            class="modal-card modal-large"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="import-modal-title"
                            tabindex="-1"
                            on:click=move |ev| ev.stop_propagation()
                            on:keydown=on_dialog_keydown
                        >
                            <div class="modal-header">
                                <h3 id="import-modal-title" class="modal-title">
                                    "Import Data"
                                </h3>
                                <button
                                    node_ref=close_btn_ref
                                    class="modal-close-btn"
                                    on:click=close_modal
                                    aria-label="Close dialog"
                                >
                                    "✕"
                                </button>
                            </div>
                            <div class="modal-body">
                                <div class="format-toggle-bar" style="flex-wrap: wrap;">
                                    <button
                                        class=move || {
                                            format!(
                                                "tab-btn {}",
                                                if import_format.get() == ImportFormat::Json {
                                                    "active"
                                                } else {
                                                    ""
                                                },
                                            )
                                        }
                                        on:click=move |_| {
                                            import_format.set(ImportFormat::Json);
                                            status_message.set(None);
                                        }
                                    >
                                        "JSON"
                                    </button>
                                    <button
                                        class=move || {
                                            format!(
                                                "tab-btn {}",
                                                if import_format.get() == ImportFormat::Csv {
                                                    "active"
                                                } else {
                                                    ""
                                                },
                                            )
                                        }
                                        on:click=move |_| {
                                            import_format.set(ImportFormat::Csv);
                                            status_message.set(None);
                                        }
                                    >
                                        "CSV"
                                    </button>
                                    <button
                                        class=move || {
                                            format!(
                                                "tab-btn {}",
                                                if import_format.get() == ImportFormat::Bson {
                                                    "active"
                                                } else {
                                                    ""
                                                },
                                            )
                                        }
                                        on:click=move |_| {
                                            import_format.set(ImportFormat::Bson);
                                            status_message.set(None);
                                        }
                                    >
                                        "Base64 BSON"
                                    </button>
                                    <button
                                        class=move || {
                                            format!(
                                                "tab-btn {}",
                                                if import_format.get() == ImportFormat::ShareLink {
                                                    "active"
                                                } else {
                                                    ""
                                                },
                                            )
                                        }
                                        on:click=move |_| {
                                            import_format.set(ImportFormat::ShareLink);
                                            status_message.set(None);
                                        }
                                    >
                                        "#state Link"
                                    </button>
                                </div>

                                <p class="modal-instruction">
                                    {move || match import_format.get() {
                                        ImportFormat::Json => {
                                            "Paste a valid JSON AppState or ItemCollection string below:"
                                        }
                                        ImportFormat::Csv => {
                                            "Paste valid CSV data below (Header: id,title,description,priority,completed,created_at):"
                                        }
                                        ImportFormat::Bson => {
                                            "Paste Base64 encoded compressed BSON data below:"
                                        }
                                        ImportFormat::ShareLink => {
                                            "Paste a #state=... fragment or full share URL below:"
                                        }
                                    }}
                                </p>

                                <textarea
                                    class="modal-textarea"
                                    rows="8"
                                    placeholder=move || match import_format.get() {
                                        ImportFormat::Json => "{\n  \"collection\": { ... }\n}",
                                        ImportFormat::Csv => {
                                            "id,title,description,priority,completed,created_at\n1,Sample Task,Task details,High,false,0"
                                        }
                                        ImportFormat::Bson => "Paste Base64 string here...",
                                        ImportFormat::ShareLink => "#state=...",
                                    }
                                    prop:value=move || raw_text.get()
                                    on:input=move |ev| raw_text.set(event_target_value(&ev))
                                ></textarea>

                                {move || match status_message.get() {
                                    Some(Ok(msg)) => {
                                        view! { <div class="alert alert-success">{msg}</div> }
                                            .into_any()
                                    }
                                    Some(Err(err)) => {
                                        view! { <div class="alert alert-error">{err}</div> }
                                            .into_any()
                                    }
                                    None => view! {}.into_any(),
                                }}
                            </div>
                            <div class="modal-footer">
                                <button class="btn btn-secondary" on:click=close_modal>
                                    "Close"
                                </button>
                                <button class="btn btn-primary" on:click=do_import>
                                    "Import & Apply"
                                </button>
                            </div>
                        </div>
                    </div>
                }
                    .into_any()
            } else {
                view! {}.into_any()
            }
        }}
    }
}

/// Supported export formats in [`ExportModal`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ExportFormat {
    /// Formatted JSON document.
    #[default]
    Json,
    /// RFC 4180 CSV spreadsheet.
    Csv,
    /// Base64 preview / binary `.bson` compressed backup.
    Bson,
    /// Standalone vector SVG priority chart.
    Svg,
    /// Self-contained printable HTML5 report with inline SVG.
    Html,
}

/// Modal dialog for previewing, copying (with verified clipboard feedback), and downloading exports.
#[component]
pub fn ExportModal(
    /// Controls modal visibility.
    is_open: RwSignal<bool>,
    /// Root application state signal.
    state: RwSignal<AppState>,
) -> impl IntoView {
    let export_format = RwSignal::new(ExportFormat::Json);
    let copy_feedback = RwSignal::new(Option::<Result<String, String>>::None);

    let close_modal = move |_| {
        is_open.set(false);
        copy_feedback.set(None);
    };

    let close_btn_ref = NodeRef::<leptos::html::Button>::new();

    Effect::new(move |_| {
        if let (true, Some(btn)) = (is_open.get(), close_btn_ref.get()) {
            let _ = btn.focus();
        }
    });

    let on_dialog_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            ev.stop_propagation();
            is_open.set(false);
        } else {
            handle_modal_tab_trap(&ev, "export-modal-dialog");
        }
    };

    let exported_content = Memo::new(move |_| {
        let current_state = state.get();
        match export_format.get() {
            ExportFormat::Json => {
                export_to_json(&current_state).unwrap_or_else(|e| format!("Error: {}", e))
            }
            ExportFormat::Csv => {
                export_to_csv(&current_state.collection).unwrap_or_else(|e| format!("Error: {}", e))
            }
            ExportFormat::Bson => {
                if let Ok(bytes) = export_to_compressed_bson(&current_state) {
                    use base64::Engine;
                    base64::engine::general_purpose::STANDARD.encode(&bytes)
                } else {
                    "Error exporting BSON".to_string()
                }
            }
            ExportFormat::Svg => export_to_svg(&current_state),
            ExportFormat::Html => export_to_printable_html(&current_state),
        }
    });

    let on_copy = move |_| {
        let text = exported_content.get_untracked();
        copy_feedback.set(None);
        leptos::task::spawn_local(async move {
            match spodeian_web_utils::write_clipboard_text(&text).await {
                Ok(()) => copy_feedback.set(Some(Ok("Copied to clipboard!".to_string()))),
                Err(e) => copy_feedback.set(Some(Err(format!(
                    "Could not copy: browser clipboard unavailable ({e})"
                )))),
            }
        });
    };

    let on_copy_share_link = move |_| {
        let current_state = state.get_untracked();
        copy_feedback.set(None);
        match encode_shareable_url_hash(&current_state) {
            Ok(hash) => {
                leptos::task::spawn_local(async move {
                    match spodeian_web_utils::write_clipboard_text(&hash).await {
                        Ok(()) => {
                            copy_feedback.set(Some(Ok("Copied #state share link!".to_string())))
                        }
                        Err(e) => copy_feedback.set(Some(Err(format!(
                            "Share link generated, but clipboard blocked ({e})"
                        )))),
                    }
                });
            }
            Err(e) => {
                copy_feedback.set(Some(Err(format!("Failed to encode share link: {e}"))));
            }
        }
    };

    let on_download = move |_| {
        let current_state = state.get_untracked();
        match export_format.get() {
            ExportFormat::Json => {
                let content = exported_content.get_untracked();
                trigger_file_download("app_state.json", &content, "application/json;charset=utf-8");
            }
            ExportFormat::Csv => {
                let content = exported_content.get_untracked();
                trigger_file_download("items.csv", &content, "text/csv;charset=utf-8");
            }
            ExportFormat::Bson => {
                if let Ok(bytes) = export_to_compressed_bson(&current_state) {
                    trigger_binary_download("app_backup.bson", &bytes, "application/octet-stream");
                }
            }
            ExportFormat::Svg => {
                let content = exported_content.get_untracked();
                trigger_file_download(
                    "priority_chart.svg",
                    &content,
                    "image/svg+xml;charset=utf-8",
                );
            }
            ExportFormat::Html => {
                let content = exported_content.get_untracked();
                trigger_file_download("workspace_report.html", &content, "text/html;charset=utf-8");
            }
        }
    };

    view! {
        {move || {
            if is_open.get() {
                view! {
                    <div class="modal-backdrop" on:click=close_modal>
                        <div
                            id="export-modal-dialog"
                            class="modal-card modal-large"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="export-modal-title"
                            tabindex="-1"
                            on:click=move |ev| ev.stop_propagation()
                            on:keydown=on_dialog_keydown
                        >
                            <div class="modal-header">
                                <h3 id="export-modal-title" class="modal-title">
                                    "Export Data"
                                </h3>
                                <button
                                    node_ref=close_btn_ref
                                    class="modal-close-btn"
                                    on:click=close_modal
                                    aria-label="Close dialog"
                                >
                                    "✕"
                                </button>
                            </div>
                            <div class="modal-body">
                                <div class="format-toggle-bar" style="flex-wrap: wrap;">
                                    <button
                                        class=move || {
                                            format!(
                                                "tab-btn {}",
                                                if export_format.get() == ExportFormat::Json {
                                                    "active"
                                                } else {
                                                    ""
                                                },
                                            )
                                        }
                                        on:click=move |_| {
                                            export_format.set(ExportFormat::Json);
                                            copy_feedback.set(None);
                                        }
                                    >
                                        "JSON"
                                    </button>
                                    <button
                                        class=move || {
                                            format!(
                                                "tab-btn {}",
                                                if export_format.get() == ExportFormat::Csv {
                                                    "active"
                                                } else {
                                                    ""
                                                },
                                            )
                                        }
                                        on:click=move |_| {
                                            export_format.set(ExportFormat::Csv);
                                            copy_feedback.set(None);
                                        }
                                    >
                                        "CSV"
                                    </button>
                                    <button
                                        class=move || {
                                            format!(
                                                "tab-btn {}",
                                                if export_format.get() == ExportFormat::Bson {
                                                    "active"
                                                } else {
                                                    ""
                                                },
                                            )
                                        }
                                        on:click=move |_| {
                                            export_format.set(ExportFormat::Bson);
                                            copy_feedback.set(None);
                                        }
                                    >
                                        "BSON"
                                    </button>
                                    <button
                                        class=move || {
                                            format!(
                                                "tab-btn {}",
                                                if export_format.get() == ExportFormat::Svg {
                                                    "active"
                                                } else {
                                                    ""
                                                },
                                            )
                                        }
                                        on:click=move |_| {
                                            export_format.set(ExportFormat::Svg);
                                            copy_feedback.set(None);
                                        }
                                    >
                                        "SVG Chart"
                                    </button>
                                    <button
                                        class=move || {
                                            format!(
                                                "tab-btn {}",
                                                if export_format.get() == ExportFormat::Html {
                                                    "active"
                                                } else {
                                                    ""
                                                },
                                            )
                                        }
                                        on:click=move |_| {
                                            export_format.set(ExportFormat::Html);
                                            copy_feedback.set(None);
                                        }
                                    >
                                        "HTML Report"
                                    </button>
                                </div>

                                <textarea
                                    class="modal-textarea"
                                    rows="10"
                                    readonly=true
                                    prop:value=move || exported_content.get()
                                ></textarea>

                                {move || match copy_feedback.get() {
                                    Some(Ok(msg)) => {
                                        view! { <div class="alert alert-success">{msg}</div> }
                                            .into_any()
                                    }
                                    Some(Err(err)) => {
                                        view! { <div class="alert alert-error">{err}</div> }
                                            .into_any()
                                    }
                                    None => view! {}.into_any(),
                                }}
                            </div>
                            <div class="modal-footer" style="flex-wrap: wrap;">
                                <button class="btn btn-secondary" on:click=close_modal>
                                    "Close"
                                </button>
                                <button class="btn btn-outline" on:click=on_copy_share_link>
                                    "Copy #state Link"
                                </button>
                                <button class="btn btn-outline" on:click=on_copy>
                                    "Copy to Clipboard"
                                </button>
                                <button class="btn btn-primary" on:click=on_download>
                                    "Download File"
                                </button>
                            </div>
                        </div>
                    </div>
                }
                    .into_any()
            } else {
                view! {}.into_any()
            }
        }}
    }
}

/// Storage persistence diagnostics and backup modal.
#[component]
pub fn StorageModal(
    /// Controls modal visibility.
    is_open: RwSignal<bool>,
    /// Signal to open the Import modal directly.
    show_import_modal: RwSignal<bool>,
    /// Root application state signal.
    state: RwSignal<AppState>,
) -> impl IntoView {
    let diag = Memo::new(move |_| {
        if !crate::client::client_ready() {
            return crate::storage::StorageDiagnostics::default();
        }
        query_storage_diagnostics()
    });

    let close_modal = move |_| {
        is_open.set(false);
    };

    let close_btn_ref = NodeRef::<leptos::html::Button>::new();

    Effect::new(move |_| {
        if let (true, Some(btn)) = (is_open.get(), close_btn_ref.get()) {
            let _ = btn.focus();
        }
    });

    let on_dialog_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            ev.stop_propagation();
            is_open.set(false);
        } else {
            handle_modal_tab_trap(&ev, "storage-modal-dialog");
        }
    };

    let on_request_persistence = move |_| {
        request_persistent_storage();
    };

    let on_trigger_pwa = move |_| {
        trigger_pwa_install();
    };

    let on_export_bson = move |_| {
        let current_state = state.get_untracked();
        if let Ok(bytes) = export_to_compressed_bson(&current_state) {
            trigger_binary_download("app_backup.bson", &bytes, "application/octet-stream");
        }
    };

    view! {
        {move || {
            if is_open.get() {
                let d = diag.get();
                let status_text = match d.is_persisted {
                    Some(true) => "Persistent (Immune to browser eviction)",
                    Some(false) => "Ephemeral (May be cleared under storage pressure)",
                    None => "Unknown / Querying...",
                };
                let pwa_text = if d.is_pwa_installed {
                    "Installed (Permanent App)"
                } else if d.pwa_install_available {
                    "Available to Install"
                } else {
                    "Not Available in Tab"
                };

                view! {
                    <div class="modal-backdrop" on:click=close_modal>
                        <div
                            id="storage-modal-dialog"
                            class="modal-card modal-large"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="storage-modal-title"
                            tabindex="-1"
                            on:click=move |ev| ev.stop_propagation()
                            on:keydown=on_dialog_keydown
                        >
                            <div class="modal-header">
                                <h3 id="storage-modal-title" class="modal-title">
                                    "Storage & Data Management"
                                </h3>
                                <button
                                    node_ref=close_btn_ref
                                    class="modal-close-btn"
                                    on:click=close_modal
                                    aria-label="Close dialog"
                                >
                                    "✕"
                                </button>
                            </div>
                            <div class="modal-body">
                                <div class="storage-diag-section">
                                    <h4>"Storage Status"</h4>
                                    <ul class="diag-list">
                                        <li>
                                            <strong>"Durability: "</strong>
                                            <span>{status_text}</span>
                                        </li>
                                        <li>
                                            <strong>"Active Storage Tier: "</strong>
                                            <span>{d.backend.label()}</span>
                                        </li>
                                        <li>
                                            <strong>"In-Memory Working Set: "</strong>
                                            <span>
                                                {format!(
                                                    "{} items (Capacity limit: {} items)",
                                                    state.get().collection.total_count(),
                                                    d.memory_capacity_limit,
                                                )}
                                            </span>
                                        </li>
                                        <li>
                                            <strong>"IndexedDB Fallback: "</strong>
                                            <span style="color: #10b981; font-weight: 600;">
                                                {if d.idb_active || d.fallback_tier_active {
                                                    "Active (Protected against RAM overfill)"
                                                } else {
                                                    "Synchronized & Available"
                                                }}
                                            </span>
                                        </li>
                                        <li>
                                            <strong>"PWA Installation: "</strong>
                                            <span>{pwa_text}</span>
                                        </li>
                                    </ul>
                                </div>

                                <div class="storage-actions-section" style="margin-top: 16px;">
                                    <h4>"Storage Actions"</h4>
                                    <div style="display: flex; gap: 8px; flex-wrap: wrap; margin-top: 8px;">
                                        {if d.is_persisted != Some(true) {
                                            view! {
                                                <button
                                                    class="btn btn-outline"
                                                    on:click=on_request_persistence
                                                >
                                                    "Request Persistent Storage"
                                                </button>
                                            }
                                                .into_any()
                                        } else {
                                            view! {}.into_any()
                                        }}
                                        {if d.pwa_install_available && !d.is_pwa_installed {
                                            view! {
                                                <button class="btn btn-outline" on:click=on_trigger_pwa>
                                                    "Install Web App"
                                                </button>
                                            }
                                                .into_any()
                                        } else {
                                            view! {}.into_any()
                                        }} <button class="btn btn-primary" on:click=on_export_bson>
                                            "Export Compressed .bson Backup"
                                        </button>
                                        <button
                                            class="btn btn-secondary"
                                            on:click=move |_| {
                                                is_open.set(false);
                                                show_import_modal.set(true);
                                            }
                                        >
                                            "Import Backup"
                                        </button>
                                    </div>
                                </div>
                            </div>
                            <div class="modal-footer">
                                <button class="btn btn-secondary" on:click=close_modal>
                                    "Close"
                                </button>
                            </div>
                        </div>
                    </div>
                }
                    .into_any()
            } else {
                view! {}.into_any()
            }
        }}
    }
}

/// Searchable Command Palette modal (`Ctrl+K` / `Cmd+K`) for rapid keyboard navigation and actions.
#[component]
pub fn CommandPaletteModal(
    /// Controls modal visibility.
    is_open: RwSignal<bool>,
    /// Root application state signal.
    state: RwSignal<AppState>,
    /// Active theme signal.
    theme: RwSignal<ThemeMode>,
    /// Active dyslexia typography signal.
    dyslexia: RwSignal<bool>,
    /// Signal opening the Import modal.
    show_import_modal: RwSignal<bool>,
    /// Signal opening the Export modal.
    show_export_modal: RwSignal<bool>,
    /// Signal opening the Storage modal.
    show_storage_modal: RwSignal<bool>,
    /// Signal opening the Telemetry modal.
    show_telemetry_modal: RwSignal<bool>,
    /// Signal opening the Help modal.
    show_help_modal: RwSignal<bool>,
    /// Signal opening the Reset modal.
    show_reset_modal: RwSignal<bool>,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if let (true, Some(el)) = (is_open.get(), input_ref.get()) {
            query.set(String::new());
            let _ = el.focus();
        }
    });

    let close_modal = move || is_open.set(false);

    let on_dialog_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            ev.stop_propagation();
            is_open.set(false);
        } else {
            handle_modal_tab_trap(&ev, "command-palette-dialog");
        }
    };

    let commands: Vec<(&'static str, &'static str, &'static str, &'static str)> = vec![
        ("nav.dashboard", "Go to Dashboard", "Navigation", "/"),
        (
            "nav.analytics",
            "Go to Analytics & SVG Explorer",
            "Navigation",
            "/analytics",
        ),
        (
            "nav.about",
            "Go to About & Diagnostics",
            "Navigation",
            "/about",
        ),
        ("edit.undo", "Undo Last Collection Change", "Edit", "Ctrl+Z"),
        ("edit.redo", "Redo Collection Change", "Edit", "Ctrl+Y"),
        ("edit.clear", "Clear Completed Items", "Edit", ""),
        ("theme.cycle", "Cycle Visual Theme", "Appearance", ""),
        (
            "a11y.dyslexia",
            "Toggle Dyslexia-Friendly Typography",
            "Accessibility",
            "",
        ),
        ("data.export", "Open Data Export Dialog", "Data", ""),
        ("data.import", "Open Data Import Dialog", "Data", ""),
        ("system.storage", "Open Storage Diagnostics", "System", ""),
        ("system.telemetry", "Open System Telemetry", "System", ""),
        ("system.help", "Open Help & Keyboard Shortcuts", "Help", ""),
        ("system.reset", "Reset to Sample Data", "System", ""),
    ];

    let run_command = move |id: &str| {
        match id {
            "nav.dashboard" => navigate(AppPage::Dashboard.path(), Default::default()),
            "nav.analytics" => navigate(AppPage::Analytics.path(), Default::default()),
            "nav.about" => navigate(AppPage::About.path(), Default::default()),
            "edit.undo" => {
                if let Some(undo_sig) = use_context::<RwSignal<UndoStack<ItemCollection>>>() {
                    let current = state.get_untracked().collection;
                    let mut restored = None;
                    undo_sig.update(|s| restored = s.undo(current));
                    if let Some(prev) = restored {
                        state.update(|st| {
                            st.collection = prev;
                            let _ = save_state_to_storage(st);
                        });
                    }
                }
            }
            "edit.redo" => {
                if let Some(undo_sig) = use_context::<RwSignal<UndoStack<ItemCollection>>>() {
                    let current = state.get_untracked().collection;
                    let mut restored = None;
                    undo_sig.update(|s| restored = s.redo(current));
                    if let Some(next) = restored {
                        state.update(|st| {
                            st.collection = next;
                            let _ = save_state_to_storage(st);
                        });
                    }
                }
            }
            "edit.clear" => {
                push_undo_if_present(state);
                state.update(|st| {
                    st.collection.clear_completed();
                    let _ = save_state_to_storage(st);
                });
            }
            "theme.cycle" => {
                let next = theme.get_untracked().next();
                theme.set(next);
                apply_document_theme(next);
            }
            "a11y.dyslexia" => {
                let next = !dyslexia.get_untracked();
                dyslexia.set(next);
                apply_document_dyslexia(next);
            }
            "data.export" => show_export_modal.set(true),
            "data.import" => show_import_modal.set(true),
            "system.storage" => show_storage_modal.set(true),
            "system.telemetry" => show_telemetry_modal.set(true),
            "system.help" => show_help_modal.set(true),
            "system.reset" => show_reset_modal.set(true),
            _ => {}
        }
        is_open.set(false);
    };

    view! {
        {move || {
            if is_open.get() {
                let commands_clone = commands.clone();
                let run_outer = run_command.clone();
                view! {
                    <div class="modal-backdrop" on:click=move |_| close_modal()>
                        <div
                            id="command-palette-dialog"
                            class="modal-card modal-large"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="command-palette-title"
                            tabindex="-1"
                            on:click=move |ev| ev.stop_propagation()
                            on:keydown=on_dialog_keydown
                        >
                            <div class="modal-header">
                                <h3 id="command-palette-title" class="modal-title">
                                    "Command Palette"
                                </h3>
                                <button
                                    class="modal-close-btn"
                                    on:click=move |_| close_modal()
                                    aria-label="Close command palette"
                                >
                                    "✕"
                                </button>
                            </div>
                            <div class="modal-body">
                                <input
                                    node_ref=input_ref
                                    type="text"
                                    class="input-text"
                                    placeholder="Filter commands by name or category..."
                                    aria-label="Filter commands"
                                    prop:value=move || query.get()
                                    on:input=move |ev| query.set(event_target_value(&ev))
                                />
                                <div style="display: flex; flex-direction: column; gap: 6px; max-height: 320px; overflow-y: auto;">
                                    {move || {
                                        let q = query.get().trim().to_ascii_lowercase();
                                        commands_clone
                                            .iter()
                                            .filter(|(id, label, cat, _)| {
                                                q.is_empty() || label.to_ascii_lowercase().contains(&q)
                                                    || cat.to_ascii_lowercase().contains(&q)
                                                    || id.to_ascii_lowercase().contains(&q)
                                            })
                                            .map(|&(id, label, cat, hint)| {
                                                let run = run_outer.clone();
                                                view! {
                                                    <button
                                                        class="drawer-btn"
                                                        style="justify-content: space-between;"
                                                        on:click=move |_| run(id)
                                                    >
                                                        <span>
                                                            <strong>{format!("[{cat}] ")}</strong>
                                                            {label}
                                                        </span>
                                                        {(!hint.is_empty()).then(|| view! { <kbd>{hint}</kbd> })}
                                                    </button>
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                    }}
                                </div>
                            </div>
                        </div>
                    </div>
                }
                    .into_any()
            } else {
                view! {}.into_any()
            }
        }}
    }
}
