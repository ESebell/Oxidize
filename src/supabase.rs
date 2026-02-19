use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, Headers};
use serde::{Deserialize, Serialize};

const SUPABASE_URL: &str = "https://ytnwppbepeojvyedrbnb.supabase.co";
const SUPABASE_KEY: &str = "sb_publishable_Oqp9Oc-Io5o3o3MUwIVD2A_Tvv_dCuS";
const AUTH_SESSION_KEY: &str = "oxidize_auth_session";
const LAST_ACTIVITY_KEY: &str = "oxidize_last_activity";
const INACTIVITY_TIMEOUT_SECS: i64 = 4 * 60 * 60; // 4 hours

use crate::types::{Session, AuthSession, AuthUser, SavedRoutine, Pass};

// ============ AUTH ============

#[derive(Deserialize, Debug)]
struct SupabaseAuthResponse {
    access_token: String,
    refresh_token: Option<String>,
    user: SupabaseUser,
}

#[derive(Deserialize, Debug)]
struct RefreshTokenResponse {
    access_token: String,
    refresh_token: String,
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
        refresh_token: auth_resp.refresh_token,
        user: AuthUser {
            id: auth_resp.user.id,
            email: auth_resp.user.email,
            display_name: crate::storage::load_display_name(),
        },
    };
    
    save_auth_session(&session);
    update_last_activity();
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
        refresh_token: auth_resp.refresh_token,
        user: AuthUser {
            id: auth_resp.user.id,
            email: auth_resp.user.email,
            display_name: crate::storage::load_display_name(),
        },
    };
    
    save_auth_session(&session);
    update_last_activity();
    Ok(session)
}

/// Sign out - clears local session and calls Supabase sign-out API
pub fn sign_out() {
    // Call Supabase sign-out API to invalidate refresh token on server
    wasm_bindgen_futures::spawn_local(async {
        let _ = sign_out_api().await;
    });
    
    // Clear local storage
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok()).flatten() {
        let _ = storage.remove_item(AUTH_SESSION_KEY);
        let _ = storage.remove_item(LAST_ACTIVITY_KEY);
    }
}

/// Call Supabase sign-out API
async fn sign_out_api() -> Result<(), String> {
    let window = web_sys::window().ok_or("no window")?;
    let session = load_auth_session().ok_or("No session")?;
    
    let headers = Headers::new().map_err(|_| "Failed to create headers")?;
    headers.set("apikey", SUPABASE_KEY).map_err(|_| "Failed to set apikey")?;
    headers.set("Authorization", &format!("Bearer {}", session.access_token)).map_err(|_| "Failed to set auth")?;
    
    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);
    opts.set_headers(&JsValue::from(&headers));
    
    let url = format!("{}/auth/v1/logout", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts).map_err(|_| "Failed to create request")?;
    
    let _ = JsFuture::from(window.fetch_with_request(&request)).await;
    Ok(())
}

/// Update last activity timestamp
pub fn update_last_activity() {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok()).flatten() {
        let now = js_sys::Date::now() as i64 / 1000;
        let _ = storage.set_item(LAST_ACTIVITY_KEY, &now.to_string());
    }
}

/// Check if session has expired due to inactivity (>4 hours)
fn is_session_expired() -> bool {
    if load_auth_session().is_none() {
        return false;
    }
    
    let storage = match web_sys::window().and_then(|w| w.local_storage().ok()).flatten() {
        Some(s) => s,
        None => return false,
    };
    
    let last_activity = storage
        .get_item(LAST_ACTIVITY_KEY)
        .ok()
        .flatten()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    
    if last_activity == 0 {
        return false;
    }
    
    let now = js_sys::Date::now() as i64 / 1000;
    (now - last_activity) > INACTIVITY_TIMEOUT_SECS
}

