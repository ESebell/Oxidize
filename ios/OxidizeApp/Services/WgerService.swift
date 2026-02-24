import Foundation

enum WgerService {
    // Wger muscle ID → English name mapping
    // Source: https://wger.de/api/v2/muscle/?format=json
    private static let muscleNames: [Int: String] = [
        1: "Biceps brachii",
        2: "Anterior deltoid",
        3: "Serratus anterior",
        4: "Pectoralis major",
        5: "Obliquus externus abdominis",
        6: "Rectus abdominis",
        7: "Gastrocnemius",
        8: "Gluteus maximus",
        9: "Trapezius",
        10: "Quadriceps femoris",
        11: "Biceps femoris",
        12: "Latissimus dorsi",
        13: "Brachialis",
        14: "Obliquus externus abdominis",
        15: "Soleus",
    ]

    static func muscleName(for id: Int) -> String {
        muscleNames[id] ?? "Unknown"
    }

    // Wger saknar erector spinae — override för övningar som tränar nedre ryggen
    private static let muscleOverrides: [String: (primary: [String], secondary: [String])] = [
        "hyperextensions": (["Erector spinae"], ["Gluteus maximus", "Biceps femoris"]),
        "back extension": (["Erector spinae"], ["Gluteus maximus"]),
        "lower back extensions": (["Erector spinae"], []),
        "good mornings": (["Erector spinae", "Biceps femoris"], ["Gluteus maximus"]),
        "good morning": (["Erector spinae", "Biceps femoris"], ["Gluteus maximus"]),
    ]

    /// Apply overrides for exercises Wger maps incorrectly
    static func applyOverrides(name: String, primary: [String], secondary: [String]) -> (primary: [String], secondary: [String]) {
        if let override = muscleOverrides[name.lowercased()] {
            return override
        }
        return (primary, secondary)
    }

    static func searchExercises(query: String) async throws -> [WgerExercise] {
        guard !query.isEmpty else { return [] }

        let encoded = query.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? query
        let url = URL(string: "https://wger.de/api/v2/exercise/search/?language=2&term=\(encoded)")!

        let (data, _) = try await URLSession.shared.data(from: url)
        let response = try JSONDecoder().decode(WgerSearchResponse.self, from: data)

        let suggestions = Array(response.suggestions.prefix(10))

        // Fetch exerciseinfo for each result concurrently to get actual muscles
        return await withTaskGroup(of: WgerExercise?.self) { group in
            for suggestion in suggestions {
                let d = suggestion.data
                group.addTask {
                    var imageUrl = d.image
                    if let img = imageUrl, !img.hasPrefix("http") {
                        imageUrl = "https://wger.de\(img)"
                    }

                    // Fetch muscles from exerciseinfo + apply overrides
                    let raw = await fetchMuscles(baseId: d.baseId)
                    let (primary, secondary) = applyOverrides(name: d.name, primary: raw.primary, secondary: raw.secondary)

                    return WgerExercise(
                        id: d.id,
                        baseId: d.baseId,
                        name: d.name,
                        primaryMuscles: primary,
                        secondaryMuscles: secondary,
                        imageUrl: imageUrl,
                        equipment: nil
                    )
                }
            }

            var results: [WgerExercise] = []
            for await exercise in group {
                if let ex = exercise { results.append(ex) }
            }
            return results
        }
    }

    /// Fetch muscle data from exerciseinfo endpoint using base_id
    static func fetchMuscles(baseId: Int) async -> (primary: [String], secondary: [String]) {
        do {
            let url = URL(string: "https://wger.de/api/v2/exerciseinfo/\(baseId)/?format=json")!
            let (data, _) = try await URLSession.shared.data(from: url)
            let info = try JSONDecoder().decode(WgerExerciseInfo.self, from: data)

            let primary = info.muscles.map { muscleName(for: $0.id) }
            let secondary = info.musclesSecondary.map { muscleName(for: $0.id) }
            return (primary, secondary)
        } catch {
            return ([], [])
        }
    }

    /// Search for an exercise by name and return its muscles (single exerciseinfo call)
    /// Used for enrichment — lightweight alternative to searchExercises
    static func lookupMuscles(exerciseName: String) async -> (primary: [String], secondary: [String])? {
        guard !exerciseName.isEmpty else { return nil }
        let encoded = exerciseName.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? exerciseName
        let url = URL(string: "https://wger.de/api/v2/exercise/search/?language=2&term=\(encoded)")!

        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            let response = try JSONDecoder().decode(WgerSearchResponse.self, from: data)

            // Find exact name match, or fall back to first result
            guard let match = response.suggestions.first(where: { $0.data.name == exerciseName })
                    ?? response.suggestions.first else { return nil }

            let raw = await fetchMuscles(baseId: match.data.baseId)
            let (primary, secondary) = applyOverrides(name: match.data.name, primary: raw.primary, secondary: raw.secondary)
            return primary.isEmpty ? nil : (primary, secondary)
        } catch {
            return nil
        }
    }
}
