//! Telemetry viewer: live event counts and recent log entries from `spodeian-telemetry`.

use leptos::prelude::*;
use spodeian_telemetry::{LogEntry, TelemetryMetrics, global_ring_buffer, metrics_snapshot};

/// How many of the most recent log entries the viewer lists.
const RECENT_ENTRIES: usize = 25;

#[derive(Clone)]
struct Snapshot {
    metrics: TelemetryMetrics,
    /// Newest first.
    entries: Vec<LogEntry>,
}

fn take_snapshot() -> Snapshot {
    let mut entries = global_ring_buffer().snapshot_recent(RECENT_ENTRIES);
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.timestamp_millis));
    Snapshot {
        metrics: metrics_snapshot(),
        entries,
    }
}

fn format_uptime(millis: u64) -> String {
    let secs = millis / 1000;
    match secs {
        0..=59 => format!("{secs}s"),
        60..=3599 => format!("{}m {}s", secs / 60, secs % 60),
        _ => format!("{}h {}m", secs / 3600, (secs % 3600) / 60),
    }
}

/// Seconds between a log entry and the snapshot, as "12s ago".
fn format_age(entry_millis: u64, newest_millis: u64) -> String {
    format!("{}s ago", newest_millis.saturating_sub(entry_millis) / 1000)
}

#[component]
fn Kpi(label: &'static str, value: String, note: &'static str) -> impl IntoView {
    view! {
        <div class="kpi-card">
            <div class="kpi-label">{label}</div>
            <div class="kpi-value">{value}</div>
            <div class="kpi-note">{note}</div>
        </div>
    }
}

#[component]
pub fn TelemetryModal(
    is_open: RwSignal<bool>,
    #[prop(optional)] announcement: Option<RwSignal<String>>,
) -> impl IntoView {
    let snapshot = RwSignal::new(None::<Snapshot>);
    // Take a fresh snapshot each time the dialog opens.
    Effect::new(move |_| {
        if is_open.get() {
            snapshot.set(Some(take_snapshot()));
        }
    });

    let announce = move |message: &str| {
        if let Some(announcer) = announcement {
            announcer.set(message.to_string());
        }
    };

    let copy_snapshot = move |_| {
        let Some(current) = snapshot.get_untracked() else {
            return;
        };
        let Ok(json) = serde_json::to_string_pretty(&(&current.metrics, &current.entries)) else {
            announce("Could not prepare the telemetry snapshot");
            return;
        };
        // Only confirm the copy once the browser reports that it happened.
        leptos::task::spawn_local(async move {
            match spodeian_web_utils::write_clipboard_text(&json).await {
                Ok(()) => announce("Telemetry snapshot copied to clipboard"),
                Err(_) => announce("Could not copy: the browser blocked clipboard access"),
            }
        });
    };

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
                    {move || {
                        snapshot
                            .get()
                            .map(|snap| {
                                let m = snap.metrics.clone();
                                let newest = snap
                                    .entries
                                    .first()
                                    .map(|e| e.timestamp_millis)
                                    .unwrap_or(0);
                                let (badge_class, badge_text) = if m.errors == 0 {
                                    ("badge badge-success", "No errors logged".to_string())
                                } else {
                                    ("badge badge-danger", format!("{} errors logged", m.errors))
                                };
                                view! {
                                    <div class="modal-header">
                                        <h2 id="telemetry-title">"Telemetry"</h2>
                                        <span class=badge_class>{badge_text}</span>
                                    </div>

                                    <div class="telemetry-kpis">
                                        <Kpi
                                            label="Uptime"
                                            value=format_uptime(m.uptime_millis)
                                            note="since the app started"
                                        />
                                        <Kpi
                                            label="Events"
                                            value=m.total_events().to_string()
                                            note="logged this session"
                                        />
                                        <Kpi
                                            label="Warnings"
                                            value=m.warnings.to_string()
                                            note="logged this session"
                                        />
                                        <Kpi
                                            label="Errors"
                                            value=m.errors.to_string()
                                            note="logged this session"
                                        />
                                    </div>

                                    <h3 class="telemetry-heading">
                                        {format!(
                                            "Recent log entries ({} of {} buffered)",
                                            snap.entries.len(),
                                            m.buffered_entries,
                                        )}
                                    </h3>
                                    <div class="telemetry-log">
                                        <table>
                                            <thead>
                                                <tr>
                                                    <th scope="col">"When"</th>
                                                    <th scope="col">"Level"</th>
                                                    <th scope="col">"Source"</th>
                                                    <th scope="col">"Message"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {snap
                                                    .entries
                                                    .into_iter()
                                                    .map(|entry| {
                                                        view! {
                                                            <tr>
                                                                <td>{format_age(entry.timestamp_millis, newest)}</td>
                                                                <td>{entry.level}</td>
                                                                <td>{entry.target}</td>
                                                                <td>{entry.message}</td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()}
                                            </tbody>
                                        </table>
                                    </div>
                                }
                            })
                    }}
                    <div class="modal-footer telemetry-footer">
                        <button class="btn btn-secondary" on:click=copy_snapshot>
                            "Copy snapshot"
                        </button>
                        <button
                            class="btn btn-secondary"
                            on:click=move |_| snapshot.set(Some(take_snapshot()))
                        >
                            "Refresh"
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

#[cfg(test)]
mod tests {
    use super::{format_age, format_uptime};

    #[test]
    fn uptime_is_readable_at_every_scale() {
        assert_eq!(format_uptime(999), "0s");
        assert_eq!(format_uptime(61_000), "1m 1s");
        assert_eq!(format_uptime(3_720_000), "1h 2m");
    }

    #[test]
    fn age_never_goes_negative() {
        assert_eq!(format_age(5_000, 17_000), "12s ago");
        assert_eq!(format_age(9_000, 1_000), "0s ago");
    }
}
