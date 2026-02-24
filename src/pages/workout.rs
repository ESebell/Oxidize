use leptos::*;
use crate::types::{
    AppView, WorkoutData, SetRecord, ExerciseRecord, ExerciseWorkoutState,
};
use crate::storage;
use crate::supabase;
use crate::app::{format_time, format_weight, parse_target_range, parse_target_reps};

#[component]
pub fn Workout(routine: String, set_view: WriteSignal<AppView>) -> impl IntoView {
    let pass_name = routine.clone();

    if let Some(paused) = storage::load_paused_workout() {
        if paused.routine_name == pass_name {
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
pub fn WorkoutActive(
    data: WorkoutData,
    set_view: WriteSignal<AppView>,
    #[prop(default = 0)] resumed_from: usize,
    #[prop(default = 0)] start_elapsed: i64,
) -> impl IntoView {
    let routine = data.routine.clone();
    let routine_name = routine.name.clone();
    let routine_name_save = routine_name.clone();
    let routine_name_pause = routine_name.clone();

    let db = storage::load_data();
    let bodyweight = db.get_bodyweight().unwrap_or(80.0);

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

    let (timer_running, set_timer_running) = create_signal(false);
    let (timer_selected_duration, set_timer_selected_duration) = create_signal(30u32);
    let (timer_remaining, set_timer_remaining) = create_signal(0i32);
    let (show_timer_flash, set_show_timer_flash) = create_signal(false);

    let jump_to_exercise = move |idx: usize| {
        set_current_idx.set(idx);
        set_is_resting.set(false);
        set_show_overview.set(false);
    };

    let (timer_just_completed, set_timer_just_completed) = create_signal(false);

    create_effect(move |_| {
        let handle = gloo_timers::callback::Interval::new(1000, move || {
            let now = js_sys::Date::now() as i64 / 1000;
            set_elapsed.set(now - start_time.get());
            if is_resting.get() && last_set_time.get() > 0 {
                set_rest_elapsed.set(now - last_set_time.get());
            }
            if timer_running.get() {
                let remaining = timer_remaining.get() - 1;
                if remaining <= 0 {
                    set_timer_remaining.set(0);
                    set_timer_running.set(false);
                    set_show_timer_flash.set(true);
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

    let find_partner_idx = move |exs: &[ExerciseWorkoutState], current_idx: usize| -> Option<usize> {
        let current = &exs[current_idx];
        if !current.exercise.is_superset {
            return None;
        }
        let partner_name = current.exercise.superset_with.as_ref()?;
        exs.iter().position(|e| &e.exercise.name == partner_name)
    };

    let complete_set = move |reps: u8| {
        let now = js_sys::Date::now() as i64 / 1000;
        let rest = if last_set_time.get() > 0 { Some(now - last_set_time.get()) } else { None };
        let idx = current_idx.get();

        let exs = exercises.get();
        let sets_done = exs[idx].sets_completed.len();
        let sets_target = exs[idx].exercise.sets as usize;
        let is_superset = exs[idx].exercise.is_superset;
        let _is_last_exercise = idx + 1 >= exs.len();

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

        let just_finished_exercise = sets_done + 1 >= sets_target;

        if just_finished_exercise {
            if is_superset {
                if let Some(partner_idx) = find_partner_idx(&exs, idx) {
                    let partner = &exs[partner_idx];
                    let partner_done = partner.sets_completed.len() >= partner.exercise.sets as usize;
                    if !partner_done {
                        set_current_idx.set(partner_idx);
                        set_is_resting.set(true);
                        return;
                    }
                }
            }
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
                set_is_finished.set(true);
                return;
            }
        } else if is_superset {
            if let Some(partner_idx) = find_partner_idx(&exs, idx) {
                let partner = &exs[partner_idx];
                let partner_done = partner.sets_completed.len() >= partner.exercise.sets as usize;
                if !partner_done {
                    set_current_idx.set(partner_idx);
                }
            }
        }
        set_is_resting.set(true);
    };

    let complete_timed_set = move || {
        let duration = timer_selected_duration.get();
        complete_set(duration as u8);
    };

    create_effect(move |_| {
        if timer_just_completed.get() {
            set_timer_just_completed.set(false);
            complete_timed_set();
        }
    });

    let start_timer = move |_| {
        let duration = timer_selected_duration.get();
        set_timer_remaining.set(duration as i32);
        set_timer_running.set(true);
    };

    let continue_workout = move |_| {
        set_is_resting.set(false);
    };

    let skip_exercise = move |_| {
        let idx = current_idx.get();
        let exs = exercises.get();
        let is_last = idx + 1 >= exs.len();

        if is_last {
            set_is_finished.set(true);
        } else {
            set_current_idx.set(idx + 1);
            set_is_resting.set(false);
        }
    };

    let adjust_weight = move |delta: f64| {
        let idx = current_idx.get();
        set_exercises.update(|exs| {
            if let Some(ex) = exs.get_mut(idx) {
                ex.current_weight = (ex.current_weight + delta).max(0.0);
            }
        });
    };

    let (routine_name_sig, _) = create_signal(routine_name_save);

    view! {
        <div class="workout">
            <div class="workout-header">
                <div class="workout-title">{&routine.name}</div>

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
                                        let mut result: Vec<View> = Vec::new();
                                        let mut i = 0;
                                        while i < exs.len() {
                                            let ex = &exs[i];
                                            let is_superset = ex.exercise.is_superset;

                                            let has_partner = is_superset && i + 1 < exs.len() &&
                                                exs[i + 1].exercise.is_superset &&
                                                exs[i + 1].exercise.superset_with.as_ref() == Some(&ex.exercise.name);

                                            if has_partner {
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

            <div class="workout-main">
                {move || {
                    if is_finished.get() {
                        let duration_mins = std::cmp::max(1, (elapsed.get() + 30) / 60);

                        let total_volume: f64 = exercises.get().iter()
                            .flat_map(|ex| ex.sets_completed.iter())
                            .map(|set| set.weight * set.reps as f64)
                            .sum();

                        let efficiency = if duration_mins > 0 {
                            total_volume / duration_mins as f64
                        } else {
                            0.0
                        };

                        let efficiency_bonus = (efficiency / 200.0).min(1.0) * 1.5;
                        let met = 5.0 + efficiency_bonus;

                        let hours = duration_mins as f64 / 60.0;
                        let calories = (hours * bodyweight * met).round() as i64;

                        let health_url = format!(
                            "shortcuts://run-shortcut?name=Oxidize&input=text&text={},{}",
                            duration_mins, calories
                        );
                        view! {
                            <div class="finish-screen">
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

                                            supabase::clear_sync_failed();

                                            let exs = exercises.get();
                                            let records: Vec<ExerciseRecord> = exs.iter()
                                                .filter(|e| !e.sets_completed.is_empty())
                                                .map(|e| ExerciseRecord {
                                                    name: e.exercise.name.clone(),
                                                    sets: e.sets_completed.clone(),
                                                    primary_muscles: e.exercise.primary_muscles.clone(),
                                                    secondary_muscles: e.exercise.secondary_muscles.clone(),
                                                })
                                                .collect();
                                            storage::save_session(routine_name_sig.get(), records, elapsed.get());

                                            use gloo_timers::callback::Interval;
                                            let check_count = std::rc::Rc::new(std::cell::RefCell::new(0));
                                            let check_count_clone = check_count.clone();
                                            let interval = Interval::new(500, move || {
                                                *check_count_clone.borrow_mut() += 1;
                                                let count = *check_count_clone.borrow();

                                                if supabase::get_sync_failed_session().is_some() {
                                                    set_is_saving.set(false);
                                                    set_show_sync_warning.set(true);
                                                    return;
                                                }

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
                        let ex = current_exercise();
                        let ex_name = ex.as_ref().map(|e| e.exercise.name.clone()).unwrap_or_default();
                        let is_superset = ex.as_ref().map(|e| e.exercise.is_superset).unwrap_or(false);
                        let is_bodyweight = ex.as_ref().map(|e| e.exercise.is_bodyweight).unwrap_or(false);
                        let is_timed = ex.as_ref().and_then(|e| e.exercise.duration_secs).is_some();
                        let target_duration = ex.as_ref().and_then(|e| e.exercise.duration_secs).unwrap_or(30);
                        let ss_with = ex.as_ref().and_then(|e| e.exercise.superset_with.clone());
                        let is_dumbbell = matches!(ex_name.as_str(), "Hammercurls" | "Sidolyft");
                        let is_alternating = matches!(ex_name.as_str(), "Utfallssteg" | "Dead Bug");

                        let last_duration = ex.as_ref()
                            .and_then(|e| e.last_data.as_ref())
                            .map(|d| d.reps as u32);

                        view! {
                            <div class=move || if show_timer_flash.get() { "exercise-screen timer-flash" } else { "exercise-screen" }>
                                <div class="exercise-progress">
                                    {move || format!("Set {} av {}", current_set_num(), total_sets())}
                                </div>

                                {is_superset.then(|| view! {
                                    <div class="superset-indicator">
                                        "Superset → " {ss_with.unwrap_or_default()}
                                    </div>
                                })}

                                {is_bodyweight.then(|| view! {
                                    <div class="bodyweight-indicator">
                                        "FINISHER"
                                    </div>
                                })}

                                <div class="exercise-name-big">{ex_name}</div>

                                {is_dumbbell.then(|| view! {
                                    <div class="exercise-hint">"Lägg ihop båda hantlarnas vikt"</div>
                                })}

                                {is_alternating.then(|| view! {
                                    <div class="exercise-hint">"Totalt antal reps (båda sidor)"</div>
                                })}

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

                                {is_timed.then(|| view! {
                                    <div class="timer-section">
                                        {move || if timer_running.get() {
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

                                            let center = (min + max) / 2;
                                            let start = (center as i32 - 5).max(1) as u8;
                                            let end = start + 11;

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

                                <button class="skip-exercise-btn" on:click=skip_exercise>
                                    "Hoppa över övning →"
                                </button>
                            </div>
                        }.into_view()
                    }
                }}
            </div>

            <div class="workout-footer">
                <button class="back-btn" on:click=move |_| {
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