/// Refresh access token using refresh token
pub async fn refresh_access_token() -> Result<(), String> {
    let session = load_auth_session().ok_or("No session")?;
    let refresh_token = session.refresh_token.ok_or("No refresh token")?;
    
    let window = web_sys::window().ok_or("no window")?;
    
    let body = serde_json::json!({
        "refresh_token": refresh_token
    }).to_string();
    
    let headers = Headers::new().map_err(|_| "Failed to create headers")?;
    headers.set("apikey", SUPABASE_KEY).map_err(|_| "Failed to set apikey")?;
    headers.set("Content-Type", "application/json").map_err(|_| "Failed to set content-type")?;
    
    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);
    opts.set_body(&JsValue::from_str(&body));
    opts.set_headers(&JsValue::from(&headers));
    
    let url = format!("{}/auth/v1/token?grant_type=refresh_token", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts).map_err(|_| "Failed to create request")?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.map_err(|_| "Fetch failed")?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "Invalid response")?;
    
    if !resp.ok() {
        return Err("Token refresh failed".into());
    }
    
    let json = JsFuture::from(resp.json().map_err(|_| "No JSON")?).await.map_err(|_| "JSON parse failed")?;
    let auth_resp: RefreshTokenResponse = serde_wasm_bindgen::from_value(json).map_err(|_| "Invalid response")?;
    
    let new_session = AuthSession {
        access_token: auth_resp.access_token,
        refresh_token: Some(auth_resp.refresh_token),
        user: AuthUser {
            id: auth_resp.user.id,
            email: auth_resp.user.email,
            display_name: crate::storage::load_display_name(),
        },
    };
    
    save_auth_session(&new_session);
    web_sys::console::log_1(&"Access token refreshed".into());
    Ok(())
}

/// Check session timeout and refresh token if needed
/// Call this on app start
pub fn check_and_refresh_session() {
    // If session expired due to inactivity, sign out
    if is_session_expired() {
        web_sys::console::log_1(&"Session expired due to inactivity (4h), signing out".into());
        sign_out();
        return;
    }
    
    // If logged in, try to refresh token and update activity
    if load_auth_session().is_some() {
        update_last_activity();
        
        // Refresh token in background
        wasm_bindgen_futures::spawn_local(async {
            if let Err(e) = refresh_access_token().await {
                web_sys::console::log_1(&format!("Token refresh failed: {}", e).into());
            }
        });
    }
}

/// Save auth session to localStorage
pub fn save_auth_session(session: &AuthSession) {
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

/// Fetch with timeout - returns Err if request takes longer than timeout_ms
async fn fetch_with_timeout(request: Request, timeout_ms: u32) -> Result<Response, String> {
    use futures::future::{select, Either};
    use std::pin::pin;
    
    let window = web_sys::window().ok_or("no window")?;
    let fetch_future = JsFuture::from(window.fetch_with_request(&request));
    let timeout_future = gloo_timers::future::TimeoutFuture::new(timeout_ms);
    
    let fetch_pinned = pin!(fetch_future);
    let timeout_pinned = pin!(timeout_future);
    
    match select(fetch_pinned, timeout_pinned).await {
        Either::Left((result, _)) => {
            let resp_value = result.map_err(|e| format!("{:?}", e))?;
            resp_value.dyn_into().map_err(|_| "Invalid response".to_string())
        }
        Either::Right((_, _)) => {
            Err(format!("Request timed out after {}ms", timeout_ms))
        }
    }
}

/// Upsert session to Supabase (insert or update if exists)
/// This is idempotent - calling multiple times is safe
/// Timeout: 5 seconds per request
async fn upsert_session(session: &Session) -> Result<(), String> {
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
    
    // 5 second timeout per request
    let resp = fetch_with_timeout(request, 5000).await?;
    
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
    
    // Only fetch sessions for current user
    let user_id = get_current_user_id().ok_or("Not logged in")?;
    
    let headers = get_headers()?;
    let opts = create_request_init("GET", None, &headers);
    
    let url = format!("{}/rest/v1/sessions?select=*&user_id=eq.{}", SUPABASE_URL, user_id);
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
    
    // Only fetch weights for current user
    let user_id = get_current_user_id().ok_or("Not logged in")?;
    
    let headers = get_headers()?;
    let opts = create_request_init("GET", None, &headers);
    
    let url = format!("{}/rest/v1/last_weights?select=*&user_id=eq.{}", SUPABASE_URL, user_id);
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
    update_last_activity();
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = save_bodyweight_async(weight).await {
            web_sys::console::log_1(&format!("Supabase bodyweight save failed: {:?}", e).into());
        }
    });
}

