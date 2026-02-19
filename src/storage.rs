use serde::{Deserialize, Serialize};
use crate::types::*;

const PAUSED_WORKOUT_KEY: &str = "oxidize_paused_workout";
const SYNC_STATUS_KEY: &str = "oxidize_sync_status";
const DATA_VERSION_KEY: &str = "oxidize_data_version";
const ACTIVE_ROUTINE_KEY: &str = "oxidize_active_routine";
const DISPLAY_NAME_KEY: &str = "oxidize_display_name";

// Sync status: "pending", "success", "failed"
pub fn get_sync_status() -> &'static str {
    get_local_storage()
        .and_then(|s| s.get_item(SYNC_STATUS_KEY).ok())
        .flatten()
        .map(|v| match v.as_str() {
            "success" => "success",
            "failed" => "failed",
            _ => "pending"
        })
        .unwrap_or("pending")
}

pub fn is_sync_complete() -> bool {
    get_sync_status() == "success"
}

pub fn get_data_version() -> u32 {
    get_local_storage()
        .and_then(|s| s.get_item(DATA_VERSION_KEY).ok())
        .flatten()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

pub fn increment_data_version() {
    let new_version = get_data_version() + 1;
    if let Some(storage) = get_local_storage() {
        let _ = storage.set_item(DATA_VERSION_KEY, &new_version.to_string());
    }
}

pub fn mark_sync_success() {
    if let Some(storage) = get_local_storage() {
        let _ = storage.set_item(SYNC_STATUS_KEY, "success");
    }
    increment_data_version(); // Trigger UI refresh
}

pub fn load_display_name() -> Option<String> {
    get_local_storage()
        .and_then(|s| s.get_item(DISPLAY_NAME_KEY).ok())
        .flatten()
}

pub fn save_display_name(name: &str) {
    if let Some(storage) = get_local_storage() {
        if name.is_empty() {
            let _ = storage.remove_item(DISPLAY_NAME_KEY);
        } else {
            let _ = storage.set_item(DISPLAY_NAME_KEY, name);
        }
    }
}

pub fn mark_sync_failed() {
    if let Some(storage) = get_local_storage() {
        let _ = storage.set_item(SYNC_STATUS_KEY, "failed");
    }
}

pub fn reset_sync_status() {
    if let Some(storage) = get_local_storage() {
        let _ = storage.set_item(SYNC_STATUS_KEY, "pending");
    }
}

// Active routine storage
pub fn save_active_routine(routine: &SavedRoutine) {
    if let Some(storage) = get_local_storage() {
        if let Ok(json) = serde_json::to_string(routine) {
            let _ = storage.set_item(ACTIVE_ROUTINE_KEY, &json);
        }
    }
}

pub fn load_active_routine() -> Option<SavedRoutine> {
    let storage = get_local_storage()?;
    let json = storage.get_item(ACTIVE_ROUTINE_KEY).ok()??;
    let mut routine: SavedRoutine = serde_json::from_str(&json).ok()?;
    if migrate_routine_names(&mut routine) {
        save_active_routine(&routine);
    }
    Some(routine)
}

pub fn clear_active_routine() {
    if let Some(storage) = get_local_storage() {
        let _ = storage.remove_item(ACTIVE_ROUTINE_KEY);
    }
}

// LocalStorage fallback for simpler key-value storage
pub fn get_local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

// Save workout data to localStorage (simpler than IndexedDB for this use case)
pub fn save_data(data: &Database) -> Result<(), String> {
    let storage = get_local_storage().ok_or("No localStorage")?;
    let json = serde_json::to_string(data).map_err(|e| e.to_string())?;
    storage.set_item("oxidize_db_v2", &json).map_err(|_| "Failed to save")?;
    Ok(())
}

pub fn load_data() -> Database {
    let storage = match get_local_storage() {
        Some(s) => s,
        None => return Database::default(),
    };

    let json = match storage.get_item("oxidize_db_v2") {
        Ok(Some(j)) => j,
        _ => return Database::default(),
    };

    let mut db: Database = serde_json::from_str(&json).unwrap_or_default();
    migrate_exercise_names(&mut db);
    db
}

/// One-time migration: rename Swedish exercise names to match Wger
pub fn migrate_exercise_names(db: &mut Database) {
    let renames = EXERCISE_RENAMES;
    let mut changed = false;

    for session in &mut db.sessions {
        for ex in &mut session.exercises {
            for &(old, new) in renames {
                if ex.name == old {
                    ex.name = new.to_string();
                    changed = true;
                }
            }
        }
    }

    for &(old, new) in renames {
        if let Some(data) = db.last_weights.remove(old) {
            db.last_weights.insert(new.to_string(), data);
            changed = true;
        }
    }

    if changed {
        let _ = save_data(db);
    }
}

pub const EXERCISE_RENAMES: &[(&str, &str)] = &[
    ("Knäböj", "Squats"),
    ("Bänkpress", "Bench Press"),
    ("Marklyft", "Deadlift"),
    ("Militärpress", "Shoulder Press"),
];

/// Migrate exercise names in a routine's passes. Returns true if changed.
pub fn migrate_routine_names(routine: &mut SavedRoutine) -> bool {
    let mut changed = false;
    for pass in &mut routine.passes {
        for ex in pass.exercises.iter_mut().chain(pass.finishers.iter_mut()) {
            for &(old, new) in EXERCISE_RENAMES {
                if ex.name == old {
                    ex.name = new.to_string();
                    changed = true;
                }
            }
        }
    }
    changed
}

// Paused workout functions
pub fn save_paused_workout(paused: &PausedWorkout) -> Result<(), String> {
    let storage = get_local_storage().ok_or("No localStorage")?;
    let json = serde_json::to_string(paused).map_err(|e| e.to_string())?;
    storage.set_item(PAUSED_WORKOUT_KEY, &json).map_err(|_| "Failed to save paused workout")?;
    Ok(())
}

pub fn load_paused_workout() -> Option<PausedWorkout> {
    let storage = get_local_storage()?;
    let json = storage.get_item(PAUSED_WORKOUT_KEY).ok()??;
    serde_json::from_str(&json).ok()
}

pub fn clear_paused_workout() {
    if let Some(storage) = get_local_storage() {
        let _ = storage.remove_item(PAUSED_WORKOUT_KEY);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BodyweightEntry {
    pub timestamp: i64,
    pub weight: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Database {
    pub sessions: Vec<Session>,
    pub last_weights: std::collections::HashMap<String, LastExerciseData>,
    #[serde(default)]
    pub bodyweight: Option<f64>,
    #[serde(default)]
    pub bodyweight_history: Vec<BodyweightEntry>,
}

impl Database {
    pub fn set_bodyweight(&mut self, weight: f64) {
        self.bodyweight = Some(weight);
        self.bodyweight_history.push(BodyweightEntry {
            timestamp: chrono::Utc::now().timestamp(),
            weight,
        });
    }
    
    pub fn get_bodyweight(&self) -> Option<f64> {
        self.bodyweight
    }
}

impl Database {
    pub fn add_session(&mut self, session: Session) {
        // Update last weights for each exercise
        for ex in &session.exercises {
            if let Some(last_set) = ex.sets.last() {
                self.last_weights.insert(
                    ex.name.clone(),
                    LastExerciseData {
                        weight: last_set.weight,
                        reps: last_set.reps,
                    },
                );
            }
        }
        self.sessions.push(session);
    }

    pub fn get_last_exercise_data(&self, exercise: &str) -> Option<LastExerciseData> {
        self.last_weights.get(exercise).cloned()
    }

    pub fn get_recent_sessions(&self, limit: usize) -> Vec<Session> {
        let mut sessions = self.sessions.clone();
        sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sessions.into_iter().take(limit).collect()
    }

    pub fn get_total_stats(&self) -> TotalStats {
        let total_sessions = self.sessions.len();
        let total_volume: f64 = self.sessions.iter().map(|s| s.total_volume).sum();
        let total_sets: usize = self.sessions.iter()
            .flat_map(|s| &s.exercises)
            .map(|e| e.sets.len())
            .sum();
        
        let avg_duration = if total_sessions > 0 {
            self.sessions.iter().map(|s| s.duration_secs).sum::<i64>() / total_sessions as i64
        } else {
            0
        };

        TotalStats {
            total_sessions,
            total_volume,
            total_sets,
            avg_duration_secs: avg_duration,
        }
    }
}

/// Single source of truth for the default routine.
/// Used as fallback when no Supabase routine is cached.
pub fn create_default_routine() -> SavedRoutine {
    let pass_a = Pass {
        name: "Pass A".to_string(),
        description: "Ben · Press · Triceps".to_string(),
        exercises: vec![
            Exercise::standard("Squats", 3, "5-8"),
            Exercise::standard("Bench Press", 3, "5-8"),
            Exercise::standard("Hip Thrusts", 3, "8-12"),
            Exercise::standard("Latsdrag", 3, "8-10"),
            Exercise::superset("Leg Curls", 2, "12-15", "Dips", Some("Ben/Triceps")),
            Exercise::superset("Dips", 2, "AMRAP", "Leg Curls", Some("Ben/Triceps")),
            Exercise::standard("Stående vadpress", 3, "12-15"),
        ],
        finishers: vec![
            Exercise::finisher("Shoulder Taps", 3, "20"),
            Exercise::timed_finisher("Mountain Climbers", 3, 30),
        ],
    };

    let pass_b = Pass {
        name: "Pass B".to_string(),
        description: "Rygg · Axlar · Biceps".to_string(),
        exercises: vec![
            Exercise::standard("Deadlift", 3, "5"),
            Exercise::standard("Shoulder Press", 3, "8-10"),
            Exercise::standard("Sittande rodd", 3, "10-12"),
            Exercise::superset("Sidolyft", 3, "12-15", "Hammercurls", Some("Axlar/Armar")),
            Exercise::superset("Hammercurls", 3, "10-12", "Sidolyft", Some("Axlar/Armar")),
            Exercise::superset("Facepulls", 3, "15", "Sittande vadpress", Some("Rygg/Vader")),
            Exercise::superset("Sittande vadpress", 3, "15-20", "Facepulls", Some("Rygg/Vader")),
        ],
        finishers: vec![
            Exercise::finisher("Dead Bug", 3, "12"),
            Exercise::finisher("Utfallssteg", 3, "20"),
        ],
    };

    let now = js_sys::Date::now() as i64 / 1000;
    let id = format!("default_{}", now);

    SavedRoutine {
        id,
        user_id: None,
        name: "Överkropp/Underkropp".to_string(),
        focus: "Styrka & Hypertrofi".to_string(),
        passes: vec![pass_a, pass_b],
        is_active: true,
        created_at: now,
    }
}

pub fn get_workout(pass_name: &str) -> Option<WorkoutData> {
    let db = load_data();

    // Use active routine from Supabase, fallback to defaults
    let saved_routine = load_active_routine().unwrap_or_else(create_default_routine);

    let pass = saved_routine.passes.iter().find(|p| p.name == pass_name)?;
    let routine = Routine {
        name: pass.name.clone(),
        focus: saved_routine.focus.clone(),
        exercises: pass.exercises.clone(),
        finishers: pass.finishers.clone(),
    };

    Some(create_workout_data(routine, &db))
}

fn create_workout_data(routine: Routine, db: &Database) -> WorkoutData {
    // Combine main exercises and finishers
    let all_exercises: Vec<&Exercise> = routine
        .exercises
        .iter()
        .chain(routine.finishers.iter())
        .collect();

    let exercises: Vec<ExerciseWorkoutState> = all_exercises
        .iter()
        .map(|ex| {
            let last_data = db.get_last_exercise_data(&ex.name);
            // Bodyweight exercises default to 0 weight
            let current_weight = if ex.is_bodyweight {
                0.0
            } else {
                last_data.as_ref().map(|d| d.weight).unwrap_or(20.0)
            };
            ExerciseWorkoutState {
                exercise: (*ex).clone(),
                last_data,
                current_weight,
                sets_completed: Vec::new(),
            }
        })
        .collect();

    WorkoutData { routine, exercises }
}

pub fn save_session(routine_name: String, exercises: Vec<ExerciseRecord>, duration_secs: i64) {
    let mut db = load_data();

    let total_volume: f64 = exercises
        .iter()
        .flat_map(|e| &e.sets)
        .map(|s| s.weight * s.reps as f64)
        .sum();

    let session = Session {
        id: uuid_simple(),
        routine: routine_name,
        timestamp: chrono::Utc::now().timestamp(),
        duration_secs,
        exercises,
        total_volume,
    };

    // Save last weights to cloud
    for ex in &session.exercises {
        if let Some(last_set) = ex.sets.last() {
            crate::supabase::save_weight_to_cloud(&ex.name, last_set.weight, last_set.reps);
        }
    }
    
    // Update activity timestamp
    crate::supabase::update_last_activity();
    
    // Save session to cloud
    crate::supabase::save_session_to_cloud(&session);

    // Save locally (instant, works offline)
    db.add_session(session);
    let _ = save_data(&db);
}

fn uuid_simple() -> String {
    let now = js_sys::Date::now() as u64;
    let random = (js_sys::Math::random() * 1_000_000.0) as u64;
    format!("{:x}{:x}", now, random)
}

