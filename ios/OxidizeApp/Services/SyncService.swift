import Foundation

@MainActor
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

        // One-time migration: re-enrich all exercises with correct Wger data
        let muscleDataVersion = UserDefaults.standard.integer(forKey: "muscle_data_version")
        print("MUSCLE_DATA_VERSION: \(muscleDataVersion)")
        if muscleDataVersion < 10 {
            let enrichedCount = await migrateAllMuscleData(db: &db)
            StorageService.shared.saveData(db)
            UserDefaults.standard.set(10, forKey: "muscle_data_version")
            print("MIGRATION: enriched \(enrichedCount) exercises")
        } else {
            StorageService.shared.saveData(db)
        }

        print("SYNC COMPLETE: \(db.sessions.count) sessions")
    }

    /// One-time migration: fetch muscles by wgerId (from routines), fall back to name override.
    private func migrateAllMuscleData(db: inout Database) async -> Int {
        let routines = (try? await SupabaseService.shared.fetchRoutines()) ?? []

        // Step 1: Collect wgerId → name mapping from routine exercises
        var wgerIds: [String: Int] = [:]  // exercise name → wgerId
        for routine in routines {
            for pass in routine.passes {
                for ex in pass.exercises + pass.finishers {
                    if let wid = ex.wgerId, wid > 0 {
                        wgerIds[ex.name] = wid
                    }
                }
            }
        }

        // Collect all unique exercise names from sessions + routines
        var allNames: Set<String> = []
        for session in db.sessions {
            for rec in session.exercises { allNames.insert(rec.name) }
        }
        for routine in routines {
            for pass in routine.passes {
                for ex in pass.exercises + pass.finishers { allNames.insert(ex.name) }
            }
        }
        guard !allNames.isEmpty else { return 0 }

        // Step 2: Fetch muscles by wgerId (reliable, ID-based)
        var lookup: [String: (baseId: Int, primary: [String], secondary: [String])] = [:]
        let namesWithId = allNames.filter { wgerIds[$0] != nil }
        let namesWithoutId = allNames.filter { wgerIds[$0] == nil }

        await withTaskGroup(of: (String, Int, [String], [String])?.self) { group in
            for name in namesWithId {
                let baseId = wgerIds[name]!
                group.addTask {
                    let raw = await WgerService.fetchMuscles(baseId: baseId)
                    // Trust Wger data when we have ID — only supplement erector spinae (missing from Wger)
                    let primary = raw.primary.isEmpty ? [] : raw.primary
                    let secondary = raw.secondary
                    let (finalPrimary, finalSecondary) = WgerService.supplementErectorSpinae(name: name, primary: primary, secondary: secondary)
                    guard !finalPrimary.isEmpty else { return nil }
                    return (name, baseId, finalPrimary, finalSecondary)
                }
            }
            for await result in group {
                if let (name, baseId, primary, secondary) = result {
                    lookup[name] = (baseId, primary, secondary)
                }
            }
        }

        // Step 3: Fallback for exercises without wgerId (overrides + name search)
        for name in namesWithoutId {
            if let result = await WgerService.lookupByName(exerciseName: name) {
                lookup[name] = result
            }
        }

        // Apply to sessions
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

        // Apply to routines
        for var routine in routines {
            var changed = false
            for pi in routine.passes.indices {
                for ei in routine.passes[pi].exercises.indices {
                    if let data = lookup[routine.passes[pi].exercises[ei].name] {
                        routine.passes[pi].exercises[ei].primaryMuscles = data.primary
                        routine.passes[pi].exercises[ei].secondaryMuscles = data.secondary
                        if routine.passes[pi].exercises[ei].wgerId == nil {
                            routine.passes[pi].exercises[ei].wgerId = data.baseId
                        }
                        changed = true
                    }
                }
                for ei in routine.passes[pi].finishers.indices {
                    if let data = lookup[routine.passes[pi].finishers[ei].name] {
                        routine.passes[pi].finishers[ei].primaryMuscles = data.primary
                        routine.passes[pi].finishers[ei].secondaryMuscles = data.secondary
                        if routine.passes[pi].finishers[ei].wgerId == nil {
                            routine.passes[pi].finishers[ei].wgerId = data.baseId
                        }
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

        let unmatched = allNames.filter { lookup[$0] == nil }
        if !unmatched.isEmpty {
            print("UNMATCHED exercises: \(unmatched.sorted().joined(separator: ", "))")
        }
        print("ID-based: \(namesWithId.count), name-fallback: \(namesWithoutId.count), matched: \(lookup.count)/\(allNames.count)")

        return lookup.count
    }
}
