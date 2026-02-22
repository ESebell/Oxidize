import Foundation

enum ProgressStatus {
    case improved
    case maintained
    case regressed
    case firstTime
}

enum MuscleGroup: String, CaseIterable, Hashable {
    case chest, back, shoulders, biceps, triceps
    case quads, hamstrings, glutes, calves, core

    var displayName: String {
        switch self {
        case .chest: "Bröst"
        case .back: "Rygg"
        case .shoulders: "Axlar"
        case .biceps: "Biceps"
        case .triceps: "Triceps"
        case .quads: "Lår (fram)"
        case .hamstrings: "Lår (bak)"
        case .glutes: "Rumpa"
        case .calves: "Vader"
        case .core: "Mage"
        }
    }
}

struct StatsSummary {
    var powerScore: Double = 0
    var bodyweight: Double = 0
    var totalSessions: Int = 0
    var weeklySets: [MuscleGroup: Int] = [:]
    var e1rmByExercise: [String: Double] = [:]
}
