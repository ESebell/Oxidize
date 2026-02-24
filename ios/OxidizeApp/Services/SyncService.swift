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

        // Enrich sessions with proper muscle data from Wger exerciseinfo
        // Force re-enrich if muscle data version is outdated (v2 = exerciseinfo-based)
        let muscleDataVersion = UserDefaults.standard.integer(forKey: "muscle_data_version")
        let needsFullReenrich = muscleDataVersion < 2

        let genericCategories: Set<String> = [
            "Armar", "Ben", "Bröst", "Axlar", "Rygg", "Vader", "Magmuskler",
            "Arms", "Legs", "Chest", "Shoulders", "Back", "Calves", "Abs",
        ]
        var needsEnrichment: Set<String> = []
        for session in db.sessions {
            for rec in session.exercises {
                if needsFullReenrich {
                    needsEnrichment.insert(rec.name)
                } else {
                    let isGeneric = rec.primaryMuscles.isEmpty ||
                        rec.primaryMuscles.allSatisfy { genericCategories.contains($0) }
                    if isGeneric { needsEnrichment.insert(rec.name) }
                }
            }
        }

        // Fetch muscles from Wger for each unique exercise (search by name → exerciseinfo)
        var muscleLookup: [String: (primary: [String], secondary: [String])] = [:]
        if !needsEnrichment.isEmpty {
            await withTaskGroup(of: (String, [String], [String])?.self) { group in
                for name in needsEnrichment {
                    group.addTask {
                        if let muscles = await WgerService.lookupMuscles(exerciseName: name) {
                            return (name, muscles.primary, muscles.secondary)
                        }
                        return nil
                    }
                }
                for await result in group {
                    if let (name, primary, secondary) = result {
                        muscleLookup[name] = (primary, secondary)
                    }
                }
            }
        }

        // Apply to sessions
        var enriched: [Session] = []
        for i in db.sessions.indices {
            var changed = false
            for j in db.sessions[i].exercises.indices {
                let rec = db.sessions[i].exercises[j]
                if let muscles = muscleLookup[rec.name] {
                    db.sessions[i].exercises[j].primaryMuscles = muscles.primary
                    db.sessions[i].exercises[j].secondaryMuscles = muscles.secondary
                    changed = true
                }
            }
            if changed { enriched.append(db.sessions[i]) }
        }

        StorageService.shared.saveData(db)

        for session in enriched {
            try? await SupabaseService.shared.upsertSession(session)
        }

        if !enriched.isEmpty || needsEnrichment.isEmpty {
            UserDefaults.standard.set(2, forKey: "muscle_data_version")
        }

        print("SYNC COMPLETE: \(db.sessions.count) sessions, enriched \(enriched.count)")
    }
}
