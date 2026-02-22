import Foundation
import Supabase

@Observable
final class SupabaseService {
    static let shared = SupabaseService()

    let client: SupabaseClient

    private init() {
        client = SupabaseClient(
            supabaseURL: URL(string: SupabaseConfig.url)!,
            supabaseKey: SupabaseConfig.anonKey
        )
    }

    // MARK: - Auth

    func signUp(email: String, password: String) async throws -> AuthSession {
        let response = try await client.auth.signUp(email: email, password: password)
        let session = authSessionFromSupabase(response.session)
        StorageService.shared.saveAuthSession(session)
        StorageService.shared.updateLastActivity()
        return session
    }

    func signIn(email: String, password: String) async throws -> AuthSession {
        let response = try await client.auth.signIn(email: email, password: password)
        let session = authSessionFromSupabase(response)
        StorageService.shared.saveAuthSession(session)
        StorageService.shared.updateLastActivity()
        return session
    }

    func signOut() async {
        try? await client.auth.signOut()
        StorageService.shared.clearAuthSession()
        StorageService.shared.clearLastActivity()
    }

    func refreshSession() async throws {
        let response = try await client.auth.refreshSession()
        let session = authSessionFromSupabase(response)
        StorageService.shared.saveAuthSession(session)
    }

    func checkAndRefreshSession() async {
        guard StorageService.shared.loadAuthSession() != nil else { return }

        if StorageService.shared.isSessionExpired() {
            print("Session expired due to inactivity (4h), signing out")
            await signOut()
            return
        }

        StorageService.shared.updateLastActivity()

        do {
            try await refreshSession()
        } catch {
            print("Token refresh failed: \(error)")
        }
    }

    var currentUserId: String? {
        StorageService.shared.loadAuthSession()?.user.id
    }

    private func authSessionFromSupabase(_ session: Supabase.Session?) -> AuthSession {
        guard let session else {
            return AuthSession(accessToken: "", refreshToken: nil, user: AuthUser(id: "", email: ""))
        }
        let displayName = StorageService.shared.loadDisplayName()
        return AuthSession(
            accessToken: session.accessToken,
            refreshToken: session.refreshToken,
            user: AuthUser(
                id: session.user.id.uuidString,
                email: session.user.email ?? "",
                displayName: displayName
            )
        )
    }

    private func authSessionFromSupabase(_ session: Supabase.Session) -> AuthSession {
        let displayName = StorageService.shared.loadDisplayName()
        return AuthSession(
            accessToken: session.accessToken,
            refreshToken: session.refreshToken,
            user: AuthUser(
                id: session.user.id.uuidString,
                email: session.user.email ?? "",
                displayName: displayName
            )
        )
    }

    // MARK: - Headers helper for REST calls

    private func authHeaders() -> [String: String] {
        var headers = [
            "apikey": SupabaseConfig.anonKey,
            "Content-Type": "application/json"
        ]
        if let session = StorageService.shared.loadAuthSession() {
            headers["Authorization"] = "Bearer \(session.accessToken)"
        } else {
            headers["Authorization"] = "Bearer \(SupabaseConfig.anonKey)"
        }
        return headers
    }

    // MARK: - Sessions

    private struct SessionRow: Codable {
        var id: String
        var routine: String
        var timestamp: Int64
        var durationSecs: Int64
        var totalVolume: Double
        var exercises: String // JSON string
        var userId: String?

        enum CodingKeys: String, CodingKey {
            case id, routine, timestamp, exercises
            case durationSecs = "duration_secs"
            case totalVolume = "total_volume"
            case userId = "user_id"
        }
    }

