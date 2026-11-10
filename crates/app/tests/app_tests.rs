#![cfg(not(target_arch = "wasm32"))]

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
fn test_app_storage_diagnostics_defaults_and_labels() {
    let diag = StorageDiagnostics::default();
    // Nothing is durable until storage has loaded.
    assert_eq!(diag.backend, StorageBackend::MemoryOnly);
    assert_eq!(
        StorageBackend::IndexedDb.label(),
        "IndexedDB (Structured Relational Tier)"
    );
    assert!(!diag.quota_exceeded);
    assert!(!diag.idb_active);
    assert!(diag.last_error.is_none());
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

fn fresh_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("leptos_template_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn bulk_state(count: usize) -> AppState {
    let mut state = AppState::default();
    for i in 0..count {
        state.collection.add(Item::new(
            format!("bulk-{i:04}"),
            format!("Bulk Item {i}"),
            "Working-set test",
            Priority::Low,
        ));
    }
    state
}

#[test]
fn test_store_round_trip_past_the_working_set_limit() {
    use app::storage::{AppStore, MEMORY_WORKING_SET_LIMIT, StorageDiagnostics, StorageStatus};

    let dir = fresh_dir("working_set");
    pollster::block_on(async {
        let store = AppStore::open_native(&dir).expect("open");
        assert!(store.load().await.expect("load empty").is_none());

        let mut state = bulk_state(MEMORY_WORKING_SET_LIMIT + 1);
        state.config.theme = ThemeMode::Light;
        store.save(&state).await.expect("save");

        // The cache holds the limit; the 101st record lives only in the durable tier.
        let (cached, stored) = store.item_counts();
        assert_eq!(cached, MEMORY_WORKING_SET_LIMIT);
        assert_eq!(stored, MEMORY_WORKING_SET_LIMIT + 1);
        let diag = StorageDiagnostics::from_status(&StorageStatus {
            cached_items: cached,
            stored_items: stored,
            ..Default::default()
        });
        assert!(diag.fallback_tier_active);

        // A new session gets everything back, in the original order.
        let reopened = AppStore::open_native(&dir).expect("reopen");
        let loaded = reopened.load().await.expect("load").expect("saved state");
        assert_eq!(loaded, state);
    });
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_store_saves_only_what_changed_and_removes_deleted_items() {
    use app::storage::AppStore;

    let dir = fresh_dir("incremental");
    pollster::block_on(async {
        let store = AppStore::open_native(&dir).expect("open");
        let mut state = bulk_state(3);
        store.save(&state).await.expect("first save");

        state.collection.items[1].completed = true;
        state.collection.remove("bulk-0000");
        state
            .collection
            .add(Item::new("new", "New", "", Priority::High));
        store.save(&state).await.expect("second save");

        let reopened = AppStore::open_native(&dir).expect("reopen");
        let loaded = reopened.load().await.expect("load").expect("saved state");
        assert_eq!(loaded, state);
        let ids: Vec<&str> = loaded
            .collection
            .items
            .iter()
            .map(|i| i.id.as_str())
            .collect();
        assert_eq!(ids, vec!["bulk-0001", "bulk-0002", "new"]);
        assert_eq!(reopened.item_counts().1, 3);
    });
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_saving_before_storage_is_ready_is_reported() {
    use app::storage::{StorageError, save_state_to_storage};

    // No store has been opened in this thread, so the save must not claim success.
    assert!(matches!(
        save_state_to_storage(&AppState::new()),
        Err(StorageError::NotReady)
    ));
}
