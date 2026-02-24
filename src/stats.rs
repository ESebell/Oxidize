use std::collections::HashMap;
use crate::types::*;
use crate::storage::Database;

/// Brzycki formula for 1RM: weight × (36 / (37 - reps))
/// More accurate than Epley for rep ranges 1-10
pub fn calculate_e1rm(weight: f64, reps: u8) -> f64 {
    if reps == 0 { return 0.0; }
    if reps == 1 { return weight; }
    if reps >= 37 { return weight * 2.0; } // Cap it
    weight * (36.0 / (37.0 - reps as f64))
}

/// Get the best E1RM for an exercise from a session
pub fn session_best_e1rm(session: &Session, exercise_name: &str) -> Option<f64> {
    session.exercises.iter()
        .find(|e| e.name == exercise_name)
        .and_then(|e| {
            e.sets.iter()
                .map(|s| calculate_e1rm(s.weight, s.reps))
                .fold(None, |max, val| match max {
                    None => Some(val),
                    Some(m) if val > m => Some(val),
                    _ => max,
                })
        })
}

/// The "Big 4" lifts for power score
pub const BIG_FOUR: [&str; 4] = ["Squats", "Deadlift", "Bench Press", "Shoulder Press"];

/// Calculate total power score (sum of E1RM for big 4)
pub fn calculate_power_score(db: &Database) -> f64 {
    BIG_FOUR.iter()
        .map(|&name| {
            db.sessions.iter()
                .filter_map(|s| session_best_e1rm(s, name))
                .fold(0.0, f64::max)
        })
        .sum()
}

/// Check if an exercise showed progressive overload vs last time
#[derive(Clone, Debug, PartialEq)]
pub enum ProgressStatus {
    Improved,    // 🔥 Increased weight, reps, or sets
    Maintained,  // ➡️ Same as before
    Regressed,   // ⬇️ Did less
    FirstTime,   // 🆕 No previous data
}

pub fn check_progressive_overload(db: &Database, exercise_name: &str, current_session: &Session) -> ProgressStatus {
    // Find the current exercise data
    let current = match current_session.exercises.iter().find(|e| e.name == exercise_name) {
        Some(e) => e,
        None => return ProgressStatus::FirstTime,
    };
    
    // Find the previous session with this exercise
    let previous_session = db.sessions.iter()
        .filter(|s| s.id != current_session.id && s.timestamp < current_session.timestamp)
        .filter(|s| s.exercises.iter().any(|e| e.name == exercise_name))
        .max_by_key(|s| s.timestamp);
    
    let previous = match previous_session {
        Some(s) => match s.exercises.iter().find(|e| e.name == exercise_name) {
            Some(e) => e,
            None => return ProgressStatus::FirstTime,
        },
        None => return ProgressStatus::FirstTime,
    };
    
    // Compare E1RM
    let current_e1rm = current.sets.iter()
        .map(|s| calculate_e1rm(s.weight, s.reps))
        .fold(0.0, f64::max);
    let previous_e1rm = previous.sets.iter()
        .map(|s| calculate_e1rm(s.weight, s.reps))
        .fold(0.0, f64::max);
    
    // Compare volume
    let current_volume: f64 = current.sets.iter().map(|s| s.weight * s.reps as f64).sum();
    let previous_volume: f64 = previous.sets.iter().map(|s| s.weight * s.reps as f64).sum();
    
    // Improved if E1RM or volume increased
    if current_e1rm > previous_e1rm * 1.005 || current_volume > previous_volume * 1.01 {
        ProgressStatus::Improved
    } else if current_e1rm >= previous_e1rm * 0.95 && current_volume >= previous_volume * 0.95 {
        ProgressStatus::Maintained
    } else {
        ProgressStatus::Regressed
    }
}

/// Muscle groups
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MuscleGroup {
    Chest,
    Back,
    Shoulders,
    Biceps,
    Triceps,
    Quads,
    Hamstrings,
    Glutes,
    Calves,
    Core,
}

