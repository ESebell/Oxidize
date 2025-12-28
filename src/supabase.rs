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

/// Save session to Supabase with proper error handling
/// Uses UPSERT to be idempotent - safe to call multiple times
/// Save session to cloud with retry logic (3 attempts)
/// Returns immediately, runs async in background
/// Sets SYNC_FAILED flag if all retries fail
pub fn save_session_to_cloud(session: &Session) {
    // Don't try to save if not logged in - RLS will block it anyway
    if get_current_user_id().is_none() {
        web_sys::console::log_1(&"Skipping cloud save: not logged in".into());
        return;
    }
    
    let session = session.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let max_retries = 3;
        let mut last_error = String::new();
        
        for attempt in 1..=max_retries {
            web_sys::console::log_1(&format!("☁️ Saving session {} (attempt {}/{})", session.id, attempt, max_retries).into());
            
            match upsert_session(&session).await {
                Ok(_) => {
                    web_sys::console::log_1(&format!("✓ Session {} saved to cloud", session.id).into());
                    clear_sync_failed_flag();
                    return; // Success!
                }
                Err(e) => {
                    last_error = e;
                    web_sys::console::log_1(&format!("✗ Attempt {} failed: {}", attempt, last_error).into());
                    
                    if attempt < max_retries {
                        // Wait before retry: 1s, 2s
                        let delay_ms = attempt * 1000;
                        gloo_timers::future::TimeoutFuture::new(delay_ms as u32).await;
                    }
                }
            }
        }
        
        // All retries failed
        web_sys::console::log_1(&format!("❌ Session {} save FAILED after {} retries: {}", session.id, max_retries, last_error).into());
        set_sync_failed_flag(&session.id);
    });
}

// Sync failure tracking
const SYNC_FAILED_KEY: &str = "oxidize_sync_failed";

fn set_sync_failed_flag(session_id: &str) {
    if let Some(storage) = crate::storage::get_local_storage() {
        let _ = storage.set_item(SYNC_FAILED_KEY, session_id);
    }
}

fn clear_sync_failed_flag() {
    if let Some(storage) = crate::storage::get_local_storage() {
        let _ = storage.remove_item(SYNC_FAILED_KEY);
    }
}

pub fn clear_sync_failed() {
    clear_sync_failed_flag();
}

pub fn get_sync_failed_session() -> Option<String> {
    crate::storage::get_local_storage()
        .and_then(|s| s.get_item(SYNC_FAILED_KEY).ok())
        .flatten()
}

/// Upsert session to Supabase (insert or update if exists)
/// This is idempotent - calling multiple times is safe
async fn upsert_session(session: &Session) -> Result<(), String> {
    let window = web_sys::window().ok_or("no window")?;
    let user_id = get_current_user_id().ok_or("Not logged in")?;
    
    // Convert session to row format
    let exercises_json = serde_json::to_value(&session.exercises).map_err(|e| e.to_string())?;
    let row = SessionRow {
        id: session.id.clone(),
        routine: session.routine.clone(),
        timestamp: session.timestamp,
        duration_secs: session.duration_secs,
        total_volume: session.total_volume,
        exercises: exercises_json,
        user_id: Some(user_id),
    };
    
    let body = serde_json::to_string(&row).map_err(|e| e.to_string())?;
    
    // Use UPSERT via Prefer header - requires unique constraint on 'id' column
    let headers = get_headers().map_err(|e| format!("{:?}", e))?;
    headers.set("Prefer", "resolution=merge-duplicates").map_err(|e| format!("{:?}", e))?;
    
    let opts = create_request_init("POST", Some(&body), &headers);
    let url = format!("{}/rest/v1/sessions", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{:?}", e))?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.map_err(|e| format!("{:?}", e))?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "Invalid response")?;
    
    if !resp.ok() {
        let status = resp.status();
        let error_text = JsFuture::from(resp.text().map_err(|_| "No text")?)
            .await
            .map(|v| v.as_string().unwrap_or_default())
            .unwrap_or_default();
        return Err(format!("HTTP {}: {}", status, error_text));
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
            Ok(_) => {
                web_sys::console::log_1(&"Synced from Supabase".into());
                crate::storage::mark_sync_success();
            },
            Err(e) => {
                web_sys::console::log_1(&format!("Sync failed: {:?}", e).into());
                crate::storage::mark_sync_failed();
            },
        }
    });
}

