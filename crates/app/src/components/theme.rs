//! Theme management component with instantaneous DOM synchronization and persistence.

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
pub fn ThemeToggle(
    theme: RwSignal<ThemeMode>,
    #[prop(optional)] announcement: Option<RwSignal<String>>,
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
            <span class="theme-icon">{move || theme.get().icon()}</span>
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
    dyslexia: RwSignal<bool>,
    #[prop(optional)] announcement: Option<RwSignal<String>>,
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
            <span class="theme-icon">"🔤"</span>
            <span class="theme-label">
                {move || if dyslexia.get() { "Dyslexia: On" } else { "Dyslexia: Off" }}
            </span>
        </button>
    }
}
