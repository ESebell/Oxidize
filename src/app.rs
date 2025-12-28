use leptos::*;
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
            format!("{} dagar sedan", diff_days)
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
    let db = storage::load_data();
    let total = db.get_total_stats();
    let recent = db.get_recent_sessions(3);
    let paused = storage::load_paused_workout();
    
    let user_email = auth.get().map(|a| a.user.email).unwrap_or_default();
    
    // State for confirmation dialog
    let (show_confirm, set_show_confirm) = create_signal(false);
    let (pending_routine, set_pending_routine) = create_signal(String::new());
    
    let start_workout = move |routine: &str| {
        if storage::load_paused_workout().is_some() {
            set_pending_routine.set(routine.to_string());
            set_show_confirm.set(true);
        } else {
            set_view.set(AppView::Workout(routine.to_string()));
        }
    };
    
    let confirm_start = move |_| {
        storage::clear_paused_workout();
        set_view.set(AppView::Workout(pending_routine.get()));
    };

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
            
            // Paused workout banner
            {paused.map(|p| {
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
                        <button class="resume-btn" on:click=move |_| set_view.set(AppView::Workout(
                            if routine_name.contains("A") { "A".to_string() } else { "B".to_string() }
                        ))>
                            "Fortsätt →"
                        </button>
                    </div>
                }
            })}
            
            <button 
                class="start-btn pass-a"
                on:click=move |_| start_workout("A")
            >
                <span class="start-btn-label">"Pass A"</span>
                <span class="start-btn-focus">"Ben · Press · Triceps"</span>
            </button>
            
            <button 
                class="start-btn pass-b"
                on:click=move |_| start_workout("B")
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
            
            <button class="stats-link" on:click=move |_| set_view.set(AppView::Stats)>
                "Statistik →"
            </button>
            
            <div class="logged-in-info">
                "inloggad:"<br/>
                {user_email.clone()}<br/>
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
    }
}

#[component]
fn Workout(routine: String, set_view: WriteSignal<AppView>) -> impl IntoView {
    // Check for paused workout first
    if let Some(paused) = storage::load_paused_workout() {
        let expected_routine = if routine == "A" { "Pass A" } else { "Pass B" };
        if paused.routine_name == expected_routine {
            // Resume paused workout
            let data = storage::get_workout(&routine);
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
    
    let data = storage::get_workout(&routine);
    
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
    let is_pass_a = routine_name.contains("A");
    
    // Load bodyweight for calorie calculation
    let db = storage::load_data();
    let bodyweight = db.get_bodyweight();
    
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
    
    // Jump to specific exercise
    let jump_to_exercise = move |idx: usize| {
        set_current_idx.set(idx);
        set_is_resting.set(false);
        set_show_overview.set(false);
    };
    
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
            // Header with progress dots
            <div class="workout-header">
                <div class=format!("workout-title {}", accent_class)>{&routine.name}</div>
                
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
                                <div class="finish-icon">"✓"</div>
                                <div class="finish-title">"Bra jobbat!"</div>
                                <div class="finish-time">{format_time(elapsed.get())}</div>
                                <div class="finish-stats">
                                    <span class="finish-stat">{format!("{:.0} kg volym", total_volume)}</span>
                                    <span class="finish-stat">{format!("{} kcal", calories)}</span>
                                </div>
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
                                <a class="health-link" href={health_url} target="_blank">
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
                        let ss_with = ex.as_ref().and_then(|e| e.exercise.superset_with.clone());
                        // Exercise hints
                        let is_dumbbell = matches!(ex_name.as_str(), "Hammercurls" | "Sidolyft");
                        let is_alternating = matches!(ex_name.as_str(), "Utfallssteg" | "Dead Bug");
                        
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
                                
                                // Rep buttons - dynamically centered around target
                                <div class="rep-label">"Tryck antal reps:"</div>
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
fn Stats(set_view: WriteSignal<AppView>, auth: ReadSignal<Option<AuthSession>>, set_auth: WriteSignal<Option<AuthSession>>) -> impl IntoView {
    let db = storage::load_data();
    let initial_bodyweight = db.get_bodyweight();
    let summary = stats::get_stats_summary(&db, initial_bodyweight);
    let power_history = stats::get_power_score_history(&db);
    let all_stats = db.get_all_exercise_stats();
    
    let user_email = auth.get().map(|a| a.user.email).unwrap_or_default();
    
    let do_logout = move |_| {
        supabase::sign_out();
        set_auth.set(None);
        set_view.set(AppView::Login);
    };
    // Get sessions from last 61 days (2 months)
    let now = chrono::Utc::now().timestamp();
    let two_months_ago = now - (61 * 24 * 60 * 60);
    let sessions: Vec<_> = db.sessions.iter()
        .filter(|s| s.timestamp >= two_months_ago)
        .cloned()
        .collect();
    
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
                                        <button class="bw-display" on:click=move |_| {
                                            set_weight_input.set(format!("{:.1}", bodyweight.get()));
                                            set_editing_weight.set(true);
                                        }>
                                            <span class="bw-label">"Din vikt: "</span>
                                            <span class="bw-value">{format!("{:.1}", bw)}</span>
                                            <span class="bw-kg">" kg"</span>
                                            <span class="bw-edit-icon">" ✎"</span>
                                        </button>
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
                })}
                
                // ═══════════════════════════════════════════════════════════════
                // 5. MUSCLE HEATMAP (7 dagar)
                // Weighted scoring: primary muscles +3, secondary +1, × sets
                // ═══════════════════════════════════════════════════════════════
                <div class="stat-card">
                    <div class="stat-card-title">"Muskelaktivitet (7 dagar)"</div>
                    <div class="heatmap-grid">
                        {MuscleGroup::all().into_iter().map(|mg| {
                            let score = *summary.muscle_frequency.get(&mg).unwrap_or(&0);
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
                        }).collect_view()}
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
                            <div class="exercise-stats-grid">
                                {all_stats.into_iter().map(|s| {
                                    view! { <ExerciseStatsCard stats=s /> }
                                }).collect_view()}
                            </div>
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
