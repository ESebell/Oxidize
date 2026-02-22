import Foundation

struct Exercise: Codable, Hashable, Identifiable {
    var id = UUID()
    var name: String
    var sets: Int = 0
    var repsTarget: String = ""
    var isSuperset: Bool = false
    var supersetWith: String?
    var supersetName: String?
    var isBodyweight: Bool = false
    var durationSecs: Int?
    var primaryMuscles: [String] = []
    var secondaryMuscles: [String] = []
    var imageUrl: String?
    var equipment: String?
    var wgerId: Int?

    enum CodingKeys: String, CodingKey {
        case name, sets, equipment
        case repsTarget = "reps_target"
        case isSuperset = "is_superset"
        case supersetWith = "superset_with"
        case supersetName = "superset_name"
        case isBodyweight = "is_bodyweight"
        case durationSecs = "duration_secs"
        case primaryMuscles = "primary_muscles"
        case secondaryMuscles = "secondary_muscles"
        case imageUrl = "image_url"
        case wgerId = "wger_id"
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        name = try c.decode(String.self, forKey: .name)
        sets = try c.decodeIfPresent(Int.self, forKey: .sets) ?? 0
        repsTarget = try c.decodeIfPresent(String.self, forKey: .repsTarget) ?? ""
        isSuperset = try c.decodeIfPresent(Bool.self, forKey: .isSuperset) ?? false
        supersetWith = try c.decodeIfPresent(String.self, forKey: .supersetWith)
        supersetName = try c.decodeIfPresent(String.self, forKey: .supersetName)
        isBodyweight = try c.decodeIfPresent(Bool.self, forKey: .isBodyweight) ?? false
        durationSecs = try c.decodeIfPresent(Int.self, forKey: .durationSecs)
        primaryMuscles = try c.decodeIfPresent([String].self, forKey: .primaryMuscles) ?? []
        secondaryMuscles = try c.decodeIfPresent([String].self, forKey: .secondaryMuscles) ?? []
        imageUrl = try c.decodeIfPresent(String.self, forKey: .imageUrl)
        equipment = try c.decodeIfPresent(String.self, forKey: .equipment)
        wgerId = try c.decodeIfPresent(Int.self, forKey: .wgerId)
    }

    init(name: String, sets: Int = 0, repsTarget: String = "", isSuperset: Bool = false, supersetWith: String? = nil, supersetName: String? = nil, isBodyweight: Bool = false, durationSecs: Int? = nil, primaryMuscles: [String] = [], secondaryMuscles: [String] = [], imageUrl: String? = nil, equipment: String? = nil, wgerId: Int? = nil) {
        self.name = name
        self.sets = sets
        self.repsTarget = repsTarget
        self.isSuperset = isSuperset
        self.supersetWith = supersetWith
        self.supersetName = supersetName
        self.isBodyweight = isBodyweight
        self.durationSecs = durationSecs
        self.primaryMuscles = primaryMuscles
        self.secondaryMuscles = secondaryMuscles
        self.imageUrl = imageUrl
        self.equipment = equipment
        self.wgerId = wgerId
    }

    static func standard(_ name: String, sets: Int, reps: String) -> Exercise {
        Exercise(name: name, sets: sets, repsTarget: reps)
    }

    static func superset(_ name: String, sets: Int, reps: String, partner: String, ssName: String? = nil) -> Exercise {
        Exercise(name: name, sets: sets, repsTarget: reps, isSuperset: true, supersetWith: partner, supersetName: ssName)
    }

    static func finisher(_ name: String, sets: Int, reps: String) -> Exercise {
        Exercise(name: name, sets: sets, repsTarget: reps, isBodyweight: true)
    }

    static func timedFinisher(_ name: String, sets: Int, duration: Int) -> Exercise {
        Exercise(name: name, sets: sets, repsTarget: "\(duration) sek", isBodyweight: true, durationSecs: duration)
    }

    static func fromWger(name: String, sets: Int, reps: String, primaryMuscles: [String], secondaryMuscles: [String], imageUrl: String?, equipment: String?, wgerId: Int) -> Exercise {
        Exercise(name: name, sets: sets, repsTarget: reps, primaryMuscles: primaryMuscles, secondaryMuscles: secondaryMuscles, imageUrl: imageUrl, equipment: equipment, wgerId: wgerId)
    }
}

struct SetRecord: Codable, Hashable {
    var weight: Double
    var reps: Int
    var timestamp: Int64
    var restBeforeSecs: Int64?

    enum CodingKeys: String, CodingKey {
        case weight, reps, timestamp
        case restBeforeSecs = "rest_before_secs"
    }
}

struct LastExerciseData: Codable, Hashable {
    var weight: Double
    var reps: Int
}
