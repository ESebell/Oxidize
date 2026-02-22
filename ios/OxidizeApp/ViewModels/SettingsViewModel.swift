import Foundation

@Observable
final class SettingsViewModel {
    var routines: [SavedRoutine] = []
    var isLoading = true
    var bodyweight: Double?
    var editingWeight = false
    var weightInput = ""
    var displayName = ""
    var editingName = false
    var nameInput = ""

    func loadSettings() async {
        isLoading = true

        let db = StorageService.shared.loadData()
        bodyweight = db.bodyweight
        displayName = StorageService.shared.loadDisplayName() ?? ""

        routines = (try? await SupabaseService.shared.fetchRoutines()) ?? []

        isLoading = false
    }

    func setActiveRoutine(id: String) async {
        do {
            try await SupabaseService.shared.setActiveRoutine(id)
            // Update local cache
            if let routine = routines.first(where: { $0.id == id }) {
                StorageService.shared.saveActiveRoutine(routine)
            }
            routines = (try? await SupabaseService.shared.fetchRoutines()) ?? routines
        } catch {
            print("Failed to set active routine: \(error)")
        }
    }

    func saveBodyweight() async {
        guard let weight = Double(weightInput.replacingOccurrences(of: ",", with: ".")) else { return }
        bodyweight = weight
        editingWeight = false

        var db = StorageService.shared.loadData()
        db.setBodyweight(weight)
        StorageService.shared.saveData(db)

        await SupabaseService.shared.saveBodyweightToCloud(weight)
        await HealthKitService.shared.saveBodyweight(weight)
    }

    func saveDisplayName() async {
        displayName = nameInput
        editingName = false
        StorageService.shared.saveDisplayName(displayName)

        // Update auth session
        if var session = StorageService.shared.loadAuthSession() {
            session.user.displayName = displayName
            StorageService.shared.saveAuthSession(session)
        }

        await SupabaseService.shared.saveDisplayNameToCloud(displayName)
    }
}