async fn save_bodyweight_async(weight: f64) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let user_id = get_current_user_id().ok_or("Not logged in")?;
    
    let timestamp = js_sys::Date::now() as i64 / 1000;
    
    // 1. Save to history table (bodyweight) - this is for the curve
    let history_body = format!(r#"{{"weight": {}, "timestamp": {}, "user_id": "{}"}}"#, weight, timestamp, user_id);
    let headers = get_headers()?;
    let opts = create_request_init("POST", Some(&history_body), &headers);
    let url = format!("{}/rest/v1/bodyweight", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    let _ = JsFuture::from(window.fetch_with_request(&request)).await?;

    // 2. Save to settings table (user_settings) for CURRENT weight
    // Create a row with ONLY user_id and bodyweight to avoid clobbering display_name
    let settings_row = UserSettingsRow {
        user_id: Some(user_id.clone()),
        display_name: None, // Will be skipped during serialization
        bodyweight: Some(weight),
    };
    let settings_body = serde_json::to_string(&settings_row).map_err(|e| e.to_string())?;
    let settings_headers = get_headers()?;
    settings_headers.set("Prefer", "resolution=merge-duplicates")?;
    let settings_opts = create_request_init("POST", Some(&settings_body), &settings_headers);
    let settings_url = format!("{}/rest/v1/user_settings", SUPABASE_URL);
    let settings_request = Request::new_with_str_and_init(&settings_url, &settings_opts)?;
    let _ = JsFuture::from(window.fetch_with_request(&settings_request)).await?;
    
    Ok(())
}

/// Fetch bodyweight history and current settings from Supabase
pub async fn fetch_bodyweight() -> Result<(Option<f64>, Vec<crate::storage::BodyweightEntry>), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let user_id = get_current_user_id().ok_or("Not logged in")?;
    let headers = get_headers()?;

    // 1. Fetch current weight from user_settings
    let settings_opts = create_request_init("GET", None, &headers);
    let settings_url = format!("{}/rest/v1/user_settings?user_id=eq.{}&select=user_id,bodyweight", SUPABASE_URL, user_id);
    let settings_request = Request::new_with_str_and_init(&settings_url, &settings_opts)?;
    let settings_resp: Response = JsFuture::from(window.fetch_with_request(&settings_request)).await?.dyn_into()?;
    
    let mut current_weight = None;
    if settings_resp.ok() {
        let json = JsFuture::from(settings_resp.json()?).await?;
        let rows: Vec<UserSettingsRow> = serde_wasm_bindgen::from_value(json).unwrap_or_default();
        current_weight = rows.first().and_then(|r| r.bodyweight);
    }

    // 2. Fetch history from bodyweight table
    let history_opts = create_request_init("GET", None, &headers);
    let history_url = format!("{}/rest/v1/bodyweight?select=*&user_id=eq.{}&order=timestamp.desc", SUPABASE_URL, user_id);
    let history_request = Request::new_with_str_and_init(&history_url, &history_opts)?;
    let history_resp: Response = JsFuture::from(window.fetch_with_request(&history_request)).await?.dyn_into()?;
    
    let mut history = vec![];
    if history_resp.ok() {
        let json = JsFuture::from(history_resp.json()?).await?;
        let rows: Vec<BodyweightRow> = serde_wasm_bindgen::from_value(json).unwrap_or_default();
        history = rows.into_iter()
            .map(|r| crate::storage::BodyweightEntry {
                timestamp: r.timestamp,
                weight: r.weight,
            })
            .collect();
    }
    
    // If no weight in settings but exists in history, use latest history
    if current_weight.is_none() {
        current_weight = history.first().map(|h| h.weight);
    }
    
    Ok((current_weight, history))
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
    let cloud_display_name = fetch_display_name().await.unwrap_or(None);
    
    // Save display name to local storage if fetched from cloud
    if let Some(name) = &cloud_display_name {
        crate::storage::save_display_name(name);
        web_sys::console::log_1(&format!("Synced display_name from cloud: {}", name).into());
        
        // Update auth session in localStorage and signal if possible
        if let Some(mut session) = load_auth_session() {
            session.user.display_name = Some(name.clone());
            save_auth_session(&session);
        }
    }
    
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

// ============ ROUTINES ============

#[derive(Serialize, Deserialize, Debug)]
struct RoutineRow {
    id: String,
    user_id: Option<String>,
    name: String,
    focus: String,
    passes: serde_json::Value,
    is_active: bool,
    created_at: i64,
}

/// Fetch all routines for the current user
pub async fn fetch_routines() -> Result<Vec<SavedRoutine>, JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let user_id = match get_current_user_id() {
        Some(id) => id,
        None => return Ok(vec![]),
    };
    
    let headers = get_headers()?;
    
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);
    opts.set_headers(&JsValue::from(&headers));
    
    let url = format!("{}/rest/v1/routines?user_id=eq.{}&order=created_at.desc", SUPABASE_URL, user_id);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        return Ok(vec![]);
    }
    
    let json = JsFuture::from(resp.json()?).await?;
    let rows: Vec<RoutineRow> = serde_wasm_bindgen::from_value(json).unwrap_or_default();
    
    let routines: Vec<SavedRoutine> = rows.into_iter().map(|r| {
        let passes: Vec<Pass> = serde_json::from_value(r.passes).unwrap_or_default();
        SavedRoutine {
            id: r.id,
            user_id: r.user_id,
            name: r.name,
            focus: r.focus,
            passes,
            is_active: r.is_active,
            created_at: r.created_at,
        }
    }).collect();
    
    Ok(routines)
}

