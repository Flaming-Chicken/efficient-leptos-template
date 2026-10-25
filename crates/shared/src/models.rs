//! Domain models and core state definitions.

use serde::{Deserialize, Serialize};

pub use spodeian_tokens::ThemeMode;

/// Task item priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }

    pub fn badge_class(self) -> &'static str {
        match self {
            Self::Low => "priority-low",
            Self::Medium => "priority-medium",
            Self::High => "priority-high",
            Self::Critical => "priority-critical",
        }
    }
}

/// Core domain item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub completed: bool,
    pub created_at: u64,
}

impl Item {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        priority: Priority,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            priority,
            completed: false,
            created_at: 0,
        }
    }
}

/// Collection wrapper for item operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ItemCollection {
    pub items: Vec<Item>,
}

impl ItemCollection {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_sample_data() -> Self {
        Self {
            items: vec![
                Item {
                    id: "item-1".to_string(),
                    title: "Deploy Serverless WebApp".to_string(),
                    description:
                        "Publish high-performance WASM application directly to Cloudflare Pages"
                            .to_string(),
                    priority: Priority::Critical,
                    completed: false,
                    created_at: 1,
                },
                Item {
                    id: "item-2".to_string(),
                    title: "Configure Native Desktop (Tauri)".to_string(),
                    description:
                        "Verify cross-platform desktop compilation and local asset loading"
                            .to_string(),
                    priority: Priority::High,
                    completed: false,
                    created_at: 2,
                },
                Item {
                    id: "item-3".to_string(),
                    title: "Verify Offline PWA Caching".to_string(),
                    description: "Check Service Worker caching strategy for offline web support"
                        .to_string(),
                    priority: Priority::Medium,
                    completed: true,
                    created_at: 3,
                },
                Item {
                    id: "item-4".to_string(),
                    title: "Test JSON & CSV Interchange".to_string(),
                    description:
                        "Import and export data with clipboard toasts and direct downloads"
                            .to_string(),
                    priority: Priority::Low,
                    completed: false,
                    created_at: 4,
                },
            ],
        }
    }

    pub fn add(&mut self, item: Item) {
        self.items.push(item);
    }

    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.items.iter().position(|i| i.id == id) {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn toggle_completed(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.completed = !item.completed;
        }
    }

    pub fn total_count(&self) -> usize {
        self.items.len()
    }

    pub fn completed_count(&self) -> usize {
        self.items.iter().filter(|i| i.completed).count()
    }

    pub fn pending_count(&self) -> usize {
        self.items.iter().filter(|i| !i.completed).count()
    }

    pub fn high_priority_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| !i.completed && matches!(i.priority, Priority::High | Priority::Critical))
            .count()
    }
}

/// Global application configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub theme: ThemeMode,
    pub auto_save: bool,
    pub show_completed: bool,
    pub filter_priority: Option<Priority>,
    pub search_query: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
            auto_save: true,
            show_completed: true,
            filter_priority: None,
            search_query: String::new(),
        }
    }
}

/// Root state container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppState {
    pub collection: ItemCollection,
    pub config: AppConfig,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            collection: ItemCollection::with_sample_data(),
            config: AppConfig::default(),
        }
    }
}
