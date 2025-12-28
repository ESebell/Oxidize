use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, Headers};
use serde::{Deserialize, Serialize};

const SUPABASE_URL: &str = "https://ytnwppbepeojvyedrbnb.supabase.co";
const SUPABASE_KEY: &str = "sb_publishable_Oqp9Oc-Io5o3o3MUwIVD2A_Tvv_dCuS";
const AUTH_SESSION_KEY: &str = "oxidize_auth_session";

use crate::types::{Session, AuthSession, AuthUser};

// ============ AUTH ============

#[derive(Deserialize, Debug)]
struct SupabaseAuthResponse {
    access_token: String,
    user: SupabaseUser,
}

#[derive(Deserialize, Debug)]
struct SupabaseUser {
    id: String,
    email: String,
}

#[derive(Deserialize, Debug)]
struct SupabaseError {
    error: Option<String>,
    error_description: Option<String>,
    msg: Option<String>,
}

/// Sign up with email and password
pub async fn sign_up(email: &str, password: &str) -> Result<AuthSession, String> {
    let window = web_sys::window().ok_or("no window")?;
    
    let body = serde_json::json!({
        "email": email,
        "password": password
    }).to_string();
    
    let headers = Headers::new().map_err(|_| "Failed to create headers")?;
    headers.set("apikey", SUPABASE_KEY).map_err(|_| "Failed to set apikey")?;
    headers.set("Content-Type", "application/json").map_err(|_| "Failed to set content-type")?;
    
    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);
    opts.set_body(&JsValue::from_str(&body));
    opts.set_headers(&JsValue::from(&headers));
    
    let url = format!("{}/auth/v1/signup", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts).map_err(|_| "Failed to create request")?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.map_err(|_| "Fetch failed")?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "Invalid response")?;
    
    let json = JsFuture::from(resp.json().map_err(|_| "No JSON")?).await.map_err(|_| "JSON parse failed")?;
    
    if !resp.ok() {
        let err: SupabaseError = serde_wasm_bindgen::from_value(json).unwrap_or(SupabaseError { error: Some("Unknown error".into()), error_description: None, msg: None });
        return Err(err.error_description.or(err.msg).or(err.error).unwrap_or("Registration failed".into()));
    }
    
    let auth_resp: SupabaseAuthResponse = serde_wasm_bindgen::from_value(json).map_err(|_| "Invalid auth response")?;
    
    let session = AuthSession {
        access_token: auth_resp.access_token,
        user: AuthUser {
            id: auth_resp.user.id,
            email: auth_resp.user.email,
        },
    };
    
    save_auth_session(&session);
    Ok(session)
}

/// Sign in with email and password
pub async fn sign_in(email: &str, password: &str) -> Result<AuthSession, String> {
    let window = web_sys::window().ok_or("no window")?;
    
    let body = serde_json::json!({
        "email": email,
        "password": password
    }).to_string();
    
    let headers = Headers::new().map_err(|_| "Failed to create headers")?;
    headers.set("apikey", SUPABASE_KEY).map_err(|_| "Failed to set apikey")?;
    headers.set("Content-Type", "application/json").map_err(|_| "Failed to set content-type")?;
    
    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);
    opts.set_body(&JsValue::from_str(&body));
    opts.set_headers(&JsValue::from(&headers));
    
    let url = format!("{}/auth/v1/token?grant_type=password", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts).map_err(|_| "Failed to create request")?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.map_err(|_| "Fetch failed")?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "Invalid response")?;
    
    let json = JsFuture::from(resp.json().map_err(|_| "No JSON")?).await.map_err(|_| "JSON parse failed")?;
    
    if !resp.ok() {
        let err: SupabaseError = serde_wasm_bindgen::from_value(json).unwrap_or(SupabaseError { error: Some("Unknown error".into()), error_description: None, msg: None });
        return Err(err.error_description.or(err.msg).or(err.error).unwrap_or("Login failed".into()));
    }
    
    let auth_resp: SupabaseAuthResponse = serde_wasm_bindgen::from_value(json).map_err(|_| "Invalid auth response")?;
    
    let session = AuthSession {
        access_token: auth_resp.access_token,
        user: AuthUser {
            id: auth_resp.user.id,
            email: auth_resp.user.email,
        },
    };
    
    save_auth_session(&session);
    Ok(session)
}

/// Sign out
pub fn sign_out() {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok()).flatten() {
        let _ = storage.remove_item(AUTH_SESSION_KEY);
    }
}

/// Save auth session to localStorage
fn save_auth_session(session: &AuthSession) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok()).flatten() {
        if let Ok(json) = serde_json::to_string(session) {
            let _ = storage.set_item(AUTH_SESSION_KEY, &json);
        }
    }
}

