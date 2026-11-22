//! Native desktop and mobile application runner using Tauri v2.

/// Tauri v2 backend IPC commands backed by native Rayon parallelism.
pub mod commands {
    /// Returns the native desktop host compute runtime summary (Rayon thread pool and WGSL shader readiness).
    #[tauri::command]
    pub fn desktop_runtime_summary() -> shared::ComputeRuntimeSummary {
        shared::ComputeRuntimeSummary::current()
    }

    /// Evaluates a parallel harmonic sweep on the native Tauri host via `spodeian-parallel` / Rayon.
    #[tauri::command]
    pub fn compute_harmonic_sweep_native(
        config: shared::ParametricPreviewConfig,
        multipliers: Vec<f32>,
        samples: usize,
    ) -> Vec<Vec<(f32, f32)>> {
        config.batch_harmonic_sweep(&multipliers, samples)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    spodeian_telemetry::init_default();

    tracing::info!("Starting Leptos Native Application (Tauri v2)...");

    if let Err(e) = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::desktop_runtime_summary,
            commands::compute_harmonic_sweep_native
        ])
        .setup(|_app| {
            tracing::info!("Tauri window setup successfully initialized");
            Ok(())
        })
        .run(tauri::generate_context!())
    {
        tracing::error!("Error while running Tauri application: {e}");
    }
}
