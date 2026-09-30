//! Telemetry Dashboard Viewer Component for Leptos Web.

use crate::storage::copy_to_clipboard;
use leptos::prelude::*;
use shared::TelemetryDashboardData;

#[component]
pub fn TelemetryModal(
    is_open: RwSignal<bool>,
    #[prop(optional)] announcement: Option<RwSignal<String>>,
) -> impl IntoView {
    let data = RwSignal::new(TelemetryDashboardData::default());

    view! {
        <Show when=move || is_open.get()>
            <div class="modal-backdrop" on:click=move |_| is_open.set(false) role="presentation">
                <div
                    class="modal-card modal-telemetry"
                    on:click=|e| e.stop_propagation()
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="telemetry-title"
                >
                    <div class="modal-header">
                        <h2 id="telemetry-title">"📊 System Performance Telemetry"</h2>
                        <span class="badge badge-success">"● Operational"</span>
                    </div>

                    <div class="telemetry-kpis" style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin: 16px 0;">
                        <div class="kpi-card" style="padding: 10px; border: 1px solid var(--border-color, #333); border-radius: 6px;">
                            <div style="font-size: 11px; opacity: 0.7;">"Uptime"</div>
                            <div style="font-size: 18px; font-weight: bold;">{move || format!("{}s", data.get().uptime_secs)}</div>
                            <div style="font-size: 10px; opacity: 0.6;">"Online Duration"</div>
                        </div>
                        <div class="kpi-card" style="padding: 10px; border: 1px solid var(--border-color, #333); border-radius: 6px;">
                            <div style="font-size: 11px; opacity: 0.7;">"Operations"</div>
                            <div style="font-size: 18px; font-weight: bold;">{move || data.get().total_operations}</div>
                            <div style="font-size: 10px; opacity: 0.6;">"Total Processed"</div>
                        </div>
                        <div class="kpi-card" style="padding: 10px; border: 1px solid var(--border-color, #333); border-radius: 6px;">
                            <div style="font-size: 11px; opacity: 0.7;">"Success Rate"</div>
                            <div style="font-size: 18px; font-weight: bold;">{move || format!("{:.2}%", data.get().success_rate)}</div>
                            <div style="font-size: 10px; opacity: 0.6;">"Zero-Fault SLA"</div>
                        </div>
                        <div class="kpi-card" style="padding: 10px; border: 1px solid var(--border-color, #333); border-radius: 6px;">
                            <div style="font-size: 11px; opacity: 0.7;">"Throughput"</div>
                            <div style="font-size: 18px; font-weight: bold;">{move || format!("{:.0} ops/s", data.get().throughput_ops_sec)}</div>
                            <div style="font-size: 10px; opacity: 0.6;">"Peak Velocity"</div>
                        </div>
                    </div>

                    <div class="telemetry-channels" style="margin-top: 16px;">
                        <h4 style="margin-bottom: 8px;">"Subsystem Channels & Health Telemetry"</h4>
                        <table style="width: 100%; border-collapse: collapse; font-size: 13px;">
                            <thead>
                                <tr style="border-bottom: 1px solid var(--border-color, #444); text-align: left;">
                                    <th style="padding: 6px;">"Channel"</th>
                                    <th style="padding: 6px;">"Processed"</th>
                                    <th style="padding: 6px;">"Latency"</th>
                                    <th style="padding: 6px;">"Status"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                    data.get().channels.into_iter().map(|ch| {
                                        view! {
                                            <tr style="border-bottom: 1px solid var(--border-subtle, #222);">
                                                <td style="padding: 6px;">{ch.name}</td>
                                                <td style="padding: 6px;">{ch.count}</td>
                                                <td style="padding: 6px;">{format!("{:.2} ms", ch.latency_ms)}</td>
                                                <td style="padding: 6px; color: var(--color-success, #4ade80);">{ch.status}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()
                                }}
                            </tbody>
                        </table>
                    </div>

                    <div class="modal-footer" style="margin-top: 20px; display: flex; justify-content: space-between;">
                        <button
                            class="btn btn-secondary"
                            on:click=move |_| {
                                if let Ok(json) = serde_json::to_string_pretty(&data.get()) {
                                    copy_to_clipboard(&json);
                                    if let Some(announcer) = announcement {
                                        announcer.set("Telemetry snapshot copied to clipboard".to_string());
                                    }
                                }
                            }
                        >
                            "📋 Copy Telemetry Snapshot"
                        </button>
                        <button class="btn btn-primary" on:click=move |_| is_open.set(false)>
                            "Close"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
