import Foundation

struct BodyweightEntry: Codable, Hashable {
    var timestamp: Int64
    var weight: Double
}

struct Database: Codable {
    var sessions: [Session] = []
    var lastWeights: [String: LastExerciseData] = [:]
    var bodyweight: Double?
    var bodyweightHistory: [BodyweightEntry] = []

    enum CodingKeys: String, CodingKey {
        case sessions
        case lastWeights = "last_weights"
        case bodyweight
        case bodyweightHistory = "bodyweight_history"
    }

    mutating func addSession(_ session: Session) {
        for ex in session.exercises {
            if let lastSet = ex.sets.last {
                lastWeights[ex.name] = LastExerciseData(weight: lastSet.weight, reps: lastSet.reps)
            }
        }
        sessions.append(session)
    }

    func getLastExerciseData(_ exercise: String) -> LastExerciseData? {
        lastWeights[exercise]
    }

    func getRecentSessions(limit: Int) -> [Session] {
        sessions.sorted { $0.timestamp > $1.timestamp }.prefix(limit).map { $0 }
    }

    func getTotalStats() -> TotalStats {
        let totalSessions = sessions.count
        let totalVolume = sessions.reduce(0.0) { $0 + $1.totalVolume }
        let totalSets = sessions.flatMap(\.exercises).reduce(0) { $0 + $1.sets.count }
        let avgDuration: Int64 = totalSessions > 0
            ? sessions.reduce(Int64(0)) { $0 + $1.durationSecs } / Int64(totalSessions)
            : 0

        return TotalStats(
            totalSessions: totalSessions,
            totalVolume: totalVolume,
            totalSets: totalSets,
            avgDurationSecs: avgDuration
        )
    }

    mutating func setBodyweight(_ weight: Double) {
        bodyweight = weight
        bodyweightHistory.append(BodyweightEntry(
            timestamp: Int64(Date().timeIntervalSince1970),
            weight: weight
        ))
    }
}
