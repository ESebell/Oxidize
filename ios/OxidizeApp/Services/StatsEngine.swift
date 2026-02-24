import Foundation

enum StatsEngine {
    /// Brzycki formula: weight * (36 / (37 - reps))
    static func calculateE1RM(weight: Double, reps: Int) -> Double {
        if reps == 0 { return 0 }
        if reps == 1 { return weight }
        if reps >= 37 { return weight * 2.0 }
        return weight * (36.0 / (37.0 - Double(reps)))
    }

    static func sessionBestE1RM(session: Session, exerciseName: String) -> Double? {
        guard let exercise = session.exercises.first(where: { $0.name == exerciseName }) else {
            return nil
        }
        let best = exercise.sets.map { calculateE1RM(weight: $0.weight, reps: $0.reps) }.max()
        return best
    }

    static func calculatePowerScore(db: Database) -> Double {
        BIG_FOUR.reduce(0.0) { total, name in
            let best = db.sessions.compactMap { sessionBestE1RM(session: $0, exerciseName: name) }.max() ?? 0
            return total + best
        }
    }

    static func checkProgressiveOverload(db: Database, exerciseName: String, currentSession: Session) -> ProgressStatus {
        guard let current = currentSession.exercises.first(where: { $0.name == exerciseName }) else {
            return .firstTime
        }

        let previousSession = db.sessions
            .filter { $0.id != currentSession.id && $0.timestamp < currentSession.timestamp }
            .filter { $0.exercises.contains { $0.name == exerciseName } }
            .max(by: { $0.timestamp < $1.timestamp })

        guard let previous = previousSession?.exercises.first(where: { $0.name == exerciseName }) else {
            return .firstTime
        }

        let currentE1rm = current.sets.map { calculateE1RM(weight: $0.weight, reps: $0.reps) }.max() ?? 0
        let previousE1rm = previous.sets.map { calculateE1RM(weight: $0.weight, reps: $0.reps) }.max() ?? 0

        let currentVolume = current.sets.reduce(0.0) { $0 + $1.weight * Double($1.reps) }
        let previousVolume = previous.sets.reduce(0.0) { $0 + $1.weight * Double($1.reps) }

        if currentE1rm > previousE1rm * 1.005 || currentVolume > previousVolume * 1.01 {
            return .improved
        } else if currentE1rm >= previousE1rm * 0.95 && currentVolume >= previousVolume * 0.95 {
            return .maintained
        } else {
            return .regressed
        }
    }

    // MARK: - Muscle Groups

    /// Maps muscle names to MuscleGroup.
    /// Handles: Wger Latin names (exerciseinfo), English names, Swedish category names (legacy data)
    static func parseMuscleGroup(_ name: String) -> MuscleGroup? {
        let n = name.lowercased()
        switch n {
        // Chest — Wger: "Pectoralis major" (id 4)
        case "pectoralis major", "chest", "bröst":
            return .chest
        // Back — Wger: "Latissimus dorsi" (id 12), "Trapezius" (id 9), "Serratus anterior" (id 3)
        // + "Erector spinae" (manual override, not in Wger)
        case "latissimus dorsi", "trapezius", "serratus anterior", "erector spinae",
             "back", "lats", "rygg":
            return .back
        // Shoulders — Wger: "Anterior deltoid" (id 2)
        case "anterior deltoid", "posterior deltoid", "lateral deltoid",
             "shoulders", "deltoids", "delts", "axlar":
            return .shoulders
        // Biceps — Wger: "Biceps brachii" (id 1), "Brachialis" (id 13)
        case "biceps brachii", "brachialis",
             "biceps", "bicep":
            return .biceps
        // Triceps — Wger: "Triceps brachii" (id 5)
        case "triceps brachii",
             "triceps", "tricep":
            return .triceps
        // Quads — Wger: "Quadriceps femoris" (id 10)
        case "quadriceps femoris",
             "quads", "quadriceps", "legs", "ben":
            return .quads
        // Hamstrings — Wger: "Biceps femoris" (id 11)
        case "biceps femoris",
             "hamstrings", "hamstring":
            return .hamstrings
        // Glutes — Wger: "Gluteus maximus" (id 8)
        case "gluteus maximus", "gluteus medius",
             "glutes", "gluteals", "rumpa":
            return .glutes
        // Calves — Wger: "Gastrocnemius" (id 7), "Soleus" (id 15)
        case "gastrocnemius", "soleus",
             "calves", "calf", "vader":
            return .calves
        // Core — Wger: "Rectus abdominis" (id 6), "Obliquus externus abdominis" (id 14)
        case "rectus abdominis", "obliquus externus abdominis",
             "core", "abs", "abdominals", "obliques", "mage", "magmuskler":
            return .core
        // Swedish legacy category names that are too generic — map to most common
        case "armar":  // "Arms" — could be biceps or triceps, default to biceps
            return .biceps
        default:
            return nil
        }
    }

