//! Domain models and core state definitions.

use serde::{Deserialize, Serialize};
use spodeian_parallel::prelude::*;
use std::collections::VecDeque;

pub use spodeian_tokens::ThemeMode;

/// Default visual theme used across SSR shell, Trunk desktop shell, and initial app state.
pub const DEFAULT_THEME: ThemeMode = ThemeMode::Dark;

/// Task item priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Priority {
    /// Low-urgency backlog task.
    Low,
    /// Standard priority task.
    #[default]
    Medium,
    /// Important near-term deliverable.
    High,
    /// Blocking or release-critical task.
    Critical,
}

impl Priority {
    /// Human-readable label for UI badges and exports.
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }

    /// CSS badge class name for styling priority pills.
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
    /// Unique identifier for the item.
    pub id: String,
    /// Short display title.
    pub title: String,
    /// Optional extended description.
    pub description: String,
    /// Priority classification.
    pub priority: Priority,
    /// Completion status flag.
    pub completed: bool,
    /// Creation timestamp in Unix milliseconds (bit-preserved across BSON `i64` limits).
    #[serde(default, with = "spodeian_export::u64_i64_compat")]
    pub created_at: u64,
}

impl Item {
    /// Constructs a new uncompleted [`Item`] with `created_at = 0`.
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
    /// Ordered items managed in the application.
    pub items: Vec<Item>,
}

impl ItemCollection {
    /// Creates an empty item collection.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Creates a collection seeded with four representative sample items.
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

    /// Appends an item to the collection.
    pub fn add(&mut self, item: Item) {
        self.items.push(item);
    }

    /// Removes the first item matching `id`, returning `true` if removed.
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.items.iter().position(|i| i.id == id) {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    /// Toggles the `completed` status of the item matching `id`.
    pub fn toggle_completed(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.completed = !item.completed;
        }
    }

    /// Removes all completed items from the collection.
    pub fn clear_completed(&mut self) {
        self.items.retain(|i| !i.completed);
    }

    /// Removes all items from the collection.
    pub fn reset(&mut self) {
        self.items.clear();
    }

    /// Total number of items.
    pub fn total_count(&self) -> usize {
        self.items.len()
    }

    /// Number of completed items.
    pub fn completed_count(&self) -> usize {
        self.items.iter().filter(|i| i.completed).count()
    }

    /// Number of pending items.
    pub fn pending_count(&self) -> usize {
        self.items.iter().filter(|i| !i.completed).count()
    }

    /// Completion fraction in `[0.0, 1.0]`.
    pub fn completion_ratio(&self) -> f32 {
        if self.items.is_empty() {
            0.0
        } else {
            self.completed_count() as f32 / self.total_count() as f32
        }
    }

    /// Number of pending items with [`Priority::High`] or [`Priority::Critical`].
    pub fn high_priority_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| !i.completed && matches!(i.priority, Priority::High | Priority::Critical))
            .count()
    }

    /// Returns item counts grouped by priority in order `(Critical, High, Medium, Low)`.
    pub fn priority_counts(&self) -> [usize; 4] {
        let mut counts = [0usize; 4];
        for item in &self.items {
            match item.priority {
                Priority::Critical => counts[0] += 1,
                Priority::High => counts[1] += 1,
                Priority::Medium => counts[2] += 1,
                Priority::Low => counts[3] += 1,
            }
        }
        counts
    }
}

/// The application's pages. The URL path is the source of truth for which one is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AppPage {
    /// Interactive item management dashboard (`/`).
    #[default]
    Dashboard,
    /// Priority distribution analytics and interactive SVG parametric preview (`/analytics`).
    Analytics,
    /// Architecture overview, storage diagnostics, and shortcuts (`/about`).
    About,
}

impl AppPage {
    /// Every page, in navigation order.
    pub const ALL: [AppPage; 3] = [AppPage::Dashboard, AppPage::Analytics, AppPage::About];

    /// Display label used in navigation controls and page titles.
    pub fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Analytics => "Analytics",
            Self::About => "About",
        }
    }

    /// The URL path that shows this page.
    pub fn path(self) -> &'static str {
        match self {
            Self::Dashboard => "/",
            Self::Analytics => "/analytics",
            Self::About => "/about",
        }
    }

    /// The page shown at `path`, ignoring a trailing slash, or `None` if no page lives there.
    pub fn from_path(path: &str) -> Option<Self> {
        let trimmed = path.trim_end_matches('/');
        let normalized = if trimmed.is_empty() { "/" } else { trimmed };
        Self::ALL.into_iter().find(|page| page.path() == normalized)
    }

    /// One-sentence page description for search engines and link previews.
    pub fn description(self) -> &'static str {
        match self {
            Self::Dashboard => {
                "Manage items with search, priority filters, undo/redo and live statistics, saved in your browser."
            }
            Self::Analytics => {
                "Inspect priority charts, interactive SVG parametric curves and printable HTML/SVG reports."
            }
            Self::About => {
                "How this Leptos template is built: architecture, storage tiers, diagnostics and keyboard shortcuts."
            }
        }
    }
}