    func upsertSession(_ session: Session) async throws {
        guard let userId = currentUserId else { throw OxidizeError.notLoggedIn }

        let exercisesJson = try JSONEncoder().encode(session.exercises)
        let exercisesString = String(data: exercisesJson, encoding: .utf8) ?? "[]"

        let row = SessionRow(
            id: session.id,
            routine: session.routine,
            timestamp: session.timestamp,
            durationSecs: session.durationSecs,
            totalVolume: session.totalVolume,
            exercises: exercisesString,
            userId: userId
        )

        var request = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/sessions")!)
        request.httpMethod = "POST"
        for (key, value) in authHeaders() {
            request.setValue(value, forHTTPHeaderField: key)
        }
        request.setValue("resolution=merge-duplicates", forHTTPHeaderField: "Prefer")

        // Build JSON manually to keep exercises as raw JSON
        let body: [String: Any] = [
            "id": session.id,
            "routine": session.routine,
            "timestamp": session.timestamp,
            "duration_secs": session.durationSecs,
            "total_volume": session.totalVolume,
            "exercises": try JSONSerialization.jsonObject(with: exercisesJson),
            "user_id": userId
        ]
        request.httpBody = try JSONSerialization.data(withJSONObject: body)
        request.timeoutInterval = 5

        let (_, response) = try await URLSession.shared.data(for: request)
        guard let httpResponse = response as? HTTPURLResponse, (200...299).contains(httpResponse.statusCode) else {
            let statusCode = (response as? HTTPURLResponse)?.statusCode ?? 0
            throw OxidizeError.httpError(statusCode)
        }
    }

    func saveSessionToCloud(_ session: Session) async {
        guard currentUserId != nil else { return }

        for attempt in 1...3 {
            do {
                try await upsertSession(session)
                print("Session \(session.id) saved to cloud")
                StorageService.shared.clearSyncFailed()
                return
            } catch {
                print("Attempt \(attempt) failed: \(error)")
                if attempt < 3 {
                    try? await Task.sleep(for: .seconds(attempt))
                }
            }
        }

        print("Session \(session.id) save FAILED after 3 retries")
        StorageService.shared.setSyncFailed(session.id)
    }

    func fetchSessions() async throws -> [Session] {
        guard let userId = currentUserId else { throw OxidizeError.notLoggedIn }

        var request = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/sessions?select=*&user_id=eq.\(userId)")!)
        request.httpMethod = "GET"
        for (key, value) in authHeaders() {
            request.setValue(value, forHTTPHeaderField: key)
        }

        let (data, response) = try await URLSession.shared.data(for: request)
        guard let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 else {
            throw OxidizeError.httpError((response as? HTTPURLResponse)?.statusCode ?? 0)
        }

        let rows = try JSONSerialization.jsonObject(with: data) as? [[String: Any]] ?? []
        return rows.compactMap { row -> Session? in
            guard let id = row["id"] as? String,
                  let routine = row["routine"] as? String,
                  let timestamp = row["timestamp"] as? Int64,
                  let durationSecs = row["duration_secs"] as? Int64,
                  let totalVolume = row["total_volume"] as? Double,
                  let exercisesRaw = row["exercises"]
            else { return nil }

            let exercisesData = try? JSONSerialization.data(withJSONObject: exercisesRaw)
            let exercises = exercisesData.flatMap { try? JSONDecoder().decode([ExerciseRecord].self, from: $0) } ?? []

            return Session(
                id: id,
                routine: routine,
                timestamp: timestamp,
                durationSecs: durationSecs,
                exercises: exercises,
                totalVolume: totalVolume
            )
        }
    }

    // MARK: - Last Weights

    private struct LastWeightRow: Codable {
        var exerciseName: String
        var weight: Double
        var reps: Int
        var userId: String?

        enum CodingKeys: String, CodingKey {
            case weight, reps
            case exerciseName = "exercise_name"
            case userId = "user_id"
        }
    }

    func saveWeightToCloud(exerciseName: String, weight: Double, reps: Int) async {
        guard let userId = currentUserId else { return }

        do {
            var request = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/last_weights")!)
            request.httpMethod = "POST"
            for (key, value) in authHeaders() {
                request.setValue(value, forHTTPHeaderField: key)
            }
            request.setValue("resolution=merge-duplicates", forHTTPHeaderField: "Prefer")

            let body: [String: Any] = [
                "exercise_name": exerciseName,
                "weight": weight,
                "reps": reps,
                "user_id": userId
            ]
            request.httpBody = try JSONSerialization.data(withJSONObject: body)

            let (_, _) = try await URLSession.shared.data(for: request)
        } catch {
            print("Weight save failed: \(error)")
        }
    }

