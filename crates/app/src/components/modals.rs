//! Modal dialogs for State Reset, Help & Architecture, Data Import, Data Export, and Storage Diagnostics.

use crate::storage::{
    copy_to_clipboard, query_storage_diagnostics, request_persistent_storage,
    save_state_to_storage, trigger_binary_download, trigger_file_download, trigger_pwa_install,
};
use leptos::prelude::*;
use shared::{
    AppState, export_to_compressed_bson, export_to_csv, export_to_json,
    import_from_compressed_bson, import_from_csv, import_from_json,
};
use tracing::info;

/// Helper to trap Tab navigation inside modal dialogs (WCAG 2.4.3).
#[allow(unused_variables)]
pub fn handle_modal_tab_trap(
    ev: &web_sys::KeyboardEvent,
    first_el: Option<&web_sys::HtmlElement>,
    last_el: Option<&web_sys::HtmlElement>,
) {
    if ev.key() == "Tab" {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                let active = doc.active_element();
                if ev.shift_key() {
                    if let (Some(active), Some(first)) = (active.as_ref(), first_el) {
                        if first.is_same_node(Some(active.as_ref())) {
                            ev.prevent_default();
                            if let Some(last) = last_el {
                                let _ = last.focus();
                            }
                        }
                    }
                } else {
                    if let (Some(active), Some(last)) = (active.as_ref(), last_el) {
                        if last.is_same_node(Some(active.as_ref())) {
                            ev.prevent_default();
                            if let Some(first) = first_el {
                                let _ = first.focus();
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ResetModal(is_open: RwSignal<bool>, state: RwSignal<AppState>) -> impl IntoView {
    let confirm_reset = move |_| {
        let default_state = AppState::new();
        state.set(default_state.clone());
        if let Err(e) = save_state_to_storage(&default_state) {
            tracing::error!("Failed to persist state on reset: {e}");
        }
        is_open.set(false);
        info!("Application state reset to default sample dataset");
    };

    let close_modal = move |_| {
        is_open.set(false);
    };

    let cancel_btn_ref = NodeRef::<leptos::html::Button>::new();
    let reset_btn_ref = NodeRef::<leptos::html::Button>::new();

    // Auto-focus Cancel button on open for safe default
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
            let first = cancel_btn_ref.get();
            let last = reset_btn_ref.get();
            handle_modal_tab_trap(&ev, first.as_deref(), last.as_deref());
        }
    };

    view! {
        {move || {
            if is_open.get() {
                view! {
                    <div class="modal-backdrop" on:click=close_modal>
                        <div
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
                                    "Are you sure you want to reset all tasks and configuration to the default sample dataset?"
                                </p>
                                <p class="text-warning">
                                    "Any custom items and changes will be replaced in local browser storage."
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
                                <button
                                    node_ref=reset_btn_ref
                                    class="btn btn-danger"
                                    on:click=confirm_reset
                                >
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

#[component]
pub fn HelpModal(is_open: RwSignal<bool>) -> impl IntoView {
    let close_modal = move |_| {
        is_open.set(false);
    };

    let close_btn_ref = NodeRef::<leptos::html::Button>::new();
    let got_it_btn_ref = NodeRef::<leptos::html::Button>::new();

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
            let first = close_btn_ref.get();
            let last = got_it_btn_ref.get();
            handle_modal_tab_trap(&ev, first.as_deref(), last.as_deref());
        }
    };

    view! {
        {move || {
            if is_open.get() {
                view! {
                    <div class="modal-backdrop" on:click=close_modal>
                        <div
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
                                        " compiled to Client-Side WebAssembly (WASM) for Serverless Static hosting (Cloudflare Pages / GitHub Pages) and Native Desktop (Tauri v2)."
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
                                            ") and robust JSON / CSV / BSON export/import engines."
                                        </li>
                                        <li>
                                            <strong>"crates/app:"</strong>
                                            " Universal Leptos UI components, reactive signals, responsive layout, theme engine, and dialogs."
                                        </li>
                                        <li>
                                            <strong>"crates/web:"</strong>
                                            " Web client entrypoint with wasm-bindgen, Trunk configuration, and PWA assets."
                                        </li>
                                        <li>
                                            <strong>"crates/desktop:"</strong>
                                            " Tauri v2 native desktop runner with cross-platform window management."
                                        </li>
                                    </ul>
                                </section>

                                <section class="help-section">
                                    <h4>"PWA & Caching Strategy"</h4>
                                    <p>
                                        "Features a hybrid service worker caching strategy: "
                                        <strong>"Network-First"</strong>
                                        " for HTML entrypoint to guarantee instant zero-downtime updates, and "
                                        <strong>"Cache-First"</strong>
                                        " for content-hashed WASM, JS, and CSS bundles."
                                    </p>
                                </section>

                                <section class="help-section">
                                    <h4>"Keyboard Shortcuts"</h4>
                                    <ul>
                                        <li>
                                            <kbd>"Enter"</kbd>
                                            " — Quickly submit new item creation in form"
                                        </li>
                                        <li>
                                            <kbd>"Escape"</kbd>
                                            " — Dismiss any open modal dialog"
                                        </li>
                                    </ul>
                                </section>
                            </div>
                            <div class="modal-footer">
                                <button
                                    node_ref=got_it_btn_ref
                                    class="btn btn-primary"
                                    on:click=close_modal
                                >
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ImportFormat {
    #[default]
    Json,
    Csv,
    Bson,
}

#[component]
pub fn ImportModal(is_open: RwSignal<bool>, state: RwSignal<AppState>) -> impl IntoView {
    let import_format = RwSignal::new(ImportFormat::Json);
    let raw_text = RwSignal::new(String::new());
    let status_message = RwSignal::new(Option::<Result<String, String>>::None);

    let close_modal = move |_| {
        is_open.set(false);
        raw_text.set(String::new());
        status_message.set(None);
    };

    let close_btn_ref = NodeRef::<leptos::html::Button>::new();
    let import_btn_ref = NodeRef::<leptos::html::Button>::new();

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
            let first = close_btn_ref.get();
            let last = import_btn_ref.get();
            handle_modal_tab_trap(&ev, first.as_deref(), last.as_deref());
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
        }
    };

    view! {
        {move || {
            if is_open.get() {
                view! {
                    <div class="modal-backdrop" on:click=close_modal>
                        <div
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
                                <div class="format-toggle-bar">
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
                                </div>

                                <p class="modal-instruction">
                                    {move || match import_format.get() {
                                        ImportFormat::Json => {
                                            "Paste a valid JSON AppState string below:"
                                        }
                                        ImportFormat::Csv => {
                                            "Paste valid CSV data below (Header: id,title,description,priority,completed,created_at):"
                                        }
                                        ImportFormat::Bson => {
                                            "Paste Base64 encoded compressed BSON data below:"
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
                                <button
                                    node_ref=import_btn_ref
                                    class="btn btn-primary"
                                    on:click=do_import
                                >
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ExportFormat {
    #[default]
    Json,
    Csv,
    Bson,
}

#[component]
pub fn ExportModal(is_open: RwSignal<bool>, state: RwSignal<AppState>) -> impl IntoView {
    let export_format = RwSignal::new(ExportFormat::Json);
    let copy_feedback = RwSignal::new(false);

    let close_modal = move |_| {
        is_open.set(false);
        copy_feedback.set(false);
    };

    let close_btn_ref = NodeRef::<leptos::html::Button>::new();
    let download_btn_ref = NodeRef::<leptos::html::Button>::new();

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
            let first = close_btn_ref.get();
            let last = download_btn_ref.get();
            handle_modal_tab_trap(&ev, first.as_deref(), last.as_deref());
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
        }
    });

    let on_copy = move |_| {
        let text = exported_content.get();
        copy_to_clipboard(&text);
        copy_feedback.set(true);
        set_timeout(
            move || copy_feedback.set(false),
            std::time::Duration::from_millis(2500),
        );
    };

    let on_download = move |_| {
        let current_state = state.get();
        match export_format.get() {
            ExportFormat::Json => {
                let content = exported_content.get();
                trigger_file_download("app_state.json", &content, "application/json");
            }
            ExportFormat::Csv => {
                let content = exported_content.get();
                trigger_file_download("items.csv", &content, "text/csv");
            }
            ExportFormat::Bson => {
                if let Ok(bytes) = export_to_compressed_bson(&current_state) {
                    trigger_binary_download("app_backup.bson", &bytes, "application/octet-stream");
                }
            }
        }
    };

    view! {
        {move || {
            if is_open.get() {
                view! {
                    <div class="modal-backdrop" on:click=close_modal>
                        <div
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
                                <div class="format-toggle-bar">
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
                                        on:click=move |_| export_format.set(ExportFormat::Json)
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
                                        on:click=move |_| export_format.set(ExportFormat::Csv)
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
                                        on:click=move |_| export_format.set(ExportFormat::Bson)
                                    >
                                        "Compressed BSON"
                                    </button>
                                </div>

                                <textarea
                                    class="modal-textarea"
                                    rows="10"
                                    readonly=true
                                    prop:value=move || exported_content.get()
                                ></textarea>

                                {move || {
                                    if copy_feedback.get() {
                                        view! {
                                            <div class="alert alert-success">
                                                "Copied to clipboard!"
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        view! {}.into_any()
                                    }
                                }}
                            </div>
                            <div class="modal-footer">
                                <button class="btn btn-secondary" on:click=close_modal>
                                    "Close"
                                </button>
                                <button class="btn btn-outline" on:click=on_copy>
                                    "Copy to Clipboard"
                                </button>
                                <button
                                    node_ref=download_btn_ref
                                    class="btn btn-primary"
                                    on:click=on_download
                                >
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

#[component]
pub fn StorageModal(
    is_open: RwSignal<bool>,
    show_import_modal: RwSignal<bool>,
    state: RwSignal<AppState>,
) -> impl IntoView {
    let diag = Memo::new(move |_| {
        let mut d = query_storage_diagnostics();
        let count = state.get().collection.total_count();
        d.memory_items_count = count;
        d.fallback_tier_active = count > crate::storage::MEMORY_WORKING_SET_LIMIT || d.idb_active;
        d
    });

    let close_modal = move |_| {
        is_open.set(false);
    };

    let close_btn_ref = NodeRef::<leptos::html::Button>::new();
    let footer_close_btn_ref = NodeRef::<leptos::html::Button>::new();

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
            let first = close_btn_ref.get();
            let last = footer_close_btn_ref.get();
            handle_modal_tab_trap(&ev, first.as_deref(), last.as_deref());
        }
    };

    let on_request_persistence = move |_| {
        request_persistent_storage();
    };

    let on_trigger_pwa = move |_| {
        trigger_pwa_install();
    };

    let on_export_bson = move |_| {
        let current_state = state.get();
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
                                <button
                                    node_ref=footer_close_btn_ref
                                    class="btn btn-secondary"
                                    on:click=close_modal
                                >
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