/// Bounded generic undo/redo state history stack matching `spodeian_ui::UndoStack`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoStack<T: Clone> {
    max_history: usize,
    undo_stack: VecDeque<T>,
    redo_stack: Vec<T>,
}

impl<T: Clone> Default for UndoStack<T> {
    fn default() -> Self {
        Self::new(64)
    }
}

impl<T: Clone> UndoStack<T> {
    /// Creates an empty undo/redo stack retaining up to `max_history` undo snapshots (at least 1).
    pub fn new(max_history: usize) -> Self {
        Self {
            max_history: max_history.max(1),
            undo_stack: VecDeque::with_capacity(max_history.clamp(1, 128)),
            redo_stack: Vec::new(),
        }
    }

    /// Records `current` state before a mutation and clears any pending redo history.
    pub fn push(&mut self, current: T) {
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(current);
        self.redo_stack.clear();
    }

    /// Reverts to the most recent undo snapshot, pushing `current` onto the redo stack.
    pub fn undo(&mut self, current: T) -> Option<T> {
        let prev = self.undo_stack.pop_back()?;
        self.redo_stack.push(current);
        Some(prev)
    }

    /// Reapplies the most recent undone snapshot, pushing `current` back onto the undo stack.
    pub fn redo(&mut self, current: T) -> Option<T> {
        let next = self.redo_stack.pop()?;
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(current);
        Some(next)
    }

    /// Returns `true` if at least one state is available to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if at least one state is available to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Number of stored undo steps.
    pub fn undo_len(&self) -> usize {
        self.undo_stack.len()
    }

    /// Number of stored redo steps.
    pub fn redo_len(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clears both undo and redo histories.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

/// Parameters controlling the deterministic 2D parametric signal curve in [`AppPage::Analytics`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ParametricPreviewConfig {
    /// Fundamental frequency multiplier in Hz (`0.5..=12.0`).
    pub frequency_hz: f32,
    /// Exponential decay damping factor (`0.0..=2.5`).
    pub damping: f32,
    /// Second-harmonic amplitude ratio (`0.0..=1.0`).
    pub harmonic_ratio: f32,
}

impl Default for ParametricPreviewConfig {
    fn default() -> Self {
        Self {
            frequency_hz: 2.5,
            damping: 0.45,
            harmonic_ratio: 0.35,
        }
    }
}

/// WGSL compute shader (`@compute @workgroup_size(64)`) evaluating the same damped harmonic
/// waveform as [`ParametricPreviewConfig::sample_points`] for WebGPU (`sml-kernel` / `wgpu`) dispatch.
pub const PARAMETRIC_HARMONIC_WGSL_SHADER: &str = r#"struct HarmonicParams {
    frequency_hz: f32,
    damping: f32,
    harmonic_ratio: f32,
    count: u32,
};

@group(0) @binding(0) var<storage, read_write> output_samples: array<f32>;
@group(0) @binding(1) var<uniform> params: HarmonicParams;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= params.count) {
        return;
    }
    let denom = f32(max(params.count - 1u, 1u));
    let t = f32(idx) / denom;
    let angle = 6.28318530718 * params.frequency_hz * t;
    let envelope = exp(-params.damping * t);
    output_samples[idx] = envelope * (sin(angle) + params.harmonic_ratio * cos(2.0 * angle));
}"#;

/// Portable compute runtime capability summary shared across SSR, WASM browsers, and Tauri desktop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeRuntimeSummary {
    /// Active runtime target and concurrency model.
    pub runtime_label: &'static str,
    /// Available hardware parallelism (`>= 1`).
    pub parallel_threads: usize,
    /// Whether `wasm32` SIMD128 vector instructions are enabled at compile time.
    pub simd128_enabled: bool,
    /// Whether bundled WGSL compute shaders are compiled and ready for WebGPU dispatch.
    pub wgsl_shader_ready: bool,
}

