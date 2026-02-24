use leptos::*;
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;
use crate::types::AppView;
use crate::storage;

#[derive(Serialize, Deserialize, Debug)]
struct AiRoutineResponse {
    name: String,
    focus: String,
    passes: Vec<crate::types::Pass>,
}

const AI_SYSTEM_PROMPT: &str = r#"You are Oxidize AI, an expert fitness coach. Your task is to generate a workout routine in JSON format.
The JSON must strictly follow this structure:
{
  "name": "Routine Name",
  "focus": "Brief focus description",
  "passes": [
    {
      "name": "Pass A",
      "description": "Short pass description",
      "exercises": [
        {
          "name": "Exercise Name",
          "sets": 3,
          "reps_target": "8-10",
          "is_superset": false,
          "superset_with": null,
          "is_bodyweight": false
        }
      ],
      "finishers": []
    }
  ]
}

Rules:
1. 'reps_target' can be "5", "8-10", "12-15", or "30 sek" for timed exercises.
2. SUPERSETS - IMPORTANT:
   - A superset is TWO exercises performed back-to-back with no rest between them.
   - Only use supersets to SAVE TIME in passes with 5+ exercises.
   - Pair exercises for ANTAGONIST muscles (biceps+triceps, chest+back, quads+hamstrings).
   - If user doesn't want supersets, set is_superset: false for ALL exercises.
   - If you DO create a superset, BOTH exercises MUST be marked:
     * Exercise A: "is_superset": true, "superset_with": "Exercise B"
     * Exercise B: "is_superset": true, "superset_with": "Exercise A"
   - Never create a superset with only ONE exercise marked!
3. 'name' in 'passes' must be max 8 chars (e.g., "PASS A", "BEN", "PUSH").
4. Each pass should have 4-8 exercises depending on session duration requested.
5. RESPOND ONLY WITH THE RAW JSON. NO MARKDOWN OR EXPLANATIONS."#;

// Wger API types
#[derive(Clone, Debug, Serialize, Deserialize)]
struct WgerExercise {
    id: u32,
    base_id: u32,
    name: String,
    primary_muscles: Vec<String>,
    secondary_muscles: Vec<String>,
    image_url: Option<String>,
    equipment: Option<String>,
}

// Wger muscle ID → English name mapping
// Source: https://wger.de/api/v2/muscle/?format=json
fn wger_muscle_name(id: u32) -> &'static str {
    match id {
        1 => "Biceps brachii",
        2 => "Anterior deltoid",
        3 => "Serratus anterior",
        4 => "Pectoralis major",
        5 => "Obliquus externus abdominis",
        6 => "Rectus abdominis",
        7 => "Gastrocnemius",
        8 => "Gluteus maximus",
        9 => "Trapezius",
        10 => "Quadriceps femoris",
        11 => "Biceps femoris",
        12 => "Latissimus dorsi",
        13 => "Brachialis",
        14 => "Obliquus externus abdominis",
        15 => "Soleus",
        _ => "Unknown",
    }
}

async fn fetch_wger_muscles(base_id: u32) -> (Vec<String>, Vec<String>) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return (vec![], vec![]),
    };

    let url = format!("https://wger.de/api/v2/exerciseinfo/{}/?format=json", base_id);
    let resp_value = match JsFuture::from(window.fetch_with_str(&url)).await {
        Ok(v) => v,
        Err(_) => return (vec![], vec![]),
    };
    let resp: Response = match resp_value.dyn_into() {
        Ok(r) => r,
        Err(_) => return (vec![], vec![]),
    };
    if !resp.ok() { return (vec![], vec![]); }

    let json = match JsFuture::from(match resp.json() { Ok(j) => j, Err(_) => return (vec![], vec![]) }).await {
        Ok(j) => j,
        Err(_) => return (vec![], vec![]),
    };

    #[derive(Deserialize)]
    struct WgerMuscle { id: u32 }
    #[derive(Deserialize)]
    struct WgerExerciseInfo {
        muscles: Vec<WgerMuscle>,
        #[serde(default)]
        muscles_secondary: Vec<WgerMuscle>,
    }

    let info: WgerExerciseInfo = match serde_wasm_bindgen::from_value(json) {
        Ok(i) => i,
        Err(_) => return (vec![], vec![]),
    };

    let primary: Vec<String> = info.muscles.iter().map(|m| wger_muscle_name(m.id).to_string()).collect();
    let secondary: Vec<String> = info.muscles_secondary.iter().map(|m| wger_muscle_name(m.id).to_string()).collect();
    (primary, secondary)
}

