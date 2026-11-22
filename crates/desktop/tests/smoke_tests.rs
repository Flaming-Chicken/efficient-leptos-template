#[test]
fn test_desktop_smoke() {
    let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    assert_eq!(
        context.config().product_name.as_deref(),
        Some("Leptos Desktop")
    );
}

#[test]
fn test_desktop_ipc_commands() {
    let summary = desktop_lib::commands::desktop_runtime_summary();
    assert!(summary.parallel_threads >= 1);
    assert!(summary.wgsl_shader_ready);

    let sweep = desktop_lib::commands::compute_harmonic_sweep_native(
        shared::ParametricPreviewConfig::default(),
        vec![0.5, 1.0, 1.5],
        32,
    );
    assert_eq!(sweep.len(), 3);
    assert!(sweep.iter().all(|curve| curve.len() == 32));
}
