//! Core shared domain models, serialization, state management, and export logic.

pub mod export;
pub mod models;
pub mod telemetry;

pub use export::{
    ExportError, export_to_compressed_bson, export_to_csv, export_to_json,
    import_from_compressed_bson, import_from_csv, import_from_json,
};
pub use models::{AppConfig, AppState, Item, ItemCollection, Priority, ThemeMode};
pub use telemetry::{TelemetryChannelMetric, TelemetryDashboardData};