/// Get the active routine for the current user
pub async fn get_active_routine() -> Result<Option<SavedRoutine>, JsValue> {
    let routines = fetch_routines().await?;
    Ok(routines.into_iter().find(|r| r.is_active))
}

/// Save a routine to Supabase
pub async fn save_routine(routine: &SavedRoutine) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let user_id = match get_current_user_id() {
        Some(id) => id,
        None => {
            web_sys::console::log_1(&"Cannot save routine: not logged in".into());
            return Err("Not logged in".into());
        }
    };
    
    let passes_json = serde_json::to_value(&routine.passes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    let body = serde_json::json!({
        "id": routine.id,
        "user_id": user_id,
        "name": routine.name,
        "focus": routine.focus,
        "passes": passes_json,
        "is_active": routine.is_active,
        "created_at": routine.created_at
    }).to_string();
    
    let headers = get_headers()?;
    headers.set("Content-Type", "application/json")?;
    headers.set("Prefer", "resolution=merge-duplicates")?;
    
    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);
    opts.set_body(&JsValue::from_str(&body));
    opts.set_headers(&JsValue::from(&headers));
    
    let url = format!("{}/rest/v1/routines", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        let text = JsFuture::from(resp.text()?).await?;
        web_sys::console::log_1(&format!("Save routine failed: {:?}", text).into());
        return Err(text);
    }
    
    web_sys::console::log_1(&format!("Routine '{}' saved", routine.name).into());
    Ok(())
}

