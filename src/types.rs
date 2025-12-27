use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Exercise {
    pub name: String,
    pub sets: u8,
    pub reps_target: String,
    pub is_superset: bool,
    pub superset_with: Option<String>,
    pub superset_name: Option<String>,
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
    pub finisher: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorkoutData {
    pub routine: Routine,
    pub exercises: Vec<ExerciseWorkoutState>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExerciseRecord {
    pub name: String,
    pub sets: Vec<SetRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub routine: String,
    pub timestamp: i64,
    pub duration_secs: i64,
    pub exercises: Vec<ExerciseRecord>,
    pub total_volume: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExerciseStats {
    pub name: String,
    pub current_weight: f64,
    pub estimated_1rm: f64,
    pub total_volume: f64,
    pub best_set: Option<SetRecord>,
    pub sessions_count: usize,
    pub avg_rest_time: f64,
    pub volume_trend: Vec<(i64, f64)>,
    pub one_rm_trend: Vec<(i64, f64)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TotalStats {
    pub total_sessions: usize,
    pub total_volume: f64,
    pub total_sets: usize,
    pub avg_duration_secs: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppView {
    Dashboard,
    Workout(String),
    Stats,
}

/// Paused workout state - saved when leaving mid-workout
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PausedWorkout {
    pub routine_name: String,
    pub exercises: Vec<ExerciseWorkoutState>,
    pub current_exercise_idx: usize,
    pub start_timestamp: i64,
    pub elapsed_secs: i64,
}