    func fetchLastWeights() async throws -> [String: LastExerciseData] {
        guard let userId = currentUserId else { throw OxidizeError.notLoggedIn }

        var request = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/last_weights?select=*&user_id=eq.\(userId)")!)
        request.httpMethod = "GET"
        for (key, value) in authHeaders() {
            request.setValue(value, forHTTPHeaderField: key)
        }

        let (data, _) = try await URLSession.shared.data(for: request)
        let rows = try JSONDecoder().decode([LastWeightRow].self, from: data)

        var map: [String: LastExerciseData] = [:]
        for row in rows {
            map[row.exerciseName] = LastExerciseData(weight: row.weight, reps: row.reps)
        }
        return map
    }

    // MARK: - Bodyweight

    private struct BodyweightRow: Codable {
        var id: Int?
        var weight: Double
        var timestamp: Int64
        var userId: String?

        enum CodingKeys: String, CodingKey {
            case id, weight, timestamp
            case userId = "user_id"
        }
    }

    private struct UserSettingsRow: Codable {
        var userId: String?
        var displayName: String?
        var bodyweight: Double?

        enum CodingKeys: String, CodingKey {
            case userId = "user_id"
            case displayName = "display_name"
            case bodyweight
        }
    }

    func saveBodyweightToCloud(_ weight: Double) async {
        guard let userId = currentUserId else { return }
        StorageService.shared.updateLastActivity()

        do {
            // 1. Save to bodyweight history table
            var historyRequest = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/bodyweight")!)
            historyRequest.httpMethod = "POST"
            for (key, value) in authHeaders() {
                historyRequest.setValue(value, forHTTPHeaderField: key)
            }
            let historyBody: [String: Any] = [
                "weight": weight,
                "timestamp": currentTimestamp(),
                "user_id": userId
            ]
            historyRequest.httpBody = try JSONSerialization.data(withJSONObject: historyBody)
            let (_, _) = try await URLSession.shared.data(for: historyRequest)

            // 2. Save to user_settings for current weight
            var settingsRequest = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/user_settings")!)
            settingsRequest.httpMethod = "POST"
            for (key, value) in authHeaders() {
                settingsRequest.setValue(value, forHTTPHeaderField: key)
            }
            settingsRequest.setValue("resolution=merge-duplicates", forHTTPHeaderField: "Prefer")
            let settingsBody: [String: Any] = [
                "user_id": userId,
                "bodyweight": weight
            ]
            settingsRequest.httpBody = try JSONSerialization.data(withJSONObject: settingsBody)
            let (_, _) = try await URLSession.shared.data(for: settingsRequest)
        } catch {
            print("Bodyweight save failed: \(error)")
        }
    }

    func fetchBodyweight() async throws -> (Double?, [BodyweightEntry]) {
        guard let userId = currentUserId else { throw OxidizeError.notLoggedIn }

        // 1. Fetch current weight from user_settings
        var settingsRequest = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/user_settings?user_id=eq.\(userId)&select=user_id,bodyweight")!)
        settingsRequest.httpMethod = "GET"
        for (key, value) in authHeaders() {
            settingsRequest.setValue(value, forHTTPHeaderField: key)
        }

        let (settingsData, _) = try await URLSession.shared.data(for: settingsRequest)
        let settingsRows = try JSONDecoder().decode([UserSettingsRow].self, from: settingsData)
        var currentWeight = settingsRows.first?.bodyweight

        // 2. Fetch history
        var historyRequest = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/bodyweight?select=*&user_id=eq.\(userId)&order=timestamp.desc")!)
        historyRequest.httpMethod = "GET"
        for (key, value) in authHeaders() {
            historyRequest.setValue(value, forHTTPHeaderField: key)
        }

        let (historyData, _) = try await URLSession.shared.data(for: historyRequest)
        let historyRows = try JSONDecoder().decode([BodyweightRow].self, from: historyData)
        let history = historyRows.map { BodyweightEntry(timestamp: $0.timestamp, weight: $0.weight) }

        if currentWeight == nil {
            currentWeight = history.first?.weight
        }

        return (currentWeight, history)
    }

