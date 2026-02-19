use leptos::*;
use crate::types::AppView;
use crate::types::AuthSession;
use crate::storage;
use crate::supabase;

#[component]
pub fn Login(set_view: WriteSignal<AppView>, set_auth: WriteSignal<Option<AuthSession>>) -> impl IntoView {
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
pub fn Register(set_view: WriteSignal<AppView>, set_auth: WriteSignal<Option<AuthSession>>) -> impl IntoView {
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
