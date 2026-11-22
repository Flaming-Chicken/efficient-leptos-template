//! Theme management component with instantaneous DOM synchronization, persistence, and crisp SVG icons.

use leptos::prelude::*;
use shared::ThemeMode;
use tracing::info;

/// Applies the theme to the HTML document element and persists it via `spodeian-leptos-ui`.
pub fn apply_document_theme(theme: ThemeMode) {
    spodeian_leptos_ui::apply_document_theme_with_key(
        theme.as_str(),
        crate::storage::STORAGE_KEY_THEME,
    );
}

/// Reads the stored theme via `spodeian-leptos-ui` or defaults to Dark mode.
pub fn get_initial_theme() -> ThemeMode {
    let raw = spodeian_leptos_ui::get_initial_theme_with_key(crate::storage::STORAGE_KEY_THEME);
    ThemeMode::from_str(&raw)
}

#[component]
fn ThemeSvgIcon(theme: RwSignal<ThemeMode>) -> impl IntoView {
    view! {
        {move || match theme.get() {
            ThemeMode::Dark => {
                view! {
                    <svg
                        aria-hidden="true"
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                    </svg>
                }
                    .into_any()
            }
            ThemeMode::Light => {
                view! {
                    <svg
                        aria-hidden="true"
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <circle cx="12" cy="12" r="5"></circle>
                        <line x1="12" y1="1" x2="12" y2="3"></line>
                        <line x1="12" y1="21" x2="12" y2="23"></line>
                        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                        <line x1="1" y1="12" x2="3" y2="12"></line>
                        <line x1="21" y1="12" x2="23" y2="12"></line>
                        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                        <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                    </svg>
                }
                    .into_any()
            }
            ThemeMode::HighContrastDark => {
                view! {
                    <svg
                        aria-hidden="true"
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <circle cx="12" cy="12" r="9"></circle>
                        <path d="M12 3v18a9 9 0 0 0 0-18z" fill="currentColor"></path>
                    </svg>
                }
                    .into_any()
            }
            ThemeMode::HighContrastLight => {
                view! {
                    <svg
                        aria-hidden="true"
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <circle cx="12" cy="12" r="9"></circle>
                        <path d="M12 3v18a9 9 0 0 1 0-18z" fill="currentColor"></path>
                    </svg>
                }
                    .into_any()
            }
        }}
    }
}

/// Interactive theme switcher cycling Dark, Warm Light, High Contrast Dark, and High Contrast Light.
#[component]
pub fn ThemeToggle(
    /// Reactive active theme signal.
    theme: RwSignal<ThemeMode>,
    /// Optional screen-reader live announcement signal.
    #[prop(optional)]
    announcement: Option<RwSignal<String>>,
) -> impl IntoView {
    let on_toggle = move |_| {
        let new_theme = theme.get().next();
        theme.set(new_theme);
        apply_document_theme(new_theme);
        if let Some(announcer) = announcement {
            announcer.set(format!("Theme changed to {}", new_theme.display_label()));
        }
        info!("Theme switched to: {:?}", new_theme);
    };

    view! {
        <button
            class="btn-theme-toggle"
            on:click=on_toggle
            title=move || format!("Theme: {}. Click to switch theme.", theme.get().display_label())
            aria-label=move || {
                format!("Theme selector. Currently set to {}.", theme.get().display_label())
            }
        >
            <span class="theme-icon" style="display: inline-flex; align-items: center;">
                <ThemeSvgIcon theme=theme />
            </span>
            <span class="theme-label">{move || theme.get().display_label()}</span>
        </button>
    }
}

/// Applies the dyslexia typography attribute to the HTML document element and saves it via
/// `spodeian-leptos-ui`.
pub fn apply_document_dyslexia(enabled: bool) {
    spodeian_leptos_ui::apply_document_dyslexia_with_key(
        enabled,
        crate::storage::STORAGE_KEY_DYSLEXIA,
    );
}

/// Reads the stored dyslexia typography setting via `spodeian-leptos-ui` or defaults to `false`.
#[must_use]
pub fn get_initial_dyslexia() -> bool {
    spodeian_leptos_ui::get_initial_dyslexia_with_key(crate::storage::STORAGE_KEY_DYSLEXIA)
}

/// Accessible toggle button for dyslexia-friendly typography (Atkinson Hyperlegible & OpenDyslexic with WCAG 1.4.12 spacing).
#[component]
pub fn DyslexiaToggle(
    /// Reactive dyslexia reading toggle signal.
    dyslexia: RwSignal<bool>,
    /// Optional screen-reader live announcement signal.
    #[prop(optional)]
    announcement: Option<RwSignal<String>>,
) -> impl IntoView {
    let on_toggle = move |_| {
        let new_val = !dyslexia.get();
        dyslexia.set(new_val);
        apply_document_dyslexia(new_val);
        if let Some(announcer) = announcement {
            announcer.set(if new_val {
                "Dyslexia-friendly typography enabled: Atkinson Hyperlegible with enhanced spacing"
                    .to_string()
            } else {
                "Dyslexia-friendly typography disabled".to_string()
            });
        }
        info!("Dyslexia font toggled to: {}", new_val);
    };

    view! {
        <button
            class="btn-theme-toggle btn-dyslexia-toggle"
            on:click=on_toggle
            title=move || {
                if dyslexia.get() {
                    "Dyslexia font enabled. Click to return to standard font."
                } else {
                    "Click to enable dyslexia-friendly font."
                }
            }
            aria-pressed=move || if dyslexia.get() { "true" } else { "false" }
            aria-label=move || {
                if dyslexia.get() {
                    "Dyslexia-friendly font enabled. Click to disable."
                } else {
                    "Dyslexia-friendly font disabled. Click to enable."
                }
            }
        >
            <span class="theme-icon" style="display: inline-flex; align-items: center;">
                <svg
                    aria-hidden="true"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <polyline points="4 7 4 4 20 4 20 7"></polyline>
                    <line x1="9" y1="20" x2="15" y2="20"></line>
                    <line x1="12" y1="4" x2="12" y2="20"></line>
                </svg>
            </span>
            <span class="theme-label">
                {move || if dyslexia.get() { "Dyslexia: On" } else { "Dyslexia: Off" }}
            </span>
        </button>
    }
}