/// Load auth session from localStorage
pub fn load_auth_session() -> Option<AuthSession> {
    let storage = web_sys::window()?.local_storage().ok()??;
    let json = storage.get_item(AUTH_SESSION_KEY).ok()??;
    serde_json::from_str(&json).ok()
}

/// Get current user ID
pub fn get_current_user_id() -> Option<String> {
    load_auth_session().map(|s| s.user.id)
}

// ============ DATA (with user_id) ============

#[derive(Serialize, Deserialize, Debug)]
struct SessionRow {
    id: String,
    routine: String,
    timestamp: i64,
    duration_secs: i64,
    total_volume: f64,
    exercises: serde_json::Value,
    user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LastWeightRow {
    exercise_name: String,
    weight: f64,
    reps: i16,
    user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BodyweightRow {
    id: Option<i32>,
    weight: f64,
    timestamp: i64,
    user_id: Option<String>,
}

fn get_headers() -> Result<Headers, JsValue> {
    let headers = Headers::new()?;
    headers.set("apikey", SUPABASE_KEY)?;
    
    // Use user's token if logged in, otherwise anon key
    if let Some(session) = load_auth_session() {
        headers.set("Authorization", &format!("Bearer {}", session.access_token))?;
    } else {
        headers.set("Authorization", &format!("Bearer {}", SUPABASE_KEY))?;
    }
    
    headers.set("Content-Type", "application/json")?;
    Ok(headers)
}

fn create_request_init(method: &str, body: Option<&str>, headers: &Headers) -> RequestInit {
    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::Cors);
    if let Some(b) = body {
        opts.set_body(&JsValue::from_str(b));
    }
    opts.set_headers(&JsValue::from(headers));
    opts
}

/// Save session to Supabase (fire and forget)
pub fn save_session_to_cloud(session: &Session) {
    let session = session.clone();
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = save_session_async(&session).await {
            web_sys::console::log_1(&format!("Supabase save failed: {:?}", e).into());
        } else {
            web_sys::console::log_1(&"Saved to Supabase".into());
        }
    });
}

async fn save_session_async(session: &Session) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let user_id = get_current_user_id();
    
    // Convert session to row format
    let exercises_json = serde_json::to_value(&session.exercises).map_err(|e| e.to_string())?;
    let row = SessionRow {
        id: session.id.clone(),
        routine: session.routine.clone(),
        timestamp: session.timestamp,
        duration_secs: session.duration_secs,
        total_volume: session.total_volume,
        exercises: exercises_json,
        user_id,
    };
    
    let body = serde_json::to_string(&row).map_err(|e| e.to_string())?;
    let headers = get_headers()?;
    let opts = create_request_init("POST", Some(&body), &headers);
    
    let url = format!("{}/rest/v1/sessions", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        let status = resp.status();
        return Err(format!("HTTP error: {}", status).into());
    }
    
    Ok(())
}

/// Save last weight to Supabase
pub fn save_weight_to_cloud(exercise_name: &str, weight: f64, reps: u8) {
    let exercise_name = exercise_name.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = save_weight_async(&exercise_name, weight, reps).await {
            web_sys::console::log_1(&format!("Supabase weight save failed: {:?}", e).into());
        }
    });
}

async fn save_weight_async(exercise_name: &str, weight: f64, reps: u8) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let user_id = get_current_user_id();
    
    let row = LastWeightRow {
        exercise_name: exercise_name.to_string(),
        weight,
        reps: reps as i16,
        user_id,
    };
    
    let body = serde_json::to_string(&row).map_err(|e| e.to_string())?;
    
    let headers = get_headers()?;
    headers.set("Prefer", "resolution=merge-duplicates")?;
    
    let opts = create_request_init("POST", Some(&body), &headers);
    
    let url = format!("{}/rest/v1/last_weights", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        let status = resp.status();
        return Err(format!("HTTP error: {}", status).into());
    }
    
    Ok(())
}

/// Fetch all sessions from Supabase
pub async fn fetch_sessions() -> Result<Vec<Session>, JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    
    let headers = get_headers()?;
    let opts = create_request_init("GET", None, &headers);
    
    let url = format!("{}/rest/v1/sessions?select=*", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()).into());
    }
    
    let json = JsFuture::from(resp.json()?).await?;
    let rows: Vec<SessionRow> = serde_wasm_bindgen::from_value(json)?;
    
    // Convert rows to Sessions
    let sessions: Vec<Session> = rows.into_iter().filter_map(|row| {
        let exercises = serde_json::from_value(row.exercises).ok()?;
        Some(Session {
            id: row.id,
            routine: row.routine,
            timestamp: row.timestamp,
            duration_secs: row.duration_secs,
            total_volume: row.total_volume,
            exercises,
        })
    }).collect();
    
    Ok(sessions)
}

