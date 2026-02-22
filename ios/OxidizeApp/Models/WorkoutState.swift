import Foundation

struct ExerciseWorkoutState: Codable, Hashable, Identifiable {
    var id = UUID()
    var exercise: Exercise
    var lastData: LastExerciseData?
    var currentWeight: Double
    var setsCompleted: [SetRecord]

    enum CodingKeys: String, CodingKey {
        case exercise
        case lastData = "last_data"
        case currentWeight = "current_weight"
        case setsCompleted = "sets_completed"
    }
}

struct WorkoutData: Codable {
    var routine: WorkoutRoutine
    var exercises: [ExerciseWorkoutState]
}

struct PausedWorkout: Codable {
    var routineName: String
    var exercises: [ExerciseWorkoutState]
    var currentExerciseIdx: Int
    var startTimestamp: Int64
    var elapsedSecs: Int64

    enum CodingKeys: String, CodingKey {
        case exercises
        case routineName = "routine_name"
        case currentExerciseIdx = "current_exercise_idx"
        case startTimestamp = "start_timestamp"
        case elapsedSecs = "elapsed_secs"
    }
}
