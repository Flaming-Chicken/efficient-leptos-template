//! Durable app state, built on `spodeian-cache`.
//!
//! - Each item is one record in an [`AsyncTieredStore`]: IndexedDB in the browser (and in
//!   the desktop app's webview), behind an in-memory cache of [`MEMORY_WORKING_SET_LIMIT`]
//!   records. A change writes only the records that changed.
//! - Settings and the item order go through [`PreferentialStore`], which puts each value
//!   in the tier `PreferentialRouter` chooses.
//! - Saves are asynchronous. Their real outcome lands in [`StorageStatus`], which the
//!   interface shows; nothing reports success before the browser confirms the write.
//!
//! The working-set limit bounds the storage cache, not the list on screen: the interface
//! still holds every item it displays.

use serde::{Deserialize, Serialize};
use shared::{AppConfig, AppState, Item};
use spodeian_cache::{AsyncTieredStore, PreferentialStore, StorageTier};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use thiserror::Error;
#[allow(unused_imports)]
use tracing::{error, info, warn};

use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Errors arising during state persistence.
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Export error: {0}")]
    Export(#[from] shared::ExportError),
    #[error("Storage is not ready yet")]
    NotReady,
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("{0}")]
    Cache(#[from] spodeian_cache::CacheError),
}

/// Key of the state saved by versions before 0.2 (whole state as one JSON string in
/// local storage). Read once to migrate, then removed.
pub const LEGACY_STORAGE_KEY_STATE: &str = "serverless_leptos_app_state";
pub const STORAGE_KEY_THEME: &str = "serverless_leptos_theme";
pub const STORAGE_KEY_DYSLEXIA: &str = "serverless_leptos_dyslexia";

const KEY_CONFIG: &str = "config";
const KEY_ITEM_INDEX: &str = "item-index";

/// How many item records the storage layer keeps cached in memory. Records beyond this
/// are read back from the durable tier when needed.
pub const MEMORY_WORKING_SET_LIMIT: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StorageBackend {
    LocalStorage,
    IndexedDb,
    CacheApi,
    NativeCas,
    /// Nothing durable is in use (not loaded yet, or storage is unavailable).
    #[default]
    MemoryOnly,
}

impl StorageBackend {
    pub fn label(self) -> &'static str {
        match self {
            Self::LocalStorage => "Local Storage (Fast Tier)",
            Self::IndexedDb => "IndexedDB (Structured Relational Tier)",
            Self::CacheApi => "Cache API (Large Binary Weights Tier)",
            Self::NativeCas => "Native CAS (Content-Addressed Disk Tier)",
            Self::MemoryOnly => "In-Memory Only (Ephemeral)",
        }
    }
}

impl From<StorageTier> for StorageBackend {
    fn from(tier: StorageTier) -> Self {
        match tier {
            StorageTier::LocalStorage => Self::LocalStorage,
            StorageTier::IndexedDb => Self::IndexedDb,
            StorageTier::CacheApi => Self::CacheApi,
            StorageTier::NativeCas => Self::NativeCas,
        }
    }
}

/// Live facts about persistence, updated after every load, save and permission request.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StorageStatus {
    /// Tier holding the settings and item order.
    pub backend: StorageBackend,
    /// Item records currently cached in memory by the storage layer.
    pub cached_items: usize,
    /// Item records in the durable tier.
    pub stored_items: usize,
    /// Whether the browser has promised not to evict this site's data.
    pub is_persisted: Option<bool>,
    /// The last save failed because the browser's storage quota is full.
    pub quota_exceeded: bool,
    /// Why the last save or load failed, if it did.
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageDiagnostics {
    pub is_persisted: Option<bool>,
    pub pwa_install_available: bool,
    pub is_pwa_installed: bool,
    pub backend: StorageBackend,
    pub quota_exceeded: bool,
    pub idb_active: bool,
    pub memory_items_count: usize,
    pub memory_capacity_limit: usize,
    pub fallback_tier_active: bool,
    pub last_error: Option<String>,
}

