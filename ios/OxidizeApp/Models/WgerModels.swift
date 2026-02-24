import Foundation

struct WgerExercise: Codable, Identifiable, Hashable {
    var id: Int
    var baseId: Int
    var name: String
    var primaryMuscles: [String]
    var secondaryMuscles: [String]
    var imageUrl: String?
    var equipment: String?
}

struct WgerSearchResponse: Codable {
    var suggestions: [WgerSuggestion]
}

struct WgerSuggestion: Codable {
    var data: WgerSuggestionData
}

struct WgerSuggestionData: Codable {
    var id: Int
    var baseId: Int
    var name: String
    var category: String?
    var image: String?

    enum CodingKeys: String, CodingKey {
        case id, name, category, image
        case baseId = "base_id"
    }
}

struct WgerMuscle: Codable {
    var id: Int
    var name: String
    var nameEn: String?

    enum CodingKeys: String, CodingKey {
        case id, name
        case nameEn = "name_en"
    }
}

struct WgerEquipment: Codable {
    var name: String
}

// Response from /api/v2/exerciseinfo/{base_id}/
struct WgerExerciseInfo: Codable {
    var muscles: [WgerMuscle]
    var musclesSecondary: [WgerMuscle]

    enum CodingKeys: String, CodingKey {
        case muscles
        case musclesSecondary = "muscles_secondary"
    }
}
