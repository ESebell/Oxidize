use leptos::*;
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;
use crate::types::{
    AppView, WorkoutData, SetRecord, ExerciseRecord, ExerciseStats, ExerciseWorkoutState, AuthSession,
};
use crate::storage;
use crate::stats::{self, MuscleGroup, ProgressStatus, BIG_FOUR};
use crate::supabase;

fn format_time(secs: i64) -> String {
    let mins = secs / 60;
    let s = secs % 60;
    format!("{:02}:{:02}", mins, s)
}

fn format_date(ts: i64) -> String {
    // Get current date in local timezone
    let now_date = js_sys::Date::new_0();
    let now_day = now_date.get_date();
    let now_month = now_date.get_month();
    let now_year = now_date.get_full_year();
    
    // Get timestamp date in local timezone
    let ts_date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64((ts * 1000) as f64));
    let ts_day = ts_date.get_date();
    let ts_month = ts_date.get_month();
    let ts_year = ts_date.get_full_year();
    
    // Compare calendar dates
    if ts_year == now_year && ts_month == now_month && ts_day == now_day {
        "Idag".to_string()
    } else {
        // Check if it was yesterday
        let yesterday = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
            js_sys::Date::now() - 86400.0 * 1000.0
        ));
        let yday = yesterday.get_date();
        let ymonth = yesterday.get_month();
        let yyear = yesterday.get_full_year();
        
        if ts_year == yyear && ts_month == ymonth && ts_day == yday {
            "Igår".to_string()
        } else {
            let diff_days = ((js_sys::Date::now() / 1000.0) as i64 - ts) / 86400;
            format!("{} dgr sen", diff_days)
        }
    }
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
    // Check if user is already logged in
    let initial_view = if supabase::load_auth_session().is_some() {
        AppView::Dashboard
    } else {
        AppView::Login
    };
    
    let (view, set_view) = create_signal(initial_view);
    let (auth, set_auth) = create_signal(supabase::load_auth_session());
    
    view! {
        <div class="app">
            {move || match view.get() {
                AppView::Login => view! { <Login set_view=set_view set_auth=set_auth /> }.into_view(),
                AppView::Register => view! { <Register set_view=set_view set_auth=set_auth /> }.into_view(),
                AppView::Dashboard => view! { <Dashboard set_view=set_view auth=auth /> }.into_view(),
                AppView::Workout(routine) => view! { <Workout routine=routine set_view=set_view /> }.into_view(),
                AppView::Stats => view! { <Stats set_view=set_view auth=auth set_auth=set_auth /> }.into_view(),
                AppView::Settings => view! { <Settings set_view=set_view auth=auth set_auth=set_auth /> }.into_view(),
                AppView::RoutineBuilder(id) => view! { <RoutineBuilder routine_id=id set_view=set_view /> }.into_view(),
            }}
        </div>
    }
}

