use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Exercise {
    pub name: String,
    #[serde(default)]
    pub sets: u8,
    #[serde(default)]
    pub reps_target: String,
    #[serde(default)]
    pub is_superset: bool,
    #[serde(default)]
    pub superset_with: Option<String>,
    #[serde(default)]
    pub superset_name: Option<String>,
    #[serde(default)]
    pub is_bodyweight: bool,
    #[serde(default)]
    pub duration_secs: Option<u32>,  // Some(30) = timed exercise, None = reps-based
    // Wger API data (optional - for routine builder)
    #[serde(default)]
    pub primary_muscles: Vec<String>,
    #[serde(default)]
    pub secondary_muscles: Vec<String>,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub equipment: Option<String>,
    #[serde(default)]
    pub wger_id: Option<u32>,
}

impl Exercise {
    pub fn standard(name: &str, sets: u8, reps: &str) -> Self {
        Self {
            name: name.to_string(),
            sets,
            reps_target: reps.to_string(),
            is_superset: false,
            superset_with: None,
            superset_name: None,
            is_bodyweight: false,
            duration_secs: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            image_url: None,
            equipment: None,
            wger_id: None,
        }
    }

    pub fn superset(name: &str, sets: u8, reps: &str, partner: &str, ss_name: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            sets,
            reps_target: reps.to_string(),
            is_superset: true,
            superset_with: Some(partner.to_string()),
            superset_name: ss_name.map(|s| s.to_string()),
            is_bodyweight: false,
            duration_secs: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            image_url: None,
            equipment: None,
            wger_id: None,
        }
    }
    
    pub fn finisher(name: &str, sets: u8, reps: &str) -> Self {
        Self {
            name: name.to_string(),
            sets,
            reps_target: reps.to_string(),
            is_superset: false,
            superset_with: None,
            superset_name: None,
            is_bodyweight: true,
            duration_secs: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            image_url: None,
            equipment: None,
            wger_id: None,
        }
    }
    
    pub fn timed_finisher(name: &str, sets: u8, duration: u32) -> Self {
        Self {
            name: name.to_string(),
            sets,
            reps_target: format!("{} sek", duration),
            is_superset: false,
            superset_with: None,
            superset_name: None,
            is_bodyweight: true,
            duration_secs: Some(duration),
            primary_muscles: vec![],
            secondary_muscles: vec![],
            image_url: None,
            equipment: None,
            wger_id: None,
        }
    }
    
    /// Create exercise from Wger API data
    pub fn from_wger(
        name: &str,
        sets: u8,
        reps: &str,
        primary_muscles: Vec<String>,
        secondary_muscles: Vec<String>,
        image_url: Option<String>,
        equipment: Option<String>,
        wger_id: u32,
    ) -> Self {
        Self {
            name: name.to_string(),
            sets,
            reps_target: reps.to_string(),
            is_superset: false,
            superset_with: None,
            superset_name: None,
            is_bodyweight: false,
            duration_secs: None,
            primary_muscles,
            secondary_muscles,
            image_url,
            equipment,
            wger_id: Some(wger_id),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SetRecord {
    pub weight: f64,
    pub reps: u8,
    pub timestamp: i64,
    pub rest_before_secs: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LastExerciseData {
    pub weight: f64,
    pub reps: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExerciseWorkoutState {
    pub exercise: Exercise,
    pub last_data: Option<LastExerciseData>,
    pub current_weight: f64,
    pub sets_completed: Vec<SetRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Routine {
    pub name: String,
    pub focus: String,
    pub exercises: Vec<Exercise>,
    pub finishers: Vec<Exercise>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorkoutData {
    pub routine: Routine,
    pub exercises: Vec<ExerciseWorkoutState>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExerciseRecord {
    pub name: String,
    pub sets: Vec<SetRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub id: String,
    pub routine: String,
    pub timestamp: i64,
    pub duration_secs: i64,
    pub exercises: Vec<ExerciseRecord>,
    pub total_volume: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct TotalStats {
    pub total_sessions: usize,
    pub total_volume: f64,
    pub total_sets: usize,
    pub avg_duration_secs: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppView {
    Login,
    Register,
    Dashboard,
    Workout(String),
    Stats,
    Settings,
    RoutineBuilder(Option<String>), // Some(id) = editing, None = new
}

/// Stored routine in Supabase
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SavedRoutine {
    pub id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    pub name: String,
    pub focus: String,
    pub passes: Vec<Pass>,
    pub is_active: bool,
    pub created_at: i64,
}

/// A single pass within a routine (e.g., "Pass A")
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pass {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub exercises: Vec<Exercise>,
    pub finishers: Vec<Exercise>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub user: AuthUser,
}

/// Paused workout state - saved when leaving mid-workout
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PausedWorkout {
    pub routine_name: String,
    pub exercises: Vec<ExerciseWorkoutState>,
    pub current_exercise_idx: usize,
    pub start_timestamp: i64,
    pub elapsed_secs: i64,
}
