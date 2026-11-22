//! Modular Leptos components, views, modals, and navigation layout.

pub mod about;
pub mod analytics;
pub mod item_list;
pub mod modals;
pub mod navbar;
pub mod telemetry_viewer;
pub mod theme;

pub use about::AboutPage;
pub use analytics::AnalyticsPage;
pub use item_list::ItemList;
pub use modals::{
    CommandPaletteModal, ExportModal, HelpModal, ImportModal, ResetModal, StorageModal,
};
pub use navbar::Navbar;
pub use telemetry_viewer::TelemetryModal;
pub use theme::{DyslexiaToggle, ThemeToggle};
