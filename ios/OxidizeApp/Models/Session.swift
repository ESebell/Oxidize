import Foundation

struct ExerciseRecord: Codable, Hashable {
    var name: String
    var sets: [SetRecord]
    var primaryMuscles: [String]
    var secondaryMuscles: [String]

    enum CodingKeys: String, CodingKey {
        case name, sets
        case primaryMuscles = "primary_muscles"
        case secondaryMuscles = "secondary_muscles"
    }

    init(name: String, sets: [SetRecord], primaryMuscles: [String] = [], secondaryMuscles: [String] = []) {
        self.name = name
        self.sets = sets
        self.primaryMuscles = primaryMuscles
        self.secondaryMuscles = secondaryMuscles
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        name = try c.decode(String.self, forKey: .name)
        sets = try c.decode([SetRecord].self, forKey: .sets)
        primaryMuscles = try c.decodeIfPresent([String].self, forKey: .primaryMuscles) ?? []
        secondaryMuscles = try c.decodeIfPresent([String].self, forKey: .secondaryMuscles) ?? []
    }
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