/// Set a routine as active (and deactivate others)
pub async fn set_active_routine(routine_id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let user_id = match get_current_user_id() {
        Some(id) => id,
        None => return Err("Not logged in".into()),
    };
    
    // First, deactivate all routines
    let headers = get_headers()?;
    headers.set("Content-Type", "application/json")?;
    
    let deactivate_body = serde_json::json!({ "is_active": false }).to_string();
    let opts = RequestInit::new();
    opts.set_method("PATCH");
    opts.set_mode(RequestMode::Cors);
    opts.set_body(&JsValue::from_str(&deactivate_body));
    opts.set_headers(&JsValue::from(&headers));
    
    let url = format!("{}/rest/v1/routines?user_id=eq.{}", SUPABASE_URL, user_id);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    let _ = JsFuture::from(window.fetch_with_request(&request)).await?;
    
    // Then activate the selected routine
    let activate_body = serde_json::json!({ "is_active": true }).to_string();
    let headers2 = get_headers()?;
    headers2.set("Content-Type", "application/json")?;
    
    let opts2 = RequestInit::new();
    opts2.set_method("PATCH");
    opts2.set_mode(RequestMode::Cors);
    opts2.set_body(&JsValue::from_str(&activate_body));
    opts2.set_headers(&JsValue::from(&headers2));
    
    let url2 = format!("{}/rest/v1/routines?id=eq.{}&user_id=eq.{}", SUPABASE_URL, routine_id, user_id);
    let request2 = Request::new_with_str_and_init(&url2, &opts2)?;
    let _ = JsFuture::from(window.fetch_with_request(&request2)).await?;
    
    web_sys::console::log_1(&format!("Routine {} activated", routine_id).into());
    Ok(())
}

/// Delete a routine
pub async fn delete_routine(routine_id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let user_id = match get_current_user_id() {
        Some(id) => id,
        None => return Err("Not logged in".into()),
    };
    
    let headers = get_headers()?;
    
    let opts = RequestInit::new();
    opts.set_method("DELETE");
    opts.set_mode(RequestMode::Cors);
    opts.set_headers(&JsValue::from(&headers));
    
    let url = format!("{}/rest/v1/routines?id=eq.{}&user_id=eq.{}", SUPABASE_URL, routine_id, user_id);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        return Err("Delete failed".into());
    }
    
    web_sys::console::log_1(&format!("Routine {} deleted", routine_id).into());
    Ok(())
}

// ============ USER SETTINGS (Display Name) ============

#[derive(Serialize, Deserialize, Debug)]
struct UserSettingsRow {
    #[serde(default)]
    user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bodyweight: Option<f64>,
}

/// Save display name to Supabase (partial update)
pub fn save_display_name_to_cloud(name: &str) {
    let name = name.to_string();
    update_last_activity();
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = save_display_name_async(&name).await {
            web_sys::console::log_1(&format!("Supabase display_name save failed: {:?}", e).into());
        }
    });
}

async fn save_display_name_async(name: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let user_id = get_current_user_id().ok_or("Not logged in")?;
    
    // Create a row with ONLY user_id and display_name to avoid clobbering bodyweight
    let row = UserSettingsRow {
        user_id: Some(user_id.clone()),
        display_name: Some(if name.is_empty() { " ".to_string() } else { name.to_string() }),
        bodyweight: None, // Will be skipped during serialization
    };
    
    let body = serde_json::to_string(&row).map_err(|e| e.to_string())?;
    let headers = get_headers().map_err(|_| "Failed to get headers")?;
    headers.set("Prefer", "resolution=merge-duplicates").map_err(|_| "Failed to set Prefer header")?;
    
    let opts = create_request_init("POST", Some(&body), &headers);
    let url = format!("{}/rest/v1/user_settings", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if resp.ok() {
        Ok(())
    } else {
        let text = JsFuture::from(resp.text()?).await?.as_string().unwrap_or_default();
        Err(format!("HTTP {}: {}", resp.status(), text).into())
    }
}

/// Fetch display name from Supabase
pub async fn fetch_display_name() -> Result<Option<String>, JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let user_id = get_current_user_id().ok_or("Not logged in")?;
    
    let headers = get_headers()?;
    let opts = create_request_init("GET", None, &headers);
    
    let url = format!("{}/rest/v1/user_settings?user_id=eq.{}&select=user_id,display_name", SUPABASE_URL, user_id);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        return Ok(None);
    }
    
    let json = JsFuture::from(resp.json()?).await?;
    web_sys::console::log_1(&format!("DEBUG: fetch_display_name JSON: {:?}", json).into());
    let rows: Vec<UserSettingsRow> = serde_wasm_bindgen::from_value(json).unwrap_or_default();
    web_sys::console::log_1(&format!("DEBUG: fetch_display_name rows: {:?}", rows).into());
    
    Ok(rows.first().and_then(|r| r.display_name.clone()))
}