async fn search_wger_exercises(query: &str) -> Result<Vec<WgerExercise>, JsValue> {
    let window = web_sys::window().ok_or("no window")?;

    let encoded_query = js_sys::encode_uri_component(query);
    let url = format!("https://wger.de/api/v2/exercise/search/?language=2&term={}", encoded_query);
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
        #[allow(dead_code)]
        value: String,
        data: WgerSuggestionData,
    }

    #[derive(Deserialize)]
    struct WgerSuggestionData {
        id: u32,
        base_id: u32,
        name: String,
        image: Option<String>,
    }

    let search_resp: WgerSearchResponse = serde_wasm_bindgen::from_value(json).unwrap_or(WgerSearchResponse { suggestions: vec![] });

    let suggestions: Vec<_> = search_resp.suggestions.into_iter().take(10).collect();
    let mut exercises = Vec::new();

    for s in suggestions {
        let image_url = s.data.image.map(|img| {
            if img.starts_with("http") {
                img
            } else {
                format!("https://wger.de{}", img)
            }
        });

        let (primary_muscles, secondary_muscles) = fetch_wger_muscles(s.data.base_id).await;

        exercises.push(WgerExercise {
            id: s.data.id,
            base_id: s.data.base_id,
            name: s.data.name,
            primary_muscles,
            secondary_muscles,
            image_url,
            equipment: None,
        });
    }

    Ok(exercises)
}