    // MARK: - Routines

    private struct RoutineRow: Codable {
        var id: String
        var userId: String?
        var name: String
        var focus: String
        var passes: String // JSON
        var isActive: Bool
        var createdAt: Int64

        enum CodingKeys: String, CodingKey {
            case id, name, focus, passes
            case userId = "user_id"
            case isActive = "is_active"
            case createdAt = "created_at"
        }
    }

    func fetchRoutines() async throws -> [SavedRoutine] {
        guard let userId = currentUserId else { return [] }

        var request = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/routines?user_id=eq.\(userId)&order=created_at.desc")!)
        request.httpMethod = "GET"
        for (key, value) in authHeaders() {
            request.setValue(value, forHTTPHeaderField: key)
        }

        let (data, _) = try await URLSession.shared.data(for: request)
        let rows = try JSONSerialization.jsonObject(with: data) as? [[String: Any]] ?? []

        return rows.compactMap { row -> SavedRoutine? in
            guard let id = row["id"] as? String,
                  let name = row["name"] as? String,
                  let focus = row["focus"] as? String,
                  let isActive = row["is_active"] as? Bool,
                  let createdAt = row["created_at"] as? Int64,
                  let passesRaw = row["passes"]
            else { return nil }

            let passesData = try? JSONSerialization.data(withJSONObject: passesRaw)
            let passes = passesData.flatMap { try? JSONDecoder().decode([Pass].self, from: $0) } ?? []

            return SavedRoutine(
                id: id,
                userId: row["user_id"] as? String,
                name: name,
                focus: focus,
                passes: passes,
                isActive: isActive,
                createdAt: createdAt
            )
        }
    }

    func getActiveRoutine() async throws -> SavedRoutine? {
        let routines = try await fetchRoutines()
        return routines.first { $0.isActive }
    }

    func saveRoutine(_ routine: SavedRoutine) async throws {
        guard let userId = currentUserId else { throw OxidizeError.notLoggedIn }

        let passesData = try JSONEncoder().encode(routine.passes)

        var request = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/routines")!)
        request.httpMethod = "POST"
        for (key, value) in authHeaders() {
            request.setValue(value, forHTTPHeaderField: key)
        }
        request.setValue("resolution=merge-duplicates", forHTTPHeaderField: "Prefer")

        let body: [String: Any] = [
            "id": routine.id,
            "user_id": userId,
            "name": routine.name,
            "focus": routine.focus,
            "passes": try JSONSerialization.jsonObject(with: passesData),
            "is_active": routine.isActive,
            "created_at": routine.createdAt
        ]
        request.httpBody = try JSONSerialization.data(withJSONObject: body)

        let (_, response) = try await URLSession.shared.data(for: request)
        guard let httpResponse = response as? HTTPURLResponse, (200...299).contains(httpResponse.statusCode) else {
            throw OxidizeError.httpError((response as? HTTPURLResponse)?.statusCode ?? 0)
        }
    }

    func setActiveRoutine(_ routineId: String) async throws {
        guard let userId = currentUserId else { throw OxidizeError.notLoggedIn }

        // Deactivate all
        var deactivateRequest = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/routines?user_id=eq.\(userId)")!)
        deactivateRequest.httpMethod = "PATCH"
        for (key, value) in authHeaders() {
            deactivateRequest.setValue(value, forHTTPHeaderField: key)
        }
        deactivateRequest.httpBody = try JSONSerialization.data(withJSONObject: ["is_active": false])
        let (_, _) = try await URLSession.shared.data(for: deactivateRequest)

        // Activate selected
        var activateRequest = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/routines?id=eq.\(routineId)&user_id=eq.\(userId)")!)
        activateRequest.httpMethod = "PATCH"
        for (key, value) in authHeaders() {
            activateRequest.setValue(value, forHTTPHeaderField: key)
        }
        activateRequest.httpBody = try JSONSerialization.data(withJSONObject: ["is_active": true])
        let (_, _) = try await URLSession.shared.data(for: activateRequest)
    }

