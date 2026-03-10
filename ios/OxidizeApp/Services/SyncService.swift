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

        // One-time migration: enrich exercises missing muscle data
        let muscleDataVersion = UserDefaults.standard.integer(forKey: "muscle_data_version")
        if muscleDataVersion < 5 {
            let enrichedCount = await migrateAllMuscleData(db: &db)
            StorageService.shared.saveData(db)
            UserDefaults.standard.set(5, forKey: "muscle_data_version")
            print("MIGRATION: enriched \(enrichedCount) exercises")
        } else {
            StorageService.shared.saveData(db)
        }

        print("SYNC COMPLETE: \(db.sessions.count) sessions")
    }

    /// One-time migration: look up all exercises by exact name in Wger,
    /// fetch muscles via exerciseinfo ID, and save back to Supabase.
    private func migrateAllMuscleData(db: inout Database) async -> Int {
        // Collect all unique exercise names that need enrichment (from sessions + routines)
        var needsEnrichment: Set<String> = []

        for session in db.sessions {
            for rec in session.exercises {
                if rec.primaryMuscles.isEmpty {
                    needsEnrichment.insert(rec.name)
                }
            }
        }

        let routines = (try? await SupabaseService.shared.fetchRoutines()) ?? []
        for routine in routines {
            for pass in routine.passes {
                for ex in pass.exercises + pass.finishers {
                    if ex.primaryMuscles.isEmpty {
                        needsEnrichment.insert(ex.name)
                    }
                }
            }
        }

        guard !needsEnrichment.isEmpty else { return 0 }

        // Lookup via Wger: exact name → baseId → exerciseinfo
        var lookup: [String: (baseId: Int, primary: [String], secondary: [String])] = [:]
        await withTaskGroup(of: (String, Int, [String], [String])?.self) { group in
            for name in needsEnrichment {
                group.addTask {
                    guard let result = await WgerService.lookupByName(exerciseName: name) else { return nil }
                    return (name, result.baseId, result.primary, result.secondary)
                }
            }
            for await result in group {
                if let (name, baseId, primary, secondary) = result {
                    lookup[name] = (baseId, primary, secondary)
                }
            }
        }

        // Apply to sessions and save to Supabase
        for i in db.sessions.indices {
            var changed = false
            for j in db.sessions[i].exercises.indices {
                if let data = lookup[db.sessions[i].exercises[j].name] {
                    db.sessions[i].exercises[j].primaryMuscles = data.primary
                    db.sessions[i].exercises[j].secondaryMuscles = data.secondary
                    changed = true
                }
            }
            if changed {
                try? await SupabaseService.shared.upsertSession(db.sessions[i])
            }
        }

        // Apply to routines and save to Supabase
        for var routine in routines {
            var changed = false
            for pi in routine.passes.indices {
                for ei in routine.passes[pi].exercises.indices {
                    if let data = lookup[routine.passes[pi].exercises[ei].name] {
                        routine.passes[pi].exercises[ei].primaryMuscles = data.primary
                        routine.passes[pi].exercises[ei].secondaryMuscles = data.secondary
                        routine.passes[pi].exercises[ei].wgerId = data.baseId
                        changed = true
                    }
                }
                for ei in routine.passes[pi].finishers.indices {
                    if let data = lookup[routine.passes[pi].finishers[ei].name] {
                        routine.passes[pi].finishers[ei].primaryMuscles = data.primary
                        routine.passes[pi].finishers[ei].secondaryMuscles = data.secondary
                        routine.passes[pi].finishers[ei].wgerId = data.baseId
                        changed = true
                    }
                }
            }
            if changed {
                try? await SupabaseService.shared.saveRoutine(routine)
                if routine.isActive {
                    StorageService.shared.saveActiveRoutine(routine)
                }
            }
        }

        let unmatched = needsEnrichment.filter { lookup[$0] == nil }
        if !unmatched.isEmpty {
            print("UNMATCHED exercises: \(unmatched.sorted().joined(separator: ", "))")
        }

        return lookup.count
    }
}