#[component]
pub fn RoutineBuilder(
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
    let (adding_exercise_to, set_adding_exercise_to) = create_signal(Option::<(usize, bool)>::None);
    let (linking_superset, set_linking_superset) = create_signal(Option::<(usize, usize)>::None);
    let (show_delete_confirm, set_show_delete_confirm) = create_signal(false);
    let (deleting, set_deleting) = create_signal(false);

    // AI Wizard state
    let (show_ai_wizard, set_show_ai_wizard) = create_signal(false);
    let (ai_step, set_ai_step) = create_signal(1u8);
    let (ai_pass_count, set_ai_pass_count) = create_signal(3u8);
    let (ai_focus, set_ai_focus) = create_signal("Styrka".to_string());
    let (ai_description, set_ai_description) = create_signal(String::new());
    let (ai_areas, set_ai_areas) = create_signal(String::new());
    let (ai_style, set_ai_style) = create_signal("Tunga lyft, få reps".to_string());
    let (ai_equipment, set_ai_equipment) = create_signal("Fullt gym".to_string());
    let (ai_duration, set_ai_duration) = create_signal("Normala (45-60 min)".to_string());
    let (ai_supersets, set_ai_supersets) = create_signal(true);
    let (ai_finishers, set_ai_finishers) = create_signal(true);
    let (ai_generating, set_ai_generating) = create_signal(false);
    let (ai_error, set_ai_error) = create_signal(None::<String>);

    let is_editing = routine_id.is_some();
    let routine_id_for_save = routine_id.clone();
    let routine_id_for_delete = routine_id.clone();

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
        set_passes.set(vec![crate::types::Pass {
            name: "Pass 1".to_string(),
            description: String::new(),
            exercises: vec![],
            finishers: vec![],
        }]);
    }

    let trigger_search = move || {
        let query = search_query.get();
        if query.len() < 2 {
            set_search_results.set(vec![]);
            return;
        }

        set_searching.set(true);
        spawn_local(async move {
            match search_wger_exercises(&query).await {
                Ok(results) => set_search_results.set(results),
                Err(_) => set_search_results.set(vec![]),
            }
            set_searching.set(false);
        });
    };

    // Debounced auto-search: store timeout handle so dropping cancels previous
    let debounce_handle = store_value(None::<gloo_timers::callback::Timeout>);

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

    let generate_with_ai = move || {
        set_ai_generating.set(true);
        set_ai_error.set(None);

        let pass_count = ai_pass_count.get();
        let focus = ai_focus.get();
        let desc = ai_description.get();
        let areas = ai_areas.get();
        let style = ai_style.get();
        let equip = ai_equipment.get();
        let duration = ai_duration.get();
        let ss = ai_supersets.get();
        let fin = ai_finishers.get();
        let bw = storage::load_data().get_bodyweight().unwrap_or(80.0);

        let user_prompt = format!(
            "Build a routine with {} unique passes. Goal: {}. \
             Qualitative Context: {}. Body parts to focus on: {}. Training style: {}. \
             Equipment available: {}. Preferred session duration: {}. \
             Include supersets: {}. Include finishers: {}. \
             User current bodyweight: {}kg.",
            pass_count, focus, desc, areas, style, equip, duration, ss, fin, bw
        );

        spawn_local(async move {
            match crate::supabase::fetch_api_key().await {
                Ok(Some(key)) => {
                    match crate::supabase::call_gemini(&key, AI_SYSTEM_PROMPT, &user_prompt).await {
                        Ok(json_str) => {
                            let clean_json = json_str.trim()
                                .trim_start_matches("```json")
                                .trim_start_matches("```")
                                .trim_end_matches("```")
                                .trim();

                            match serde_json::from_str::<AiRoutineResponse>(clean_json) {
                                Ok(mut resp) => {
                                    for pass in &mut resp.passes {
                                        let valid_supersets: Vec<(String, bool)> = pass.exercises.iter()
                                            .map(|ex| {
                                                if !ex.is_superset {
                                                    return (ex.name.clone(), true);
                                                }
                                                if let Some(partner_name) = &ex.superset_with {
                                                    let partner_valid = pass.exercises.iter()
                                                        .any(|e| &e.name == partner_name
                                                             && e.is_superset
                                                             && e.superset_with.as_ref() == Some(&ex.name));
                                                    (ex.name.clone(), partner_valid)
                                                } else {
                                                    (ex.name.clone(), false)
                                                }
                                            })
                                            .collect();

                                        for exercise in &mut pass.exercises {
                                            if let Some((_, is_valid)) = valid_supersets.iter()
                                                .find(|(name, _)| name == &exercise.name)
                                            {
                                                if !is_valid && exercise.is_superset {
                                                    web_sys::console::log_1(&format!(
                                                        "Removing broken superset from '{}'",
                                                        exercise.name
                                                    ).into());
                                                    exercise.is_superset = false;
                                                    exercise.superset_with = None;
                                                }
                                            }
                                        }
                                    }

                                    set_routine_name.set(resp.name);
                                    set_routine_focus.set(resp.focus);
                                    set_passes.set(resp.passes);
                                    set_show_ai_wizard.set(false);
                                }
                                Err(e) => {
                                    web_sys::console::log_1(&format!("JSON Parse Error: {}. Raw: {}", e, clean_json).into());
                                    set_ai_error.set(Some(format!("JSON-fel: {}", e)));
                                }
                            }
                        }
                        Err(e) => set_ai_error.set(Some(format!("AI-fel: {:?}", e))),
                    }
                }
                Ok(None) => set_ai_error.set(Some("AI-nyckel saknas i databasen (app_config)".to_string())),
                Err(e) => set_ai_error.set(Some(format!("Kopplingsfel: {:?}", e))),
            }
            set_ai_generating.set(false);
        });
    };

    let (trigger_save, set_trigger_save) = create_signal(false);

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
                        user_id: None,
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
                <button class="ai-magic-btn" on:click=move |_| set_show_ai_wizard.set(true)>
                    "AI 🪄"
                </button>
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
                                                let ex_sets = ex.sets.to_string();
                                                let ex_reps = ex.reps_target.clone();
                                                view! {
                                                    <div class={if has_superset { "exercise-item superset" } else { "exercise-item" }}>
                                                        <div class="exercise-main">
                                                            <span class="exercise-name">{&ex.name}</span>
                                                            <div class="exercise-edit">
                                                                <input type="number" class="sets-input" value=ex_sets
                                                                    on:blur=move |e| {
                                                                        let val = event_target_value(&e).parse::<u8>().unwrap_or(3);
                                                                        let mut p = passes.get();
                                                                        if let Some(pass) = p.get_mut(idx) {
                                                                            if let Some(exercise) = pass.exercises.get_mut(ei) {
                                                                                exercise.sets = val;
                                                                            }
                                                                        }
                                                                        set_passes.set(p);
                                                                    }
                                                                />
                                                                <span class="x-sep">"×"</span>
                                                                <input type="text" class="reps-input" value=ex_reps
                                                                    on:blur=move |e| {
                                                                        let val = event_target_value(&e);
                                                                        let mut p = passes.get();
                                                                        if let Some(pass) = p.get_mut(idx) {
                                                                            if let Some(exercise) = pass.exercises.get_mut(ei) {
                                                                                exercise.reps_target = val;
                                                                            }
                                                                        }
                                                                        set_passes.set(p);
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                        {if has_superset {
                                                            let ex_name_unlink = ex_name_for_unlink.clone();
                                                            view! {
                                                                <span class="superset-badge">{superset_info}</span>
                                                                <button class="unlink-superset-btn" title="Bryt superset" on:click=move |_| {
                                                                    let mut p = passes.get();
                                                                    if let Some(pass) = p.get_mut(idx) {
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

                                            <button class="add-exercise-btn" on:click=move |_| {
                                                set_search_query.set(String::new());
                                                set_search_results.set(vec![]);
                                                set_adding_exercise_to.set(Some((idx, false)));
                                            }>
                                                "+ Lägg till övning"
                                            </button>

                                            <h3>"Finishers"</h3>
                                            {pass.finishers.iter().enumerate().map(|(fi, ex)| {
                                                let fin_sets = ex.sets.to_string();
                                                let fin_reps = ex.reps_target.clone();
                                                let is_timed = ex.duration_secs.is_some();
                                                view! {
                                                    <div class="exercise-item finisher">
                                                        <span class="exercise-name">{&ex.name}</span>
                                                        <div class="exercise-edit">
                                                            <input type="number" class="sets-input" value=fin_sets
                                                                on:blur=move |e| {
                                                                    let val = event_target_value(&e).parse::<u8>().unwrap_or(2);
                                                                    let mut p = passes.get();
                                                                    if let Some(pass) = p.get_mut(idx) {
                                                                        if let Some(exercise) = pass.finishers.get_mut(fi) {
                                                                            exercise.sets = val;
                                                                        }
                                                                    }
                                                                    set_passes.set(p);
                                                                }
                                                            />
                                                            <span class="x-sep">"×"</span>
                                                            <input type="text" class="reps-input" value=fin_reps
                                                                on:blur=move |e| {
                                                                    let val = event_target_value(&e);
                                                                    let mut p = passes.get();
                                                                    if let Some(pass) = p.get_mut(idx) {
                                                                        if let Some(exercise) = pass.finishers.get_mut(fi) {
                                                                            // Sync duration_secs if timed
                                                                            if exercise.duration_secs.is_some() {
                                                                                let num = val.trim_end_matches(|c: char| !c.is_ascii_digit())
                                                                                    .parse::<u32>().unwrap_or(30);
                                                                                exercise.duration_secs = Some(num);
                                                                                exercise.reps_target = format!("{}s", num);
                                                                            } else {
                                                                                exercise.reps_target = val;
                                                                            }
                                                                        }
                                                                    }
                                                                    set_passes.set(p);
                                                                }
                                                            />
                                                            <button class={if is_timed { "timed-toggle active" } else { "timed-toggle" }}
                                                                title={if is_timed { "Timer-läge (klicka för reps)" } else { "Reps-läge (klicka för timer)" }}
                                                                on:click=move |_| {
                                                                    let mut p = passes.get();
                                                                    if let Some(pass) = p.get_mut(idx) {
                                                                        if let Some(exercise) = pass.finishers.get_mut(fi) {
                                                                            if exercise.duration_secs.is_some() {
                                                                                // Switch to reps mode
                                                                                exercise.duration_secs = None;
                                                                                exercise.reps_target = "10-15".to_string();
                                                                            } else {
                                                                                // Switch to timed mode
                                                                                exercise.duration_secs = Some(30);
                                                                                exercise.reps_target = "30s".to_string();
                                                                            }
                                                                        }
                                                                    }
                                                                    set_passes.set(p);
                                                                }
                                                            >
                                                                {if is_timed { "⏱" } else { "#" }}
                                                            </button>
                                                        </div>
                                                        <button class="remove-exercise-btn" on:click=move |_| {
                                                            let mut p = passes.get();
                                                            if let Some(pass) = p.get_mut(idx) {
                                                                pass.finishers.remove(fi);
                                                            }
                                                            set_passes.set(p);
                                                        }>"×"</button>
                                                    </div>
                                                }
                                            }).collect_view()}

                                            <button class="add-exercise-btn finisher-btn" on:click=move |_| {
                                                set_search_query.set(String::new());
                                                set_search_results.set(vec![]);
                                                set_adding_exercise_to.set(Some((idx, true)));
                                            }>
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
                                                type="search"
                                                enterkeyhint="search"
                                                placeholder={if is_finisher { "Sök övning..." } else { "Sök (t.ex. bench, squat)" }}
                                                prop:value=search_query
                                                on:input=move |e| {
                                                    let val = event_target_value(&e);
                                                    set_search_query.set(val.clone());
                                                    if val.len() >= 2 {
                                                        debounce_handle.set_value(Some(
                                                            gloo_timers::callback::Timeout::new(400, move || {
                                                                trigger_search();
                                                            })
                                                        ));
                                                    } else {
                                                        debounce_handle.set_value(None);
                                                        set_search_results.set(vec![]);
                                                    }
                                                }
                                                on:keydown=move |e| {
                                                    if e.key() == "Enter" {
                                                        e.prevent_default();
                                                        debounce_handle.set_value(None);
                                                        trigger_search();
                                                    }
                                                }
                                            />
                                            {move || searching.get().then(|| view! {
                                                <span class="search-spinner">"..."</span>
                                            })}
                                        </div>

                                        <div class="search-results">
                                            {move || search_results.get().into_iter().map(|ex| {
                                                let ex_clone = ex.clone();
                                                let thumb_url = ex.image_url.clone().unwrap_or_default();
                                                let has_thumb = !thumb_url.is_empty();
                                                view! {
                                                    <button class="search-result-item" on:click=move |_| {
                                                        let Some((pi, fin)) = adding_exercise_to.get() else { return };
                                                        let mut p = passes.get();
                                                        if let Some(pass) = p.get_mut(pi) {
                                                            let mut new_ex = crate::types::Exercise::from_wger(
                                                                &ex_clone.name,
                                                                if fin { 2 } else { 3 },
                                                                if fin { "10-15" } else { "8-12" },
                                                                ex_clone.primary_muscles.clone(),
                                                                ex_clone.secondary_muscles.clone(),
                                                                ex_clone.image_url.clone(),
                                                                ex_clone.equipment.clone(),
                                                                ex_clone.base_id,
                                                            );
                                                            if fin {
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
                                                        {if has_thumb {
                                                            view! { <img src=thumb_url class="result-thumb" /> }.into_view()
                                                        } else {
                                                            view! { <div class="result-thumb placeholder">"O"</div> }.into_view()
                                                        }}
                                                        <div class="result-info">
                                                            <span class="result-name">{&ex.name}</span>
                                                            <span class="result-muscles">{ex.primary_muscles.join(", ")}</span>
                                                        </div>
                                                    </button>
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
                                                    set_deleting.set(true);
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

            // AI Wizard Modal
            {move || show_ai_wizard.get().then(|| view! {
                <div class="ai-wizard-overlay">
                    <div class="ai-wizard-dialog">
                        <div class="ai-wizard-header">
                            <h2>"Oxidize AI Wizard 🪄"</h2>
                            <button class="close-btn" on:click=move |_| set_show_ai_wizard.set(false)>"×"</button>
                        </div>

                        <div class="ai-wizard-body">
                            {move || if ai_generating.get() {
                                view! {
                                    <div class="ai-generating-view">
                                        <div class="ai-spinner"></div>
                                        <div class="wizard-step-title">"Hjärnan jobbar..."</div>
                                        <p class="wizard-step-desc">"Bygger en optimerad rutin baserat på dina svar. Det tar bara några sekunder."</p>
                                    </div>
                                }.into_view()
                            } else if let Some(err) = ai_error.get() {
                                view! {
                                    <div class="ai-generating-view">
                                        <div class="wizard-step-title" style="color: #ff4444">"Hoppsan!"</div>
                                        <p class="wizard-step-desc">{err}</p>
                                        <button class="wizard-btn-next" on:click=move |_| set_ai_error.set(None)>"Försök igen"</button>
                                    </div>
                                }.into_view()
                            } else {
                                match ai_step.get() {
                                    1 => view! {
                                        <div class="wizard-step">
                                            <div class="wizard-step-title">"Struktur"</div>
                                            <p class="wizard-step-desc">"Hur många unika pass ska ingå i rutinen?"</p>
                                            <div class="wizard-option-grid">
                                                {[1, 2, 3, 4, 5].into_iter().map(|n| {
                                                    let is_sel = ai_pass_count.get() == n as u8;
                                                    view! {
                                                        <button
                                                            class=format!("wizard-option-btn {}", if is_sel { "selected" } else { "" })
                                                            on:click=move |_| set_ai_pass_count.set(n as u8)
                                                        >
                                                            {format!("{} pass", n)}
                                                        </button>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    }.into_view(),
                                    2 => view! {
                                        <div class="wizard-step">
                                            <div class="wizard-step-title">"Målsättning"</div>

                                            <div class="wizard-input-group">
                                                <label>"Huvudfokus"</label>
                                                <div class="wizard-option-grid">
                                                    {["Styrka", "Volym", "Funktionell"].into_iter().map(|f| {
                                                        let is_sel = ai_focus.get() == f;
                                                        view! {
                                                            <button
                                                                class=format!("wizard-option-btn {}", if is_sel { "selected" } else { "" })
                                                                on:click=move |_| set_ai_focus.set(f.to_string())
                                                            >
                                                                {f}
                                                            </button>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            </div>

                                            <div class="wizard-input-group" style="margin-top: 1rem">
                                                <label>"Beskriv med egna ord"</label>
                                                <textarea
                                                    class="wizard-textarea"
                                                    placeholder="T.ex: 'Vill bli stark i bänkpress men har ont i axeln...'"
                                                    on:input=move |e| set_ai_description.set(event_target_value(&e))
                                                    prop:value=ai_description
                                                ></textarea>
                                            </div>
                                        </div>
                                    }.into_view(),
                                    3 => view! {
                                        <div class="wizard-step">
                                            <div class="wizard-step-title">"Detaljer"</div>

                                            <div class="wizard-input-group">
                                                <label>"Prioritera områden"</label>
                                                <input
                                                    type="text"
                                                    class="name-input"
                                                    placeholder="T.ex: Armar, Rygg, Ben"
                                                    on:input=move |e| set_ai_areas.set(event_target_value(&e))
                                                    prop:value=ai_areas
                                                />
                                            </div>

                                            <div class="wizard-input-group" style="margin-top: 1rem">
                                                <label>"Träningsstil"</label>
                                                <div class="wizard-option-grid">
                                                    {["Tunga lyft, få reps", "Medeltungt, fler reps", "Hög puls, kort vila"].into_iter().map(|s| {
                                                        let is_sel = ai_style.get() == s;
                                                        view! {
                                                            <button
                                                                class=format!("wizard-option-btn {}", if is_sel { "selected" } else { "" })
                                                                on:click=move |_| set_ai_style.set(s.to_string())
                                                            >
                                                                {s}
                                                            </button>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            </div>
                                        </div>
                                    }.into_view(),
                                    4 => view! {
                                        <div class="wizard-step">
                                            <div class="wizard-step-title">"Förutsättningar"</div>

                                            <div class="wizard-input-group">
                                                <label>"Utrustning"</label>
                                                <div class="wizard-option-grid">
                                                    {["Fullt gym", "Hemma (hantlar)", "Bara kroppsvikt"].into_iter().map(|e| {
                                                        let is_sel = ai_equipment.get() == e;
                                                        view! {
                                                            <button
                                                                class=format!("wizard-option-btn {}", if is_sel { "selected" } else { "" })
                                                                on:click=move |_| set_ai_equipment.set(e.to_string())
                                                            >
                                                                {e}
                                                            </button>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            </div>

                                            <div class="wizard-input-group" style="margin-top: 1rem">
                                                <label>"Passens längd"</label>
                                                <div class="wizard-option-grid">
                                                    {["Korta (30 min)", "Normala (45-60 min)", "Långa (90 min+)"].into_iter().map(|d| {
                                                        let is_sel = ai_duration.get() == d;
                                                        view! {
                                                            <button
                                                                class=format!("wizard-option-btn {}", if is_sel { "selected" } else { "" })
                                                                on:click=move |_| set_ai_duration.set(d.to_string())
                                                            >
                                                                {d}
                                                            </button>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            </div>
                                        </div>
                                    }.into_view(),
                                    5 => view! {
                                        <div class="wizard-step">
                                            <div class="wizard-step-title">"Oxidize-val"</div>

                                            <div class="wizard-toggle-row" style="margin-bottom: 1rem">
                                                <span>"Använd Supersets"</span>
                                                <button
                                                    class=format!("wizard-option-btn {}", if ai_supersets.get() { "selected" } else { "" })
                                                    on:click=move |_| set_ai_supersets.update(|v| *v = !*v)
                                                >
                                                    {if ai_supersets.get() { "PÅ" } else { "AV" }}
                                                </button>
                                            </div>

                                            <div class="wizard-toggle-row">
                                                <span>"Inkludera Finishers"</span>
                                                <button
                                                    class=format!("wizard-option-btn {}", if ai_finishers.get() { "selected" } else { "" })
                                                    on:click=move |_| set_ai_finishers.update(|v| *v = !*v)
                                                >
                                                    {if ai_finishers.get() { "PÅ" } else { "AV" }}
                                                </button>
                                            </div>

                                            <p class="wizard-step-desc" style="margin-top: 1.5rem">
                                                "Nu är vi klara! Klicka på Generera för att låta AI:n bygga din rutin."
                                            </p>
                                        </div>
                                    }.into_view(),
                                    _ => view! { <div></div> }.into_view()
                                }
                            }}
                        </div>

                        {move || (!ai_generating.get() && ai_error.get().is_none()).then(|| view! {
                            <div class="ai-wizard-footer">
                                {move || if ai_step.get() > 1 {
                                    view! {
                                        <button class="wizard-btn-prev" on:click=move |_| set_ai_step.update(|s| *s -= 1)>
                                            "Bakåt"
                                        </button>
                                    }.into_view()
                                } else {
                                    view! { <div /> }.into_view()
                                }}

                                {move || if ai_step.get() < 5 {
                                    view! {
                                        <button class="wizard-btn-next" on:click=move |_| set_ai_step.update(|s| *s += 1)>
                                            "Nästa"
                                        </button>
                                    }.into_view()
                                } else {
                                    view! {
                                        <button class="wizard-btn-next" on:click=move |_| generate_with_ai()>
                                            "Generera Rutin 🪄"
                                        </button>
                                    }.into_view()
                                }}
                            </div>
                        })}
                    </div>
                </div>
            })}
        </div>
    }
}
