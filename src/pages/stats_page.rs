use leptos::*;
use crate::types::{AppView, AuthSession};
use crate::storage;
use crate::stats::{self, MuscleGroup, ProgressStatus, BIG_FOUR};
use crate::supabase;
use crate::app::format_date;

#[component]
pub fn Stats(set_view: WriteSignal<AppView>, set_auth: WriteSignal<Option<AuthSession>>) -> impl IntoView {
    let (sync_status, set_sync_status) = create_signal(storage::get_sync_status().to_string());
    let (data_version, set_data_version) = create_signal(0u32);

    create_effect(move |_| {
        let status = sync_status.get();
        if status == "pending" {
            let handle = gloo_timers::callback::Interval::new(200, move || {
                let new_status = storage::get_sync_status();
                if new_status != "pending" {
                    set_sync_status.set(new_status.to_string());
                }
            });
            std::mem::forget(handle);
        } else {
            set_data_version.update(|v| *v += 1);
        }
    });

    let do_logout = move |_| {
        supabase::sign_out();
        set_auth.set(None);
        set_view.set(AppView::Login);
    };

    let load_summary = move || {
        let _ = data_version.get();
        let db = storage::load_data();
        stats::get_stats_summary(&db, db.get_bodyweight().unwrap_or(80.0))
    };

    view! {
        <div class="stats">
            <div class="stats-header">
                <button class="stats-back-btn" on:click=move |_| set_view.set(AppView::Dashboard)>
                    "←"
                </button>
                <div class="stats-title">"Statistik"</div>
                <button class="logout-btn" on:click=do_logout>
                    "Logga ut"
                </button>
            </div>

            <div class="stats-body">

                // 1. STRENGTH TOTAL (hero)
                <div class="stat-card hero-card">
                    <div class="hero-label">"STYRKETOTAL"</div>
                    <div class="hero-value">{move || format!("{:.0}", load_summary().power_score)}<span class="hero-unit">" kg"</span></div>
                    <div class="hero-subtitle">
                        {move || {
                            let s = load_summary();
                            if s.bodyweight > 0.0 {
                                format!("Styrkeratio: {:.2}x kroppsvikt", s.power_score / s.bodyweight)
                            } else {
                                "Summa E1RM (Big 4)".to_string()
                            }
                        }}
                    </div>

                    <div class="big4-grid">
                        {move || {
                            let s = load_summary();
                            BIG_FOUR.iter().map(|&name| {
                                let e1rm = *s.e1rm_by_exercise.get(name).unwrap_or(&0.0);
                                view! {
                                    <div class="big4-item">
                                        <span class="big4-name">{name}</span>
                                        <span class="big4-value">{format!("{:.0}", e1rm)}</span>
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>

                    {move || {
                        let _ = data_version.get();
                        let db = storage::load_data();
                        let ph = stats::get_power_score_history(&db);
                        (!ph.is_empty()).then(|| {
                            let max = ph.iter().map(|(_, v)| *v).fold(0.0, f64::max);
                            let bars: Vec<f64> = ph.iter()
                                .rev().take(10).rev()
                                .map(|(_, v)| if max > 0.0 { (v / max) * 100.0 } else { 0.0 })
                                .collect();
                            view! {
                                <div class="power-chart">
                                    {bars.into_iter().map(|h| {
                                        view! { <div class="power-bar" style=format!("height: {}%", h.max(8.0))></div> }
                                    }).collect_view()}
                                </div>
                            }
                        })
                    }}
                </div>

                // 2. WEEKLY VOLUME PER MUSCLE
                <div class="stat-card">
                    <div class="stat-card-title">"Veckovolym per muskel"</div>
                    <div class="stat-card-hint">"Set senaste 7 dagar (10-20 set/vecka = optimalt)"</div>
                    <div class="volume-grid">
                        {move || {
                            let s = load_summary();
                            MuscleGroup::all().into_iter().map(|mg| {
                                let sets = *s.weekly_sets.get(&mg).unwrap_or(&0);
                                let zone = match sets {
                                    0 => "vol-none",
                                    1..=9 => "vol-low",
                                    10..=20 => "vol-optimal",
                                    _ => "vol-high",
                                };
                                let bar_pct = ((sets as f64 / 20.0) * 100.0).min(100.0);
                                view! {
                                    <div class=format!("volume-row {}", zone)>
                                        <span class="volume-name">{mg.name()}</span>
                                        <div class="volume-bar-track">
                                            <div class="volume-bar-fill" style=format!("width: {}%", bar_pct)></div>
                                            <div class="volume-bar-target"></div>
                                        </div>
                                        <span class="volume-count">{sets}</span>
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>
                </div>

                // 3. PROGRESSION (last session)
                {move || {
                    let _ = data_version.get();
                    let db = storage::load_data();
                    let mut sessions = db.sessions.clone();
                    sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    let statuses: Vec<(String, ProgressStatus)> = if let Some(session) = sessions.first() {
                        session.exercises.iter()
                            .map(|e| {
                                let status = stats::check_progressive_overload(&db, &e.name, session);
                                (e.name.clone(), status)
                            })
                            .collect()
                    } else {
                        vec![]
                    };
                    (!statuses.is_empty()).then(|| view! {
                        <div class="stat-card">
                            <div class="stat-card-title">"Senaste passet"</div>
                            <div class="overload-grid">
                                {statuses.iter().map(|(name, status)| {
                                    let class = match status {
                                        ProgressStatus::Improved => "improved",
                                        ProgressStatus::Maintained => "maintained",
                                        ProgressStatus::Regressed => "regressed",
                                        ProgressStatus::FirstTime => "first",
                                    };
                                    let name = name.clone();
                                    view! {
                                        <div class=format!("overload-item {}", class)>
                                            <span class="overload-icon"></span>
                                            <span class="overload-name">{name}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                            <div class="overload-legend">
                                <span class="legend-item improved"><span class="legend-icon"></span>" Ökning"</span>
                                <span class="legend-item maintained"><span class="legend-icon"></span>" Stabil"</span>
                                <span class="legend-item regressed"><span class="legend-icon"></span>" Nedgång"</span>
                            </div>
                        </div>
                    })
                }}

                // 4. BODYWEIGHT (only if data exists)
                {move || {
                    let _ = data_version.get();
                    let db = storage::load_data();
                    (db.bodyweight_history.len() >= 2).then(|| {
                        view! {
                            <div class="stat-card">
                                <div class="stat-card-title">"Viktutveckling"</div>
                                <WeightChart history=db.bodyweight_history />
                            </div>
                        }
                    })
                }}

            </div>
        </div>
    }
}

#[component]
pub fn WeightChart(history: Vec<crate::storage::BodyweightEntry>) -> impl IntoView {
    let now = (js_sys::Date::now() / 1000.0) as i64;
    let one_year_ago = now - (365 * 24 * 60 * 60);

    let mut sorted = history.clone();
    sorted.sort_by_key(|h| h.timestamp);

    let data: Vec<_> = sorted.into_iter()
        .filter(|h| h.timestamp >= one_year_ago)
        .collect();

    if data.len() < 2 {
        return view! { <div class="empty-chart">"Behöver minst två mätningar"</div> }.into_view();
    }

    let min_val = data.iter().map(|h| h.weight).fold(f64::INFINITY, f64::min);
    let max_val = data.iter().map(|h| h.weight).fold(f64::NEG_INFINITY, f64::max);

    let weight_diff = max_val - min_val;
    let padding_y = (10.0 - weight_diff).max(2.0) / 2.0;
    let min_w = min_val - padding_y;
    let max_w = max_val + padding_y;
    let range = max_w - min_w;

    let padding = 15.0;
    let width = 100.0;
    let height = 60.0;

    let first_ts = data.first().unwrap().timestamp;
    let last_ts = data.last().unwrap().timestamp;
    let time_range = (last_ts - first_ts).max(1) as f64;

    let get_x = |ts: i64| {
        padding + ((ts - first_ts) as f64 / time_range * (width - 2.0 * padding))
    };

    let get_y = |w: f64| {
        height - padding - ((w - min_w) / range * (height - 2.0 * padding))
    };

    let points: String = data.iter()
        .map(|h| format!("{},{}", get_x(h.timestamp), get_y(h.weight)))
        .collect::<Vec<_>>()
        .join(" ");

    view! {
        <div class="weight-chart-container">
            <svg viewBox=format!("0 0 {} {}", width, height) class="weight-chart-svg">
                <line x1=padding y1={get_y(min_val)} x2={width-padding} y2={get_y(min_val)} stroke="#222" stroke-width="0.3" stroke-dasharray="1,1" />
                <text x={padding - 2.0} y={get_y(min_val) + 1.0} font-size="3" fill="#666" text-anchor="end" font-family="var(--font)">{format!("{:.0}", min_val)}</text>

                <line x1=padding y1={get_y(max_val)} x2={width-padding} y2={get_y(max_val)} stroke="#222" stroke-width="0.3" stroke-dasharray="1,1" />
                <text x={padding - 2.0} y={get_y(max_val) + 1.0} font-size="3" fill="#666" text-anchor="end" font-family="var(--font)">{format!("{:.0}", max_val)}</text>

                <polyline points=points class="weight-line" />

                {let first = data.first().unwrap();
                 let last = data.last().unwrap();
                 view! {
                    <circle cx={get_x(first.timestamp)} cy={get_y(first.weight)} r="1.0" class="weight-point" />
                    <circle cx={get_x(last.timestamp)} cy={get_y(last.weight)} r="1.0" class="weight-point" />
                 }}
            </svg>

            <div class="weight-stats-row">
                <div class="weight-stat">
                    <span class="weight-stat-label">{format_date(data.first().unwrap().timestamp)}</span>
                    <span class="weight-stat-val">{format!("{:.1}", data.first().unwrap().weight)}</span>
                </div>
                <div class="weight-stat">
                    <span class="weight-stat-label">"Nu"</span>
                    <span class="weight-stat-val highlight">{format!("{:.1}", data.last().unwrap().weight)}</span>
                </div>
            </div>
        </div>
    }.into_view()
}
