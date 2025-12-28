use serde::{Deserialize, Serialize};
use crate::types::*;

const PAUSED_WORKOUT_KEY: &str = "oxidize_paused_workout";
const SYNC_STATUS_KEY: &str = "oxidize_sync_status";

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

pub fn mark_sync_success() {
    if let Some(storage) = get_local_storage() {
        let _ = storage.set_item(SYNC_STATUS_KEY, "success");
    }
}

pub fn mark_sync_failed() {
    if let Some(storage) = get_local_storage() {
        let _ = storage.set_item(SYNC_STATUS_KEY, "failed");
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
    
    serde_json::from_str(&json).unwrap_or_default()
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

    pub fn get_exercise_stats(&self, exercise_name: &str) -> ExerciseStats {
        let mut stats = ExerciseStats {
            name: exercise_name.to_string(),
            current_weight: 0.0,
            estimated_1rm: 0.0,
            total_volume: 0.0,
            best_set: None,
            sessions_count: 0,
            avg_rest_time: 0.0,
            volume_trend: Vec::new(),
            one_rm_trend: Vec::new(),
        };

        let mut rest_times: Vec<i64> = Vec::new();

        for session in &self.sessions {
            for ex in &session.exercises {
                if ex.name == exercise_name {
                    stats.sessions_count += 1;
                    let mut session_volume = 0.0;
                    let mut session_best_1rm = 0.0;

                    for set in &ex.sets {
                        let volume = set.weight * set.reps as f64;
                        stats.total_volume += volume;
                        session_volume += volume;

                        // 1RM (Epley formula)
                        let one_rm = if set.reps == 1 {
                            set.weight
                        } else {
                            set.weight * (1.0 + set.reps as f64 / 30.0)
                        };

                        if one_rm > stats.estimated_1rm {
                            stats.estimated_1rm = one_rm;
                            stats.best_set = Some(set.clone());
                        }
                        if one_rm > session_best_1rm {
                            session_best_1rm = one_rm;
                        }

                        if let Some(rest) = set.rest_before_secs {
                            if rest > 0 {
                                rest_times.push(rest);
                            }
                        }
                    }

                    stats.volume_trend.push((session.timestamp, session_volume));
                    if session_best_1rm > 0.0 {
                        stats.one_rm_trend.push((session.timestamp, session_best_1rm));
                    }
                }
            }
        }

        if let Some(last) = self.last_weights.get(exercise_name) {
            stats.current_weight = last.weight;
        }

        if !rest_times.is_empty() {
            stats.avg_rest_time = rest_times.iter().sum::<i64>() as f64 / rest_times.len() as f64;
        }

        stats
    }

    pub fn get_all_exercise_stats(&self) -> Vec<ExerciseStats> {
        let names = get_all_exercise_names();
        names.iter()
            .map(|n| self.get_exercise_stats(n))
            .filter(|s| s.sessions_count > 0)
            .collect()
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

// Routines defined here to avoid circular deps
pub fn get_routine_a() -> Routine {
    Routine {
        name: "Pass A".to_string(),
        focus: "Ben, Press & Triceps".to_string(),
        exercises: vec![
            Exercise::standard("Knäböj", 3, "5-8"),
            Exercise::standard("Bänkpress", 3, "5-8"),
            Exercise::standard("Hip Thrusts", 3, "8-12"),
            Exercise::standard("Latsdrag", 3, "8-10"),
            Exercise::superset("Leg Curls", 2, "12-15", "Dips", None),
            Exercise::superset("Dips", 2, "AMRAP", "Leg Curls", None),
            Exercise::standard("Stående vadpress", 3, "12-15"),
        ],
        finishers: vec![
            Exercise::finisher("Shoulder Taps", 3, "20"),
            Exercise::finisher("Mountain Climbers", 3, "30s"),
        ],
    }
}

pub fn get_routine_b() -> Routine {
    Routine {
        name: "Pass B".to_string(),
        focus: "Rygg, Axlar & Biceps".to_string(),
        exercises: vec![
            Exercise::standard("Marklyft", 3, "5"),
            Exercise::standard("Militärpress", 3, "8-10"),
            Exercise::standard("Sittande rodd", 3, "10-12"),
            Exercise::superset("Sidolyft", 3, "12-15", "Hammercurls", None),
            Exercise::superset("Hammercurls", 3, "10-12", "Sidolyft", None),
            Exercise::superset("Facepulls", 3, "15", "Sittande vadpress", None),
            Exercise::superset("Sittande vadpress", 3, "15-20", "Facepulls", None),
        ],
        finishers: vec![
            Exercise::finisher("Dead Bug", 3, "12/sida"),
            Exercise::finisher("Utfallssteg", 3, "20"),
        ],
    }
}

pub fn get_all_exercise_names() -> Vec<String> {
    let a = get_routine_a();
    let b = get_routine_b();
    let mut names: Vec<String> = a.exercises.iter().map(|e| e.name.clone()).collect();
    names.extend(a.finishers.iter().map(|e| e.name.clone()));
    names.extend(b.exercises.iter().map(|e| e.name.clone()));
    names.extend(b.finishers.iter().map(|e| e.name.clone()));
    names.sort();
    names.dedup();
    names
}

pub fn get_workout(routine_id: &str) -> Option<WorkoutData> {
    let db = load_data();
    let routine = match routine_id {
        "A" => get_routine_a(),
        "B" => get_routine_b(),
        _ => return None,
    };

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

    Some(WorkoutData { routine, exercises })
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

