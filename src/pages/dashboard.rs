use leptos::*;
use crate::types::{AppView, AuthSession};
use crate::storage;
use crate::supabase;
use crate::app::{format_time, format_date};

#[component]
pub fn Dashboard(set_view: WriteSignal<AppView>, auth: ReadSignal<Option<AuthSession>>) -> impl IntoView {
    supabase::check_and_refresh_session();

    if supabase::load_auth_session().is_none() {
        set_view.set(AppView::Login);
        return view! { <div class="loading">"Sessionen har gått ut..."</div> }.into_view();
    }

    storage::reset_sync_status();
    supabase::sync_from_cloud();

    let (data_version, set_data_version) = create_signal(storage::get_data_version());
    let (is_loading, set_is_loading) = create_signal(true);

    let (active_routine, set_active_routine) = create_signal(Option::<crate::types::SavedRoutine>::None);
    let (routine_loading, set_routine_loading) = create_signal(true);

    create_effect(move |_| {
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(100).await;

            match supabase::fetch_routines().await {
                Ok(routines) => {
                    let mut active = routines.into_iter().find(|r| r.is_active);
                    if let Some(ref mut r) = active {
                        storage::migrate_routine_names(r);
                        storage::save_active_routine(r);
                    }
                    set_active_routine.set(active);
                }
                Err(_) => {
                    set_active_routine.set(storage::load_active_routine());
                }
            }
            set_routine_loading.set(false);
        });
    });

    if !storage::is_sync_complete() {
        use gloo_timers::callback::Interval;
        let interval = Interval::new(200, move || {
            if storage::is_sync_complete() {
                set_is_loading.set(false);
                set_data_version.set(storage::get_data_version());
            }
        });
        leptos::on_cleanup(move || drop(interval));
    }

    let stats = create_memo(move |_| {
        let _ = data_version.get();
        let db = storage::load_data();
        let total = db.get_total_stats();
        let recent = db.get_recent_sessions(1);
        (total, recent)
    });

    let paused = create_memo(move |_| {
        let _ = data_version.get();
        storage::load_paused_workout()
    });

    let user_display = move || {
        let _ = data_version.get();
        storage::load_display_name()
            .or_else(|| auth.get().and_then(|a| a.user.display_name.clone()))
            .or_else(|| auth.get().map(|a| a.user.email.clone()))
            .unwrap_or_default()
    };

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