#[component]
fn Login(set_view: WriteSignal<AppView>, set_auth: WriteSignal<Option<AuthSession>>) -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (loading, set_loading) = create_signal(false);
    
    let do_login = move |_| {
        let email = email.get();
        let password = password.get();
        set_loading.set(true);
        set_error.set(None);
        
        spawn_local(async move {
            match supabase::sign_in(&email, &password).await {
                Ok(session) => {
                    set_auth.set(Some(session));
                    // Reset sync status and trigger new sync with user's credentials
                    storage::reset_sync_status();
                    supabase::sync_from_cloud();
                    set_view.set(AppView::Dashboard);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };
    
    view! {
        <div class="auth-container">
            <div class="auth-logo">"OXIDIZE"</div>
            <div class="auth-card">
                <h2 class="auth-title">"Logga in"</h2>
                
                {move || error.get().map(|e| view! { <div class="auth-error">{e}</div> })}
                
                <input
                    type="email"
                    class="auth-input"
                    placeholder="Email"
                    on:input=move |ev| set_email.set(event_target_value(&ev))
                    prop:value=email
                />
                
                <input
                    type="password"
                    class="auth-input"
                    placeholder="Lösenord"
                    on:input=move |ev| set_password.set(event_target_value(&ev))
                    prop:value=password
                />
                
                <button 
                    class="auth-button"
                    on:click=do_login
                    disabled=move || loading.get()
                >
                    {move || if loading.get() { "Loggar in..." } else { "Logga in" }}
                </button>
                
                <div class="auth-switch">
                    "Inget konto? "
                    <button class="auth-link" on:click=move |_| set_view.set(AppView::Register)>
                        "Registrera dig"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Register(set_view: WriteSignal<AppView>, set_auth: WriteSignal<Option<AuthSession>>) -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (password2, set_password2) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (loading, set_loading) = create_signal(false);
    
    let do_register = move |_| {
        let email = email.get();
        let password = password.get();
        let password2 = password2.get();
        
        if password != password2 {
            set_error.set(Some("Lösenorden matchar inte".into()));
            return;
        }
        
        if password.len() < 6 {
            set_error.set(Some("Lösenordet måste vara minst 6 tecken".into()));
            return;
        }
        
        set_loading.set(true);
        set_error.set(None);
        
        spawn_local(async move {
            match supabase::sign_up(&email, &password).await {
                Ok(session) => {
                    set_auth.set(Some(session));
                    set_view.set(AppView::Dashboard);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };
    
    view! {
        <div class="auth-container">
            <div class="auth-logo">"OXIDIZE"</div>
            <div class="auth-card">
                <h2 class="auth-title">"Skapa konto"</h2>
                
                {move || error.get().map(|e| view! { <div class="auth-error">{e}</div> })}
                
                <input
                    type="email"
                    class="auth-input"
                    placeholder="Email"
                    on:input=move |ev| set_email.set(event_target_value(&ev))
                    prop:value=email
                />
                
                <input
                    type="password"
                    class="auth-input"
                    placeholder="Lösenord"
                    on:input=move |ev| set_password.set(event_target_value(&ev))
                    prop:value=password
                />
                
                <input
                    type="password"
                    class="auth-input"
                    placeholder="Bekräfta lösenord"
                    on:input=move |ev| set_password2.set(event_target_value(&ev))
                    prop:value=password2
                />
                
                <button 
                    class="auth-button"
                    on:click=do_register
                    disabled=move || loading.get()
                >
                    {move || if loading.get() { "Skapar konto..." } else { "Skapa konto" }}
                </button>
                
                <div class="auth-switch">
                    "Har du redan konto? "
                    <button class="auth-link" on:click=move |_| set_view.set(AppView::Login)>
                        "Logga in"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Dashboard(set_view: WriteSignal<AppView>, auth: ReadSignal<Option<AuthSession>>) -> impl IntoView {
    // Refresh token and sync data when Dashboard is viewed
    // This ensures fresh data after app was in background
    supabase::check_and_refresh_session();
    
    // If session was expired and user got signed out, redirect to login
    if supabase::load_auth_session().is_none() {
        set_view.set(AppView::Login);
        return view! { <div class="loading">"Sessionen har gått ut..."</div> }.into_view();
    }
    
    storage::reset_sync_status();
    supabase::sync_from_cloud();
    
    // Signal to trigger data reload
    let (data_version, set_data_version) = create_signal(storage::get_data_version());
    let (is_loading, set_is_loading) = create_signal(true); // Start as loading
    
    // Active routine from Supabase
    let (active_routine, set_active_routine) = create_signal(Option::<crate::types::SavedRoutine>::None);
    let (routine_loading, set_routine_loading) = create_signal(true);
    
    // Load active routine on mount (after token refresh)
    create_effect(move |_| {
        spawn_local(async move {
            // Small delay to let token refresh complete
            gloo_timers::future::TimeoutFuture::new(100).await;
            
            match supabase::fetch_routines().await {
                Ok(routines) => {
                    let active = routines.into_iter().find(|r| r.is_active);
                    if let Some(ref r) = active {
                        // Cache locally for workout loading
                        storage::save_active_routine(r);
                    }
                    set_active_routine.set(active);
                }
                Err(_) => {
                    // Try to load from cache
                    set_active_routine.set(storage::load_active_routine());
                }
            }
            set_routine_loading.set(false);
        });
    });
    
    // Poll for sync completion
    if !storage::is_sync_complete() {
        use gloo_timers::callback::Interval;
        let interval = Interval::new(200, move || {
            if storage::is_sync_complete() {
                set_is_loading.set(false);
                set_data_version.set(storage::get_data_version());
            }
        });
        // Keep interval alive but stop after 10 seconds max
        leptos::on_cleanup(move || drop(interval));
    }
    
    let stats = create_memo(move |_| {
        let _ = data_version.get();
        let db = storage::load_data();
        let total = db.get_total_stats();
        let recent = db.get_recent_sessions(1); // Only 1 recent session
        (total, recent)
    });
    
    let paused = create_memo(move |_| {
        let _ = data_version.get();
        storage::load_paused_workout()
    });
    
    // Get display name reactively - listen to data_version to update after cloud sync
    let user_display = move || {
        let _ = data_version.get(); // IMPORTANT: Trigger re-render when sync completes
        storage::load_display_name()
            .or_else(|| auth.get().and_then(|a| a.user.display_name.clone()))
            .or_else(|| auth.get().map(|a| a.user.email.clone()))
            .unwrap_or_default()
    };
    
    // State for confirmation dialog
    let (show_confirm, set_show_confirm) = create_signal(false);
    let (pending_pass, set_pending_pass) = create_signal(String::new());
    
    let start_workout_pass = move |pass_name: String| {
        if storage::load_paused_workout().is_some() {
            set_pending_pass.set(pass_name);
            set_show_confirm.set(true);
        } else {
            set_view.set(AppView::Workout(pass_name));
        }
    };
    
    let confirm_start = move |_| {
        storage::clear_paused_workout();
        set_view.set(AppView::Workout(pending_pass.get()));
    };

    view! {
        <div class="dashboard">
            <button class="settings-gear" on:click=move |_| set_view.set(AppView::Settings)>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d="M12 15a3 3 0 100-6 3 3 0 000 6z"/>
                    <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/>
                </svg>
            </button>
            <div class="logo">"OXIDIZE"</div>
            
            // Show loading indicator while syncing
            {move || is_loading.get().then(|| view! {
                <div class="sync-loading">"Laddar data..."</div>
            })}
            
            <div class="quick-stats">
                <div class="quick-stat">
                    <span class="quick-stat-value">{move || stats.get().0.total_sessions}</span>
                    <span class="quick-stat-label">"pass"</span>
                </div>
                <div class="quick-stat">
                    <span class="quick-stat-value">{move || format!("{:.0}", stats.get().0.total_volume / 1000.0)}</span>
                    <span class="quick-stat-label">"ton"</span>
                </div>
            </div>
            
            // Paused workout banner
            {move || paused.get().map(|p| {
                let routine_name = p.routine_name.clone();
                let elapsed = format_time(p.elapsed_secs);
                let exercises_done = p.exercises.iter().filter(|e| !e.sets_completed.is_empty()).count();
                let total_ex = p.exercises.len();
                view! {
                    <div class="paused-workout-banner">
                        <div class="paused-info">
                            <span class="paused-label"><span class="pause-icon"></span>" Pågående pass"</span>
                            <span class="paused-routine">{&routine_name}</span>
                            <span class="paused-progress">{exercises_done}"/" {total_ex}" övningar · "{elapsed}</span>
                        </div>
                        <button class="resume-btn" on:click={
                            let rn = routine_name.clone();
                            move |_| set_view.set(AppView::Workout(rn.clone()))
                        }>
                            "Fortsätt →"
                        </button>
                    </div>
                }
            })}
            
            // Dynamic pass buttons from active routine
            {move || {
                if routine_loading.get() {
                    view! { <p class="loading-passes">"Laddar pass..."</p> }.into_view()
                } else if let Some(routine) = active_routine.get() {
                    let pass_count = routine.passes.len();
                    let size_class = match pass_count {
                        1..=2 => "size-large",
                        3 => "size-medium",
                        4 => "size-small",
                        _ => "size-tiny",
                    };
                    view! {
                        <div class=format!("pass-buttons {}", size_class)>
                            {routine.passes.iter().enumerate().map(|(i, pass)| {
                                let pass_name = pass.name.clone();
                                let pass_name_click = pass.name.clone();
                                let btn_class = format!("start-btn pass-{}", (b'a' + i as u8) as char);
                                let description = if pass.description.is_empty() {
                                    format!("{} övningar", pass.exercises.len())
                                } else {
                                    pass.description.clone()
                                };
                                view! {
                                    <button 
                                        class=btn_class
                                        on:click=move |_| start_workout_pass(pass_name_click.clone())
                                    >
                                        <span class="start-btn-label">{pass_name}</span>
                                        <span class="start-btn-focus">{description}</span>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    }.into_view()
                } else {
                    // No active routine - show prompt to create one
                    view! {
                        <div class="no-routine">
                            <p>"Ingen aktiv rutin"</p>
                            <button class="create-routine-link" on:click=move |_| set_view.set(AppView::Settings)>
                                "Skapa eller välj rutin →"
                            </button>
                        </div>
                    }.into_view()
                }
            }}

            {move || {
                let recent = stats.get().1;
                let active = active_routine.get();
                (!recent.is_empty()).then(|| view! {
                    <div class="recent-sessions">
                        <div class="recent-title">"Senaste"</div>
                        {recent.into_iter().map(|s| {
                            // Find pass index for color
                            let pass_idx = active.as_ref()
                                .and_then(|r| r.passes.iter().position(|p| p.name == s.routine))
                                .unwrap_or(0);
                            let color_class = format!("pass-{}", (b'a' + pass_idx as u8) as char);
                            view! {
                                <div class="recent-item">
                                    <span class=format!("recent-routine {}", color_class)>{&s.routine}</span>
                                    <span class="recent-date">{format_date(s.timestamp)}</span>
                                    <span class="recent-duration">{format_time(s.duration_secs)}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                })
            }}
            
            <button class="stats-link" on:click=move |_| set_view.set(AppView::Stats)>
                "Statistik →"
            </button>
            
            <div class="logged-in-info">
                "inloggad: "{move || user_display()}<br/>
                <button class="logout-link" on:click=move |_| {
                    supabase::sign_out();
                    set_view.set(AppView::Login);
                }>"logga ut"</button>
            </div>
            
            // Confirmation dialog
            {move || show_confirm.get().then(|| view! {
                <div class="modal-overlay">
                    <div class="confirm-dialog">
                        <div class="confirm-title">"Avbryta pågående pass?"</div>
                        <div class="confirm-text">"Du har ett pågående pass. Vill du radera det och starta ett nytt?"</div>
                        <div class="confirm-buttons">
                            <button class="confirm-cancel" on:click=move |_| set_show_confirm.set(false)>
                                "Avbryt"
                            </button>
                            <button class="confirm-ok" on:click=confirm_start>
                                "Ja, starta nytt"
                            </button>
                        </div>
                    </div>
                </div>
            })}
        </div>
    }.into_view()
}

#[component]
fn Workout(routine: String, set_view: WriteSignal<AppView>) -> impl IntoView {
    // The routine parameter is now the pass name (e.g., "Pass A", "Ben", etc.)
    let pass_name = routine.clone();
    
    // Check for paused workout first
    if let Some(paused) = storage::load_paused_workout() {
        if paused.routine_name == pass_name {
            // Resume paused workout
            let data = storage::get_workout(&pass_name);
            if let Some(mut d) = data {
                d.exercises = paused.exercises;
                return view! { 
                    <WorkoutActive 
                        data=d 
                        set_view=set_view 
                        resumed_from=paused.current_exercise_idx
                        start_elapsed=paused.elapsed_secs
                    /> 
                }.into_view();
            }
        }
    }
    
    // Clear any paused workout since we're starting fresh
    storage::clear_paused_workout();
    
    let data = storage::get_workout(&pass_name);
    
    match data {
        Some(d) => view! { 
            <WorkoutActive data=d set_view=set_view /> 
        }.into_view(),
        None => view! { <div class="loading">"Kunde inte ladda pass"</div> }.into_view(),
    }
}

#[component]
fn WorkoutActive(
    data: WorkoutData, 
    set_view: WriteSignal<AppView>,
    #[prop(default = 0)] resumed_from: usize,
    #[prop(default = 0)] start_elapsed: i64,
) -> impl IntoView {
    let routine = data.routine.clone();
    let routine_name = routine.name.clone();
    let routine_name_save = routine_name.clone();
    let routine_name_pause = routine_name.clone();
    
    // Load bodyweight for calorie calculation (use default if not set)
    let db = storage::load_data();
    let bodyweight = db.get_bodyweight().unwrap_or(80.0);
    
    // State
    let total_exercises = data.exercises.len();
    let (exercises, set_exercises) = create_signal(data.exercises);
    let (current_idx, set_current_idx) = create_signal(resumed_from);
    let (start_time, _) = create_signal(js_sys::Date::now() as i64 / 1000 - start_elapsed);
    let (elapsed, set_elapsed) = create_signal(start_elapsed);
    let (last_set_time, set_last_set_time) = create_signal(0i64);
    let (rest_elapsed, set_rest_elapsed) = create_signal(0i64);
    let (is_resting, set_is_resting) = create_signal(false);
    let (is_finished, set_is_finished) = create_signal(false);
    let (show_overview, set_show_overview) = create_signal(false);
    let (show_cancel_confirm, set_show_cancel_confirm) = create_signal(false);
    let (is_saving, set_is_saving) = create_signal(false);
    let (show_sync_warning, set_show_sync_warning) = create_signal(false);
    
    // Timer state for timed exercises (like Mountain Climbers)
    let (timer_running, set_timer_running) = create_signal(false);
    let (timer_selected_duration, set_timer_selected_duration) = create_signal(30u32);
    let (timer_remaining, set_timer_remaining) = create_signal(0i32);
    let (show_timer_flash, set_show_timer_flash) = create_signal(false);
    
    // Jump to specific exercise
    let jump_to_exercise = move |idx: usize| {
        set_current_idx.set(idx);
        set_is_resting.set(false);
        set_show_overview.set(false);
    };
    
    // Flag to signal timer completion (checked in a separate effect)
    let (timer_just_completed, set_timer_just_completed) = create_signal(false);
    
    // Timer
    create_effect(move |_| {
        let handle = gloo_timers::callback::Interval::new(1000, move || {
            let now = js_sys::Date::now() as i64 / 1000;
            set_elapsed.set(now - start_time.get());
            if is_resting.get() && last_set_time.get() > 0 {
                set_rest_elapsed.set(now - last_set_time.get());
            }
            // Countdown timer for timed exercises
            if timer_running.get() {
                let remaining = timer_remaining.get() - 1;
                if remaining <= 0 {
                    set_timer_remaining.set(0);
                    set_timer_running.set(false);
                    set_show_timer_flash.set(true);
                    // Flash for 800ms then trigger completion
                    gloo_timers::callback::Timeout::new(800, move || {
                        set_show_timer_flash.set(false);
                        set_timer_just_completed.set(true);
                    }).forget();
                } else {
                    set_timer_remaining.set(remaining);
                }
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
    
    // Helper to find superset partner index
    let find_partner_idx = move |exs: &[ExerciseWorkoutState], current_idx: usize| -> Option<usize> {
        let current = &exs[current_idx];
        if !current.exercise.is_superset {
            return None;
        }
        let partner_name = current.exercise.superset_with.as_ref()?;
        exs.iter().position(|e| &e.exercise.name == partner_name)
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
        let is_superset = exs[idx].exercise.is_superset;
        let _is_last_exercise = idx + 1 >= exs.len();
        
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
        let just_finished_exercise = sets_done + 1 >= sets_target;
        
        if just_finished_exercise {
            // This exercise is done
            if is_superset {
                // Check if partner is also done
                if let Some(partner_idx) = find_partner_idx(&exs, idx) {
                    let partner = &exs[partner_idx];
                    let partner_done = partner.sets_completed.len() >= partner.exercise.sets as usize;
                    if !partner_done {
                        // Partner still has sets, go there
                        set_current_idx.set(partner_idx);
                        set_is_resting.set(true);
                        return;
                    }
                }
            }
            // Move to next exercise (skip partner if it's right after us)
            let mut next_idx = idx + 1;
            while next_idx < exs.len() {
                let next_ex = &exs[next_idx];
                let next_done = next_ex.sets_completed.len() >= next_ex.exercise.sets as usize;
                if !next_done {
                    break;
                }
                next_idx += 1;
            }
            if next_idx < exs.len() {
                set_current_idx.set(next_idx);
            } else {
                // All exercises done!
                set_is_finished.set(true);
                return;
            }
        } else if is_superset {
            // Not done with this exercise yet - alternate to partner if they have sets remaining
            if let Some(partner_idx) = find_partner_idx(&exs, idx) {
                let partner = &exs[partner_idx];
                let partner_done = partner.sets_completed.len() >= partner.exercise.sets as usize;
                if !partner_done {
                    // Partner has sets remaining, switch to them
                    set_current_idx.set(partner_idx);
                }
            }
        }
        // Show rest screen
        set_is_resting.set(true);
    };
    
    // Complete a timed set (for exercises like Mountain Climbers)
    let complete_timed_set = move || {
        let duration = timer_selected_duration.get();
        // Use duration as "reps" value for timed exercises
        complete_set(duration as u8);
    };
    
    // Watch for timer completion and complete the set
    create_effect(move |_| {
        if timer_just_completed.get() {
            set_timer_just_completed.set(false);
            complete_timed_set();
        }
    });
    
    // Start timer for timed exercise
    let start_timer = move |_| {
        let duration = timer_selected_duration.get();
        set_timer_remaining.set(duration as i32);
        set_timer_running.set(true);
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

    view! {
        <div class="workout">
            // Header with progress dots
            <div class="workout-header">
                <div class="workout-title">{&routine.name}</div>
                
                // Progress dots - clickable to open overview
                <button class="progress-dots" on:click=move |_| set_show_overview.set(true)>
                    {move || {
                        let curr = current_idx.get();
                        let exs = exercises.get();
                        (0..total_exercises).map(|i| {
                            let is_done = exs.get(i).map(|e| {
                                e.sets_completed.len() >= e.exercise.sets as usize
                            }).unwrap_or(false);
                            let is_current = i == curr;
                            let is_started = exs.get(i).map(|e| !e.sets_completed.is_empty()).unwrap_or(false);
                            
                            let dot_class = if is_done {
                                "progress-dot done"
                            } else if is_current {
                                "progress-dot current"
                            } else if is_started {
                                "progress-dot started"
                            } else {
                                "progress-dot"
                            };
                            
                            view! { <span class=dot_class></span> }
                        }).collect_view()
                    }}
                </button>
                
                <div class="workout-timer">{move || format_time(elapsed.get())}</div>
            </div>
            
            // Overview modal
            {move || {
                if show_overview.get() {
                    let exs = exercises.get();
                    let curr = current_idx.get();
                    
                    view! {
                        <div class="overview-modal-backdrop" on:click=move |_| set_show_overview.set(false)>
                            <div class="overview-modal" on:click=|e| e.stop_propagation()>
                                <div class="overview-header">
                                    <span class="overview-title">"Pass-översikt"</span>
                                    <button class="overview-close" on:click=move |_| set_show_overview.set(false)>"✕"</button>
                                </div>
                                <div class="overview-list">
                                    {
                                        // Group exercises, detecting superset pairs
                                        let mut result: Vec<View> = Vec::new();
                                        let mut i = 0;
                                        while i < exs.len() {
                                            let ex = &exs[i];
                                            let is_superset = ex.exercise.is_superset;
                                            
                                            // Check if this starts a superset pair
                                            let has_partner = is_superset && i + 1 < exs.len() && 
                                                exs[i + 1].exercise.is_superset &&
                                                exs[i + 1].exercise.superset_with.as_ref() == Some(&ex.exercise.name);
                                            
                                            if has_partner {
                                                // Render both as a group
                                                let ex1 = &exs[i];
                                                let ex2 = &exs[i + 1];
                                                let idx1 = i;
                                                let idx2 = i + 1;
                                                
                                                let item1_class = if ex1.sets_completed.len() >= ex1.exercise.sets as usize { 
                                                    "overview-item done" 
                                                } else if idx1 == curr { 
                                                    "overview-item current" 
                                                } else { 
                                                    "overview-item" 
                                                };
                                                let item2_class = if ex2.sets_completed.len() >= ex2.exercise.sets as usize { 
                                                    "overview-item done" 
                                                } else if idx2 == curr { 
                                                    "overview-item current" 
                                                } else { 
                                                    "overview-item" 
                                                };
                                                
                                                let icon1 = if ex1.sets_completed.len() >= ex1.exercise.sets as usize { "✓" } 
                                                    else if idx1 == curr { "►" } else { "" };
                                                let icon2 = if ex2.sets_completed.len() >= ex2.exercise.sets as usize { "✓" } 
                                                    else if idx2 == curr { "►" } else { "" };
                                                
                                                let name1 = ex1.exercise.name.clone();
                                                let name2 = ex2.exercise.name.clone();
                                                let sets1 = format!("{}/{}", ex1.sets_completed.len(), ex1.exercise.sets);
                                                let sets2 = format!("{}/{}", ex2.sets_completed.len(), ex2.exercise.sets);
                                                
                                                result.push(view! {
                                                    <div class="superset-group">
                                                        <button class=item1_class on:click=move |_| jump_to_exercise(idx1)>
                                                            <span class="overview-icon">{icon1}</span>
                                                            <span class="overview-name">{name1}</span>
                                                            <span class="overview-sets">{sets1}</span>
                                                        </button>
                                                        <button class=item2_class on:click=move |_| jump_to_exercise(idx2)>
                                                            <span class="overview-icon">{icon2}</span>
                                                            <span class="overview-name">{name2}</span>
                                                            <span class="overview-sets">{sets2}</span>
                                                        </button>
                                                    </div>
                                                }.into_view());
                                                i += 2;
                                            } else {
                                                // Regular exercise
                                                let idx = i;
                                                let item_class = if ex.sets_completed.len() >= ex.exercise.sets as usize { 
                                                    "overview-item done" 
                                                } else if idx == curr { 
                                                    "overview-item current" 
                                                } else { 
                                                    "overview-item" 
                                                };
                                                let icon = if ex.sets_completed.len() >= ex.exercise.sets as usize { "✓" } 
                                                    else if idx == curr { "►" } else { "" };
                                                let name = ex.exercise.name.clone();
                                                let sets = format!("{}/{}", ex.sets_completed.len(), ex.exercise.sets);
                                                
                                                result.push(view! {
                                                    <button class=item_class on:click=move |_| jump_to_exercise(idx)>
                                                        <span class="overview-icon">{icon}</span>
                                                        <span class="overview-name">{name}</span>
                                                        <span class="overview-sets">{sets}</span>
                                                    </button>
                                                }.into_view());
                                                i += 1;
                                            }
                                        }
                                        result.into_iter().collect_view()
                                    }
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
            
            // Main content
            <div class="workout-main">
                {move || {
                    if is_finished.get() {
                        // Finished view
                        let duration_mins = std::cmp::max(1, (elapsed.get() + 30) / 60); // Round up, minimum 1 min
                        
                        // Calculate total volume (kg × reps)
                        let total_volume: f64 = exercises.get().iter()
                            .flat_map(|ex| ex.sets_completed.iter())
                            .map(|set| set.weight * set.reps as f64)
                            .sum();
                        
                        // Calculate efficiency (kg/min)
                        let efficiency = if duration_mins > 0 { 
                            total_volume / duration_mins as f64 
                        } else { 
                            0.0 
                        };
                        
                        // Base MET = 5.0, increase slightly for high efficiency workouts
                        // Typical efficiency: 50-150 kg/min
                        // Add up to 1.5 MET for very intense workouts (200+ kg/min)
                        let efficiency_bonus = (efficiency / 200.0).min(1.0) * 1.5;
                        let met = 5.0 + efficiency_bonus;
                        
                        // Calories = (minutes / 60) × bodyweight × MET
                        let hours = duration_mins as f64 / 60.0;
                        let calories = (hours * bodyweight * met).round() as i64;
                        
                        let health_url = format!(
                            "shortcuts://run-shortcut?name=Oxidize&input=text&text={},{}",
                            duration_mins, calories
                        );
                        view! {
                            <div class="finish-screen">
                                // Sync warning modal
                                {move || show_sync_warning.get().then(|| view! {
                                    <div class="modal-overlay">
                                        <div class="sync-warning-dialog">
                                            <div class="sync-warning-icon">"⚠️"</div>
                                            <div class="sync-warning-title">"Kunde inte spara till molnet"</div>
                                            <div class="sync-warning-text">
                                                "Passet är sparat lokalt men kunde inte synkas till Supabase efter 3 försök. "
                                                "Rensa INTE webbläsarens cache förrän du har internetanslutning och appen har synkat."
                                            </div>
                                            <button class="sync-warning-btn" on:click=move |_| {
                                                set_view.set(AppView::Dashboard);
                                            }>
                                                "Jag förstår"
                                            </button>
                                        </div>
                                    </div>
                                })}
                                
                                <div class="finish-icon">"✓"</div>
                                <div class="finish-title">"Bra jobbat!"</div>
                                <div class="finish-time">{format_time(elapsed.get())}</div>
                                <div class="finish-stats">
                                    <span class="finish-stat">{format!("{:.0} kg volym", total_volume)}</span>
                                    <span class="finish-stat">{format!("{} kcal", calories)}</span>
                                </div>
                                
                                {move || if is_saving.get() {
                                    view! {
                                        <div class="saving-indicator">"Sparar..."</div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <button class="finish-save-btn" on:click=move |_| {
                                            set_is_saving.set(true);
                                            
                                            // Clear any previous sync failure flag
                                            supabase::clear_sync_failed();
                                            
                                            let exs = exercises.get();
                                            let records: Vec<ExerciseRecord> = exs.iter()
                                                .filter(|e| !e.sets_completed.is_empty())
                                                .map(|e| ExerciseRecord {
                                                    name: e.exercise.name.clone(),
                                                    sets: e.sets_completed.clone(),
                                                })
                                                .collect();
                                            storage::save_session(routine_name_sig.get(), records, elapsed.get());
                                            
                                            // Poll for sync result (check every 500ms for up to 10 seconds)
                                            use gloo_timers::callback::Interval;
                                            let check_count = std::rc::Rc::new(std::cell::RefCell::new(0));
                                            let check_count_clone = check_count.clone();
                                            let interval = Interval::new(500, move || {
                                                *check_count_clone.borrow_mut() += 1;
                                                let count = *check_count_clone.borrow();
                                                
                                                // Check if sync failed
                                                if supabase::get_sync_failed_session().is_some() {
                                                    set_is_saving.set(false);
                                                    set_show_sync_warning.set(true);
                                                    return;
                                                }
                                                
                                                // After ~5 seconds (10 checks), assume success and go to dashboard
                                                // (retries take max ~3-4 sec, so 5 sec is safe)
                                                if count >= 10 {
                                                    set_view.set(AppView::Dashboard);
                                                }
                                            });
                                            leptos::on_cleanup(move || drop(interval));
                                        }>
                                            "Spara pass"
                                        </button>
                                    }.into_view()
                                }}
                                
                                <a class="health-link" href={health_url.clone()} target="_blank">
                                    "Logga till Health →"
                                </a>
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
                        let is_bodyweight = ex.as_ref().map(|e| e.exercise.is_bodyweight).unwrap_or(false);
                        let is_timed = ex.as_ref().and_then(|e| e.exercise.duration_secs).is_some();
                        let target_duration = ex.as_ref().and_then(|e| e.exercise.duration_secs).unwrap_or(30);
                        let ss_with = ex.as_ref().and_then(|e| e.exercise.superset_with.clone());
                        // Exercise hints
                        let is_dumbbell = matches!(ex_name.as_str(), "Hammercurls" | "Sidolyft");
                        let is_alternating = matches!(ex_name.as_str(), "Utfallssteg" | "Dead Bug");
                        
                        // Last used duration for timed exercises (stored as reps)
                        let last_duration = ex.as_ref()
                            .and_then(|e| e.last_data.as_ref())
                            .map(|d| d.reps as u32);
                        
                        view! {
                            <div class=move || if show_timer_flash.get() { "exercise-screen timer-flash" } else { "exercise-screen" }>
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
                                
                                // Bodyweight badge for finishers
                                {is_bodyweight.then(|| view! {
                                    <div class="bodyweight-indicator">
                                        "FINISHER"
                                    </div>
                                })}
                                
                                // Exercise name
                                <div class="exercise-name-big">{ex_name}</div>
                                
                                // Dumbbell hint
                                {is_dumbbell.then(|| view! {
                                    <div class="exercise-hint">"Lägg ihop båda hantlarnas vikt"</div>
                                })}
                                
                                // Alternating exercise hint
                                {is_alternating.then(|| view! {
                                    <div class="exercise-hint">"Totalt antal reps (båda sidor)"</div>
                                })}
                                
                                // Weight with controls (hidden for bodyweight exercises)
                                {(!is_bodyweight).then(|| view! {
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
                                })}
                                
                                // TIMED EXERCISE UI
                                {is_timed.then(|| view! {
                                    <div class="timer-section">
                                        {move || if timer_running.get() {
                                            // Countdown view
                                            view! {
                                                <div class="timer-countdown">
                                                    <div class="timer-display">
                                                        {move || format!("0:{:02}", timer_remaining.get())}
                                                    </div>
                                                    <button class="timer-stop-btn" on:click=move |_| {
                                                        set_timer_running.set(false);
                                                        set_timer_remaining.set(0);
                                                    }>
                                                        "Avbryt"
                                                    </button>
                                                </div>
                                            }.into_view()
                                        } else {
                                            // Duration selector view
                                            view! {
                                                <div class="timer-selector">
                                                    <div class="timer-label">"Välj tid:"</div>
                                                    <div class="duration-buttons">
                                                        {[20u32, 25, 30, 35, 40, 45, 50, 55].into_iter().map(|d| {
                                                            let is_target = d == target_duration;
                                                            let is_last = last_duration == Some(d);
                                                            let btn_class = move || {
                                                                let is_selected = timer_selected_duration.get() == d;
                                                                if is_selected {
                                                                    "duration-button selected"
                                                                } else if is_last {
                                                                    "duration-button last"
                                                                } else if is_target {
                                                                    "duration-button target"
                                                                } else {
                                                                    "duration-button"
                                                                }
                                                            };
                                                            view! {
                                                                <button 
                                                                    class=btn_class
                                                                    on:click=move |_| set_timer_selected_duration.set(d)
                                                                >
                                                                    {format!("{}s", d)}
                                                                </button>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                    <button class="timer-start-btn" on:click=start_timer>
                                                        "▶ STARTA"
                                                    </button>
                                                    <div class="timer-target-hint">
                                                        {format!("Mål: {} sek", target_duration)}
                                                    </div>
                                                </div>
                                            }.into_view()
                                        }}
                                    </div>
                                })}
                                
                                // REP-BASED EXERCISE UI (not timed)
                                {(!is_timed).then(|| view! {
                                    <div class="rep-label">"Tryck antal reps:"</div>
                                })}
                                {(!is_timed).then(|| view! {
                                    <div class="rep-buttons">
                                        {move || {
                                            let ex = current_exercise();
                                            let (min, max) = ex.as_ref()
                                                .map(|e| parse_target_range(&e.exercise.reps_target))
                                                .unwrap_or((5, 8));
                                            let last_reps = ex.as_ref()
                                                .and_then(|e| e.last_data.as_ref())
                                                .map(|d| d.reps);
                                            
                                            // Center the 12-button grid around the target
                                            let center = (min + max) / 2;
                                            let start = (center as i32 - 5).max(1) as u8;
                                            let end = start + 11; // 12 buttons total
                                            
                                            (start..=end).map(|r| {
                                                let is_last = last_reps == Some(r);
                                                let is_target = r >= min && r <= max;
                                                let btn_class = if is_last { 
                                                    "rep-button last" 
                                                } else if is_target { 
                                                    "rep-button target" 
                                                } else { 
                                                    "rep-button" 
                                                };
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
                                })}
                                {(!is_timed).then(|| view! {
                                    <div class="rep-target-hint">
                                        {move || format!("Mål: {}", target_reps_str())}
                                    </div>
                                })}
                                
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
                <button class="back-btn" on:click=move |_| {
                    // Save paused state
                    let paused = crate::types::PausedWorkout {
                        routine_name: routine_name_pause.clone(),
                        exercises: exercises.get(),
                        current_exercise_idx: current_idx.get(),
                        start_timestamp: start_time.get(),
                        elapsed_secs: elapsed.get(),
                    };
                    let _ = storage::save_paused_workout(&paused);
                    set_view.set(AppView::Dashboard);
                }>
                    <span class="pause-icon"></span>" Pausa"
                </button>
                <button class="cancel-workout-btn" on:click=move |_| {
                    set_show_cancel_confirm.set(true);
                }>
                    "Avsluta pass"
                </button>
            </div>
            
            // Cancel confirmation modal
            {move || show_cancel_confirm.get().then(|| view! {
                <div class="modal-overlay">
                    <div class="confirm-dialog">
                        <div class="confirm-title">"Avsluta pass?"</div>
                        <div class="confirm-text">"Är du säker? Passet sparas inte."</div>
                        <div class="confirm-buttons">
                            <button class="confirm-cancel" on:click=move |_| set_show_cancel_confirm.set(false)>
                                "Nej, fortsätt"
                            </button>
                            <button class="confirm-ok" on:click=move |_| {
                                storage::clear_paused_workout();
                                set_view.set(AppView::Dashboard);
                            }>
                                "Ja, avsluta"
                            </button>
                        </div>
                    </div>
                </div>
            })}
        </div>
    }
}

#[component]
fn WeightChart(history: Vec<crate::storage::BodyweightEntry>) -> impl IntoView {
    if history.len() < 2 {
        return view! { <div class="empty-chart">"Behöver minst två mätningar för en kurva"</div> }.into_view();
    }

    // Filter data to only show last 12 months
    let now = (js_sys::Date::now() / 1000.0) as i64;
    let one_year_ago = now - (365 * 24 * 60 * 60);
    
    let mut sorted = history.clone();
    sorted.sort_by_key(|h| h.timestamp);
    
    let data: Vec<_> = sorted.into_iter()
        .filter(|h| h.timestamp >= one_year_ago)
        .collect();
        
    if data.len() < 2 {
        return view! { <div class="empty-chart">"Ingen data för senaste året"</div> }.into_view();
    }
    
    let min_w = data.iter().map(|h| h.weight).fold(f64::INFINITY, f64::min);
    let max_w = data.iter().map(|h| h.weight).fold(f64::NEG_INFINITY, f64::max);
    let range = (max_w - min_w).max(1.0);
    
    let padding = 20.0;
    let width = 100.0;
    let height = 100.0;
    
    // X-axis is time-based now
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
                // Grid lines (min/max)
                <line x1=padding y1={get_y(min_w)} x2={width-padding} y2={get_y(min_w)} stroke="#222" stroke-width="1" stroke-dasharray="2,2" />
                <line x1=padding y1={get_y(max_w)} x2={width-padding} y2={get_y(max_w)} stroke="#222" stroke-width="1" stroke-dasharray="2,2" />
                
                // The line
                <polyline points=points class="weight-line" />
                
                // Points
                {data.iter().enumerate().map(|(idx, h)| {
                    let x = get_x(h.timestamp);
                    let y = get_y(h.weight);
                    view! {
                        <circle cx=x cy=y r="4" class="weight-point" />
                        {if idx == 0 || idx == data.len() - 1 || h.weight == max_w || h.weight == min_w {
                            // Alternate label position to prevent overlap
                            let y_off = if idx % 2 == 0 { -12.0 } else { 16.0 };
                            view! {
                                <text x=x y={y + y_off} font-size="12" fill="var(--fg-primary)" text-anchor="middle" font-family="var(--font)" font-weight="700">
                                    {format!("{:.1}", h.weight)}
                                </text>
                            }.into_view()
                        } else {
                            view! { <text /> }.into_view()
                        }}
                    }
                }).collect_view()}
            </svg>
            <div class="weight-chart-labels">
                <span class="weight-chart-label">{format_date(data.first().unwrap().timestamp)}</span>
                <span class="weight-chart-label">{format_date(data.last().unwrap().timestamp)}</span>
            </div>
        </div>
    }.into_view()
}

#[component]
fn Stats(set_view: WriteSignal<AppView>, auth: ReadSignal<Option<AuthSession>>, set_auth: WriteSignal<Option<AuthSession>>) -> impl IntoView {
    // Reactive signal for sync status - poll until complete
    let (sync_status, set_sync_status) = create_signal(storage::get_sync_status().to_string());
    
    // Reactive bodyweight signal (Option<f64> - None means still loading)
    let (bodyweight, set_bodyweight) = create_signal(Option::<f64>::None);
    
    // Reactive data version - increments when data should be reloaded
    let (data_version, set_data_version) = create_signal(0u32);
    
    // Poll sync status until complete, then trigger data reload
    create_effect(move |_| {
        let status = sync_status.get();
        if status == "pending" {
            // Still waiting - set up interval to check again
            let handle = gloo_timers::callback::Interval::new(200, move || {
                let new_status = storage::get_sync_status();
                if new_status != "pending" {
                    set_sync_status.set(new_status.to_string());
                }
            });
            // Keep the interval alive by leaking it (it will stop when status changes)
            std::mem::forget(handle);
        } else {
            // Sync complete - load the actual data
            let db = storage::load_data();
            let bw = db.get_bodyweight();
            let display_bw = Some(bw.unwrap_or(80.0));
            set_bodyweight.set(display_bw);
            // Trigger UI refresh
            set_data_version.update(|v| *v += 1);
        }
    });
    
    let _user_email = auth.get().map(|a| a.user.email).unwrap_or_default();
    
    let do_logout = move |_| {
        supabase::sign_out();
        set_auth.set(None);
        set_view.set(AppView::Login);
    };
    
    // Helper to load fresh data - called in reactive closures
    let load_sessions = move || {
        let _ = data_version.get(); // Subscribe to changes
        let db = storage::load_data();
        let now = chrono::Utc::now().timestamp();
        let two_months_ago = now - (61 * 24 * 60 * 60);
        let mut sessions: Vec<_> = db.sessions.iter()
            .filter(|s| s.timestamp >= two_months_ago)
            .cloned()
            .collect();
        sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sessions
    };
    
    let load_summary = move || {
        let _ = data_version.get(); // Subscribe to changes
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
                // ═══════════════════════════════════════════════════════════════
                // 1. POWER SCORE (E1RM) - Main Hero Metric
                // ═══════════════════════════════════════════════════════════════
                <div class="stat-card hero-card">
                    <div class="hero-label">"POWER SCORE"</div>
                    <div class="hero-value">{move || format!("{:.0}", load_summary().power_score)}<span class="hero-unit">" kg"</span></div>
                    <div class="hero-subtitle">"Summa E1RM (Big 4)"</div>
                    
                    // Big 4 breakdown
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
                    
                    // Mini trend chart
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
                
                // ═══════════════════════════════════════════════════════════════
                // 2. POWER-TO-WEIGHT RATIO
                // ═══════════════════════════════════════════════════════════════
                <div class="stat-card">
                    <div class="stat-card-title">"Relativ Styrka"</div>
                    <div class="ptw-section">
                        <div class="ptw-main">
                            <span class="ptw-value">
                                {move || {
                                    match bodyweight.get() {
                                        Some(bw) if bw > 0.0 => format!("{:.2}", load_summary().power_score / bw),
                                        _ => "—".to_string()
                                    }
                                }}
                            </span>
                            <span class="ptw-unit">"Styrkeratio"</span>
                        </div>
                        <div class="ptw-hint">"Total styrka (Big 4) delat med kroppsvikt"</div>
                    </div>
                </div>
                
                // ═══════════════════════════════════════════════════════════════
                // 3. EFFICIENCY (kg/min)
                // ═══════════════════════════════════════════════════════════════
                <div class="stat-card">
                    <div class="stat-card-title">"Effektivitet"</div>
                    <div class="efficiency-row">
                        <div class="efficiency-main">
                            <span class="efficiency-value">{move || format!("{:.1}", load_summary().avg_efficiency)}</span>
                            <span class="efficiency-unit">"kg/min"</span>
                        </div>
                        <div class="efficiency-explain">
                            "Snitt volym per minut"
                        </div>
                    </div>
                    // Show last 5 sessions efficiency
                    {move || {
                        let sess = load_sessions();
                        (!sess.is_empty()).then(|| {
                            let eff_history: Vec<f64> = sess.iter().take(5)
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
                        })
                    }}
                </div>
                
                // ═══════════════════════════════════════════════════════════════
                // 4. PROGRESSIVE OVERLOAD STREAK
                // ═══════════════════════════════════════════════════════════════
                {move || {
                    let _ = data_version.get();
                    let db = storage::load_data();
                    let sess = load_sessions();
                    let statuses: Vec<(String, ProgressStatus)> = if let Some(session) = sess.first() {
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
                            <div class="stat-card-title">"Senaste passet: Progression"</div>
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
                
                // ═══════════════════════════════════════════════════════════════
                // 5. MUSCLE HEATMAP (7 dagar)
                // Weighted scoring: primary muscles +3, secondary +1, × sets
                // ═══════════════════════════════════════════════════════════════
                <div class="stat-card">
                    <div class="stat-card-title">"Muskelaktivitet (7 dagar)"</div>
                    <div class="heatmap-grid">
                        {move || {
                            let s = load_summary();
                            MuscleGroup::all().into_iter().map(|mg| {
                                let score = *s.muscle_frequency.get(&mg).unwrap_or(&0);
                                // New thresholds for weighted scoring:
                                // 0 = not trained
                                // 1-8 = low (secondary muscle or skipped)
                                // 9-17 = moderate (one exercise as primary)
                                // 18-26 = good (multiple exercises)
                                // 27+ = high
                                let heat_class = match score {
                                    0 => "heat-0",
                                    1..=8 => "heat-1",
                                    9..=17 => "heat-2",
                                    18..=26 => "heat-3",
                                    _ => "heat-4",
                                };
                                view! {
                                    <div class=format!("heatmap-item {}", heat_class)>
                                        <span class="heat-name">{mg.name()}</span>
                                        <span class="heat-count">{score}</span>
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>
                    <div class="heat-legend">
                        <span class="heat-0">"Vila"</span>
                        <span class="heat-1">"Låg"</span>
                        <span class="heat-2">"Ok"</span>
                        <span class="heat-3">"Bra"</span>
                        <span class="heat-4">"Hög"</span>
                    </div>
                </div>

                // ═══════════════════════════════════════════════════════════════
                // 6. WEIGHT CURVE (Moved below heatmap)
                // ═══════════════════════════════════════════════════════════════
                <div class="stat-card weight-card-new">
                    <div class="stat-card-title">"Viktutveckling"</div>
                    {move || {
                        let _ = data_version.get();
                        let db = storage::load_data();
                        view! { <WeightChart history=db.bodyweight_history /> }
                    }}
                    <div class="bodyweight-ref">
                        {move || {
                            let bw = bodyweight.get();
                            let bw_display = bw.map(|w| format!("{:.1}", w)).unwrap_or("--.-".to_string());
                            view! {
                                <span class="bw-label">"Nuvarande vikt: "</span>
                                <span class="bw-value">{bw_display}</span>
                                <span class="bw-kg">" kg"</span>
                            }
                        }}
                    </div>
                </div>
                
                // ═══════════════════════════════════════════════════════════════
                // 6. REST TIME STATS
                // ═══════════════════════════════════════════════════════════════
                <div class="stat-card">
                    <div class="stat-card-title">"Vilotider"</div>
                    <div class="rest-stat-main">
                        <span class="rest-stat-value">{move || format_time(load_summary().avg_rest_time as i64)}</span>
                        <span class="rest-stat-label">"snitt vila"</span>
                    </div>
                    <div class="rest-stat-explain">
                        {move || {
                            let s = load_summary();
                            if s.avg_rest_time > 150.0 {
                                "⚠️ Längre vila = starkare lyft men längre pass"
                            } else if s.avg_rest_time > 90.0 {
                                "✓ Optimal vila för styrka"
                            } else if s.avg_rest_time > 45.0 {
                                "⚡ Kort vila = mer kondition, mindre styrka"
                            } else {
                                "💨 Väldigt snabb – bra för fettförbränning!"
                            }
                        }}
                    </div>
                </div>
                
                // ═══════════════════════════════════════════════════════════════
                // Exercise Details
                // ═══════════════════════════════════════════════════════════════
                {move || {
                    let _ = data_version.get();
                    let db = storage::load_data();
                    let all_ex_stats = db.get_all_exercise_stats();
                    if all_ex_stats.is_empty() {
                        view! { <div class="empty-state">"Kör ditt första pass!"</div> }.into_view()
                    } else {
                        view! {
                            <div class="exercise-stats-section">
                                <div class="section-title">"Övningar"</div>
                                <div class="exercise-stats-grid">
                                    {all_ex_stats.into_iter().map(|s| {
                                        view! { <ExerciseStatsCard stats=s /> }
                                    }).collect_view()}
                                </div>
                            </div>
                        }.into_view()
                    }
                }}
                
                // ═══════════════════════════════════════════════════════════════
                // History
                // ═══════════════════════════════════════════════════════════════
                {move || {
                    let sess = load_sessions();
                    (!sess.is_empty()).then(|| view! {
                        <div class="stat-card">
                            <div class="stat-card-title">"Historik"</div>
                            {sess.into_iter().map(|s| {
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
                    })
                }}
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

// ============ SETTINGS ============

#[component]
fn Settings(
    set_view: WriteSignal<AppView>,
    auth: ReadSignal<Option<AuthSession>>,
    set_auth: WriteSignal<Option<AuthSession>>,
) -> impl IntoView {
    let (routines, set_routines) = create_signal(Vec::<crate::types::SavedRoutine>::new());
    let (loading, set_loading) = create_signal(true);
    
    // Bodyweight state
    let db = storage::load_data();
    let initial_bw = db.get_bodyweight();
    let (bodyweight, set_bodyweight) = create_signal(initial_bw);
    let (editing_weight, set_editing_weight) = create_signal(false);
    let (weight_input, set_weight_input) = create_signal(
        initial_bw.map(|w| format!("{:.1}", w)).unwrap_or_default()
    );
    
    let save_bodyweight = move |_| {
        if let Ok(w) = weight_input.get().parse::<f64>() {
            set_bodyweight.set(Some(w));
            set_weight_input.set(format!("{:.1}", w));
            
            // Save to local storage cache
            let mut local_db = storage::load_data();
            local_db.set_bodyweight(w);
            let _ = storage::save_data(&local_db);
            
            // Sync to cloud - this now updates BOTH current weight and history
            crate::supabase::save_bodyweight_to_cloud(w);
            
            // Update auth session cache if it exists
            if let Some(mut session) = supabase::load_auth_session() {
                session.user.display_name = storage::load_display_name();
                supabase::save_auth_session(&session);
            }
        }
        set_editing_weight.set(false);
    };
    
    // Display name state
    let initial_name = storage::load_display_name().unwrap_or_default();
    let (display_name, set_display_name) = create_signal(initial_name.clone());
    let (editing_name, set_editing_name) = create_signal(false);
    let (name_input, set_name_input) = create_signal(initial_name);

    // Fetch name from cloud if it's missing or to ensure it's fresh
    create_effect(move |_| {
        spawn_local(async move {
            match supabase::fetch_display_name().await {
                Ok(Some(cloud_name)) => {
                    if !cloud_name.is_empty() {
                        set_display_name.set(cloud_name.clone());
                        set_name_input.set(cloud_name.clone());
                        storage::save_display_name(&cloud_name);
                        
                        // Update global auth signal too!
                        if let Some(mut session) = supabase::load_auth_session() {
                            session.user.display_name = Some(cloud_name);
                            supabase::save_auth_session(&session);
                            set_auth.set(Some(session));
                        }
                    }
                }
                _ => {}
            }
        });
    });
    
    let save_display_name = move |_| {
        let name = name_input.get();
        set_display_name.set(name.clone());
        storage::save_display_name(&name);
        
        // Sync to Supabase (cloud)
        supabase::save_display_name_to_cloud(&name);
        
        // Update auth session with new display name
        if let Some(mut session) = supabase::load_auth_session() {
            session.user.display_name = if name.is_empty() { None } else { Some(name.clone()) };
            supabase::save_auth_session(&session);
            // UPDATE THE GLOBAL AUTH SIGNAL
            set_auth.set(Some(session));
        }
        set_editing_name.set(false);
    };
    
    // Load routines on mount
    create_effect(move |_| {
        spawn_local(async move {
            match crate::supabase::fetch_routines().await {
                Ok(r) => {
                    set_routines.set(r);
                    set_loading.set(false);
                }
                Err(_) => {
                    set_loading.set(false);
                }
            }
        });
    });
    
    let do_set_active = move |id: String| {
        spawn_local(async move {
            let _ = crate::supabase::set_active_routine(&id).await;
            // Refresh list
            if let Ok(r) = crate::supabase::fetch_routines().await {
                set_routines.set(r);
            }
        });
    };
    
    let user_email = auth.get().map(|a| a.user.email.clone()).unwrap_or_default();
    
    view! {
        <div class="settings-container">
            <header class="settings-header">
                <button class="back-btn" on:click=move |_| set_view.set(AppView::Dashboard)>
                    "← Tillbaka"
                </button>
                <h1>"Inställningar"</h1>
            </header>
            
            <section class="settings-section">
                <h2>"Mina rutiner"</h2>
                
                {move || if loading.get() {
                    view! { <p class="loading-text">"Laddar rutiner..."</p> }.into_view()
                } else {
                    let routines_list = routines.get();
                    if routines_list.is_empty() {
                        view! {
                            <div class="empty-routines">
                                <p>"Inga rutiner ännu."</p>
                                <p>"Klicka nedan för att skapa din första!"</p>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <div class="routines-list">
                                {routines_list.into_iter().map(|r| {
                                    let id_for_edit = r.id.clone();
                                    let id_for_activate = r.id.clone();
                                    let is_active = r.is_active;
                                    view! {
                                        <div class=format!("routine-card {}", if is_active { "active" } else { "" })>
                                            <div class="routine-info">
                                                <span class="routine-name">{&r.name}</span>
                                                <span class="routine-passes">{format!("{} pass", r.passes.len())}</span>
                                                {is_active.then(|| view! { <span class="active-badge">"Aktiv"</span> })}
                                            </div>
                                            <div class="routine-actions">
                                                {(!is_active).then(|| {
                                                    let id_click = id_for_activate.clone();
                                                    view! {
                                                        <button class="activate-btn" on:click=move |_| do_set_active(id_click.clone())>
                                                            "Aktivera"
                                                        </button>
                                                    }
                                                })}
                                                <button class="edit-btn" on:click=move |_| set_view.set(AppView::RoutineBuilder(Some(id_for_edit.clone())))>
                                                    "Redigera"
                                                </button>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_view()
                    }
                }}
                
                <button class="create-routine-btn" on:click=move |_| set_view.set(AppView::RoutineBuilder(None))>
                    "+ Skapa ny rutin"
                </button>
            </section>
            
            <section class="settings-section">
                <h2>"Kroppsvikt"</h2>
                <p class="settings-hint">"Används för att beräkna relativ styrka och kalorier"</p>
                <div class="bodyweight-setting">
                    {move || {
                        if editing_weight.get() {
                            view! {
                                <div class="bw-edit-row">
                                    <input 
                                        type="number" 
                                        step="0.1"
                                        class="bw-input"
                                        prop:value=weight_input
                                        on:input=move |ev| set_weight_input.set(event_target_value(&ev))
                                    />
                                    <span class="bw-kg">"kg"</span>
                                    <button class="bw-save" on:click=save_bodyweight>"✓"</button>
                                    <button class="bw-cancel" on:click=move |_| set_editing_weight.set(false)>"✕"</button>
                                </div>
                            }.into_view()
                        } else {
                            let bw_display = bodyweight.get()
                                .map(|w| format!("{:.1} kg", w))
                                .unwrap_or("Ej angiven".to_string());
                            view! {
                                <div class="bw-display-row">
                                    <span class="bw-value">{bw_display}</span>
                                    <button class="bw-edit-btn" on:click=move |_| {
                                        let input_val = bodyweight.get()
                                            .map(|w| format!("{:.1}", w))
                                            .unwrap_or_default();
                                        set_weight_input.set(input_val);
                                        set_editing_weight.set(true);
                                    }>"Ändra"</button>
                                </div>
                            }.into_view()
                        }
                    }}
                </div>
            </section>
            
            <section class="settings-section">
                <h2>"Visningsnamn"</h2>
                <p class="settings-hint">"Visas på dashboarden istället för e-post"</p>
                <div class="display-name-setting">
                    {move || {
                        if editing_name.get() {
                            view! {
                                <div class="name-edit-row">
                                    <input 
                                        type="text"
                                        maxlength="30"
                                        class="name-input"
                                        placeholder="Ditt namn"
                                        prop:value=name_input
                                        on:input=move |ev| set_name_input.set(event_target_value(&ev))
                                    />
                                    <button class="name-save" on:click=save_display_name>"✓"</button>
                                    <button class="name-cancel" on:click=move |_| set_editing_name.set(false)>"✕"</button>
                                </div>
                            }.into_view()
                        } else {
                            let name_display = display_name.get();
                            let name_text = if name_display.is_empty() { 
                                "Ej angivet".to_string() 
                            } else { 
                                name_display 
                            };
                            view! {
                                <div class="name-display-row">
                                    <span class="name-value">{name_text}</span>
                                    <button class="name-edit-btn" on:click=move |_| {
                                        set_name_input.set(display_name.get());
                                        set_editing_name.set(true);
                                    }>"Ändra"</button>
                                </div>
                            }.into_view()
                        }
                    }}
                </div>
            </section>
            
            <section class="settings-section">
                <h2>"Konto"</h2>
                <div class="account-info">
                    <span class="account-email">{user_email}</span>
                    <button class="logout-btn" on:click=move |_| {
                        crate::supabase::sign_out();
                        set_auth.set(None);
                        set_view.set(AppView::Login);
                    }>"Logga ut"</button>
                </div>
            </section>
        </div>
    }
}

// ============ ROUTINE BUILDER ============

#[component]
fn RoutineBuilder(
    routine_id: Option<String>,
    set_view: WriteSignal<AppView>,
) -> impl IntoView {
    let (routine_name, set_routine_name) = create_signal(String::new());
    let (routine_focus, set_routine_focus) = create_signal(String::new());
    let (passes, set_passes) = create_signal(Vec::<crate::types::Pass>::new());
    let (loading, set_loading) = create_signal(routine_id.is_some());
    let (saving, set_saving) = create_signal(false);
    let (search_query, set_search_query) = create_signal(String::new());
    let (search_results, set_search_results) = create_signal(Vec::<WgerExercise>::new());
    let (searching, set_searching) = create_signal(false);
    let (selected_pass_idx, set_selected_pass_idx) = create_signal(0usize);
    // (pass_index, is_finisher)
    let (adding_exercise_to, set_adding_exercise_to) = create_signal(Option::<(usize, bool)>::None);
    // (pass_index, exercise_index) - which exercise we want to link as superset
    let (linking_superset, set_linking_superset) = create_signal(Option::<(usize, usize)>::None);
    // Delete state
    let (show_delete_confirm, set_show_delete_confirm) = create_signal(false);
    let (deleting, set_deleting) = create_signal(false);
    let is_editing = routine_id.is_some();
    let routine_id_for_save = routine_id.clone();
    let routine_id_for_delete = routine_id.clone();
    
    // Load existing routine if editing
    if let Some(ref id) = routine_id {
        let id_clone = id.clone();
        create_effect(move |_| {
            let id_inner = id_clone.clone();
            spawn_local(async move {
                if let Ok(routines) = crate::supabase::fetch_routines().await {
                    if let Some(r) = routines.into_iter().find(|r| r.id == id_inner) {
                        set_routine_name.set(r.name);
                        set_routine_focus.set(r.focus);
                        set_passes.set(r.passes);
                    }
                }
                set_loading.set(false);
            });
        });
    } else {
        // New routine - start with one empty pass
        set_passes.set(vec![crate::types::Pass {
            name: "Pass 1".to_string(),
            description: String::new(),
            exercises: vec![],
            finishers: vec![],
        }]);
    }
    
    // Search Wger API
    let trigger_search = move || {
        let query = search_query.get();
        if query.len() < 2 { return; }
        
        set_searching.set(true);
        spawn_local(async move {
            match search_wger_exercises(&query).await {
                Ok(results) => set_search_results.set(results),
                Err(_) => set_search_results.set(vec![]),
            }
            set_searching.set(false);
        });
    };
    let do_search = move |_: web_sys::MouseEvent| trigger_search();
    
    let add_pass = move |_| {
        let mut p = passes.get();
        let num = p.len() + 1;
        p.push(crate::types::Pass {
            name: format!("Pass {}", num),
            description: String::new(),
            exercises: vec![],
            finishers: vec![],
        });
        set_passes.set(p);
    };
    
    let rename_pass = move |idx: usize, new_name: String| {
        let mut p = passes.get();
        if let Some(pass) = p.get_mut(idx) {
            // Max 8 characters
            pass.name = new_name.chars().take(8).collect();
        }
        set_passes.set(p);
    };
    
    let update_pass_description = move |idx: usize, new_desc: String| {
        let mut p = passes.get();
        if let Some(pass) = p.get_mut(idx) {
            pass.description = new_desc;
        }
        set_passes.set(p);
    };
    
    let (trigger_save, set_trigger_save) = create_signal(false);
    
    // Effect to handle save
    {
        let routine_id_for_save = routine_id_for_save.clone();
        create_effect(move |_| {
            if trigger_save.get() {
                set_trigger_save.set(false);
                set_saving.set(true);
                let name = routine_name.get();
                let focus = routine_focus.get();
                let passes_data = passes.get();
                let existing_id = routine_id_for_save.clone();
                
                spawn_local(async move {
                    let now = js_sys::Date::now() as i64 / 1000;
                    let id = existing_id.unwrap_or_else(|| format!("routine_{}", now));
                    
                    let routine = crate::types::SavedRoutine {
                        id,
                        user_id: None, // Will be set by supabase::save_routine
                        name,
                        focus,
                        passes: passes_data,
                        is_active: true,
                        created_at: now,
                    };
                    
                    let _ = crate::supabase::save_routine(&routine).await;
                    if !is_editing {
                        let _ = crate::supabase::set_active_routine(&routine.id).await;
                    }
                    set_saving.set(false);
                    set_view.set(AppView::Settings);
                });
            }
        });
    }
    
    view! {
        <div class="routine-builder">
            <header class="builder-header">
                <button class="back-btn" on:click=move |_| set_view.set(AppView::Settings)>
                    "← Avbryt"
                </button>
                <h1>{if is_editing { "Redigera rutin" } else { "Ny rutin" }}</h1>
            </header>
            
            {move || if loading.get() {
                view! { <p class="loading-text">"Laddar..."</p> }.into_view()
            } else {
                view! {
                    <div class="builder-content">
                        <div class="builder-meta">
                            <input
                                type="text"
                                placeholder="Rutinens namn"
                                class="routine-name-input"
                                prop:value=routine_name
                                on:input=move |e| set_routine_name.set(event_target_value(&e))
                            />
                            <input
                                type="text"
                                placeholder="Fokus (t.ex. Styrka & Hypertrofi)"
                                class="routine-focus-input"
                                prop:value=routine_focus
                                on:input=move |e| set_routine_focus.set(event_target_value(&e))
                            />
                        </div>
                        
                        <div class="passes-tabs">
                            {move || passes.get().iter().enumerate().map(|(i, p)| {
                                let is_selected = selected_pass_idx.get() == i;
                                view! {
                                    <button
                                        class=format!("pass-tab {}", if is_selected { "selected" } else { "" })
                                        on:click=move |_| set_selected_pass_idx.set(i)
                                    >
                                        {&p.name}
                                    </button>
                                }
                            }).collect_view()}
                            <button class="add-pass-btn" on:click=add_pass>"+"</button>
                        </div>
                        
                        <div class="pass-editor">
                            {move || {
                                let idx = selected_pass_idx.get();
                                let p = passes.get();
                                if let Some(pass) = p.get(idx) {
                                    let pass_name = pass.name.clone();
                                    let pass_desc = pass.description.clone();
                                    view! {
                                        <div class="pass-meta-edit">
                                            <label class="field-label">"Namn (max 8 tecken)"</label>
                                            <input
                                                type="text"
                                                class="pass-name-input"
                                                maxlength="8"
                                                placeholder="Passnamn"
                                                value=pass_name
                                                on:blur=move |e| {
                                                    rename_pass(idx, event_target_value(&e));
                                                }
                                            />
                                            <label class="field-label">"Beskrivning"</label>
                                            <input
                                                type="text"
                                                class="pass-desc-input"
                                                placeholder="t.ex. Ben · Press · Triceps"
                                                value=pass_desc
                                                on:blur=move |e| {
                                                    update_pass_description(idx, event_target_value(&e));
                                                }
                                            />
                                        </div>
                                        <div class="pass-exercises">
                                            <h3>"Övningar"</h3>
                                            {pass.exercises.iter().enumerate().map(|(ei, ex)| {
                                                let has_superset = ex.is_superset && ex.superset_with.is_some();
                                                let superset_info = if has_superset {
                                                    format!(" ⟷ {}", ex.superset_with.as_ref().unwrap_or(&String::new()))
                                                } else {
                                                    String::new()
                                                };
                                                let ex_name_for_unlink = ex.name.clone();
                                                view! {
                                                    <div class={if has_superset { "exercise-item superset" } else { "exercise-item" }}>
                                                        <div class="exercise-main">
                                                            <span class="exercise-name">{&ex.name}</span>
                                                            <span class="exercise-detail">{format!("{}×{}", ex.sets, ex.reps_target)}</span>
                                                        </div>
                                                        {if has_superset {
                                                            let ex_name_unlink = ex_name_for_unlink.clone();
                                                            view! { 
                                                                <span class="superset-badge">{superset_info}</span>
                                                                <button class="unlink-superset-btn" title="Bryt superset" on:click=move |_| {
                                                                    let mut p = passes.get();
                                                                    if let Some(pass) = p.get_mut(idx) {
                                                                        // Find and unlink both exercises
                                                                        for ex in &mut pass.exercises {
                                                                            if ex.name == ex_name_unlink || ex.superset_with.as_ref() == Some(&ex_name_unlink) {
                                                                                ex.is_superset = false;
                                                                                ex.superset_with = None;
                                                                            }
                                                                        }
                                                                    }
                                                                    set_passes.set(p);
                                                                }>"✂"</button>
                                                            }.into_view()
                                                        } else {
                                                            view! {
                                                                <button class="link-superset-btn" on:click=move |_| {
                                                                    set_linking_superset.set(Some((idx, ei)));
                                                                }>"⟷"</button>
                                                            }.into_view()
                                                        }}
                                                        <button class="remove-exercise-btn" on:click=move |_| {
                                                            let mut p = passes.get();
                                                            if let Some(pass) = p.get_mut(idx) {
                                                                // Remove superset link from partner if exists
                                                                if let Some(partner_name) = pass.exercises.get(ei).and_then(|e| e.superset_with.clone()) {
                                                                    for other in &mut pass.exercises {
                                                                        if other.name == partner_name {
                                                                            other.is_superset = false;
                                                                            other.superset_with = None;
                                                                        }
                                                                    }
                                                                }
                                                                pass.exercises.remove(ei);
                                                            }
                                                            set_passes.set(p);
                                                        }>"×"</button>
                                                    </div>
                                                }
                                            }).collect_view()}
                                            
                                            <button class="add-exercise-btn" on:click=move |_| set_adding_exercise_to.set(Some((idx, false)))>
                                                "+ Lägg till övning"
                                            </button>
                                            
                                            <h3>"Finishers"</h3>
                                            {pass.finishers.iter().map(|ex| {
                                                view! {
                                                    <div class="exercise-item finisher">
                                                        <span class="exercise-name">{&ex.name}</span>
                                                        <span class="exercise-detail">{format!("{}×{}", ex.sets, ex.reps_target)}</span>
                                                    </div>
                                                }
                                            }).collect_view()}
                                            
                                            <button class="add-exercise-btn finisher-btn" on:click=move |_| set_adding_exercise_to.set(Some((idx, true)))>
                                                "+ Lägg till finisher"
                                            </button>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <p>"Välj ett pass"</p> }.into_view()
                                }
                            }}
                        </div>
                        
                        // Exercise search modal
                        {move || adding_exercise_to.get().map(|(pass_idx, is_finisher)| {
                            let title = if is_finisher { "Lägg till finisher" } else { "Sök övning" };
                            
                            // Common finisher exercises for quick-add
                            let common_finishers = vec![
                                ("Mountain Climbers", "30s", "Core, Cardio"),
                                ("Burpees", "30s", "Helkropp"),
                                ("Planka", "45s", "Core"),
                                ("Dead Bug", "30s", "Core"),
                                ("Jumping Jacks", "30s", "Cardio"),
                                ("Utfallssteg", "20 reps", "Ben"),
                                ("Shoulder Taps", "30s", "Core, Axlar"),
                                ("High Knees", "30s", "Cardio"),
                            ];
                            
                            view! {
                                <div class="exercise-search-modal">
                                    <div class="exercise-search-dialog">
                                        <h3>{title}</h3>
                                        
                                        // Quick-add section for finishers
                                        {if is_finisher {
                                            view! {
                                                <div class="quick-add-section">
                                                    <span class="quick-add-label">"Vanliga finishers:"</span>
                                                    <div class="quick-add-grid">
                                                        {common_finishers.into_iter().map(|(name, target, muscles)| {
                                                            let name_str = name.to_string();
                                                            let target_str = target.to_string();
                                                            let muscles_str = muscles.to_string();
                                                            view! {
                                                                <button class="quick-add-btn" on:click=move |_| {
                                                                    let mut p = passes.get();
                                                                    if let Some(pass) = p.get_mut(pass_idx) {
                                                                        let is_timed = target_str.contains("s");
                                                                        let duration = if is_timed {
                                                                            target_str.trim_end_matches('s').parse::<u32>().ok()
                                                                        } else {
                                                                            None
                                                                        };
                                                                        let new_ex = crate::types::Exercise {
                                                                            name: name_str.clone(),
                                                                            sets: 2,
                                                                            reps_target: target_str.clone(),
                                                                            is_superset: false,
                                                                            superset_with: None,
                                                                            superset_name: None,
                                                                            is_bodyweight: true,
                                                                            duration_secs: duration,
                                                                            primary_muscles: muscles_str.split(", ").map(|s| s.to_string()).collect(),
                                                                            secondary_muscles: vec![],
                                                                            image_url: None,
                                                                            equipment: Some("Kroppsvikt".to_string()),
                                                                            wger_id: None,
                                                                        };
                                                                        pass.finishers.push(new_ex);
                                                                    }
                                                                    set_passes.set(p);
                                                                    set_adding_exercise_to.set(None);
                                                                }>
                                                                    <span class="quick-name">{name}</span>
                                                                    <span class="quick-detail">{target}" · "{muscles}</span>
                                                                </button>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                </div>
                                                <div class="search-divider">"— eller sök —"</div>
                                            }.into_view()
                                        } else {
                                            view! { <span></span> }.into_view()
                                        }}
                                        
                                        <div class="search-box">
                                            <input
                                                type="text"
                                                placeholder={if is_finisher { "Sök kroppsviktsövning..." } else { "Sök (t.ex. bench, squat)" }}
                                                prop:value=search_query
                                                on:input=move |e| set_search_query.set(event_target_value(&e))
                                                on:keydown=move |e| {
                                                    if e.key() == "Enter" {
                                                        trigger_search();
                                                    }
                                                }
                                            />
                                            <button on:click=do_search disabled=searching>
                                                {if searching.get() { "..." } else { "Sök" }}
                                            </button>
                                        </div>
                                        
                                        <div class="search-results">
                                            {move || search_results.get().into_iter().map(|ex| {
                                                let ex_clone = ex.clone();
                                                view! {
                                                    <div class="search-result-item" on:click=move |_| {
                                                        // Add exercise to pass
                                                        let mut p = passes.get();
                                                        if let Some(pass) = p.get_mut(pass_idx) {
                                                            let mut new_ex = crate::types::Exercise::from_wger(
                                                                &ex_clone.name,
                                                                3,
                                                                if is_finisher { "30s" } else { "8-12" },
                                                                ex_clone.primary_muscles.clone(),
                                                                ex_clone.secondary_muscles.clone(),
                                                                ex_clone.image_url.clone(),
                                                                ex_clone.equipment.clone(),
                                                                ex_clone.id,
                                                            );
                                                            if is_finisher {
                                                                new_ex.is_bodyweight = true;
                                                                pass.finishers.push(new_ex);
                                                            } else {
                                                                pass.exercises.push(new_ex);
                                                            }
                                                        }
                                                        set_passes.set(p);
                                                        set_adding_exercise_to.set(None);
                                                        set_search_results.set(vec![]);
                                                        set_search_query.set(String::new());
                                                    }>
                                                        {ex.image_url.as_ref().map(|url| view! {
                                                            <img src=url.clone() class="result-thumb" />
                                                        })}
                                                        <div class="result-info">
                                                            <span class="result-name">{&ex.name}</span>
                                                            <span class="result-muscles">{ex.primary_muscles.join(", ")}</span>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                        
                                        <button class="close-search-btn" on:click=move |_| {
                                            set_adding_exercise_to.set(None);
                                            set_search_results.set(vec![]);
                                        }>"Stäng"</button>
                                    </div>
                                </div>
                            }
                        })}
                        
                        // Superset picker modal
                        {move || linking_superset.get().map(|(pass_idx, exercise_idx)| {
                            let p = passes.get();
                            let pass = p.get(pass_idx);
                            let source_name = pass.and_then(|p| p.exercises.get(exercise_idx)).map(|e| e.name.clone()).unwrap_or_default();
                            let available: Vec<(usize, String)> = pass.map(|p| {
                                p.exercises.iter().enumerate()
                                    .filter(|(i, ex)| *i != exercise_idx && !ex.is_superset)
                                    .map(|(i, ex)| (i, ex.name.clone()))
                                    .collect()
                            }).unwrap_or_default();
                            
                            view! {
                                <div class="superset-picker-modal">
                                    <div class="superset-picker-dialog">
                                        <h3>"Länka superset"</h3>
                                        <p class="superset-source">{format!("Länka \"{}\" med:", source_name)}</p>
                                        
                                        {if available.is_empty() {
                                            view! { <p class="no-options">"Inga övningar att länka med"</p> }.into_view()
                                        } else {
                                            view! {
                                                <div class="superset-options">
                                                    {available.into_iter().map(|(other_idx, other_name)| {
                                                        let name_for_closure = other_name.clone();
                                                        let source_for_closure = source_name.clone();
                                                        view! {
                                                            <button class="superset-option" on:click=move |_| {
                                                                let mut p = passes.get();
                                                                if let Some(pass) = p.get_mut(pass_idx) {
                                                                    // Link both exercises
                                                                    if let Some(ex1) = pass.exercises.get_mut(exercise_idx) {
                                                                        ex1.is_superset = true;
                                                                        ex1.superset_with = Some(name_for_closure.clone());
                                                                    }
                                                                    if let Some(ex2) = pass.exercises.get_mut(other_idx) {
                                                                        ex2.is_superset = true;
                                                                        ex2.superset_with = Some(source_for_closure.clone());
                                                                    }
                                                                }
                                                                set_passes.set(p);
                                                                set_linking_superset.set(None);
                                                            }>{other_name}</button>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_view()
                                        }}
                                        
                                        <button class="close-search-btn" on:click=move |_| {
                                            set_linking_superset.set(None);
                                        }>"Avbryt"</button>
                                    </div>
                                </div>
                            }
                        })}
                        
                        <button
                            class="save-routine-btn"
                            on:click=move |_| set_trigger_save.set(true)
                            disabled=saving
                        >
                            {if saving.get() { "Sparar..." } else { "Spara rutin" }}
                        </button>
                        
                        // Delete button (only when editing existing routine)
                        {is_editing.then(|| {
                            view! {
                                <button
                                    class="delete-routine-btn"
                                    on:click=move |_| set_show_delete_confirm.set(true)
                                >
                                    "Radera rutin"
                                </button>
                            }
                        })}
                    </div>
                }.into_view()
            }}
            
            // Delete confirmation modal
            {move || {
                let id_for_delete = routine_id_for_delete.clone();
                show_delete_confirm.get().then(|| {
                    let routine_name_display = routine_name.get();
                    view! {
                        <div class="delete-confirm-modal">
                            <div class="delete-confirm-dialog">
                                <h3>"Radera rutin?"</h3>
                                <p class="delete-warning">
                                    "Är du säker på att du vill radera "
                                    <strong>{routine_name_display}</strong>
                                    "? Detta kan inte ångras."
                                </p>
                                <div class="delete-confirm-actions">
                                    <button 
                                        class="cancel-delete-btn" 
                                        on:click=move |_| set_show_delete_confirm.set(false)
                                        disabled=deleting
                                    >
                                        "Avbryt"
                                    </button>
                                    <button 
                                        class="confirm-delete-btn" 
                                        on:click=move |_| {
                                            if let Some(ref id) = id_for_delete {
                                                let id_clone = id.clone();
                                                set_deleting.set(true);
                                                spawn_local(async move {
                                                    if crate::supabase::delete_routine(&id_clone).await.is_ok() {
                                                        storage::clear_active_routine();
                                                    }
                                                    set_deleting.set(false);
                                                    set_show_delete_confirm.set(false);
                                                    set_view.set(AppView::Settings);
                                                });
                                            }
                                        }
                                        disabled=deleting
                                    >
                                        {if deleting.get() { "Raderar..." } else { "Radera" }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                })
            }}
        </div>
    }
}

// Wger API types
#[derive(Clone, Debug, Serialize, Deserialize)]
struct WgerExercise {
    id: u32,
    name: String,
    primary_muscles: Vec<String>,
    secondary_muscles: Vec<String>,
    image_url: Option<String>,
    equipment: Option<String>,
}

async fn search_wger_exercises(query: &str) -> Result<Vec<WgerExercise>, JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    
    let url = format!("https://wger.de/api/v2/exercise/search/?language=2&term={}", query);
    let resp_value = JsFuture::from(window.fetch_with_str(&url)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        return Ok(vec![]);
    }
    
    let json = JsFuture::from(resp.json()?).await?;
    
    #[derive(Deserialize)]
    struct WgerSearchResponse {
        suggestions: Vec<WgerSuggestion>,
    }
    
    #[derive(Deserialize)]
    struct WgerSuggestion {
        value: String,
        data: WgerSuggestionData,
    }
    
    #[derive(Deserialize)]
    struct WgerSuggestionData {
        id: u32,
        name: String,
        category: String,
        image: Option<String>,
    }
    
    let search_resp: WgerSearchResponse = serde_wasm_bindgen::from_value(json).unwrap_or(WgerSearchResponse { suggestions: vec![] });
    
    // Convert to our format
    let exercises: Vec<WgerExercise> = search_resp.suggestions.into_iter().take(10).map(|s| {
        let image_url = s.data.image.map(|img| {
            if img.starts_with("http") {
                img
            } else {
                format!("https://wger.de{}", img)
            }
        });
        
        WgerExercise {
            id: s.data.id,
            name: s.data.name,
            primary_muscles: vec![s.data.category],
            secondary_muscles: vec![],
            image_url,
            equipment: None,
        }
    }).collect();
    
    Ok(exercises)
}
