import Foundation

@Observable
final class DashboardViewModel {
    var isLoading = true
    var activeRoutine: SavedRoutine?
    var totalStats = TotalStats()
    var recentSessions: [Session] = []
    var pausedWorkout: PausedWorkout?
    var displayName: String = ""
    var showConfirmDialog = false
    var pendingPassName = ""
    var syncStatus = "pending"

    func loadData() async {
        isLoading = true
        syncStatus = "pending"

        // Start sync
        StorageService.shared.resetSyncStatus()
        await SyncService.shared.syncFromCloud()
        syncStatus = StorageService.shared.getSyncStatus()

        // HealthKit authorization + bodyweight sync
        await HealthKitService.shared.requestAuthorization()
        if let hkData = await HealthKitService.shared.fetchLatestBodyweight() {
            var db = StorageService.shared.loadData()
            let localTimestamp = db.bodyweightHistory.last?.timestamp ?? 0
            let hkTimestamp = Int64(hkData.date.timeIntervalSince1970)
            if hkTimestamp > localTimestamp {
                db.setBodyweight(hkData.weight)
                StorageService.shared.saveData(db)
            }
        }

        // Load local data
        let db = StorageService.shared.loadData()
        totalStats = db.getTotalStats()
        recentSessions = db.getRecentSessions(limit: 5)
        pausedWorkout = StorageService.shared.loadPausedWorkout()
        displayName = StorageService.shared.loadDisplayName() ?? ""

        // Load active routine — always refresh from cloud, fall back to cache
        if let routine = try? await SupabaseService.shared.getActiveRoutine() {
            activeRoutine = routine
            StorageService.shared.saveActiveRoutine(routine)
        } else if let routine = StorageService.shared.loadActiveRoutine() {
            activeRoutine = routine
        } else {
            let defaultRoutine = StorageService.shared.createDefaultRoutine()
            activeRoutine = defaultRoutine
            StorageService.shared.saveActiveRoutine(defaultRoutine)
        }

        isLoading = false
    }

    func startWorkout(passName: String) -> String? {
        if pausedWorkout != nil && pausedWorkout?.routineName != passName {
            pendingPassName = passName
            showConfirmDialog = true
            return nil
        }
        return passName
    }

    func confirmStartNew() -> String {
        StorageService.shared.clearPausedWorkout()
        pausedWorkout = nil
        showConfirmDialog = false
        return pendingPassName
    }

    func resumeWorkout() -> String? {
        pausedWorkout?.routineName
    }
}