/// Cloud-first sync: Cloud is source of truth, local is just cache
async fn do_sync() -> Result<(), JsValue> {
    web_sys::console::log_1(&"═══════════════════════════════════════".into());
    web_sys::console::log_1(&"SYNC START".into());
    
    // Only sync if logged in
    let user_id = match get_current_user_id() {
        Some(id) => id,
        None => {
            web_sys::console::log_1(&"SYNC ABORTED: not logged in".into());
            return Ok(());
        }
    };
    web_sys::console::log_1(&format!("User ID: {}", user_id).into());
    
    // Check what's in local storage BEFORE sync
    let local_before = crate::storage::load_data();
    web_sys::console::log_1(&format!("LOCAL BEFORE: {} sessions", local_before.sessions.len()).into());
    
    // Fetch from cloud
    web_sys::console::log_1(&"Fetching from Supabase...".into());
    let cloud_sessions = fetch_sessions().await.unwrap_or_default();
    let cloud_weights = fetch_last_weights().await.unwrap_or_default();
    let (cloud_bodyweight, cloud_bw_history) = fetch_bodyweight().await.unwrap_or((None, vec![]));
    
    web_sys::console::log_1(&format!("CLOUD: {} sessions", cloud_sessions.len()).into());
    
    // PHASE 1: PUSH - Upload local sessions missing from cloud
    let cloud_ids: std::collections::HashSet<String> = cloud_sessions.iter().map(|s| s.id.clone()).collect();
    let mut pushed_count = 0;
    
    for local_session in &local_before.sessions {
        if !cloud_ids.contains(&local_session.id) {
            web_sys::console::log_1(&format!("📤 Pushing local session: {} ({})", local_session.routine, local_session.id).into());
            match upsert_session(local_session).await {
                Ok(_) => {
                    web_sys::console::log_1(&format!("  ✓ Pushed successfully").into());
                    pushed_count += 1;
                }
                Err(e) => {
                    web_sys::console::log_1(&format!("  ✗ Push failed: {}", e).into());
                }
            }
        }
    }
    
    if pushed_count > 0 {
        web_sys::console::log_1(&format!("📤 Pushed {} local sessions to cloud", pushed_count).into());
    }
    
    // PHASE 2: PULL - Fetch cloud data again (in case we just pushed something)
    let cloud_sessions = if pushed_count > 0 {
        fetch_sessions().await.unwrap_or_default()
    } else {
        cloud_sessions
    };
    
    web_sys::console::log_1(&format!("CLOUD FINAL: {} sessions", cloud_sessions.len()).into());
    for s in &cloud_sessions {
        web_sys::console::log_1(&format!("  - {} ({})", s.routine, s.id).into());
    }
    
    // Create fresh database with cloud data
    let mut db = crate::storage::Database::default();
    db.sessions = cloud_sessions;
    db.last_weights = cloud_weights;
    db.bodyweight = cloud_bodyweight;
    db.bodyweight_history = cloud_bw_history;
    db.sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    // Save to localStorage
    web_sys::console::log_1(&"Saving to localStorage...".into());
    match crate::storage::save_data(&db) {
        Ok(_) => web_sys::console::log_1(&"Save OK".into()),
        Err(e) => web_sys::console::log_1(&format!("Save FAILED: {}", e).into()),
    }
    
    // Verify by re-loading
    let local_after = crate::storage::load_data();
    web_sys::console::log_1(&format!("LOCAL AFTER: {} sessions", local_after.sessions.len()).into());
    
    web_sys::console::log_1(&"SYNC COMPLETE".into());
    web_sys::console::log_1(&"═══════════════════════════════════════".into());
    Ok(())
}
