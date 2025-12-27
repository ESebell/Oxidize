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
pub const BIG_FOUR: [&str; 4] = ["Knäböj", "Marklyft", "Bänkpress", "Hip Thrusts"];

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

/// Calculate power-to-weight ratio
pub fn calculate_power_to_weight(db: &Database, bodyweight: f64) -> f64 {
    if bodyweight <= 0.0 { return 0.0; }
    calculate_power_score(db) / bodyweight
}

/// Calculate efficiency (kg lifted per minute)
pub fn calculate_efficiency(session: &Session) -> f64 {
    if session.duration_secs <= 0 { return 0.0; }
    let minutes = session.duration_secs as f64 / 60.0;
    session.total_volume / minutes
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

/// Map exercises to muscle groups
pub fn get_muscle_groups(exercise: &str) -> Vec<MuscleGroup> {
    match exercise {
        "Knäböj" => vec![MuscleGroup::Quads, MuscleGroup::Glutes, MuscleGroup::Core],
        "Bänkpress" => vec![MuscleGroup::Chest, MuscleGroup::Triceps, MuscleGroup::Shoulders],
        "Hip Thrusts" => vec![MuscleGroup::Glutes, MuscleGroup::Hamstrings],
        "Latsdrag" => vec![MuscleGroup::Back, MuscleGroup::Biceps],
        "Leg Curls" => vec![MuscleGroup::Hamstrings],
        "Dips" => vec![MuscleGroup::Chest, MuscleGroup::Triceps],
        "Stående vadpress" | "Sittande vadpress" => vec![MuscleGroup::Calves],
        "Marklyft" => vec![MuscleGroup::Back, MuscleGroup::Hamstrings, MuscleGroup::Glutes, MuscleGroup::Core],
        "Militärpress" => vec![MuscleGroup::Shoulders, MuscleGroup::Triceps],
        "Sittande rodd" => vec![MuscleGroup::Back, MuscleGroup::Biceps],
        "Sidolyft" => vec![MuscleGroup::Shoulders],
        "Facepulls" => vec![MuscleGroup::Shoulders, MuscleGroup::Back],
        "Hammercurls" => vec![MuscleGroup::Biceps],
        _ => vec![],
    }
}

/// Calculate muscle frequency in last N days
pub fn calculate_muscle_frequency(db: &Database, days: i64) -> HashMap<MuscleGroup, u32> {
    let cutoff = chrono::Utc::now().timestamp() - (days * 86400);
    let mut freq: HashMap<MuscleGroup, u32> = HashMap::new();
    
    // Initialize all muscles to 0
    for muscle in MuscleGroup::all() {
        freq.insert(muscle, 0);
    }
    
    for session in &db.sessions {
        if session.timestamp < cutoff { continue; }
        
        for exercise in &session.exercises {
            for muscle in get_muscle_groups(&exercise.name) {
                *freq.entry(muscle).or_insert(0) += 1;
            }
        }
    }
    
    freq
}

/// Calculate average rest time across all sessions
pub fn calculate_avg_rest_time(db: &Database) -> f64 {
    let mut total_rest: i64 = 0;
    let mut count: usize = 0;
    
    for session in &db.sessions {
        for exercise in &session.exercises {
            for set in &exercise.sets {
                if let Some(rest) = set.rest_before_secs {
                    if rest > 0 && rest < 600 { // Ignore unrealistic values
                        total_rest += rest;
                        count += 1;
                    }
                }
            }
        }
    }
    
    if count == 0 { return 0.0; }
    total_rest as f64 / count as f64
}

/// Get rest time stats for an exercise
pub fn get_exercise_rest_stats(db: &Database, exercise_name: &str) -> (f64, f64, f64) {
    let mut rest_times: Vec<i64> = Vec::new();
    
    for session in &db.sessions {
        for exercise in &session.exercises {
            if exercise.name == exercise_name {
                for set in &exercise.sets {
                    if let Some(rest) = set.rest_before_secs {
                        if rest > 0 && rest < 600 {
                            rest_times.push(rest);
                        }
                    }
                }
            }
        }
    }
    
    if rest_times.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    rest_times.sort();
    let avg = rest_times.iter().sum::<i64>() as f64 / rest_times.len() as f64;
    let min = *rest_times.first().unwrap() as f64;
    let max = *rest_times.last().unwrap() as f64;
    
    (avg, min, max)
}

/// E1RM history for an exercise (for graphing)
pub fn get_e1rm_history(db: &Database, exercise_name: &str) -> Vec<(i64, f64)> {
    let mut history: Vec<(i64, f64)> = Vec::new();
    
    for session in &db.sessions {
        if let Some(e1rm) = session_best_e1rm(session, exercise_name) {
            history.push((session.timestamp, e1rm));
        }
    }
    
    history.sort_by_key(|(ts, _)| *ts);
    history
}

/// Power score history (for graphing)
pub fn get_power_score_history(db: &Database) -> Vec<(i64, f64)> {
    // Group sessions by date, calculate power score at each point
    let mut current_best: HashMap<&str, f64> = HashMap::new();
    let mut history: Vec<(i64, f64)> = Vec::new();
    
    // Sort sessions by time
    let mut sessions: Vec<_> = db.sessions.iter().collect();
    sessions.sort_by_key(|s| s.timestamp);
    
    for session in sessions {
        // Update best E1RM for each big lift
        for &lift in &BIG_FOUR {
            if let Some(e1rm) = session_best_e1rm(session, lift) {
                let current = current_best.entry(lift).or_insert(0.0);
                if e1rm > *current {
                    *current = e1rm;
                }
            }
        }
        
        // Calculate total power score at this point
        let score: f64 = current_best.values().sum();
        if score > 0.0 {
            history.push((session.timestamp, score));
        }
    }
    
    history
}

/// Comprehensive stats summary
#[derive(Clone, Debug)]
pub struct StatsSummary {
    pub power_score: f64,
    pub power_to_weight: f64,
    pub bodyweight: f64,
    pub avg_efficiency: f64,
    pub avg_rest_time: f64,
    pub total_sessions: usize,
    pub total_volume: f64,
    pub muscle_frequency: HashMap<MuscleGroup, u32>,
    pub e1rm_by_exercise: HashMap<String, f64>,
}

pub fn get_stats_summary(db: &Database, bodyweight: f64) -> StatsSummary {
    let power_score = calculate_power_score(db);
    
    // Calculate average efficiency
    let avg_efficiency = if db.sessions.is_empty() {
        0.0
    } else {
        db.sessions.iter()
            .map(calculate_efficiency)
            .sum::<f64>() / db.sessions.len() as f64
    };
    
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
        power_to_weight: calculate_power_to_weight(db, bodyweight),
        bodyweight,
        avg_efficiency,
        avg_rest_time: calculate_avg_rest_time(db),
        total_sessions: db.sessions.len(),
        total_volume: db.sessions.iter().map(|s| s.total_volume).sum(),
        muscle_frequency: calculate_muscle_frequency(db, 7),
        e1rm_by_exercise,
    }
}

