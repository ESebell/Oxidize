use leptos::*;
use crate::types::{AppView, AuthSession};
use crate::storage;
use crate::supabase;

#[component]
pub fn Settings(
    set_view: WriteSignal<AppView>,
    auth: ReadSignal<Option<AuthSession>>,
    set_auth: WriteSignal<Option<AuthSession>>,
) -> impl IntoView {
    let (routines, set_routines) = create_signal(Vec::<crate::types::SavedRoutine>::new());
    let (loading, set_loading) = create_signal(true);

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

            let mut local_db = storage::load_data();
            local_db.set_bodyweight(w);
            let _ = storage::save_data(&local_db);

            crate::supabase::save_bodyweight_to_cloud(w);

            if let Some(mut session) = supabase::load_auth_session() {
                session.user.display_name = storage::load_display_name();
                supabase::save_auth_session(&session);
            }
        }
        set_editing_weight.set(false);
    };

    let initial_name = storage::load_display_name().unwrap_or_default();
    let (display_name, set_display_name) = create_signal(initial_name.clone());
    let (editing_name, set_editing_name) = create_signal(false);
    let (name_input, set_name_input) = create_signal(initial_name);

    create_effect(move |_| {
        spawn_local(async move {
            match supabase::fetch_display_name().await {
                Ok(Some(cloud_name)) => {
                    if !cloud_name.is_empty() {
                        set_display_name.set(cloud_name.clone());
                        set_name_input.set(cloud_name.clone());
                        storage::save_display_name(&cloud_name);

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

        supabase::save_display_name_to_cloud(&name);

        if let Some(mut session) = supabase::load_auth_session() {
            session.user.display_name = if name.is_empty() { None } else { Some(name.clone()) };
            supabase::save_auth_session(&session);
            set_auth.set(Some(session));
        }
        set_editing_name.set(false);
    };

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