impl Default for StorageDiagnostics {
    fn default() -> Self {
        Self {
            is_persisted: None,
            pwa_install_available: false,
            is_pwa_installed: false,
            backend: StorageBackend::default(),
            quota_exceeded: false,
            idb_active: false,
            memory_items_count: 0,
            memory_capacity_limit: MEMORY_WORKING_SET_LIMIT,
            fallback_tier_active: false,
            last_error: None,
        }
    }
}

impl StorageDiagnostics {
    /// Combines the live storage status with the PWA install flags.
    pub fn from_status(status: &StorageStatus) -> Self {
        Self {
            is_persisted: status.is_persisted,
            backend: status.backend,
            quota_exceeded: status.quota_exceeded,
            idb_active: status.backend == StorageBackend::IndexedDb,
            memory_items_count: status.cached_items,
            // True once the durable tier holds records the memory cache has let go of.
            fallback_tier_active: status.stored_items > status.cached_items,
            last_error: status.last_error.clone(),
            ..Self::default()
        }
    }
}

// ---------------------------------------------------------------------------
// The store
// ---------------------------------------------------------------------------

/// Durable storage for [`AppState`].
pub struct AppStore {
    blobs: PreferentialStore,
    items: AsyncTieredStore<Item>,
    /// Content hash of each item as last saved, so a save writes only what changed.
    saved: RefCell<HashMap<String, u64>>,
}

fn content_hash(item: &Item) -> Result<u64, StorageError> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    serde_json::to_vec(item)?.hash(&mut hasher);
    Ok(hasher.finish())
}

impl AppStore {
    /// Opens the browser (or desktop webview) store.
    #[cfg(target_arch = "wasm32")]
    pub async fn open() -> Result<Self, StorageError> {
        Self::open_named("leptos-template").await
    }

    /// Opens a browser store under its own namespace (separate databases).
    #[cfg(target_arch = "wasm32")]
    pub async fn open_named(namespace: &str) -> Result<Self, StorageError> {
        use spodeian_cache::web::IndexedDbStore;
        let item_db = IndexedDbStore::open(&format!("spodeian-{namespace}-items"), "items").await?;
        Ok(Self {
            blobs: PreferentialStore::open_web(namespace).await?,
            items: AsyncTieredStore::with_indexed_db(MEMORY_WORKING_SET_LIMIT, item_db).await?,
            saved: RefCell::new(HashMap::new()),
        })
    }

    /// Opens a store in a directory. Used by native tests and tools; the desktop app runs
    /// in a webview and uses [`AppStore::open`].
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_native(dir: impl AsRef<std::path::Path>) -> Result<Self, StorageError> {
        let dir = dir.as_ref();
        Ok(Self {
            blobs: PreferentialStore::open_native(dir.join("blobs"))?,
            items: AsyncTieredStore::with_disk(MEMORY_WORKING_SET_LIMIT, dir.join("items"))?,
            saved: RefCell::new(HashMap::new()),
        })
    }

    /// Loads the saved state, or `None` if nothing has been saved yet.
    pub async fn load(&self) -> Result<Option<AppState>, StorageError> {
        let Some(ids) = self.blobs.load_json::<Vec<String>>(KEY_ITEM_INDEX).await? else {
            return Ok(None);
        };
        let config = self
            .blobs
            .load_json::<AppConfig>(KEY_CONFIG)
            .await?
            .unwrap_or_default();

        let mut state = AppState {
            config,
            ..AppState::default()
        };
        let mut saved = HashMap::new();
        for id in ids {
            // An interrupted save can leave an id without its record; skip it.
            if let Some(item) = self.items.get(&id).await? {
                saved.insert(id, content_hash(&item)?);
                state.collection.add(item);
            }
        }
        *self.saved.borrow_mut() = saved;
        Ok(Some(state))
    }

