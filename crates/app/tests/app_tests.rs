use app::storage::{StorageBackend, StorageDiagnostics};
use shared::{
    AppState, Item, Priority, ThemeMode, export_to_compressed_bson, export_to_csv, export_to_json,
    import_from_compressed_bson, import_from_csv, import_from_json,
};

#[test]
fn test_app_initial_state() {
    let state = AppState::new();
    assert_eq!(state.collection.total_count(), 4);
    assert_eq!(state.collection.completed_count(), 1);
    assert_eq!(state.collection.pending_count(), 3);
    assert_eq!(state.config.theme, ThemeMode::Dark);
    assert!(state.config.auto_save);
    assert!(state.config.show_completed);
    assert!(state.config.search_query.is_empty());
}

#[test]
fn test_app_item_mutations() {
    let mut state = AppState::new();

    let new_item = Item::new(
        "custom-item-1",
        "Implement Web Worker",
        "Run compute off the main thread",
        Priority::Critical,
    );
    state.collection.add(new_item);
    assert_eq!(state.collection.total_count(), 5);

    // Toggle completed
    state.collection.toggle_completed("custom-item-1");
    let item = state
        .collection
        .items
        .iter()
        .find(|i| i.id == "custom-item-1")
        .expect("Item should exist");
    assert!(item.completed);

    // Remove item
    let removed = state.collection.remove("custom-item-1");
    assert!(removed);
    assert_eq!(state.collection.total_count(), 4);
}

#[test]
fn test_app_storage_diagnostics() {
    let diag = StorageDiagnostics::default();
    assert_eq!(diag.backend, StorageBackend::LocalStorage);
    assert_eq!(
        StorageBackend::LocalStorage.label(),
        "Local Storage (Fast Tier)"
    );
    assert_eq!(
        StorageBackend::IndexedDb.label(),
        "IndexedDB (Structured Relational Tier)"
    );
    assert_eq!(
        StorageBackend::NativeCas.label(),
        "Native CAS (Content-Addressed Disk Tier)"
    );
    assert!(!diag.quota_exceeded);
    assert!(!diag.idb_active);
}

#[test]
fn test_app_export_import_json() {
    let mut state = AppState::new();
    state.config.theme = ThemeMode::Light;
    state.config.search_query = "deploy".to_string();

    let json_str = export_to_json(&state).expect("JSON export failed");
    assert!(json_str.contains("deploy"));

    let restored: AppState = import_from_json(&json_str).expect("JSON import failed");
    assert_eq!(state, restored);
}

#[test]
fn test_app_export_import_csv() {
    let state = AppState::new();
    let csv_str = export_to_csv(&state.collection).expect("CSV export failed");
    assert!(csv_str.starts_with("id,title,description,priority,completed,created_at"));

    let restored = import_from_csv(&csv_str).expect("CSV import failed");
    assert_eq!(state.collection.total_count(), restored.total_count());
    for (orig, imp) in state.collection.items.iter().zip(restored.items.iter()) {
        assert_eq!(orig.title, imp.title);
        assert_eq!(orig.description, imp.description);
        assert_eq!(orig.priority, imp.priority);
        assert_eq!(orig.completed, imp.completed);
    }
}

#[test]
fn test_app_export_import_compressed_bson() {
    let state = AppState::new();
    let compressed_bytes = export_to_compressed_bson(&state).expect("BSON export failed");
    assert!(!compressed_bytes.is_empty());

    let restored: AppState =
        import_from_compressed_bson(&compressed_bytes).expect("BSON import failed");
    assert_eq!(state, restored);
}

#[test]
fn test_app_reset_data() {
    let mut state = AppState::new();
    state.config.theme = ThemeMode::Light;
    state.config.search_query = "active query".to_string();

    assert!(!state.collection.items.is_empty());
    state.reset_data();

    // Data collection cleared
    assert_eq!(state.collection.total_count(), 0);
    assert!(state.collection.items.is_empty());

    // Config preserved
    assert_eq!(state.config.theme, ThemeMode::Light);
    assert_eq!(state.config.search_query, "active query");
}

#[test]
fn test_app_search_and_filter() {
    let state = AppState::new();

    // Filter by search query
    let query = "PWA";
    let matches: Vec<_> = state
        .collection
        .items
        .iter()
        .filter(|item| {
            item.title.to_lowercase().contains(&query.to_lowercase())
                || item
                    .description
                    .to_lowercase()
                    .contains(&query.to_lowercase())
        })
        .collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].id, "item-3");

    // Filter by priority
    let critical_items: Vec<_> = state
        .collection
        .items
        .iter()
        .filter(|item| item.priority == Priority::Critical)
        .collect();
    assert_eq!(critical_items.len(), 1);
    assert_eq!(critical_items[0].id, "item-1");
}

#[test]
fn test_app_storage_diagnostics_and_memory_capacity() {
    use app::storage::{MEMORY_WORKING_SET_LIMIT, query_storage_diagnostics};

    let mut diag = query_storage_diagnostics();
    assert_eq!(diag.memory_capacity_limit, MEMORY_WORKING_SET_LIMIT);
    assert_eq!(diag.memory_items_count, 0);
    assert!(!diag.fallback_tier_active);

    // Simulate bulk items exceeding memory capacity
    let mut state = AppState::new();
    for i in 0..110 {
        state.collection.add(shared::Item::new(
            format!("bulk-{i}"),
            format!("Bulk Item {i}"),
            "Memory stress test",
            Priority::Low,
        ));
    }

    let count = state.collection.total_count();
    diag.memory_items_count = count;
    diag.fallback_tier_active = count > MEMORY_WORKING_SET_LIMIT || diag.idb_active;

    assert_eq!(diag.memory_items_count, 114);
    assert!(diag.fallback_tier_active);
}