// ============ AI AGENT ============

#[derive(Deserialize)]
struct ConfigRow {
    config_value: String,
}

pub async fn fetch_api_key() -> Result<Option<String>, JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let headers = get_headers()?;
    let opts = create_request_init("GET", None, &headers);
    
    let url = format!("{}/rest/v1/app_config?config_key=eq.gemini_api_key&select=config_value", SUPABASE_URL);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        return Ok(None);
    }
    
    let json = JsFuture::from(resp.json()?).await?;
    let rows: Vec<ConfigRow> = serde_wasm_bindgen::from_value(json).unwrap_or_default();
    
    Ok(rows.first().map(|r| r.config_value.clone()))
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResp,
}

#[derive(Deserialize)]
struct GeminiContentResp {
    parts: Vec<GeminiPartResp>,
}

#[derive(Deserialize)]
struct GeminiPartResp {
    text: String,
}

pub async fn call_gemini(api_key: &str, system_prompt: &str, user_prompt: &str) -> Result<String, JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    
    // First, let's list available models to see what we have access to
    let list_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={}",
        api_key
    );
    web_sys::console::log_1(&format!("DEBUG: Listing available models...").into());
    
    let list_headers = Headers::new()?;
    let list_opts = RequestInit::new();
    list_opts.set_method("GET");
    list_opts.set_headers(&list_headers);
    
    let list_request = Request::new_with_str_and_init(&list_url, &list_opts)?;
    let list_resp_value = JsFuture::from(window.fetch_with_request(&list_request)).await?;
    let list_resp: Response = list_resp_value.dyn_into()?;
    
    if list_resp.ok() {
        let list_text = JsFuture::from(list_resp.text()?).await?.as_string().unwrap_or_default();
        web_sys::console::log_1(&format!("DEBUG: Available models: {}", &list_text[..500.min(list_text.len())]).into());
    } else {
        web_sys::console::log_1(&format!("DEBUG: Failed to list models: {}", list_resp.status()).into());
    }
    
    let full_prompt = format!("{}\n\nUser request: {}", system_prompt, user_prompt);
    
    let req_body = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart { text: full_prompt }],
        }],
    };
    
    let body_str = serde_json::to_string(&req_body).map_err(|e| e.to_string())?;
    
    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    
    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_headers(&headers);
    opts.set_body(&JsValue::from_str(&body_str));
    
    // Use gemini-2.5-flash which is available for this API key
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );
    
    web_sys::console::log_1(&format!("DEBUG: Calling: {}", url).into());
    
    let request = Request::new_with_str_and_init(&url, &opts)?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        let text = JsFuture::from(resp.text()?).await?.as_string().unwrap_or_default();
        return Err(format!("Gemini API error (v5-pro, {}): {}", resp.status(), text).into());
    }
    
    let json = JsFuture::from(resp.json()?).await?;
    let gemini_resp: GeminiResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Failed to parse Gemini response: {:?}", e))?;
    
    let response_text = gemini_resp.candidates.first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.clone())
        .ok_or("Empty response from Gemini")?;
        
    Ok(response_text)
}