/// Fetch all last weights from Supabase  
pub async fn fetch_last_weights() -> Result<std::collections::HashMap<String, crate::types::LastExerciseData>, JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    
    let headers = get_headers()?;
    let opts = create_request_init("GET", None, &headers);
    
    let url = format!("{}/rest/v1/last_weights?select=*", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()).into());
    }
    
    let json = JsFuture::from(resp.json()?).await?;
    let rows: Vec<LastWeightRow> = serde_wasm_bindgen::from_value(json)?;
    
    let mut map = std::collections::HashMap::new();
    for row in rows {
        map.insert(row.exercise_name, crate::types::LastExerciseData {
            weight: row.weight,
            reps: row.reps as u8,
        });
    }
    
    Ok(map)
}

/// Save bodyweight to Supabase
pub fn save_bodyweight_to_cloud(weight: f64) {
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = save_bodyweight_async(weight).await {
            web_sys::console::log_1(&format!("Supabase bodyweight save failed: {:?}", e).into());
        }
    });
}

async fn save_bodyweight_async(weight: f64) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let user_id = get_current_user_id();
    
    let timestamp = js_sys::Date::now() as i64 / 1000;
    let body = if let Some(uid) = user_id {
        format!(r#"{{"weight": {}, "timestamp": {}, "user_id": "{}"}}"#, weight, timestamp, uid)
    } else {
        format!(r#"{{"weight": {}, "timestamp": {}}}"#, weight, timestamp)
    };
    
    let headers = get_headers()?;
    let opts = create_request_init("POST", Some(&body), &headers);
    
    let url = format!("{}/rest/v1/bodyweight", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()).into());
    }
    
    Ok(())
}

/// Fetch bodyweight history from Supabase
pub async fn fetch_bodyweight() -> Result<(Option<f64>, Vec<crate::storage::BodyweightEntry>), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    
    let headers = get_headers()?;
    let opts = create_request_init("GET", None, &headers);
    
    let url = format!("{}/rest/v1/bodyweight?select=*&order=timestamp.desc", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()).into());
    }
    
    let json = JsFuture::from(resp.json()?).await?;
    let rows: Vec<BodyweightRow> = serde_wasm_bindgen::from_value(json)?;
    
    let current = rows.first().map(|r| r.weight);
    let history: Vec<crate::storage::BodyweightEntry> = rows.into_iter()
        .map(|r| crate::storage::BodyweightEntry {
            timestamp: r.timestamp,
            weight: r.weight,
        })
        .collect();
    
    Ok((current, history))
}

/// Sync local data with Supabase (call on app start)
pub fn sync_from_cloud() {
    wasm_bindgen_futures::spawn_local(async {
        match do_sync().await {
            Ok(_) => web_sys::console::log_1(&"Synced from Supabase".into()),
            Err(e) => web_sys::console::log_1(&format!("Sync failed: {:?}", e).into()),
        }
        // Mark sync as complete (even if failed, so UI doesn't show loading forever)
        crate::storage::mark_sync_complete();
    });
}

async fn do_sync() -> Result<(), JsValue> {
    // Fetch from cloud
    let cloud_sessions = fetch_sessions().await?;
    let cloud_weights = fetch_last_weights().await?;
    let (cloud_bodyweight, cloud_bw_history) = fetch_bodyweight().await.unwrap_or((None, vec![]));
    
    // Load local data
    let mut local_db = crate::storage::load_data();
    
    // Merge sessions (add any from cloud that we don't have locally)
    let local_ids: std::collections::HashSet<_> = local_db.sessions.iter().map(|s| s.id.clone()).collect();
    for session in cloud_sessions {
        if !local_ids.contains(&session.id) {
            local_db.sessions.push(session);
        }
    }
    
    // Merge weights (cloud takes precedence if newer data exists)
    for (name, data) in cloud_weights {
        local_db.last_weights.insert(name, data);
    }
    
    // Merge bodyweight (use cloud if available)
    if let Some(bw) = cloud_bodyweight {
        local_db.bodyweight = Some(bw);
    }
    
    // Merge bodyweight history
    let local_bw_timestamps: std::collections::HashSet<_> = local_db.bodyweight_history.iter().map(|e| e.timestamp).collect();
    for entry in cloud_bw_history {
        if !local_bw_timestamps.contains(&entry.timestamp) {
            local_db.bodyweight_history.push(entry);
        }
    }
    // Sort by timestamp
    local_db.bodyweight_history.sort_by_key(|e| e.timestamp);
    
    // Save merged data locally
    let _ = crate::storage::save_data(&local_db);
    
    Ok(())
}