impl MuscleGroup {
    pub fn name(&self) -> &'static str {
        match self {
            MuscleGroup::Chest => "Bröst",
            MuscleGroup::Back => "Rygg",
            MuscleGroup::Shoulders => "Axlar",
            MuscleGroup::Biceps => "Biceps",
            MuscleGroup::Triceps => "Triceps",
            MuscleGroup::Quads => "Lår (fram)",
            MuscleGroup::Hamstrings => "Lår (bak)",
            MuscleGroup::Glutes => "Rumpa",
            MuscleGroup::Calves => "Vader",
            MuscleGroup::Core => "Mage",
        }
    }
    
    pub fn all() -> Vec<MuscleGroup> {
        vec![
            MuscleGroup::Chest, MuscleGroup::Back, MuscleGroup::Shoulders,
            MuscleGroup::Biceps, MuscleGroup::Triceps, MuscleGroup::Quads,
            MuscleGroup::Hamstrings, MuscleGroup::Glutes, MuscleGroup::Calves,
            MuscleGroup::Core,
        ]
    }
}

/// Maps muscle names to MuscleGroup.
/// Handles: Wger Latin names (exerciseinfo), English names, Swedish category names (legacy data)
pub fn parse_muscle_name(name: &str) -> Option<MuscleGroup> {
    match name.to_lowercase().as_str() {
        // Chest — Wger: "Pectoralis major" (id 4)
        "pectoralis major" | "chest" | "bröst" => Some(MuscleGroup::Chest),
        // Back — Wger: "Latissimus dorsi" (id 12), "Trapezius" (id 9), "Serratus anterior" (id 3)
        "latissimus dorsi" | "trapezius" | "serratus anterior"
            | "back" | "lats" | "rygg" => Some(MuscleGroup::Back),
        // Shoulders — Wger: "Anterior deltoid" (id 2)
        "anterior deltoid" | "posterior deltoid" | "lateral deltoid"
            | "shoulders" | "deltoids" | "delts" | "axlar" => Some(MuscleGroup::Shoulders),
        // Biceps — Wger: "Biceps brachii" (id 1), "Brachialis" (id 13)
        "biceps brachii" | "brachialis"
            | "biceps" | "bicep" => Some(MuscleGroup::Biceps),
        // Triceps — Wger: "Triceps brachii" (id 5)
        "triceps brachii"
            | "triceps" | "tricep" => Some(MuscleGroup::Triceps),
        // Quads — Wger: "Quadriceps femoris" (id 10)
        "quadriceps femoris"
            | "quads" | "quadriceps" | "legs" | "ben" => Some(MuscleGroup::Quads),
        // Hamstrings — Wger: "Biceps femoris" (id 11)
        "biceps femoris"
            | "hamstrings" | "hamstring" => Some(MuscleGroup::Hamstrings),
        // Glutes — Wger: "Gluteus maximus" (id 8)
        "gluteus maximus" | "gluteus medius"
            | "glutes" | "gluteals" | "rumpa" => Some(MuscleGroup::Glutes),
        // Calves — Wger: "Gastrocnemius" (id 7), "Soleus" (id 15)
        "gastrocnemius" | "soleus"
            | "calves" | "calf" | "vader" => Some(MuscleGroup::Calves),
        // Core — Wger: "Rectus abdominis" (id 6), "Obliquus externus abdominis" (id 14)
        "rectus abdominis" | "obliquus externus abdominis"
            | "core" | "abs" | "abdominals" | "obliques" | "mage" | "magmuskler" => Some(MuscleGroup::Core),
        // Swedish legacy category names that are too generic
        "armar" => Some(MuscleGroup::Biceps),  // "Arms" — default to biceps
        _ => None,
    }
}

