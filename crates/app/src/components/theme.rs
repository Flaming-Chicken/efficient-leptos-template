//! Theme management component with instantaneous DOM synchronization and persistence.

use leptos::prelude::*;
use shared::ThemeMode;
use tracing::info;

/// Applies the theme to the HTML document element.
pub fn apply_document_theme(_theme: ThemeMode) {
    #[cfg(target_arch = "wasm32")]
    {
        let theme = _theme;
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document()
                && let Some(html) = doc.document_element()
            {
                let _ = html.set_attribute("data-theme", theme.as_str());
            }
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(crate::storage::STORAGE_KEY_THEME, theme.as_str());
            }
        }
    }
}

/// Reads the stored theme or defaults to Dark mode.
pub fn get_initial_theme() -> ThemeMode {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
            && let Ok(Some(val)) = storage.get_item(crate::storage::STORAGE_KEY_THEME)
        {
            return ThemeMode::from_str(&val);
        }
    }
    ThemeMode::Dark
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

/// Applies the dyslexia typography attribute to the HTML document element and saves it to localStorage.
pub fn apply_document_dyslexia(_enabled: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document()
                && let Some(html) = doc.document_element()
            {
                if _enabled {
                    let _ = html.set_attribute("data-dyslexia", "true");
                } else {
                    let _ = html.remove_attribute("data-dyslexia");
                }
            }
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(
                    crate::storage::STORAGE_KEY_DYSLEXIA,
                    if _enabled { "true" } else { "false" },
                );
            }
        }
    }
}

/// Reads the stored dyslexia typography setting from localStorage or defaults to false.
#[must_use]
pub fn get_initial_dyslexia() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
            && let Ok(Some(val)) = storage.get_item(crate::storage::STORAGE_KEY_DYSLEXIA)
        {
            return val == "true";
        }
    }
    false
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
