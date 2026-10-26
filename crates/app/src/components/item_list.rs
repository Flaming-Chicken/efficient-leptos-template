//! Interactive item management component with reactive search, filtering, creation, and statistics.

use crate::storage::save_state_to_storage;
use leptos::prelude::*;
use shared::{AppState, Item, Priority};

#[component]
pub fn ItemList(
    state: RwSignal<AppState>,
    #[prop(optional)] announcement: Option<RwSignal<String>>,
) -> impl IntoView {
    let new_title = RwSignal::new(String::new());
    let new_description = RwSignal::new(String::new());
    let new_priority = RwSignal::new(Priority::Medium);

    let search_query = RwSignal::new(String::new());
    let filter_priority = RwSignal::new(Option::<Priority>::None);
    let show_completed = RwSignal::new(true);

    let add_item = move || {
        let title = new_title.get().trim().to_string();
        if title.is_empty() {
            return;
        }

        let desc = new_description.get().trim().to_string();
        let priority = new_priority.get();

        let id = format!("item-{}", js_sys_time());
        let item = Item {
            id,
            title: title.clone(),
            description: desc,
            priority,
            completed: false,
            created_at: js_sys_time(),
        };

        state.update(|s| {
            s.collection.add(item);
            if let Err(e) = save_state_to_storage(s) {
                tracing::error!("Failed to persist state on item add: {e}");
            }
        });

        if let Some(announcer) = announcement {
            announcer.set(format!("Added new task: {title}"));
        }

        new_title.set(String::new());
        new_description.set(String::new());
        new_priority.set(Priority::Medium);
    };

    let toggle_item = move |id: String| {
        let mut completed_state = false;
        let mut item_title = String::new();
        state.update(|s| {
            if let Some(item) = s.collection.items.iter_mut().find(|i| i.id == id) {
                item.completed = !item.completed;
                completed_state = item.completed;
                item_title = item.title.clone();
            }
            if let Err(e) = save_state_to_storage(s) {
                tracing::error!("Failed to persist state on item toggle: {e}");
            }
        });
        if let Some(announcer) = announcement {
            let status = if completed_state {
                "completed"
            } else {
                "marked pending"
            };
            announcer.set(format!("Task {status}: {item_title}"));
        }
    };

    let remove_item = move |id: String| {
        let mut item_title = String::new();
        state.update(|s| {
            if let Some(pos) = s.collection.items.iter().position(|i| i.id == id) {
                item_title = s.collection.items[pos].title.clone();
                s.collection.items.remove(pos);
            }
            if let Err(e) = save_state_to_storage(s) {
                tracing::error!("Failed to persist state on item delete: {e}");
            }
        });
        if let Some(announcer) = announcement {
            announcer.set(format!("Deleted task: {item_title}"));
        }
    };

    // Filtered items memo
    let filtered_items = Memo::new(move |_| {
        let current_state = state.get();
        let query = search_query.get().to_lowercase();
        let p_filter = filter_priority.get();
        let allow_completed = show_completed.get();

        current_state
            .collection
            .items
            .into_iter()
            .filter(|item| {
                if !allow_completed && item.completed {
                    return false;
                }
                if let Some(p) = p_filter
                    && item.priority != p
                {
                    return false;
                }
                if !query.is_empty() {
                    let in_title = item.title.to_lowercase().contains(&query);
                    let in_desc = item.description.to_lowercase().contains(&query);
                    if !in_title && !in_desc {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<Item>>()
    });

    view! {
        <div class="content-container">
            // Statistics Summary Banner
            <div class="metrics-grid">
                <div class="metric-card">
                    <span class="metric-label">"Total Items"</span>
                    <span class="metric-value">{move || state.get().collection.total_count()}</span>
                </div>
                <div class="metric-card metric-completed">
                    <span class="metric-label">"Completed"</span>
                    <span class="metric-value">
                        {move || state.get().collection.completed_count()}
                    </span>
                </div>
                <div class="metric-card metric-pending">
                    <span class="metric-label">"Pending Tasks"</span>
                    <span class="metric-value">
                        {move || state.get().collection.pending_count()}
                    </span>
                </div>
                <div class="metric-card metric-urgent">
                    <span class="metric-label">"High & Critical"</span>
                    <span class="metric-value">
                        {move || state.get().collection.high_priority_count()}
                    </span>
                </div>
            </div>

            // Create New Item Card
            <section class="card form-card">
                <h2 class="card-title">"➕ Add New Task or Item"</h2>
                <form
                    on:submit=move |ev| {
                        ev.prevent_default();
                        add_item();
                    }
                    class="create-item-form"
                >
                    <div class="form-row">
                        <div class="form-group flex-2">
                            <label for="item-title">"Title"</label>
                            <input
                                id="item-title"
                                type="text"
                                class="input-text"
                                placeholder="Enter task title (e.g. Test Serverless Deployment)..."
                                prop:value=move || new_title.get()
                                on:input=move |ev| new_title.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="form-group flex-1">
                            <label for="item-priority">"Priority"</label>
                            <select
                                id="item-priority"
                                class="select-input"
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    let p = match val.as_str() {
                                        "Low" => Priority::Low,
                                        "High" => Priority::High,
                                        "Critical" => Priority::Critical,
                                        _ => Priority::Medium,
                                    };
                                    new_priority.set(p);
                                }
                            >
                                <option
                                    value="Medium"
                                    selected=move || new_priority.get() == Priority::Medium
                                >
                                    "Medium"
                                </option>
                                <option
                                    value="Low"
                                    selected=move || new_priority.get() == Priority::Low
                                >
                                    "Low"
                                </option>
                                <option
                                    value="High"
                                    selected=move || new_priority.get() == Priority::High
                                >
                                    "High"
                                </option>
                                <option
                                    value="Critical"
                                    selected=move || new_priority.get() == Priority::Critical
                                >
                                    "Critical"
                                </option>
                            </select>
                        </div>
                    </div>

                    <div class="form-row">
                        <div class="form-group flex-grow">
                            <label for="item-desc">"Description (Optional)"</label>
                            <input
                                id="item-desc"
                                type="text"
                                class="input-text"
                                placeholder="Add extra context or instructions..."
                                prop:value=move || new_description.get()
                                on:input=move |ev| new_description.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="form-group form-actions">
                            <label class="invisible-label">"Action"</label>
                            <button type="submit" class="btn btn-primary">
                                "Add Item"
                            </button>
                        </div>
                    </div>
                </form>
            </section>

            // Filter & Search Controls
            <section class="card filter-card">
                <div class="filter-controls">
                    <div class="search-box">
                        <input
                            type="text"
                            class="input-search"
                            placeholder="Search items by title or description..."
                            prop:value=move || search_query.get()
                            on:input=move |ev| search_query.set(event_target_value(&ev))
                        />
                        {move || {
                            if !search_query.get().is_empty() {
                                view! {
                                    <button
                                        class="btn-clear-search"
                                        on:click=move |_| search_query.set(String::new())
                                    >
                                        "✕"
                                    </button>
                                }
                                    .into_any()
                            } else {
                                view! {}.into_any()
                            }
                        }}
                    </div>

                    <div class="filter-actions">
                        <select
                            class="select-input select-filter"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                let p = match val.as_str() {
                                    "Low" => Some(Priority::Low),
                                    "Medium" => Some(Priority::Medium),
                                    "High" => Some(Priority::High),
                                    "Critical" => Some(Priority::Critical),
                                    _ => None,
                                };
                                filter_priority.set(p);
                            }
                        >
                            <option value="All">"All Priorities"</option>
                            <option value="Critical">"Critical"</option>
                            <option value="High">"High"</option>
                            <option value="Medium">"Medium"</option>
                            <option value="Low">"Low"</option>
                        </select>

                        <label class="toggle-label">
                            <input
                                type="checkbox"
                                prop:checked=move || show_completed.get()
                                on:change=move |ev| show_completed.set(event_target_checked(&ev))
                            />
                            <span>"Show Completed"</span>
                        </label>
                    </div>
                </div>
            </section>

            // Items List / Grid
            <section class="items-section">
                <div class="items-header">
                    <h3>"Items & Tasks"</h3>
                    <span class="items-count-badge">
                        {move || {
                            format!(
                                "Showing {} of {}",
                                filtered_items.get().len(),
                                state.get().collection.total_count(),
                            )
                        }}
                    </span>
                </div>

                <div class="items-list">
                    <For
                        each=move || filtered_items.get()
                        key=|item| item.id.clone()
                        children=move |item| {
                            let item_id = item.id.clone();
                            let item_id_delete = item.id.clone();
                            let is_completed = item.completed;
                            let priority_class = item.priority.badge_class();
                            let priority_label = item.priority.label();

                            view! {
                                <div class=format!(
                                    "item-card {}",
                                    if is_completed { "item-completed" } else { "" },
                                )>
                                    <div class="item-status">
                                        <input
                                            type="checkbox"
                                            class="item-checkbox"
                                            prop:checked=is_completed
                                            on:change=move |_| toggle_item(item_id.clone())
                                            title="Toggle completed"
                                        />
                                    </div>
                                    <div class="item-content">
                                        <div class="item-header-row">
                                            <h4 class="item-title">{item.title}</h4>
                                            <span class=format!(
                                                "badge {}",
                                                priority_class,
                                            )>{priority_label}</span>
                                        </div>
                                        {if !item.description.is_empty() {
                                            view! { <p class="item-description">{item.description}</p> }
                                                .into_any()
                                        } else {
                                            view! {}.into_any()
                                        }}
                                    </div>
                                    <div class="item-actions">
                                        <button
                                            class="btn-delete"
                                            on:click=move |_| remove_item(item_id_delete.clone())
                                            title="Delete item"
                                            aria-label="Delete item"
                                        >
                                            "✕"
                                        </button>
                                    </div>
                                </div>
                            }
                        }
                    />

                    {move || {
                        if filtered_items.get().is_empty() {
                            view! {
                                <div class="empty-state">
                                    <h4>"No items match your filter"</h4>
                                    <p>
                                        "Try adjusting your search query, priority filter, or add a new item above."
                                    </p>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}
                </div>
            </section>
        </div>
    }
}

fn js_sys_time() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}