    /// Get muscle groups for an Exercise (used in routine builder UI)
    static func getMuscleGroupsForExercise(_ exercise: Exercise) -> [(MuscleGroup, Int)] {
        var result: [(MuscleGroup, Int)] = []
        for name in exercise.primaryMuscles {
            if let mg = parseMuscleGroup(name) { result.append((mg, 3)) }
        }
        for name in exercise.secondaryMuscles {
            if let mg = parseMuscleGroup(name) { result.append((mg, 1)) }
        }
        return result
    }

    /// Get muscle groups from a session exercise record (used in stats)
    static func musclesFromRecord(_ record: ExerciseRecord) -> [(MuscleGroup, Int)] {
        var result: [(MuscleGroup, Int)] = []
        for name in record.primaryMuscles {
            if let mg = parseMuscleGroup(name) { result.append((mg, 3)) }
        }
        for name in record.secondaryMuscles {
            if let mg = parseMuscleGroup(name) { result.append((mg, 1)) }
        }
        return result
    }

    /// Per-muscle weekly sets using exact Wger muscle names (lowercased).
    /// Primary muscles get full set count, secondary get 1/3.
    static func calculateWeeklyMuscleDetail(db: Database, days: Int64 = 7) -> [String: Int] {
        let cutoff = currentTimestamp() - (days * 86400)
        var sets: [String: Int] = [:]
        for session in db.sessions where session.timestamp >= cutoff {
            for exercise in session.exercises {
                let count = exercise.sets.count
                if count == 0 { continue }
                for muscle in exercise.primaryMuscles {
                    sets[muscle.lowercased(), default: 0] += count
                }
                for muscle in exercise.secondaryMuscles {
                    sets[muscle.lowercased(), default: 0] += max(1, count / 3)
                }
            }
        }
        return sets
    }

    static func calculateWeeklySets(db: Database, days: Int64 = 7) -> [MuscleGroup: Int] {
        let cutoff = currentTimestamp() - (days * 86400)
        var sets: [MuscleGroup: Int] = [:]
        for muscle in MuscleGroup.allCases { sets[muscle] = 0 }

        let weeklySessions = db.sessions.filter { $0.timestamp >= cutoff }

        for session in weeklySessions {
            for exercise in session.exercises {
                let setsCompleted = exercise.sets.count
                if setsCompleted == 0 { continue }

                let muscles = musclesFromRecord(exercise)
                for (muscle, weight) in muscles {
                    if weight >= 3 {
                        sets[muscle, default: 0] += setsCompleted
                    }
                }
            }
        }
        return sets
    }

    static func getPowerScoreHistory(db: Database) -> [(Int64, Double)] {
        var currentBest: [String: Double] = [:]
        var history: [(Int64, Double)] = []

        let sortedSessions = db.sessions.sorted { $0.timestamp < $1.timestamp }

        for session in sortedSessions {
            for lift in BIG_FOUR {
                if let e1rm = sessionBestE1RM(session: session, exerciseName: lift) {
                    let current = currentBest[lift] ?? 0
                    if e1rm > current { currentBest[lift] = e1rm }
                }
            }
            let score = currentBest.values.reduce(0, +)
            if score > 0 {
                history.append((session.timestamp, score))
            }
        }

        return history
    }

    static func getStatsSummary(db: Database, bodyweight: Double) -> StatsSummary {
        let powerScore = calculatePowerScore(db: db)

        var e1rmByExercise: [String: Double] = [:]
        for session in db.sessions {
            for exercise in session.exercises {
                let best = sessionBestE1RM(session: session, exerciseName: exercise.name) ?? 0
                let current = e1rmByExercise[exercise.name] ?? 0
                if best > current { e1rmByExercise[exercise.name] = best }
            }
        }

        return StatsSummary(
            powerScore: powerScore,
            bodyweight: bodyweight,
            totalSessions: db.sessions.count,
            weeklySets: calculateWeeklySets(db: db),
            weeklyMuscleDetail: calculateWeeklyMuscleDetail(db: db),
            e1rmByExercise: e1rmByExercise
        )
    }
}
