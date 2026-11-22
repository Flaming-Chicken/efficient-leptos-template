//! Responsive top navigation bar with quick action buttons, undo/redo, command palette, and mobile drawer.

use crate::components::theme::{DyslexiaToggle, ThemeToggle};
use crate::storage::{query_storage_diagnostics, save_state_to_storage, trigger_pwa_install};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;
use shared::{AppPage, AppState, ItemCollection, ThemeMode, UndoStack};

/// Top responsive navigation header and mobile menu drawer.
#[component]
pub fn Navbar(
    /// Active visual theme signal.
    theme: RwSignal<ThemeMode>,
    /// Active dyslexia typography signal.
    dyslexia: RwSignal<bool>,
    /// Opens the reset confirmation dialog.
    show_reset_modal: RwSignal<bool>,
    /// Opens the help dialog.
    show_help_modal: RwSignal<bool>,
    /// Opens the import dialog.
    show_import_modal: RwSignal<bool>,
    /// Opens the export dialog.
    show_export_modal: RwSignal<bool>,
    /// Opens the storage diagnostics dialog.
    show_storage_modal: RwSignal<bool>,
    /// Optional telemetry dialog signal.
    #[prop(optional)]
    show_telemetry_modal: Option<RwSignal<bool>>,
    /// Optional command palette dialog signal.
    #[prop(optional)]
    show_command_palette: Option<RwSignal<bool>>,
    /// Controls mobile drawer visibility.
    mobile_menu_open: RwSignal<bool>,
    /// Optional screen-reader live announcement signal.
    #[prop(optional)]
    announcement: Option<RwSignal<String>>,
) -> impl IntoView {
    let location = use_location();
    let current_page = Memo::new(move |_| AppPage::from_path(&location.pathname.get()));
    let state_ctx = use_context::<RwSignal<AppState>>();
    let undo_ctx = use_context::<RwSignal<UndoStack<ItemCollection>>>();

    let on_undo = move |_| {
        if let (Some(st), Some(undo_sig)) = (state_ctx, undo_ctx) {
            let current = st.get_untracked().collection;
            let mut restored = None;
            undo_sig.update(|s| restored = s.undo(current));
            if let Some(prev) = restored {
                st.update(|s| {
                    s.collection = prev;
                    let _ = save_state_to_storage(s);
                });
                if let Some(ann) = announcement {
                    ann.set("Undid last item collection change".to_string());
                }
            }
        }
    };

    let on_redo = move |_| {
        if let (Some(st), Some(undo_sig)) = (state_ctx, undo_ctx) {
            let current = st.get_untracked().collection;
            let mut restored = None;
            undo_sig.update(|s| restored = s.redo(current));
            if let Some(next) = restored {
                st.update(|s| {
                    s.collection = next;
                    let _ = save_state_to_storage(s);
                });
                if let Some(ann) = announcement {
                    ann.set("Redid item collection change".to_string());
                }
            }
        }
    };

    // Header link: highlighted, and announced as the current page, when it is shown.
    let page_link = move |page: AppPage, class: &'static str, active_class: &'static str| {
        view! {
            <A
                href=page.path()
                attr:class=move || {
                    if current_page.get() == Some(page) { active_class } else { class }
                }
                attr:aria-current=move || (current_page.get() == Some(page)).then_some("page")
                on:click=move |_| mobile_menu_open.set(false)
            >
                {page.label()}
            </A>
        }
    };

    let toggle_mobile_menu = move |_| {
        mobile_menu_open.update(|open| *open = !*open);
    };

    let close_mobile_menu = move || {
        mobile_menu_open.set(false);
    };

    let diag = Memo::new(move |_| {
        if crate::client::client_ready() {
            query_storage_diagnostics()
        } else {
            crate::storage::StorageDiagnostics::default()
        }
    });

    view! {
        <header class="app-header">
            <div class="header-container">
                <div class="brand-section">
                    <a href="/" class="brand-logo" on:click=move |_| close_mobile_menu()>
                        <div class="logo-text">
                            <span class="brand-name">"Leptos Serverless"</span>
                            <span class="brand-subtitle">"& Desktop Template"</span>
                        </div>
                    </a>
                    <span class="badge-tech">{concat!("v", env!("CARGO_PKG_VERSION"))}</span>
                </div>

                <nav class="desktop-nav" style="flex-wrap: wrap;">
                    <div class="page-nav-group" style="display: flex; gap: 4px; margin-right: 8px;">
                        {page_link(AppPage::Dashboard, "nav-btn", "nav-btn nav-btn-accent")}
                        {page_link(AppPage::Analytics, "nav-btn", "nav-btn nav-btn-accent")}
                        {page_link(AppPage::About, "nav-btn", "nav-btn nav-btn-accent")}
                    </div>
                    {move || {
                        undo_ctx
                            .map(|u| {
                                view! {
                                    <button
                                        class="nav-btn"
                                        disabled=move || !u.get().can_undo()
                                        on:click=on_undo
                                        title="Undo (Ctrl+Z)"
                                    >
                                        "Undo"
                                    </button>
                                    <button
                                        class="nav-btn"
                                        disabled=move || !u.get().can_redo()
                                        on:click=on_redo
                                        title="Redo (Ctrl+Y)"
                                    >
                                        "Redo"
                                    </button>
                                }
                            })
                    }}
                    {move || {
                        show_command_palette
                            .map(|cp| {
                                view! {
                                    <button
                                        class="nav-btn"
                                        on:click=move |_| cp.set(true)
                                        title="Command Palette (Ctrl+K)"
                                    >
                                        "⌘K"
                                    </button>
                                }
                            })
                    }}
                    {move || {
                        let d = diag.get();
                        if d.pwa_install_available && !d.is_pwa_installed {
                            view! {
                                <button
                                    class="nav-btn nav-btn-accent"
                                    on:click=move |_| trigger_pwa_install()
                                >
                                    "Install App"
                                </button>
                            }
                                .into_any()
                        } else {
                            ().into_any()
                        }
                    }}
                    <button class="nav-btn" on:click=move |_| show_storage_modal.set(true)>
                        {move || {
                            let d = diag.get();
                            match d.is_persisted {
                                Some(true) => "Storage (Persistent)",
                                Some(false) => "Storage (Ephemeral)",
                                None => "Storage",
                            }
                        }}
                    </button>
                    <button class="nav-btn" on:click=move |_| show_import_modal.set(true)>
                        "Import"
                    </button>
                    <button class="nav-btn" on:click=move |_| show_export_modal.set(true)>
                        "Export"
                    </button>
                    {move || {
                        if let Some(telemetry_sig) = show_telemetry_modal {
                            view! {
                                <button class="nav-btn" on:click=move |_| telemetry_sig.set(true)>
                                    "Telemetry"
                                </button>
                            }
                                .into_any()
                        } else {
                            ().into_any()
                        }
                    }}
                    <button
                        class="nav-btn nav-btn-subtle"
                        on:click=move |_| show_help_modal.set(true)
                    >
                        "Help"
                    </button>
                    <button
                        class="nav-btn nav-btn-danger"
                        on:click=move |_| show_reset_modal.set(true)
                    >
                        "Reset"
                    </button>
                    {if let Some(ann) = announcement {
                        view! {
                            <ThemeToggle theme=theme announcement=ann />
                            <DyslexiaToggle dyslexia=dyslexia announcement=ann />
                        }
                            .into_any()
                    } else {
                        view! {
                            <ThemeToggle theme=theme />
                            <DyslexiaToggle dyslexia=dyslexia />
                        }
                            .into_any()
                    }}
                </nav>

                <button
                    class="mobile-menu-toggle"
                    on:click=toggle_mobile_menu
                    aria-label="Toggle Navigation Menu"
                    aria-expanded=move || if mobile_menu_open.get() { "true" } else { "false" }
                >
                    {move || if mobile_menu_open.get() { "✕" } else { "☰" }}
                </button>
            </div>

            // Mobile dropdown drawer
            {move || {
                if mobile_menu_open.get() {
                    view! {
                        <div class="mobile-drawer">
                            {page_link(
                                AppPage::Dashboard,
                                "drawer-btn",
                                "drawer-btn drawer-btn-active",
                            )}
                            {page_link(
                                AppPage::Analytics,
                                "drawer-btn",
                                "drawer-btn drawer-btn-active",
                            )}
                            {page_link(
                                AppPage::About,
                                "drawer-btn",
                                "drawer-btn drawer-btn-active",
                            )}
                            <hr style="border: 0; border-top: 1px solid var(--border-color); margin: 6px 0;" />
                            {move || {
                                show_command_palette
                                    .map(|cp| {
                                        view! {
                                            <button
                                                class="drawer-btn"
                                                on:click=move |_| {
                                                    close_mobile_menu();
                                                    cp.set(true);
                                                }
                                            >
                                                "⌘K Command Palette"
                                            </button>
                                        }
                                    })
                            }}
                            {move || {
                                let d = diag.get();
                                if d.pwa_install_available && !d.is_pwa_installed {
                                    view! {
                                        <button
                                            class="drawer-btn drawer-btn-accent"
                                            on:click=move |_| {
                                                close_mobile_menu();
                                                trigger_pwa_install();
                                            }
                                        >
                                            "Install App"
                                        </button>
                                    }
                                        .into_any()
                                } else {
                                    ().into_any()
                                }
                            }}
                            <button
                                class="drawer-btn"
                                on:click=move |_| {
                                    close_mobile_menu();
                                    show_storage_modal.set(true);
                                }
                            >
                                "Storage Diagnostics"
                            </button>
                            <button
                                class="drawer-btn"
                                on:click=move |_| {
                                    close_mobile_menu();
                                    show_import_modal.set(true);
                                }
                            >
                                "Import Data"
                            </button>
                            <button
                                class="drawer-btn"
                                on:click=move |_| {
                                    close_mobile_menu();
                                    show_export_modal.set(true);
                                }
                            >
                                "Export Data"
                            </button>
                            {move || {
                                if let Some(telemetry_sig) = show_telemetry_modal {
                                    view! {
                                        <button
                                            class="drawer-btn"
                                            on:click=move |_| {
                                                close_mobile_menu();
                                                telemetry_sig.set(true);
                                            }
                                        >
                                            "Telemetry"
                                        </button>
                                    }
                                        .into_any()
                                } else {
                                    ().into_any()
                                }
                            }}
                            <button
                                class="drawer-btn"
                                on:click=move |_| {
                                    close_mobile_menu();
                                    show_help_modal.set(true);
                                }
                            >
                                "Help & Shortcuts"
                            </button>
                            <button
                                class="drawer-btn drawer-btn-danger"
                                on:click=move |_| {
                                    close_mobile_menu();
                                    show_reset_modal.set(true);
                                }
                            >
                                "Reset State"
                            </button>
                            <div
                                class="drawer-theme"
                                style="display: flex; gap: 0.5rem; justify-content: center; flex-wrap: wrap;"
                            >
                                {if let Some(ann) = announcement {
                                    view! {
                                        <ThemeToggle theme=theme announcement=ann />
                                        <DyslexiaToggle dyslexia=dyslexia announcement=ann />
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <ThemeToggle theme=theme />
                                        <DyslexiaToggle dyslexia=dyslexia />
                                    }
                                        .into_any()
                                }}
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }}
        </header>
    }
}