    /// Saves `state`, writing only the items that changed since the last save or load.
    /// Returns the tier that holds the settings and item order.
    pub async fn save(&self, state: &AppState) -> Result<StorageBackend, StorageError> {
        let mut next = HashMap::with_capacity(state.collection.items.len());
        for item in &state.collection.items {
            let hash = content_hash(item)?;
            let unchanged = self.saved.borrow().get(&item.id) == Some(&hash);
            if !unchanged {
                self.items.put(item.id.clone(), item.clone()).await?;
                self.saved.borrow_mut().insert(item.id.clone(), hash);
            }
            next.insert(item.id.clone(), hash);
        }

        let removed: Vec<String> = self
            .saved
            .borrow()
            .keys()
            .filter(|id| !next.contains_key(*id))
            .cloned()
            .collect();
        for id in removed {
            self.items.remove(&id).await?;
            self.saved.borrow_mut().remove(&id);
        }

        let order: Vec<&str> = state
            .collection
            .items
            .iter()
            .map(|i| i.id.as_str())
            .collect();
        self.blobs.save_json(KEY_CONFIG, &state.config).await?;
        let tier = self.blobs.save_json(KEY_ITEM_INDEX, &order).await?;
        Ok(tier.into())
    }

    /// Records cached in memory, and records in the durable tier.
    pub fn item_counts(&self) -> (usize, usize) {
        let metrics = self.items.metrics();
        (metrics.memory_items_count, metrics.fallback_items_count)
    }
}

// ---------------------------------------------------------------------------
// App-wide handle. Browser apps are single-threaded, so thread-locals hold it.
// ---------------------------------------------------------------------------

thread_local! {
    static STORE: RefCell<Option<Rc<AppStore>>> = const { RefCell::new(None) };
    static STATUS: Cell<Option<RwSignal<StorageStatus>>> = const { Cell::new(None) };
    /// The newest state waiting to be saved. Saves run one at a time; a burst of changes
    /// collapses into one write of the latest state.
    static PENDING: RefCell<Option<AppState>> = const { RefCell::new(None) };
    static SAVING: Cell<bool> = const { Cell::new(false) };
}

fn update_status(change: impl FnOnce(&mut StorageStatus)) {
    if let Some(status) = STATUS.with(Cell::get) {
        status.update(change);
    }
}

fn record_error(context: &str, err: &StorageError) {
    let message = format!("{context}: {err}");
    error!("{message}");
    update_status(|s| {
        s.quota_exceeded = message.contains("QuotaExceeded");
        s.last_error = Some(message);
    });
}

fn record_counts(store: &AppStore) {
    let (cached, stored) = store.item_counts();
    update_status(|s| {
        s.cached_items = cached;
        s.stored_items = stored;
    });
}

