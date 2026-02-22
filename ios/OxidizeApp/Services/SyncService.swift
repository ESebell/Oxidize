import Foundation

@Observable
final class SyncService {
    static let shared = SyncService()

    private init() {}

    func syncFromCloud() async {
        do {
            try await doSync()
            StorageService.shared.markSyncSuccess()
            print("Synced from Supabase")
        } catch {
            StorageService.shared.markSyncFailed()
            print("Sync failed: \(error)")
        }
    }

    private func doSync() async throws {
        guard let userId = SupabaseService.shared.currentUserId else {
            print("SYNC ABORTED: not logged in")
            return
        }
        print("SYNC START - User: \(userId)")

        // Fetch from cloud
        let cloudSessions = (try? await SupabaseService.shared.fetchSessions()) ?? []
        let cloudWeights = (try? await SupabaseService.shared.fetchLastWeights()) ?? [:]
        let (cloudBodyweight, cloudBwHistory) = (try? await SupabaseService.shared.fetchBodyweight()) ?? (nil, [])
        let cloudDisplayName = await SupabaseService.shared.fetchDisplayName()

        // Save display name if fetched
        if let name = cloudDisplayName {
            StorageService.shared.saveDisplayName(name)
            if var session = StorageService.shared.loadAuthSession() {
                session.user.displayName = name
                StorageService.shared.saveAuthSession(session)
            }
        }

        print("CLOUD: \(cloudSessions.count) sessions")

        // Cloud-first: replace local with cloud data
        var db = Database()
        db.sessions = cloudSessions.sorted { $0.timestamp > $1.timestamp }
        db.lastWeights = cloudWeights
        db.bodyweight = cloudBodyweight
        db.bodyweightHistory = cloudBwHistory

        StorageService.shared.saveData(db)
        print("SYNC COMPLETE: \(db.sessions.count) sessions")
    }
}
