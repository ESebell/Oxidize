import Foundation

struct ExerciseRecord: Codable, Hashable {
    var name: String
    var sets: [SetRecord]
}

struct Session: Codable, Identifiable, Hashable {
    var id: String
    var routine: String
    var timestamp: Int64
    var durationSecs: Int64
    var exercises: [ExerciseRecord]
    var totalVolume: Double

    enum CodingKeys: String, CodingKey {
        case id, routine, timestamp, exercises
        case durationSecs = "duration_secs"
        case totalVolume = "total_volume"
    }
}

struct TotalStats {
    var totalSessions: Int = 0
    var totalVolume: Double = 0
    var totalSets: Int = 0
    var avgDurationSecs: Int64 = 0
}
