import Foundation

@Observable
final class StatsViewModel {
    var summary = StatsSummary()
    var powerScoreHistory: [(Int64, Double)] = []
    var bodyweightHistory: [BodyweightEntry] = []
    var lastSessionProgression: [(String, ProgressStatus)] = []
    var bodyweight: Double = 0
    var recentSessions: [Session] = []

    func loadStats() {
        let db = StorageService.shared.loadData()
        bodyweight = db.bodyweight ?? 0

        summary = StatsEngine.getStatsSummary(db: db, bodyweight: bodyweight)
        powerScoreHistory = StatsEngine.getPowerScoreHistory(db: db)
        bodyweightHistory = db.bodyweightHistory.sorted { $0.timestamp < $1.timestamp }
        recentSessions = Array(db.sessions.sorted { $0.timestamp > $1.timestamp }.prefix(10))

        // Progression for latest session
        if let latestSession = db.sessions.max(by: { $0.timestamp < $1.timestamp }) {
            lastSessionProgression = latestSession.exercises.map { ex in
                let status = StatsEngine.checkProgressiveOverload(
                    db: db, exerciseName: ex.name, currentSession: latestSession
                )
                return (ex.name, status)
            }
        }
    }
}
