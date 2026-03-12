import Foundation

@Observable
final class StorageService {
    static let shared = StorageService()

    private let defaults = UserDefaults.standard

    private init() {}

    // MARK: - Database

    func saveData(_ db: Database) {
        if let data = try? JSONEncoder().encode(db) {
            defaults.set(data, forKey: StorageKeys.database)
        }
    }

    func loadData() -> Database {
        guard let data = defaults.data(forKey: StorageKeys.database),
              let db = try? JSONDecoder().decode(Database.self, from: data)
        else { return Database() }
        return db
    }

    // MARK: - Auth Session

    func saveAuthSession(_ session: AuthSession) {
        if let data = try? JSONEncoder().encode(session) {
            defaults.set(data, forKey: StorageKeys.authSession)
        }
    }

    func loadAuthSession() -> AuthSession? {
        guard let data = defaults.data(forKey: StorageKeys.authSession),
              let session = try? JSONDecoder().decode(AuthSession.self, from: data)
        else { return nil }
        return session
    }

    func clearAuthSession() {
        defaults.removeObject(forKey: StorageKeys.authSession)
    }

    // MARK: - Paused Workout

    func savePausedWorkout(_ paused: PausedWorkout) {
        if let data = try? JSONEncoder().encode(paused) {
            defaults.set(data, forKey: StorageKeys.pausedWorkout)
        }
    }

    func loadPausedWorkout() -> PausedWorkout? {
        guard let data = defaults.data(forKey: StorageKeys.pausedWorkout),
              let paused = try? JSONDecoder().decode(PausedWorkout.self, from: data)
        else { return nil }
        return paused
    }

    func clearPausedWorkout() {
        defaults.removeObject(forKey: StorageKeys.pausedWorkout)
    }

    // MARK: - Active Routine

    func saveActiveRoutine(_ routine: SavedRoutine) {
        if let data = try? JSONEncoder().encode(routine) {
            defaults.set(data, forKey: StorageKeys.activeRoutine)
        }
    }

    func loadActiveRoutine() -> SavedRoutine? {
        guard let data = defaults.data(forKey: StorageKeys.activeRoutine),
              let routine = try? JSONDecoder().decode(SavedRoutine.self, from: data)
        else { return nil }
        return routine
    }

    func clearActiveRoutine() {
        defaults.removeObject(forKey: StorageKeys.activeRoutine)
    }

    // MARK: - Sync Status

    func getSyncStatus() -> String {
        defaults.string(forKey: StorageKeys.syncStatus) ?? "pending"
    }

    func markSyncSuccess() {
        defaults.set("success", forKey: StorageKeys.syncStatus)
        incrementDataVersion()
    }

    func markSyncFailed() {
        defaults.set("failed", forKey: StorageKeys.syncStatus)
    }

    func resetSyncStatus() {
        defaults.set("pending", forKey: StorageKeys.syncStatus)
    }

    var isSyncComplete: Bool {
        getSyncStatus() == "success"
    }

    // MARK: - Data Version

    func getDataVersion() -> Int {
        defaults.integer(forKey: StorageKeys.dataVersion)
    }

    func incrementDataVersion() {
        defaults.set(getDataVersion() + 1, forKey: StorageKeys.dataVersion)
    }

    // MARK: - Display Name

    func loadDisplayName() -> String? {
        defaults.string(forKey: StorageKeys.displayName)
    }

    func saveDisplayName(_ name: String) {
        if name.isEmpty {
            defaults.removeObject(forKey: StorageKeys.displayName)
        } else {
            defaults.set(name, forKey: StorageKeys.displayName)
        }
    }

    // MARK: - Last Activity (inactivity timeout)

    func updateLastActivity() {
        defaults.set(currentTimestamp(), forKey: StorageKeys.lastActivity)
    }

    func clearLastActivity() {
        defaults.removeObject(forKey: StorageKeys.lastActivity)
    }

    func isSessionExpired() -> Bool {
        guard loadAuthSession() != nil else { return false }
        let lastActivity = Int64(defaults.integer(forKey: StorageKeys.lastActivity))
        if lastActivity == 0 { return false }
        return (currentTimestamp() - lastActivity) > SupabaseConfig.inactivityTimeoutSecs
    }

    // MARK: - Sync Failed

    func setSyncFailed(_ sessionId: String) {
        defaults.set(sessionId, forKey: StorageKeys.syncFailed)
    }

    func getSyncFailedSession() -> String? {
        defaults.string(forKey: StorageKeys.syncFailed)
    }

    func clearSyncFailed() {
        defaults.removeObject(forKey: StorageKeys.syncFailed)
    }

    // MARK: - Workout helpers

    func getWorkout(passName: String) -> WorkoutData? {
        let db = loadData()
        guard let savedRoutine = loadActiveRoutine() else { return nil }

        guard let pass = savedRoutine.passes.first(where: { $0.name == passName }) else {
            return nil
        }

        let routine = WorkoutRoutine(
            name: pass.name,
            focus: savedRoutine.focus,
            exercises: pass.exercises,
            finishers: pass.finishers
        )

        return createWorkoutData(routine: routine, db: db)
    }

    private func createWorkoutData(routine: WorkoutRoutine, db: Database) -> WorkoutData {
        let allExercises = routine.exercises + routine.finishers
        let exercises = allExercises.map { ex -> ExerciseWorkoutState in
            let lastData = db.getLastExerciseData(ex.name)
            let currentWeight: Double = if ex.isBodyweight {
                0.0
            } else {
                lastData?.weight ?? 20.0
            }
            return ExerciseWorkoutState(
                exercise: ex,
                lastData: lastData,
                currentWeight: currentWeight,
                setsCompleted: []
            )
        }
        return WorkoutData(routine: routine, exercises: exercises)
    }

    func saveSession(routineName: String, exercises: [ExerciseRecord], durationSecs: Int64) async {
        var db = loadData()

        let totalVolume = exercises.flatMap(\.sets).reduce(0.0) { $0 + $1.weight * Double($1.reps) }

        let session = Session(
            id: generateId(),
            routine: routineName,
            timestamp: currentTimestamp(),
            durationSecs: durationSecs,
            exercises: exercises,
            totalVolume: totalVolume
        )

        // Save locally immediately
        db.addSession(session)
        saveData(db)

        // Push to cloud and wait for completion
        for ex in session.exercises {
            if let lastSet = ex.sets.last {
                await SupabaseService.shared.saveWeightToCloud(
                    exerciseName: ex.name, weight: lastSet.weight, reps: lastSet.reps
                )
            }
        }
        updateLastActivity()
        await SupabaseService.shared.saveSessionToCloud(session)
    }

}
