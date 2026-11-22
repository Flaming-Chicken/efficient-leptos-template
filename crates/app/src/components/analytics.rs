//! Analytics dashboard, priority SVG chart, and interactive parametric signal explorer.

use crate::storage::trigger_file_download;
use leptos::prelude::*;
use shared::{AppState, ParametricPreviewConfig, export_to_printable_html, export_to_svg};

/// Interactive analytics page showing live priority breakdown bars, damped harmonic SVG curve
/// controls, and one-click `.svg` / printable `.html` report exports.
#[component]
pub fn AnalyticsPage(
    /// Reactive root application state signal.
    state: RwSignal<AppState>,
    /// Optional export modal trigger signal.
    #[prop(optional)]
    show_export_modal: Option<RwSignal<bool>>,
) -> impl IntoView {
    let freq = RwSignal::new(ParametricPreviewConfig::default().frequency_hz);
    let damping = RwSignal::new(ParametricPreviewConfig::default().damping);
    let harmonic = RwSignal::new(ParametricPreviewConfig::default().harmonic_ratio);

    let polyline_points = Memo::new(move |_| {
        let cfg = ParametricPreviewConfig {
            frequency_hz: freq.get(),
            damping: damping.get(),
            harmonic_ratio: harmonic.get(),
        };
        cfg.svg_polyline_points(640.0, 200.0, 96)
    });

    let on_download_svg = move |_| {
        let svg = export_to_svg(&state.get_untracked());
        trigger_file_download("priority_chart.svg", &svg, "image/svg+xml;charset=utf-8");
    };

    let on_download_html = move |_| {
        let html = export_to_printable_html(&state.get_untracked());
        trigger_file_download("workspace_report.html", &html, "text/html;charset=utf-8");
    };

    view! {
        <div class="content-container analytics-page">
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px;">
                <div>
                    <h1 style="font-size: 1.65rem; font-weight: 700;">
                        "Analytics & Interactive SVG Explorer"
                    </h1>
                    <p style="color: var(--text-secondary); font-size: 0.92rem;">
                        "Live task priority metrics, parametric signal visualization, and self-contained vector/HTML exports."
                    </p>
                </div>
                <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                    <button class="btn btn-outline" on:click=on_download_svg>
                        "Download .svg"
                    </button>
                    <button class="btn btn-primary" on:click=on_download_html>
                        "Printable HTML Report"
                    </button>
                    {move || {
                        show_export_modal
                            .map(|sig| {
                                view! {
                                    <button
                                        class="btn btn-secondary"
                                        on:click=move |_| sig.set(true)
                                    >
                                        "All Formats"
                                    </button>
                                }
                            })
                    }}
                </div>
            </div>

            <div class="metrics-grid">
                <div class="metric-card">
                    <span class="metric-label">"Total Items"</span>
                    <span class="metric-value">{move || state.get().collection.total_count()}</span>
                </div>
                <div class="metric-card metric-completed">
                    <span class="metric-label">"Completion Rate"</span>
                    <span class="metric-value">
                        {move || {
                            format!("{:.0}%", state.get().collection.completion_ratio() * 100.0)
                        }}
                    </span>
                </div>
                <div class="metric-card metric-urgent">
                    <span class="metric-label">"High & Critical Pending"</span>
                    <span class="metric-value">
                        {move || state.get().collection.high_priority_count()}
                    </span>
                </div>
            </div>

            // Reactive Priority Distribution SVG Bar Chart (SSR-safe without inner_html)
            <section class="card">
                <h2 class="card-title">"Priority Breakdown"</h2>
                {move || {
                    let s = state.get();
                    let counts = s.collection.priority_counts();
                    let max_c = counts.iter().copied().max().unwrap_or(1).max(1) as f64;
                    let labels = ["Critical", "High", "Medium", "Low"];
                    let fills = [
                        "var(--danger)",
                        "var(--warning)",
                        "var(--accent-primary)",
                        "var(--text-secondary)",
                    ];
                    let bars: Vec<_> = (0..4)
                        .map(|i| {
                            let c = counts[i];
                            let h = if c == 0 {
                                4.0
                            } else {
                                ((c as f64 / max_c) * 120.0).max(10.0)
                            };
                            let y = 148.0 - h;
                            let x = 58.0 + (i as f64) * 145.0;
                            (labels[i], c, fills[i], x, y, h)
                        })
                        .collect();

                    view! {
                        <svg
                            viewBox="0 0 640 190"
                            role="img"
                            aria-label="Task priority breakdown chart"
                            style="width: 100%; height: auto; display: block; background: var(--bg-surface-elevated); border-radius: 8px; border: 1px solid var(--border-color);"
                        >
                            {bars
                                .into_iter()
                                .map(|(label, c, fill, x, y, h)| {
                                    view! {
                                        <g>
                                            <rect
                                                x=format!("{x:.1}")
                                                y=format!("{y:.1}")
                                                width="76"
                                                height=format!("{h:.1}")
                                                rx="6"
                                                fill=fill
                                            />
                                            <text
                                                x=format!("{:.1}", x + 38.0)
                                                y=format!("{:.1}", y - 8.0)
                                                text-anchor="middle"
                                                fill="var(--text-primary)"
                                                font-size="13"
                                                font-weight="700"
                                            >
                                                {c}
                                            </text>
                                            <text
                                                x=format!("{:.1}", x + 38.0)
                                                y="170"
                                                text-anchor="middle"
                                                fill="var(--text-secondary)"
                                                font-size="12"
                                            >
                                                {label}
                                            </text>
                                        </g>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </svg>
                    }
                }}
            </section>

            // Interactive Parametric Signal SVG Plot
            <section class="card">
                <h2 class="card-title">"Parametric Damped Harmonic Explorer"</h2>
                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 14px; margin-bottom: 16px;">
                    <label style="display: flex; flex-direction: column; gap: 4px; font-size: 0.85rem;">
                        <span>{move || format!("Frequency: {:.2} Hz", freq.get())}</span>
                        <input
                            type="range"
                            min="0.5"
                            max="12.0"
                            step="0.25"
                            prop:value=move || freq.get().to_string()
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<f32>() {
                                    freq.set(v);
                                }
                            }
                        />
                    </label>
                    <label style="display: flex; flex-direction: column; gap: 4px; font-size: 0.85rem;">
                        <span>{move || format!("Damping: {:.2}", damping.get())}</span>
                        <input
                            type="range"
                            min="0.0"
                            max="2.5"
                            step="0.05"
                            prop:value=move || damping.get().to_string()
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<f32>() {
                                    damping.set(v);
                                }
                            }
                        />
                    </label>
                    <label style="display: flex; flex-direction: column; gap: 4px; font-size: 0.85rem;">
                        <span>{move || format!("2nd Harmonic: {:.2}", harmonic.get())}</span>
                        <input
                            type="range"
                            min="0.0"
                            max="1.0"
                            step="0.05"
                            prop:value=move || harmonic.get().to_string()
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<f32>() {
                                    harmonic.set(v);
                                }
                            }
                        />
                    </label>
                </div>

                <svg
                    viewBox="0 0 640 200"
                    role="img"
                    aria-label="Interactive damped harmonic waveform"
                    style="width: 100%; height: auto; display: block; background: var(--bg-surface-elevated); border-radius: 8px; border: 1px solid var(--border-color);"
                >
                    <line
                        x1="16"
                        y1="100"
                        x2="624"
                        y2="100"
                        stroke="var(--border-color)"
                        stroke-width="1"
                        stroke-dasharray="4 4"
                    />
                    <polyline
                        fill="none"
                        stroke="var(--accent-primary)"
                        stroke-width="2.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        points=move || polyline_points.get()
                    />
                </svg>

                {
                    let summary = shared::ComputeRuntimeSummary::current();
                    view! {
                        <div style="margin-top: 14px; display: flex; flex-wrap: wrap; gap: 10px; align-items: center; justify-content: space-between; font-size: 0.85rem; color: var(--text-secondary);">
                            <span>
                                <strong>"Compute Engine: "</strong>
                                {summary.runtime_label}
                                {format!(" ({} threads)", summary.parallel_threads)}
                            </span>
                            <span>
                                {if summary.wgsl_shader_ready {
                                    "WGSL Compute Kernel: Ready"
                                } else {
                                    "WGSL Compute Kernel: Unavailable"
                                }}
                            </span>
                        </div>
                        <details style="margin-top: 10px;">
                            <summary style="cursor: pointer; font-weight: 600; font-size: 0.88rem; color: var(--accent-primary);">
                                "View Bundled WebGPU WGSL Compute Kernel (PARAMETRIC_HARMONIC_WGSL_SHADER)"
                            </summary>
                            <pre style="margin-top: 8px; padding: 12px; border-radius: 8px; background: var(--bg-surface-elevated); border: 1px solid var(--border-color); font-family: var(--font-mono); font-size: 0.8rem; overflow-x: auto;">
                                <code>{shared::PARAMETRIC_HARMONIC_WGSL_SHADER}</code>
                            </pre>
                        </details>
                    }
                }
            </section>
        </div>
    }
}