    func deleteRoutine(_ routineId: String) async throws {
        guard let userId = currentUserId else { throw OxidizeError.notLoggedIn }

        var request = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/routines?id=eq.\(routineId)&user_id=eq.\(userId)")!)
        request.httpMethod = "DELETE"
        for (key, value) in authHeaders() {
            request.setValue(value, forHTTPHeaderField: key)
        }

        let (_, response) = try await URLSession.shared.data(for: request)
        guard let httpResponse = response as? HTTPURLResponse, (200...299).contains(httpResponse.statusCode) else {
            throw OxidizeError.httpError((response as? HTTPURLResponse)?.statusCode ?? 0)
        }
    }

    // MARK: - Display Name

    func saveDisplayNameToCloud(_ name: String) async {
        guard let userId = currentUserId else { return }
        StorageService.shared.updateLastActivity()

        do {
            var request = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/user_settings")!)
            request.httpMethod = "POST"
            for (key, value) in authHeaders() {
                request.setValue(value, forHTTPHeaderField: key)
            }
            request.setValue("resolution=merge-duplicates", forHTTPHeaderField: "Prefer")

            let displayName = name.isEmpty ? " " : name
            let body: [String: Any] = [
                "user_id": userId,
                "display_name": displayName
            ]
            request.httpBody = try JSONSerialization.data(withJSONObject: body)
            let (_, _) = try await URLSession.shared.data(for: request)
        } catch {
            print("Display name save failed: \(error)")
        }
    }

    func fetchDisplayName() async -> String? {
        guard let userId = currentUserId else { return nil }

        do {
            var request = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/user_settings?user_id=eq.\(userId)&select=user_id,display_name")!)
            request.httpMethod = "GET"
            for (key, value) in authHeaders() {
                request.setValue(value, forHTTPHeaderField: key)
            }

            let (data, _) = try await URLSession.shared.data(for: request)
            let rows = try JSONDecoder().decode([UserSettingsRow].self, from: data)
            return rows.first?.displayName
        } catch {
            return nil
        }
    }

    // MARK: - API Key (for Gemini)

    private struct ConfigRow: Codable {
        var configValue: String

        enum CodingKeys: String, CodingKey {
            case configValue = "config_value"
        }
    }

    func fetchApiKey() async -> String? {
        do {
            var request = URLRequest(url: URL(string: "\(SupabaseConfig.url)/rest/v1/app_config?config_key=eq.gemini_api_key&select=config_value")!)
            request.httpMethod = "GET"
            for (key, value) in authHeaders() {
                request.setValue(value, forHTTPHeaderField: key)
            }

            let (data, response) = try await URLSession.shared.data(for: request)
            let httpResponse = response as? HTTPURLResponse
            print("[Gemini] fetchApiKey status: \(httpResponse?.statusCode ?? 0)")
            print("[Gemini] fetchApiKey body: \(String(data: data, encoding: .utf8) ?? "nil")")
            let rows = try JSONDecoder().decode([ConfigRow].self, from: data)
            print("[Gemini] fetchApiKey rows: \(rows.count)")
            return rows.first?.configValue
        } catch {
            print("[Gemini] fetchApiKey error: \(error)")
            return nil
        }
    }
}

enum OxidizeError: Error, LocalizedError {
    case notLoggedIn
    case httpError(Int)
    case networkError(String)

    var errorDescription: String? {
        switch self {
        case .notLoggedIn: "Inte inloggad"
        case .httpError(let code): "HTTP-fel: \(code)"
        case .networkError(let msg): msg
        }
    }
}
