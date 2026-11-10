// Temporary IndexedDB bridge used by the app's storage code. Replaced by spodeian-cache's
// Rust IndexedDB backend; this file is removed once nothing calls it.
// IndexedDB Multi-Tier Fallback Storage Engine
const IDB_DB_NAME = 'leptos_template_offline_store';
const IDB_STORE_NAME = 'leptos_template_kv';
const IDB_VERSION = 1;

function openIdbDatabase() {
    return new Promise((resolve, reject) => {
        if (!window.indexedDB) {
            reject(new Error('IndexedDB is not supported in this environment'));
            return;
        }
        const request = window.indexedDB.open(IDB_DB_NAME, IDB_VERSION);
        request.onupgradeneeded = (e) => {
            const db = e.target.result;
            if (!db.objectStoreNames.contains(IDB_STORE_NAME)) {
                db.createObjectStore(IDB_STORE_NAME);
            }
        };
        request.onsuccess = (e) => resolve(e.target.result);
        request.onerror = (e) => reject(e.target.error);
    });
}

window.__saveToIndexedDB = async function (key, value) {
    try {
        const db = await openIdbDatabase();
        return new Promise((resolve, reject) => {
            const tx = db.transaction(IDB_STORE_NAME, 'readwrite');
            const store = tx.objectStore(IDB_STORE_NAME);
            const req = store.put(value, key);
            req.onsuccess = () => resolve(true);
            req.onerror = (e) => reject(e.target.error);
        });
    } catch (err) {
        console.error('IndexedDB save error for key:', key, err);
        return false;
    }
};

window.__loadFromIndexedDB = async function (key) {
    try {
        const db = await openIdbDatabase();
        return new Promise((resolve, reject) => {
            const tx = db.transaction(IDB_STORE_NAME, 'readonly');
            const store = tx.objectStore(IDB_STORE_NAME);
            const req = store.get(key);
            req.onsuccess = (e) => resolve(e.target.result || null);
            req.onerror = (e) => reject(e.target.error);
        });
    } catch (err) {
        console.error('IndexedDB load error for key:', key, err);
        return null;
    }
};