/// Get muscle groups for an Exercise struct from its stored muscle data
pub fn get_muscle_groups_for_exercise(exercise: &crate::types::Exercise) -> Vec<(MuscleGroup, u32)> {
    let mut result = Vec::new();
    for name in &exercise.primary_muscles {
        if let Some(mg) = parse_muscle_name(name) {
            result.push((mg, 3));
        }
    }
    for name in &exercise.secondary_muscles {
        if let Some(mg) = parse_muscle_name(name) {
            result.push((mg, 1));
        }
    }
    result
}

/// Get muscle groups from a session exercise record
pub fn muscles_from_record(record: &ExerciseRecord) -> Vec<(MuscleGroup, u32)> {
    let mut result = Vec::new();
    for name in &record.primary_muscles {
        if let Some(mg) = parse_muscle_name(name) {
            result.push((mg, 3));
        }
    }
    for name in &record.secondary_muscles {
        if let Some(mg) = parse_muscle_name(name) {
            result.push((mg, 1));
        }
    }
    result
}

/// Calculate weekly sets per muscle group (primary muscles only).
/// Counts actual completed sets for each muscle group where the exercise
/// targets that muscle as primary (weight == 3).
/// Research suggests 10-20 sets per muscle group per week is optimal.
pub fn calculate_weekly_sets(db: &Database, days: i64) -> HashMap<MuscleGroup, u32> {
    let cutoff = chrono::Utc::now().timestamp() - (days * 86400);
    let mut sets: HashMap<MuscleGroup, u32> = HashMap::new();

    for muscle in MuscleGroup::all() {
        sets.insert(muscle, 0);
    }

    for session in &db.sessions {
        if session.timestamp < cutoff { continue; }

        for exercise in &session.exercises {
            let sets_completed = exercise.sets.len() as u32;
            if sets_completed == 0 { continue; }

            let muscles = muscles_from_record(exercise);

            // Only count primary muscles (weight == 3)
            for (muscle, weight) in muscles {
                if weight >= 3 {
                    *sets.entry(muscle).or_insert(0) += sets_completed;
                }
            }
        }
    }

    sets
}

/// Power score history (for graphing)
pub fn get_power_score_history(db: &Database) -> Vec<(i64, f64)> {
    let mut current_best: HashMap<&str, f64> = HashMap::new();
    let mut history: Vec<(i64, f64)> = Vec::new();

    let mut sessions: Vec<_> = db.sessions.iter().collect();
    sessions.sort_by_key(|s| s.timestamp);

    for session in sessions {
        for &lift in &BIG_FOUR {
            if let Some(e1rm) = session_best_e1rm(session, lift) {
                let current = current_best.entry(lift).or_insert(0.0);
                if e1rm > *current {
                    *current = e1rm;
                }
            }
        }

        let score: f64 = current_best.values().sum();
        if score > 0.0 {
            history.push((session.timestamp, score));
        }
    }

    history
}

/// Comprehensive stats summary
#[derive(Clone, Debug, PartialEq)]
pub struct StatsSummary {
    pub power_score: f64,
    pub bodyweight: f64,
    pub total_sessions: usize,
    pub weekly_sets: HashMap<MuscleGroup, u32>,
    pub e1rm_by_exercise: HashMap<String, f64>,
}

pub fn get_stats_summary(db: &Database, bodyweight: f64) -> StatsSummary {
    let power_score = calculate_power_score(db);

    // Get best E1RM for each exercise
    let mut e1rm_by_exercise: HashMap<String, f64> = HashMap::new();
    for session in &db.sessions {
        for exercise in &session.exercises {
            let best = session_best_e1rm(session, &exercise.name).unwrap_or(0.0);
            let current = e1rm_by_exercise.entry(exercise.name.clone()).or_insert(0.0);
            if best > *current {
                *current = best;
            }
        }
    }

    StatsSummary {
        power_score,
        bodyweight,
        total_sessions: db.sessions.len(),
        weekly_sets: calculate_weekly_sets(db, 7),
        e1rm_by_exercise,
    }
}