/// Opens storage and loads the saved state. Call once, in the browser, after the first
/// render. Returns `None` on first run or if storage is unavailable (the reason is then in
/// `status.last_error`); the app keeps working in memory either way.
pub async fn init(status: RwSignal<StorageStatus>) -> Option<AppState> {
    STATUS.with(|s| s.set(Some(status)));

    #[cfg(target_arch = "wasm32")]
    {
        let store = match AppStore::open().await {
            Ok(store) => Rc::new(store),
            Err(err) => {
                record_error("Could not open storage", &err);
                return None;
            }
        };

        let loaded = match store.load().await {
            Ok(Some(state)) => Some(state),
            Ok(None) => migrate_legacy_state(&store).await,
            Err(err) => {
                record_error("Could not load saved data", &err);
                None
            }
        };

        record_counts(&store);
        if loaded.is_some() {
            update_status(|s| s.backend = StorageBackend::IndexedDb);
        }
        if let Ok(persisted) = spodeian_cache::web::is_storage_persisted().await {
            update_status(|s| s.is_persisted = Some(persisted));
        }
        STORE.with(|s| *s.borrow_mut() = Some(store));
        loaded
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// Moves state saved by a pre-0.2 version (one JSON string in local storage) into the
/// new store, then removes the old copy.
#[cfg(target_arch = "wasm32")]
async fn migrate_legacy_state(store: &AppStore) -> Option<AppState> {
    let storage = web_sys::window()?.local_storage().ok()??;
    let json = storage.get_item(LEGACY_STORAGE_KEY_STATE).ok()??;
    let state = match shared::import_from_json(&json) {
        Ok(state) => state,
        Err(err) => {
            warn!("Ignoring unreadable legacy saved state: {err}");
            return None;
        }
    };
    match store.save(&state).await {
        Ok(_) => {
            let _ = storage.remove_item(LEGACY_STORAGE_KEY_STATE);
            info!("Migrated saved state from local storage to IndexedDB");
            Some(state)
        }
        Err(err) => {
            // Keep the legacy copy so nothing is lost; show its data this session.
            record_error("Could not migrate saved data", &err);
            Some(state)
        }
    }
}

/// Queues `state` to be saved. Returns `Err(NotReady)` if storage has not been opened;
/// a failure of the write itself is reported through [`StorageStatus::last_error`].
pub fn save_state_to_storage(state: &AppState) -> Result<(), StorageError> {
    let Some(store) = STORE.with(|s| s.borrow().clone()) else {
        return Err(StorageError::NotReady);
    };
    PENDING.with(|p| *p.borrow_mut() = Some(state.clone()));
    if SAVING.with(|s| s.replace(true)) {
        return Ok(()); // the running save loop will pick this state up
    }
    leptos::task::spawn_local(async move {
        while let Some(next) = PENDING.with(|p| p.borrow_mut().take()) {
            match store.save(&next).await {
                Ok(backend) => update_status(|s| {
                    s.backend = backend;
                    s.quota_exceeded = false;
                    s.last_error = None;
                }),
                Err(err) => record_error("Changes could not be saved", &err),
            }
            record_counts(&store);
        }
        SAVING.with(|s| s.set(false));
    });
    Ok(())
}

/// Current diagnostics: the live storage status plus PWA install flags. Reactive when
/// called inside a reactive scope.
#[allow(unused_mut)]
pub fn query_storage_diagnostics() -> StorageDiagnostics {
    let mut diag = STATUS
        .with(Cell::get)
        .map(|status| StorageDiagnostics::from_status(&status.get()))
        .unwrap_or_default();

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let flag = |name: &str| {
                js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str(name))
                    .ok()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            };
            diag.pwa_install_available = flag("__pwaInstallAvailable");
            diag.is_pwa_installed = flag("__pwaInstalled");
        }
    }

    diag
}

/// Asks the browser to keep this site's data when space runs low. The answer arrives in
/// [`StorageStatus::is_persisted`].
pub fn request_persistent_storage() {
    #[cfg(target_arch = "wasm32")]
    leptos::task::spawn_local(async {
        match spodeian_cache::web::request_persistent_storage().await {
            Ok(granted) => {
                info!("Persistent storage requested; granted: {granted}");
                update_status(|s| s.is_persisted = Some(granted));
            }
            Err(err) => warn!("Persistent storage request failed: {err}"),
        }
    });
}

/// Trigger native PWA install prompt
pub fn trigger_pwa_install() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(func) = js_sys::Reflect::get(
                &window,
                &wasm_bindgen::JsValue::from_str("__triggerPWAInstall"),
            )
            && let Some(func) = func.dyn_ref::<js_sys::Function>()
        {
            let _ = func.call0(&window);
            info!("Triggered __triggerPWAInstall from Leptos");
        }
    }
}

pub use spodeian_web_utils::{
    copy_to_clipboard, trigger_binary_download, trigger_text_download,
    trigger_text_download as trigger_file_download,
};
