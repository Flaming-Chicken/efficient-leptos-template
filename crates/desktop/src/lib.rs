//! Native desktop and mobile application runner using Tauri v2.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    spodeian_telemetry::init_default();

    tracing::info!("Starting Leptos Native Application (Tauri v2)...");

    tauri::Builder::default()
        .setup(|_app| {
            tracing::info!("Tauri window setup successfully initialized");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
