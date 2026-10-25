use shared::{
    AppState, Item, ItemCollection, Priority, ThemeMode, export_to_compressed_bson, export_to_csv,
    export_to_json, import_from_compressed_bson, import_from_csv, import_from_json,
};

#[test]
fn test_theme_mode_toggle() {
    let theme = ThemeMode::Dark;
    assert!(theme.is_dark());
    let toggled = theme.toggle();
    assert_eq!(toggled, ThemeMode::Light);
    assert!(!toggled.is_dark());
}

#[test]
fn test_item_collection_operations() {
    let mut collection = ItemCollection::new();
    assert_eq!(collection.total_count(), 0);

    let item = Item::new("test-1", "Test Title", "Test Description", Priority::High);
    collection.add(item);
    assert_eq!(collection.total_count(), 1);
    assert_eq!(collection.pending_count(), 1);
    assert_eq!(collection.completed_count(), 0);
    assert_eq!(collection.high_priority_count(), 1);

    collection.toggle_completed("test-1");
    assert_eq!(collection.completed_count(), 1);
    assert_eq!(collection.pending_count(), 0);
    assert_eq!(collection.high_priority_count(), 0);

    assert!(collection.remove("test-1"));
    assert_eq!(collection.total_count(), 0);
}

#[test]
fn test_json_roundtrip() {
    let state = AppState::new();
    let json_str = export_to_json(&state).expect("Export JSON failed");
    let imported: AppState = import_from_json(&json_str).expect("Import JSON failed");
    assert_eq!(state, imported);
}

#[test]
fn test_bson_roundtrip() {
    let state = AppState::new();
    let bytes = export_to_compressed_bson(&state).expect("Export BSON failed");
    assert!(!bytes.is_empty());
    let imported: AppState = import_from_compressed_bson(&bytes).expect("Import BSON failed");
    assert_eq!(
        state.collection.items.len(),
        imported.collection.items.len()
    );
}

#[test]
fn test_backward_compatibility() {
    let legacy_json = r#"{"collection":{"items":[]},"config":{}}"#;
    let imported: Result<AppState, _> = serde_json::from_str(legacy_json);
    assert!(imported.is_ok());
}

#[test]
fn test_csv_roundtrip() {
    let collection = ItemCollection::with_sample_data();
    let csv_str = export_to_csv(&collection).expect("Export CSV failed");
    let imported = import_from_csv(&csv_str).expect("Import CSV failed");
    assert_eq!(collection.items.len(), imported.items.len());
    for (orig, imp) in collection.items.iter().zip(imported.items.iter()) {
        assert_eq!(orig.title, imp.title);
        assert_eq!(orig.description, imp.description);
        assert_eq!(orig.priority, imp.priority);
        assert_eq!(orig.completed, imp.completed);
    }
}

#[test]
fn test_reset_data() {
    let mut state = AppState::new();
    assert!(!state.collection.items.is_empty());
    state.config.theme = ThemeMode::Light;

    state.reset_data();
    assert!(state.collection.items.is_empty());
    assert_eq!(state.collection.total_count(), 0);
    // Preserves configuration
    assert_eq!(state.config.theme, ThemeMode::Light);
}
