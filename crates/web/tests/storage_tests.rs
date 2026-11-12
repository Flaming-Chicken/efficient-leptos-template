//! Browser tests: app state really persists in IndexedDB.
#![cfg(target_arch = "wasm32")]

use app::storage::{
    AppStore, LEGACY_STORAGE_KEY_STATE, MEMORY_WORKING_SET_LIMIT, StorageBackend, StorageStatus,
};
use leptos::prelude::*;
use shared::{AppState, Item, Priority, ThemeMode};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn unique(name: &str) -> String {
    format!("{name}-{}", (js_sys::Math::random() * 1e12) as u64)
}

fn bulk_state(count: usize) -> AppState {
    let mut state = AppState::default();
    for i in 0..count {
        state.collection.add(Item::new(
            format!("bulk-{i:04}"),
            format!("Bulk Item {i}"),
            "A description long enough that the whole state is well over sixteen kilobytes.",
            Priority::Medium,
        ));
    }
    state
}

#[wasm_bindgen_test]
async fn large_state_survives_a_new_session() {
    let namespace = unique("large");
    let mut state = bulk_state(MEMORY_WORKING_SET_LIMIT + 1);
    state.config.theme = ThemeMode::Light;
    assert!(serde_json::to_string(&state).unwrap().len() > 16 * 1024);

    let store = AppStore::open_named(&namespace).await.expect("open");
    assert!(store.load().await.expect("load empty").is_none());
    assert_eq!(
        store.save(&state).await.expect("save"),
        StorageBackend::IndexedDb
    );
    // The 101st record is only in IndexedDB, not in the memory cache.
    assert_eq!(
        store.item_counts(),
        (MEMORY_WORKING_SET_LIMIT, MEMORY_WORKING_SET_LIMIT + 1)
    );
    drop(store);

    let reopened = AppStore::open_named(&namespace).await.expect("reopen");
    let loaded = reopened.load().await.expect("load").expect("saved state");
    assert_eq!(loaded, state);
}

#[wasm_bindgen_test]
async fn changes_and_deletions_persist() {
    let namespace = unique("changes");
    let store = AppStore::open_named(&namespace).await.expect("open");
    let mut state = bulk_state(3);
    store.save(&state).await.expect("first save");

    state.collection.items[2].completed = true;
    state.collection.remove("bulk-0000");
    store.save(&state).await.expect("second save");
    drop(store);

    let reopened = AppStore::open_named(&namespace).await.expect("reopen");
    assert_eq!(reopened.load().await.expect("load"), Some(state));
}

#[wasm_bindgen_test]
async fn state_saved_by_the_previous_version_is_migrated() {
    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let mut legacy = AppState::new();
    legacy.collection.add(Item::new(
        "legacy-1",
        "From the old version",
        "",
        Priority::High,
    ));
    storage
        .set_item(
            LEGACY_STORAGE_KEY_STATE,
            &serde_json::to_string(&legacy).unwrap(),
        )
        .unwrap();

    let owner = Owner::new();
    owner.set();
    let status = RwSignal::new(StorageStatus::default());
    let loaded = app::storage::init(status)
        .await
        .expect("legacy state is loaded");
    assert_eq!(loaded, legacy);
    assert_eq!(status.get_untracked().backend, StorageBackend::IndexedDb);
    assert_eq!(status.get_untracked().stored_items, 5);
    assert!(status.get_untracked().last_error.is_none());
    // The old copy is gone, and the data now loads from the new store.
    assert_eq!(storage.get_item(LEGACY_STORAGE_KEY_STATE).unwrap(), None);
    let store = AppStore::open().await.expect("open default store");
    assert_eq!(store.load().await.expect("load"), Some(legacy));
}
