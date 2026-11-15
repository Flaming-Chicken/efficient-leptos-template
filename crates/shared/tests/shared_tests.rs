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

#[test]
fn test_app_page_paths_round_trip() {
    use shared::AppPage;

    for page in AppPage::ALL {
        assert_eq!(AppPage::from_path(page.path()), Some(page));
        assert!(!page.description().is_empty());
    }
    assert_eq!(AppPage::from_path("/about/"), Some(AppPage::About));
    assert_eq!(AppPage::from_path(""), Some(AppPage::Dashboard));
    assert_eq!(AppPage::from_path("/missing"), None);
    assert_eq!(AppPage::About.label(), "About");
}

#[test]
fn test_state_saved_before_url_routing_still_loads() {
    // Saved states from before URL routing carry an `active_page` field; it is ignored.
    let legacy = r#"{"collection":{"items":[]},"config":{"theme":"Light","active_page":"About"}}"#;
    let state: AppState = serde_json::from_str(legacy).expect("legacy state should load");
    assert_eq!(state.config.theme, ThemeMode::Light);
}

#[test]
fn test_versioned_envelope_and_legacy_schema_migration() {
    use shared::CURRENT_SCHEMA_VERSION;
    use spodeian_persist::StateMigrator;

    let legacy_unversioned = r#"{"collection":{"items":[]},"config":{"theme":"Dark"}}"#;
    let migrated =
        import_from_json(legacy_unversioned).expect("legacy unversioned JSON should migrate");
    assert_eq!(migrated.schema_version, CURRENT_SCHEMA_VERSION);

    let state = AppState::new();
    let wrapped =
        StateMigrator::serialize_versioned(&state, CURRENT_SCHEMA_VERSION, 1_700_000_000.0)
            .expect("serialize versioned");
    let restored = import_from_json(&wrapped).expect("import versioned snapshot");
    assert_eq!(state, restored);
}

mod prop_tests {
    use super::*;
    use proptest::prelude::*;
    use shared::CURRENT_SCHEMA_VERSION;
    use spodeian_leptos_ui::{normalize_theme, toggle_theme_value};
    use spodeian_persist::StateMigrator;

    fn arb_priority() -> impl Strategy<Value = Priority> {
        prop_oneof![
            Just(Priority::Low),
            Just(Priority::Medium),
            Just(Priority::High),
            Just(Priority::Critical),
        ]
    }

    fn arb_theme() -> impl Strategy<Value = ThemeMode> {
        prop_oneof![
            Just(ThemeMode::Dark),
            Just(ThemeMode::Light),
            Just(ThemeMode::HighContrastDark),
            Just(ThemeMode::HighContrastLight),
        ]
    }

    fn arb_single_line_text() -> impl Strategy<Value = String> {
        proptest::collection::vec(
            prop_oneof![
                Just('a'),
                Just('Z'),
                Just('0'),
                Just(','),
                Just('"'),
                Just(' '),
                Just('-'),
                Just('!'),
                Just('é'),
                Just('λ'),
            ],
            1..32,
        )
        .prop_map(|chars| {
            let s: String = chars.into_iter().collect();
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                "item".to_string()
            } else {
                trimmed
            }
        })
    }

    fn arb_item() -> impl Strategy<Value = Item> {
        (
            arb_single_line_text(),
            arb_single_line_text(),
            arb_single_line_text(),
            arb_priority(),
            any::<bool>(),
            any::<u64>(),
        )
            .prop_map(
                |(id, title, description, priority, completed, created_at)| Item {
                    id,
                    title,
                    description,
                    priority,
                    completed,
                    created_at,
                },
            )
    }

    proptest! {
        #[test]
        fn prop_versioned_state_json_and_bson_roundtrip(
            items in proptest::collection::vec(arb_item(), 0..16),
            theme in arb_theme(),
            auto_save in any::<bool>(),
            show_completed in any::<bool>(),
            filter_priority in proptest::option::of(arb_priority()),
            search_query in "[a-zA-Z0-9 _,-]{0,24}",
        ) {
            let state = AppState {
                schema_version: CURRENT_SCHEMA_VERSION,
                collection: ItemCollection { items },
                config: shared::AppConfig {
                    theme,
                    auto_save,
                    show_completed,
                    filter_priority,
                    search_query,
                },
            };

            let json = export_to_json(&state).expect("json export");
            let from_json = import_from_json(&json).expect("json import");
            prop_assert_eq!(&from_json, &state);
            prop_assert_eq!(from_json.schema_version, CURRENT_SCHEMA_VERSION);

            let enveloped = StateMigrator::serialize_versioned(&state, CURRENT_SCHEMA_VERSION, 42.0)
                .expect("versioned wrap");
            let from_envelope = import_from_json(&enveloped).expect("versioned import");
            prop_assert_eq!(&from_envelope, &state);

            let bson_bytes = export_to_compressed_bson(&state).expect("bson export");
            let from_bson = import_from_compressed_bson(&bson_bytes).expect("bson import");
            prop_assert_eq!(from_bson, state);
        }

        #[test]
        fn prop_csv_roundtrip_handles_commas_quotes_and_unicode(
            items in proptest::collection::vec(arb_item(), 1..12),
        ) {
            let collection = ItemCollection { items };
            let csv = export_to_csv(&collection).expect("csv export");
            let restored = import_from_csv(&csv).expect("csv import");
            prop_assert_eq!(restored.items.len(), collection.items.len());
            for (orig, imp) in collection.items.iter().zip(restored.items.iter()) {
                prop_assert_eq!(&imp.id, &orig.id);
                prop_assert_eq!(&imp.title, &orig.title);
                prop_assert_eq!(&imp.description, &orig.description);
                prop_assert_eq!(imp.priority, orig.priority);
                prop_assert_eq!(imp.completed, orig.completed);
                prop_assert_eq!(imp.created_at, orig.created_at);
            }
        }

        #[test]
        fn prop_theme_normalization_and_cycle_invariants(raw in ".*") {
            let normalized = normalize_theme(&raw);
            prop_assert!(matches!(
                normalized,
                "dark" | "light" | "high-contrast-dark" | "high-contrast-light"
            ));
            let mode = ThemeMode::from_str(normalized);
            prop_assert_eq!(mode.as_str(), normalized);

            // Cycling 4 times always returns to the same normalized mode.
            let mut cur = normalized;
            for _ in 0..4 {
                cur = toggle_theme_value(cur);
            }
            prop_assert_eq!(cur, normalized);
        }
    }
}
