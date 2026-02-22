import Foundation

struct Pass: Codable, Hashable, Identifiable {
    var id = UUID()
    var name: String
    var description: String = ""
    var exercises: [Exercise]
    var finishers: [Exercise]

    enum CodingKeys: String, CodingKey {
        case name, description, exercises, finishers
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        name = try c.decode(String.self, forKey: .name)
        description = try c.decodeIfPresent(String.self, forKey: .description) ?? ""
        exercises = try c.decode([Exercise].self, forKey: .exercises)
        finishers = try c.decodeIfPresent([Exercise].self, forKey: .finishers) ?? []
    }

    init(name: String, description: String = "", exercises: [Exercise], finishers: [Exercise] = []) {
        self.name = name
        self.description = description
        self.exercises = exercises
        self.finishers = finishers
    }
}

struct SavedRoutine: Codable, Identifiable, Hashable {
    var id: String
    var userId: String?
    var name: String
    var focus: String
    var passes: [Pass]
    var isActive: Bool
    var createdAt: Int64

    enum CodingKeys: String, CodingKey {
        case id, name, focus, passes
        case userId = "user_id"
        case isActive = "is_active"
        case createdAt = "created_at"
    }
}

struct WorkoutRoutine: Codable {
    var name: String
    var focus: String
    var exercises: [Exercise]
    var finishers: [Exercise]
}
