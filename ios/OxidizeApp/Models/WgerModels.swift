import Foundation

struct WgerExercise: Codable, Identifiable, Hashable {
    var id: Int
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
    var name: String
    var category: WgerCategory?
    var muscles: [WgerMuscle]?
    var musclesSecondary: [WgerMuscle]?
    var image: String?
    var equipment: [WgerEquipment]?

    enum CodingKeys: String, CodingKey {
        case id, name, category, muscles, image, equipment
        case musclesSecondary = "muscles_secondary"
    }
}

struct WgerCategory: Codable {
    var name: String
}

struct WgerMuscle: Codable {
    var name: String
    var nameEn: String?

    enum CodingKeys: String, CodingKey {
        case name
        case nameEn = "name_en"
    }
}

struct WgerEquipment: Codable {
    var name: String
}
