use leptos::*;
use crate::types::{
    AppView, WorkoutData, SetRecord, ExerciseRecord, ExerciseStats,
};
use crate::storage;
use crate::stats::{self, MuscleGroup, ProgressStatus, BIG_FOUR};

fn format_time(secs: i64) -> String {
    let mins = secs / 60;
    let s = secs % 60;
    format!("{:02}:{:02}", mins, s)
}

fn format_date(ts: i64) -> String {
    let now = js_sys::Date::now() as i64 / 1000;
    let diff = now - ts;
    
    if diff < 86400 { "Idag".to_string() }
    else if diff < 172800 { "Igår".to_string() }
    else { format!("{} dagar sedan", diff / 86400) }
}

fn format_weight(w: f64) -> String {
    if w.fract() == 0.0 { format!("{:.0}", w) }
    else { format!("{:.1}", w) }
}

// Parse target reps from string like "5-8" or "5" or "AMRAP"
// Returns (min, max)
fn parse_target_range(target: &str) -> (u8, u8) {
    if target.contains("AMRAP") { return (8, 15); }
    if let Some(dash) = target.find('-') {
        let min = target[..dash].parse().unwrap_or(5);
        let max = target[dash+1..].parse().unwrap_or(min + 3);
        (min, max)
    } else {
        let n = target.parse().unwrap_or(5);
        (n, n)
    }
}

fn parse_target_reps(target: &str) -> u8 {
    parse_target_range(target).0
}

#[component]
pub fn App() -> impl IntoView {
    let (view, set_view) = create_signal(AppView::Dashboard);
    
    view! {
        <div class="app">
            {move || match view.get() {
                AppView::Dashboard => view! { <Dashboard set_view=set_view /> }.into_view(),
                AppView::Workout(routine) => view! { <Workout routine=routine set_view=set_view /> }.into_view(),
                AppView::Stats => view! { <Stats set_view=set_view /> }.into_view(),
            }}
        </div>
    }
}

