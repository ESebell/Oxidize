use leptos::*;
use crate::types::AppView;
use crate::supabase;
use crate::pages::*;

pub(crate) fn format_time(secs: i64) -> String {
    let mins = secs / 60;
    let s = secs % 60;
    format!("{:02}:{:02}", mins, s)
}

pub(crate) fn format_date(ts: i64) -> String {
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

pub(crate) fn format_weight(w: f64) -> String {
    if w.fract() == 0.0 { format!("{:.0}", w) }
    else { format!("{:.1}", w) }
}

pub(crate) fn parse_target_range(target: &str) -> (u8, u8) {
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

pub(crate) fn parse_target_reps(target: &str) -> u8 {
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
                AppView::Stats => view! { <Stats set_view=set_view set_auth=set_auth /> }.into_view(),
                AppView::Settings => view! { <Settings set_view=set_view auth=auth set_auth=set_auth /> }.into_view(),
                AppView::RoutineBuilder(id) => view! { <RoutineBuilder routine_id=id set_view=set_view /> }.into_view(),
            }}
        </div>
    }
}
