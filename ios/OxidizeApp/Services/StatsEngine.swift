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

    static func parseMuscleGroup(_ name: String) -> MuscleGroup? {
        switch name.lowercased() {
        case "chest", "bröst", "pectoralis": return .chest
        case "back", "rygg", "lats", "latissimus": return .back
        case "shoulders", "axlar", "deltoids", "delts": return .shoulders
        case "biceps", "bicep": return .biceps
        case "triceps", "tricep": return .triceps
        case "quads", "quadriceps", "lår (fram)", "legs": return .quads
        case "hamstrings", "lår (bak)", "hamstring": return .hamstrings
        case "glutes", "rumpa", "gluteus", "gluteals": return .glutes
        case "calves", "vader", "calf": return .calves
        case "core", "abs", "mage", "abdominals": return .core
        default: return nil
        }
    }

    static func getMuscleGroupsWeighted(_ exerciseName: String) -> [(MuscleGroup, Int)] {
        switch exerciseName {
        case "Squats", "Knäböj":
            return [(.quads, 3), (.glutes, 3), (.core, 1)]
        case "Bench Press", "Bänkpress":
            return [(.chest, 3), (.triceps, 1), (.shoulders, 1)]
        case "Hip Thrusts":
            return [(.glutes, 3), (.hamstrings, 1)]
        case "Latsdrag":
            return [(.back, 3), (.biceps, 1)]
        case "Leg Curls":
            return [(.hamstrings, 3)]
        case "Dips":
            return [(.chest, 3), (.triceps, 3)]
        case "Stående vadpress", "Sittande vadpress":
            return [(.calves, 3)]
        case "Deadlift", "Marklyft":
            return [(.back, 3), (.hamstrings, 3), (.glutes, 3), (.core, 1)]
        case "Shoulder Press", "Militärpress":
            return [(.shoulders, 3), (.triceps, 1)]
        case "Sittande rodd":
            return [(.back, 3), (.biceps, 1)]
        case "Sidolyft":
            return [(.shoulders, 3)]
        case "Facepulls":
            return [(.shoulders, 3), (.back, 1)]
        case "Hammercurls":
            return [(.biceps, 3)]
        case "Shoulder Taps":
            return [(.core, 3), (.shoulders, 1)]
        case "Mountain Climbers":
            return [(.core, 3), (.quads, 1)]
        case "Dead Bug":
            return [(.core, 3)]
        case "Utfallssteg":
            return [(.quads, 3), (.glutes, 3)]
        default:
            return []
        }
    }

    static func getMuscleGroupsForExercise(_ exercise: Exercise) -> [(MuscleGroup, Int)] {
        let hardcoded = getMuscleGroupsWeighted(exercise.name)
        if !hardcoded.isEmpty { return hardcoded }

        var result: [(MuscleGroup, Int)] = []
        for name in exercise.primaryMuscles {
            if let mg = parseMuscleGroup(name) { result.append((mg, 3)) }
        }
        for name in exercise.secondaryMuscles {
            if let mg = parseMuscleGroup(name) { result.append((mg, 1)) }
        }
        return result
    }

    static func calculateWeeklySets(db: Database, days: Int64 = 7) -> [MuscleGroup: Int] {
        let cutoff = currentTimestamp() - (days * 86400)
        var sets: [MuscleGroup: Int] = [:]
        for muscle in MuscleGroup.allCases { sets[muscle] = 0 }

        let routine = StorageService.shared.loadActiveRoutine()
        var exerciseLookup: [String: Exercise] = [:]
        if let routine {
            for pass in routine.passes {
                for ex in pass.exercises + pass.finishers {
                    exerciseLookup[ex.name] = ex
                }
            }
        }

        for session in db.sessions {
            if session.timestamp < cutoff { continue }

            for exercise in session.exercises {
                let setsCompleted = exercise.sets.count
                if setsCompleted == 0 { continue }

                var muscles = getMuscleGroupsWeighted(exercise.name)
                if muscles.isEmpty, let ex = exerciseLookup[exercise.name] {
                    muscles = getMuscleGroupsForExercise(ex)
                }

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
            e1rmByExercise: e1rmByExercise
        )
    }
}