#[component]
fn Dashboard(set_view: WriteSignal<AppView>) -> impl IntoView {
    let db = storage::load_data();
    let total = db.get_total_stats();
    let recent = db.get_recent_sessions(3);

    view! {
        <div class="dashboard">
            <div class="logo">"OXIDIZE"</div>
            
            <div class="quick-stats">
                <div class="quick-stat">
                    <span class="quick-stat-value">{total.total_sessions}</span>
                    <span class="quick-stat-label">"pass"</span>
                </div>
                <div class="quick-stat">
                    <span class="quick-stat-value">{format!("{:.0}", total.total_volume / 1000.0)}</span>
                    <span class="quick-stat-label">"ton"</span>
                </div>
            </div>
            
            <button 
                class="start-btn pass-a"
                on:click=move |_| set_view.set(AppView::Workout("A".to_string()))
            >
                <span class="start-btn-label">"Pass A"</span>
                <span class="start-btn-focus">"Ben · Press · Triceps"</span>
            </button>
            
            <button 
                class="start-btn pass-b"
                on:click=move |_| set_view.set(AppView::Workout("B".to_string()))
            >
                <span class="start-btn-label">"Pass B"</span>
                <span class="start-btn-focus">"Rygg · Axlar · Biceps"</span>
            </button>

            {(!recent.is_empty()).then(|| view! {
                <div class="recent-sessions">
                    <div class="recent-title">"Senaste"</div>
                    {recent.into_iter().map(|s| {
                        let routine_class = if s.routine.contains("A") { "pass-a" } else { "pass-b" };
                        view! {
                            <div class="recent-item">
                                <span class=format!("recent-routine {}", routine_class)>{&s.routine}</span>
                                <span class="recent-date">{format_date(s.timestamp)}</span>
                                <span class="recent-duration">{format_time(s.duration_secs)}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            })}
            
            <div class="stats-link" on:click=move |_| set_view.set(AppView::Stats)>
                "Statistik →"
            </div>
        </div>
    }
}

#[component]
fn Workout(routine: String, set_view: WriteSignal<AppView>) -> impl IntoView {
    let data = storage::get_workout(&routine);
    
    match data {
        Some(d) => view! { <WorkoutActive data=d set_view=set_view /> }.into_view(),
        None => view! { <div class="loading">"Kunde inte ladda pass"</div> }.into_view(),
    }
}

#[component]
fn WorkoutActive(data: WorkoutData, set_view: WriteSignal<AppView>) -> impl IntoView {
    let routine = data.routine.clone();
    let routine_name = routine.name.clone();
    let routine_name_save = routine_name.clone();
    let is_pass_a = routine_name.contains("A");
    
    // State
    let (exercises, set_exercises) = create_signal(data.exercises);
    let (current_idx, set_current_idx) = create_signal(0usize);
    let (start_time, _) = create_signal(js_sys::Date::now() as i64 / 1000);
    let (elapsed, set_elapsed) = create_signal(0i64);
    let (last_set_time, set_last_set_time) = create_signal(0i64);
    let (rest_elapsed, set_rest_elapsed) = create_signal(0i64);
    let (is_resting, set_is_resting) = create_signal(false);
    let (is_finished, set_is_finished) = create_signal(false);
    
    // Timer
    create_effect(move |_| {
        let handle = gloo_timers::callback::Interval::new(1000, move || {
            let now = js_sys::Date::now() as i64 / 1000;
            set_elapsed.set(now - start_time.get());
            if is_resting.get() && last_set_time.get() > 0 {
                set_rest_elapsed.set(now - last_set_time.get());
            }
        });
        on_cleanup(move || drop(handle));
    });
    
    // Current exercise info
    let current_exercise = move || exercises.get().get(current_idx.get()).cloned();
    let current_set_num = move || {
        current_exercise().map(|e| e.sets_completed.len() + 1).unwrap_or(1)
    };
    let total_sets = move || {
        current_exercise().map(|e| e.exercise.sets as usize).unwrap_or(0)
    };
    let current_weight = move || {
        current_exercise().map(|e| e.current_weight).unwrap_or(0.0)
    };
    let _target_reps = move || {
        current_exercise().map(|e| parse_target_reps(&e.exercise.reps_target)).unwrap_or(5)
    };
    let target_reps_str = move || {
        current_exercise().map(|e| e.exercise.reps_target.clone()).unwrap_or_default()
    };
    
    // Complete a set
    let complete_set = move |reps: u8| {
        let now = js_sys::Date::now() as i64 / 1000;
        let rest = if last_set_time.get() > 0 { Some(now - last_set_time.get()) } else { None };
        let idx = current_idx.get();
        
        // Get current state BEFORE update
        let exs = exercises.get();
        let sets_done = exs[idx].sets_completed.len();
        let sets_target = exs[idx].exercise.sets as usize;
        let is_last_exercise = idx + 1 >= exs.len();
        
        // Add the set
        set_exercises.update(|exs| {
            if let Some(ex) = exs.get_mut(idx) {
                ex.sets_completed.push(SetRecord {
                    weight: ex.current_weight,
                    reps,
                    timestamp: now,
                    rest_before_secs: rest,
                });
            }
        });
        
        set_last_set_time.set(now);
        set_rest_elapsed.set(0);
        
        // Check if we just completed the LAST set of this exercise
        if sets_done + 1 >= sets_target {
            // Move to next exercise
            if !is_last_exercise {
                set_current_idx.set(idx + 1);
            } else {
                // All exercises done!
                set_is_finished.set(true);
                return;
            }
        }
        // Show rest screen (whether staying on same exercise or moving to next)
        set_is_resting.set(true);
    };
    
    // Continue from rest
    let continue_workout = move |_| {
        set_is_resting.set(false);
    };
    
    // Skip current exercise
    let skip_exercise = move |_| {
        let idx = current_idx.get();
        let exs = exercises.get();
        let is_last = idx + 1 >= exs.len();
        
        if is_last {
            // This was the last exercise, go to finish
            set_is_finished.set(true);
        } else {
            // Move to next exercise
            set_current_idx.set(idx + 1);
            set_is_resting.set(false);
        }
    };
    
    // Adjust weight
    let adjust_weight = move |delta: f64| {
        let idx = current_idx.get();
        set_exercises.update(|exs| {
            if let Some(ex) = exs.get_mut(idx) {
                ex.current_weight = (ex.current_weight + delta).max(0.0);
            }
        });
    };
    
    // Store routine name in a signal so it can be accessed from nested closures
    let (routine_name_sig, _) = create_signal(routine_name_save);
    
    let accent_class = if is_pass_a { "accent-a" } else { "accent-b" };

    view! {
        <div class="workout">
            // Header
            <div class="workout-header">
                <div class=format!("workout-title {}", accent_class)>{&routine.name}</div>
                <div class="workout-timer">{move || format_time(elapsed.get())}</div>
            </div>
            
            // Main content
            <div class="workout-main">
                {move || {
                    if is_finished.get() {
                        // Finished view
                        view! {
                            <div class="finish-screen">
                                <div class="finish-icon">"✓"</div>
                                <div class="finish-title">"Bra jobbat!"</div>
                                <div class="finish-time">{format_time(elapsed.get())}</div>
                                <button class="finish-save-btn" on:click=move |_| {
                                    let exs = exercises.get();
                                    let records: Vec<ExerciseRecord> = exs.iter()
                                        .filter(|e| !e.sets_completed.is_empty())
                                        .map(|e| ExerciseRecord {
                                            name: e.exercise.name.clone(),
                                            sets: e.sets_completed.clone(),
                                        })
                                        .collect();
                                    storage::save_session(routine_name_sig.get(), records, elapsed.get());
                                    set_view.set(AppView::Dashboard);
                                }>
                                    "Spara pass"
                                </button>
                            </div>
                        }.into_view()
                    } else if is_resting.get() {
                        // Rest view
                        let next_ex = current_exercise();
                        let next_name = next_ex.as_ref().map(|e| e.exercise.name.clone()).unwrap_or_default();
                        let next_set = current_set_num();
                        let next_total = total_sets();
                        
                        view! {
                            <div class="rest-screen">
                                <div class="rest-label">"VILA"</div>
                                <div class="rest-timer">{move || format_time(rest_elapsed.get())}</div>
                                <div class="rest-next">
                                    <span class="rest-next-label">"Nästa:"</span>
                                    <span class="rest-next-exercise">{next_name}</span>
                                    <span class="rest-next-set">{format!("Set {}/{}", next_set, next_total)}</span>
                                </div>
                                <button class="rest-continue-btn" on:click=continue_workout>
                                    "Fortsätt"
                                </button>
                            </div>
                        }.into_view()
                    } else {
                        // Active exercise view
                        let ex = current_exercise();
                        let ex_name = ex.as_ref().map(|e| e.exercise.name.clone()).unwrap_or_default();
                        let is_superset = ex.as_ref().map(|e| e.exercise.is_superset).unwrap_or(false);
                        let ss_with = ex.as_ref().and_then(|e| e.exercise.superset_with.clone());
                        
                        view! {
                            <div class="exercise-screen">
                                // Progress
                                <div class="exercise-progress">
                                    {move || format!("Set {} av {}", current_set_num(), total_sets())}
                                </div>
                                
                                // Superset badge
                                {is_superset.then(|| view! {
                                    <div class="superset-indicator">
                                        "Superset → " {ss_with.unwrap_or_default()}
                                    </div>
                                })}
                                
                                // Exercise name
                                <div class="exercise-name-big">{ex_name}</div>
                                
                                // Weight with controls
                                <div class="weight-section">
                                    <button class="weight-adjust" on:click=move |_| adjust_weight(-2.5)>
                                        "−"
                                    </button>
                                    <div class="weight-display-big">
                                        <span class="weight-value">{move || format_weight(current_weight())}</span>
                                        <span class="weight-unit">"kg"</span>
                                    </div>
                                    <button class="weight-adjust" on:click=move |_| adjust_weight(2.5)>
                                        "+"
                                    </button>
                                </div>
                                
                                // Rep buttons - dynamically centered around target
                                <div class="rep-label">"Tryck antal reps:"</div>
                                <div class="rep-buttons">
                                    {move || {
                                        let (min, max) = current_exercise()
                                            .map(|e| parse_target_range(&e.exercise.reps_target))
                                            .unwrap_or((5, 8));
                                        
                                        // Center the 12-button grid around the target
                                        let center = (min + max) / 2;
                                        let start = (center as i32 - 5).max(1) as u8;
                                        let end = start + 11; // 12 buttons total
                                        
                                        (start..=end).map(|r| {
                                            let is_target = r >= min && r <= max;
                                            let btn_class = if is_target { "rep-button target" } else { "rep-button" };
                                            view! {
                                                <button 
                                                    class=btn_class
                                                    on:click=move |_| complete_set(r)
                                                >
                                                    {r}
                                                </button>
                                            }
                                        }).collect_view()
                                    }}
                                </div>
                                <div class="rep-target-hint">
                                    {move || format!("Mål: {}", target_reps_str())}
                                </div>
                                
                                // Skip button
                                <button class="skip-exercise-btn" on:click=skip_exercise>
                                    "Hoppa över övning →"
                                </button>
                            </div>
                        }.into_view()
                    }
                }}
            </div>
            
            // Footer
            <div class="workout-footer">
                <button class="cancel-workout-btn" on:click=move |_| set_view.set(AppView::Dashboard)>
                    "Avbryt pass"
                </button>
            </div>
        </div>
    }
}

#[component]
fn Stats(set_view: WriteSignal<AppView>) -> impl IntoView {
    let db = storage::load_data();
    let initial_bodyweight = db.get_bodyweight();
    let summary = stats::get_stats_summary(&db, initial_bodyweight);
    let power_history = stats::get_power_score_history(&db);
    let all_stats = db.get_all_exercise_stats();
    let sessions = db.get_recent_sessions(10);
    
    // Reactive bodyweight signal
    let (bodyweight, set_bodyweight) = create_signal(initial_bodyweight);
    
    // State for bodyweight input
    let (editing_weight, set_editing_weight) = create_signal(false);
    let (weight_input, set_weight_input) = create_signal(format!("{:.1}", initial_bodyweight));
    
    let save_bodyweight = move |_| {
        if let Ok(w) = weight_input.get().parse::<f64>() {
            // Update reactive signal (updates UI immediately)
            set_bodyweight.set(w);
            set_weight_input.set(format!("{:.1}", w));
            
            // Save to local storage
            let mut db = storage::load_data();
            db.set_bodyweight(w);
            let _ = storage::save_data(&db);
            
            // Sync to cloud
            crate::supabase::save_bodyweight_to_cloud(w);
        }
        set_editing_weight.set(false);
    };
    
    // Get progressive overload status for last session
    let last_session = sessions.first().cloned();
    let overload_statuses: Vec<(String, ProgressStatus)> = if let Some(ref session) = last_session {
        session.exercises.iter()
            .map(|e| {
                let status = stats::check_progressive_overload(&db, &e.name, session);
                (e.name.clone(), status)
            })
            .collect()
    } else {
        vec![]
    };
    let overload_statuses_clone = overload_statuses.clone();
    
    // E1RM for big 4
    let big4_e1rm: Vec<(String, f64)> = BIG_FOUR.iter()
        .map(|&name| (name.to_string(), *summary.e1rm_by_exercise.get(name).unwrap_or(&0.0)))
        .collect();

    view! {
        <div class="stats">
            <div class="stats-header">
                <button class="back-btn" on:click=move |_| set_view.set(AppView::Dashboard)>
                    "←"
                </button>
                <div class="stats-title">"Statistik"</div>
                <div></div>
            </div>
            
            <div class="stats-body">
                // ═══════════════════════════════════════════════════════════════
                // 1. POWER SCORE (E1RM) - Main Hero Metric
                // ═══════════════════════════════════════════════════════════════
                <div class="stat-card hero-card">
                    <div class="hero-label">"POWER SCORE"</div>
                    <div class="hero-value">{format!("{:.0}", summary.power_score)}<span class="hero-unit">" kg"</span></div>
                    <div class="hero-subtitle">"Summa E1RM (De fyra stora)"</div>
                    
                    // Big 4 breakdown
                    <div class="big4-grid">
                        {big4_e1rm.iter().map(|(name, e1rm)| {
                            let name = name.clone();
                            let e1rm = *e1rm;
                            view! {
                                <div class="big4-item">
                                    <span class="big4-name">{name}</span>
                                    <span class="big4-value">{format!("{:.0}", e1rm)}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                    
                    // Mini trend chart
                    {(!power_history.is_empty()).then(|| {
                        let max = power_history.iter().map(|(_, v)| *v).fold(0.0, f64::max);
                        let bars: Vec<f64> = power_history.iter()
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
                    })}
                </div>
                
                // ═══════════════════════════════════════════════════════════════
                // 2. POWER-TO-WEIGHT RATIO
                // ═══════════════════════════════════════════════════════════════
                <div class="stat-card">
                    <div class="stat-card-title">"Relativ Styrka"</div>
                    <div class="ptw-section">
                        <div class="ptw-main">
                            <span class="ptw-value">
                                {move || {
                                    let bw = bodyweight.get();
                                    if bw > 0.0 {
                                        format!("{:.2}", summary.power_score / bw)
                                    } else {
                                        "—".to_string()
                                    }
                                }}
                            </span>
                            <span class="ptw-unit">"× kroppsvikt"</span>
                        </div>
                        
                        // Bodyweight editor
                        <div class="bodyweight-edit">
                            {move || {
                                if editing_weight.get() {
                                    view! {
                                        <div class="bw-input-row">
                                            <input 
                                                type="number" 
                                                class="bw-input"
                                                prop:value=weight_input
                                                on:input=move |ev| set_weight_input.set(event_target_value(&ev))
                                            />
                                            <span class="bw-kg">"kg"</span>
                                            <button class="bw-save" on:click=save_bodyweight>"✓"</button>
                                        </div>
                                    }.into_view()
                                } else {
                                    let bw = bodyweight.get();
                                    view! {
                                        <div class="bw-display" on:click=move |_| {
                                            set_weight_input.set(format!("{:.1}", bodyweight.get()));
                                            set_editing_weight.set(true);
                                        }>
                                            <span class="bw-label">"Din vikt: "</span>
                                            <span class="bw-value">{format!("{:.1}", bw)}</span>
                                            <span class="bw-kg">" kg"</span>
                                            <span class="bw-edit-icon">" ✎"</span>
                                        </div>
                                    }.into_view()
                                }
                            }}
                        </div>
                    </div>
                    <div class="ptw-hint">"Ju mer vikt du tappar med bibehållen styrka, desto högre ratio"</div>
                </div>
                
                // ═══════════════════════════════════════════════════════════════
                // 3. EFFICIENCY (kg/min)
                // ═══════════════════════════════════════════════════════════════
                <div class="stat-card">
                    <div class="stat-card-title">"Effektivitet"</div>
                    <div class="efficiency-row">
                        <div class="efficiency-main">
                            <span class="efficiency-value">{format!("{:.1}", summary.avg_efficiency)}</span>
                            <span class="efficiency-unit">"kg/min"</span>
                        </div>
                        <div class="efficiency-explain">
                            "Snitt volym per minut"
                        </div>
                    </div>
                    // Show last 5 sessions efficiency
                    {(!sessions.is_empty()).then(|| {
                        let eff_history: Vec<f64> = sessions.iter().take(5)
                            .map(|s| stats::calculate_efficiency(s))
                            .collect();
                        let max = eff_history.iter().cloned().fold(0.0, f64::max);
                        view! {
                            <div class="efficiency-bars">
                                {eff_history.into_iter().enumerate().map(|(i, e)| {
                                    let pct = if max > 0.0 { (e / max) * 100.0 } else { 0.0 };
                                    let is_latest = i == 0;
                                    view! {
                                        <div class="eff-bar-wrap">
                                            <div 
                                                class=if is_latest { "eff-bar latest" } else { "eff-bar" }
                                                style=format!("height: {}%", pct.max(10.0))
                                            ></div>
                                            <span class="eff-label">{format!("{:.0}", e)}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }
                    })}
                </div>
                
                // ═══════════════════════════════════════════════════════════════
                // 4. PROGRESSIVE OVERLOAD STREAK
                // ═══════════════════════════════════════════════════════════════
                {(!overload_statuses_clone.is_empty()).then(|| view! {
                    <div class="stat-card">
                        <div class="stat-card-title">"Senaste passet: Progression"</div>
                        <div class="overload-grid">
                            {overload_statuses_clone.iter().map(|(name, status)| {
                                let (icon, class) = match status {
                                    ProgressStatus::Improved => ("🔥", "improved"),
                                    ProgressStatus::Maintained => ("➡️", "maintained"),
                                    ProgressStatus::Regressed => ("⬇️", "regressed"),
                                    ProgressStatus::FirstTime => ("🆕", "first"),
                                };
                                let name = name.clone();
                                view! {
                                    <div class=format!("overload-item {}", class)>
                                        <span class="overload-icon">{icon}</span>
                                        <span class="overload-name">{name}</span>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                        <div class="overload-legend">
                            <span>"🔥 Ökning"</span>
                            <span>"➡️ Stabil"</span>
                            <span>"⬇️ Nedgång"</span>
                        </div>
                    </div>
                })}
                
                // ═══════════════════════════════════════════════════════════════
                // 5. MUSCLE HEATMAP (7 dagar)
                // ═══════════════════════════════════════════════════════════════
                <div class="stat-card">
                    <div class="stat-card-title">"Muskelaktivitet (7 dagar)"</div>
                    <div class="heatmap-grid">
                        {MuscleGroup::all().into_iter().map(|mg| {
                            let count = *summary.muscle_frequency.get(&mg).unwrap_or(&0);
                            let heat_class = match count {
                                0 => "heat-0",
                                1 => "heat-1",
                                2 => "heat-2",
                                3 => "heat-3",
                                _ => "heat-4",
                            };
                            view! {
                                <div class=format!("heatmap-item {}", heat_class)>
                                    <span class="heat-name">{mg.name()}</span>
                                    <span class="heat-count">{count}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                    <div class="heat-legend">
                        <span class="heat-0">"0"</span>
                        <span class="heat-1">"1"</span>
                        <span class="heat-2">"2"</span>
                        <span class="heat-3">"3"</span>
                        <span class="heat-4">"4+"</span>
                    </div>
                </div>
                
                // ═══════════════════════════════════════════════════════════════
                // 6. REST TIME STATS
                // ═══════════════════════════════════════════════════════════════
                <div class="stat-card">
                    <div class="stat-card-title">"Vilotider"</div>
                    <div class="rest-stat-main">
                        <span class="rest-stat-value">{format_time(summary.avg_rest_time as i64)}</span>
                        <span class="rest-stat-label">"snitt vila"</span>
                    </div>
                    <div class="rest-stat-explain">
                        {if summary.avg_rest_time > 150.0 {
                            "⚠️ Längre vila = starkare lyft men längre pass"
                        } else if summary.avg_rest_time > 90.0 {
                            "✓ Optimal vila för styrka"
                        } else if summary.avg_rest_time > 45.0 {
                            "⚡ Kort vila = mer kondition, mindre styrka"
                        } else {
                            "💨 Väldigt snabb – bra för fettförbränning!"
                        }}
                    </div>
                </div>
                
                // ═══════════════════════════════════════════════════════════════
                // Exercise Details
                // ═══════════════════════════════════════════════════════════════
                {if all_stats.is_empty() {
                    view! { <div class="empty-state">"Kör ditt första pass!"</div> }.into_view()
                } else {
                    view! {
                        <div class="exercise-stats-section">
                            <div class="section-title">"Övningar"</div>
                            {all_stats.into_iter().map(|s| {
                                view! { <ExerciseStatsCard stats=s /> }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }}
                
                // ═══════════════════════════════════════════════════════════════
                // History
                // ═══════════════════════════════════════════════════════════════
                {(!sessions.is_empty()).then(|| view! {
                    <div class="stat-card">
                        <div class="stat-card-title">"Historik"</div>
                        {sessions.into_iter().map(|s| {
                            let routine_class = if s.routine.contains("A") { "pass-a" } else { "pass-b" };
                            let eff = stats::calculate_efficiency(&s);
                            view! {
                                <div class="history-item">
                                    <span class=format!("history-routine {}", routine_class)>{&s.routine}</span>
                                    <span class="history-date">{format_date(s.timestamp)}</span>
                                    <span class="history-eff">{format!("{:.0} kg/min", eff)}</span>
                                    <span class="history-duration">{format_time(s.duration_secs)}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                })}
            </div>
        </div>
    }
}

#[component]
fn ExerciseStatsCard(stats: ExerciseStats) -> impl IntoView {
    let bars: Vec<f64> = if stats.one_rm_trend.is_empty() {
        vec![]
    } else {
        let trend: Vec<f64> = stats.one_rm_trend.iter().rev().take(8).map(|(_, v)| *v).collect();
        let max = trend.iter().cloned().fold(0.0, f64::max);
        if max > 0.0 { trend.iter().map(|v| (v / max) * 100.0).collect() }
        else { vec![] }
    };

    view! {
        <div class="stat-card">
            <div class="stat-card-title">{&stats.name}</div>
            <div class="stat-grid">
                <div class="stat-item">
                    <span class="stat-value">{format_weight(stats.current_weight)}</span>
                    <span class="stat-label">"kg nu"</span>
                </div>
                <div class="stat-item">
                    <span class="stat-value highlight">{format_weight(stats.estimated_1rm)}</span>
                    <span class="stat-label">"1RM"</span>
                </div>
            </div>
            {(!bars.is_empty()).then(|| view! {
                <div class="mini-chart">
                    {bars.into_iter().rev().map(|h| {
                        view! { <div class="mini-bar" style=format!("height: {}%", h.max(8.0))></div> }
                    }).collect_view()}
                </div>
            })}
        </div>
    }
}