impl ComputeRuntimeSummary {
    /// Captures the compute runtime summary for the active compilation target.
    pub fn current() -> Self {
        let simd128_enabled = cfg!(all(target_arch = "wasm32", target_feature = "simd128"));
        let parallel_threads = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(1)
            }
            #[cfg(target_arch = "wasm32")]
            {
                1usize
            }
        };
        let runtime_label = if cfg!(not(target_arch = "wasm32")) {
            "Native / SSR / Tauri (Rayon Parallel Pool)"
        } else if simd128_enabled {
            "WASM32 Browser / Webview (SIMD128 Sequential)"
        } else {
            "WASM32 Portable Sequential"
        };
        Self {
            runtime_label,
            parallel_threads,
            simd128_enabled,
            wgsl_shader_ready: PARAMETRIC_HARMONIC_WGSL_SHADER.contains("@compute"),
        }
    }
}

impl ParametricPreviewConfig {
    /// Generates `count` evenly spaced `(t, y)` samples across `t in [0.0, 1.0]` using
    /// `spodeian_parallel` (Rayon on native/Tauri/SSR hosts; sequential on `wasm32`).
    pub fn sample_points(&self, count: usize) -> Vec<(f32, f32)> {
        if count == 0 {
            return Vec::new();
        }
        let freq = if self.frequency_hz.is_finite() {
            self.frequency_hz.clamp(0.25, 20.0)
        } else {
            2.5
        };
        let damp = if self.damping.is_finite() {
            self.damping.clamp(0.0, 5.0)
        } else {
            0.45
        };
        let harmonic = if self.harmonic_ratio.is_finite() {
            self.harmonic_ratio.clamp(0.0, 1.0)
        } else {
            0.35
        };

        let denom = (count.saturating_sub(1)).max(1) as f32;
        (0..count)
            .into_par_iter()
            .map(|idx| {
                let t = idx as f32 / denom;
                let angle = std::f32::consts::TAU * freq * t;
                let envelope = (-damp * t).exp();
                let y = envelope * (angle.sin() + harmonic * (2.0 * angle).cos());
                (t, y)
            })
            .collect()
    }

    /// Evaluates a parallel harmonic sweep across `multipliers` using `spodeian_parallel`.
    pub fn batch_harmonic_sweep(&self, multipliers: &[f32], count: usize) -> Vec<Vec<(f32, f32)>> {
        (0..multipliers.len())
            .into_par_iter()
            .map(|idx| {
                let scale = multipliers[idx];
                let scaled = Self {
                    frequency_hz: if scale.is_finite() {
                        (self.frequency_hz * scale).clamp(0.25, 20.0)
                    } else {
                        self.frequency_hz
                    },
                    damping: self.damping,
                    harmonic_ratio: self.harmonic_ratio,
                };
                scaled.sample_points(count)
            })
            .collect()
    }

    /// Returns scalar `y` values matching [`PARAMETRIC_HARMONIC_WGSL_SHADER`]'s output buffer layout.
    pub fn evaluate_harmonic_wgsl_cpu_reference(&self, count: usize) -> Vec<f32> {
        self.sample_points(count)
            .into_iter()
            .map(|(_, y)| y)
            .collect()
    }

    /// Formats sampled `(t, y)` coordinates as an SVG `polyline` `points` string inside
    /// `width x height`.
    pub fn svg_polyline_points(&self, width: f32, height: f32, count: usize) -> String {
        let w = width.max(20.0);
        let h = height.max(20.0);
        let pad_x = 16.0;
        let usable_w = (w - 2.0 * pad_x).max(4.0);
        let mid_y = h * 0.5;
        let amp_y = h * 0.36;

        self.sample_points(count)
            .into_iter()
            .map(|(t, y)| {
                let px = pad_x + t * usable_w;
                let py = mid_y - y * amp_y;
                format!("{px:.1},{py:.1}")
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Global application configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Active visual theme.
    pub theme: ThemeMode,
    /// Whether edits automatically persist to browser storage.
    pub auto_save: bool,
    /// Whether completed items remain visible in the dashboard list.
    pub show_completed: bool,
    /// Optional priority filter.
    pub filter_priority: Option<Priority>,
    /// Text filter query across item titles and descriptions.
    pub search_query: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: DEFAULT_THEME,
            auto_save: true,
            show_completed: true,
            filter_priority: None,
            search_query: String::new(),
        }
    }
}

/// Current persisted JSON schema version for [`AppState`].
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

fn default_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

/// Root state container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    /// Schema revision for forward/backwards state migration.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// Managed domain item collection.
    pub collection: ItemCollection,
    /// Persisted application configuration.
    pub config: AppConfig,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            collection: ItemCollection::default(),
            config: AppConfig::default(),
        }
    }
}

impl AppState {
    /// Creates a new [`AppState`] initialized with default sample items.
    pub fn new() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            collection: ItemCollection::with_sample_data(),
            config: AppConfig::default(),
        }
    }

    /// Resets app domain data while preserving configuration preferences like theme.
    pub fn reset_data(&mut self) {
        self.collection.reset();
    }
}
