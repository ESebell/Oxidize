import Foundation

enum WgerService {
    // Wger muscle ID → English name mapping
    // Source: https://wger.de/api/v2/muscle/?format=json
    private static let muscleNames: [Int: String] = [
        1: "Biceps brachii",
        2: "Anterior deltoid",
        3: "Serratus anterior",
        4: "Pectoralis major",
        5: "Triceps brachii",
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

    // Erector spinae overrides — Wger saknar denna muskel helt (inte i deras muskel-DB)
    // Namnvarianter från Wger-sökresultat inkluderade.
    private static let erectorSpinaeExercises: [String: (primary: [String], secondary: [String])] = [
        // Deadlifts (Wger: baseId 184 "Deadlifts")
        "deadlifts": (["Erector spinae", "Gluteus maximus"], ["Biceps femoris", "Quadriceps femoris"]),
        "deadlift": (["Erector spinae", "Gluteus maximus"], ["Biceps femoris", "Quadriceps femoris"]),
        // Romanian Deadlift (Wger: baseId 507, 1750, 1652, 1700)
        "romanian deadlift": (["Erector spinae", "Biceps femoris"], ["Gluteus maximus"]),
        "barbell romanian deadlift (rdl)": (["Erector spinae", "Biceps femoris"], ["Gluteus maximus"]),
        "dumbbell romanian deadlift": (["Erector spinae", "Biceps femoris"], ["Gluteus maximus"]),
        "dumbbell romanian deadlifts": (["Erector spinae", "Biceps femoris"], ["Gluteus maximus"]),
        "romanian deadlift, single leg": (["Erector spinae", "Biceps femoris"], ["Gluteus maximus"]),
        "single leg rdl": (["Erector spinae", "Biceps femoris"], ["Gluteus maximus"]),
        // Hyperextensions (Wger: baseId 301, 1809, 1348, 1143)
        "hyperextensions": (["Erector spinae"], ["Gluteus maximus", "Biceps femoris"]),
        "reverse hyperextension": (["Erector spinae"], ["Gluteus maximus", "Biceps femoris"]),
        "lower back extensions": (["Erector spinae"], []),
        "back extensión": (["Erector spinae"], ["Gluteus maximus"]),
    ]

    /// Supplement erector spinae for exercises where Wger lacks this muscle.
    /// Only replaces data for known erector spinae exercises; trusts Wger otherwise.
    static func supplementErectorSpinae(name: String, primary: [String], secondary: [String]) -> (primary: [String], secondary: [String]) {
        if let override = erectorSpinaeExercises[name.lowercased()] {
            return override
        }
        return (primary, secondary)
    }

    // Name-based fallback overrides — för övningar utan wgerId
    // Täcker: övningar som inte finns i Wger, svenska namn, och övningar
    // vars Wger-namn inte matchar exakt vid sökning
    private static let nameOverrides: [String: (primary: [String], secondary: [String])] = [
        // Övningar som inte finns i Wger
        "mountain climbers": (["Rectus abdominis"], ["Quadriceps femoris", "Anterior deltoid"]),
        "dead bug": (["Rectus abdominis"], ["Obliquus externus abdominis"]),
        "shoulder taps": (["Rectus abdominis"], ["Anterior deltoid", "Obliquus externus abdominis"]),
        // Övningar vars Wger-namn inte matchar vid sökning
        "facepulls": (["Trapezius"], ["Anterior deltoid"]),
        "face pulls": (["Trapezius"], ["Anterior deltoid"]),
        "hip thrusts": (["Gluteus maximus"], ["Biceps femoris"]),
        "hip thrust": (["Gluteus maximus"], ["Biceps femoris"]),
        "squats": (["Quadriceps femoris", "Gluteus maximus"], ["Biceps femoris"]),
        "lunges": (["Quadriceps femoris", "Gluteus maximus"], ["Biceps femoris"]),
        "seated cable row": (["Latissimus dorsi", "Trapezius"], ["Biceps brachii"]),
        "shoulder press": (["Anterior deltoid"], ["Triceps brachii"]),
        // Svenska övningsnamn
        "latsdrag": (["Latissimus dorsi"], ["Biceps brachii"]),
        "sidolyft": (["Anterior deltoid"], []),
        "sittande rodd": (["Latissimus dorsi", "Trapezius"], ["Biceps brachii"]),
        "sittande vadpress": (["Soleus"], ["Gastrocnemius"]),
        "stående vadpress": (["Gastrocnemius"], ["Soleus"]),
        "utfallssteg": (["Quadriceps femoris", "Gluteus maximus"], ["Biceps femoris"]),
    ]

    /// Apply name-based overrides — only used in fallback path (no wgerId available)
    static func applyOverrides(name: String, primary: [String], secondary: [String]) -> (primary: [String], secondary: [String]) {
        // Check erector spinae first
        if let override = erectorSpinaeExercises[name.lowercased()] {
            return override
        }
        // Then name overrides
        if let override = nameOverrides[name.lowercased()] {
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

    // Name aliases for exercises whose local name differs from Wger's name
    private static let nameAliases: [String: String] = [
        "hammercurls": "Hammer Curls",
        "squats": "Barbell Squat",
        "deadlift": "Deadlift, Barbell",
        "lunges": "Lunges",
        "leg curls": "Leg Curl",
        "seated cable row": "Seated Row",
    ]

    /// Search for an exercise by exact name and return its muscles + baseId.
    /// Only accepts exact name match (case-insensitive) — never falls back to partial matches.
    static func lookupByName(exerciseName: String) async -> (baseId: Int, primary: [String], secondary: [String])? {
        guard !exerciseName.isEmpty else { return nil }

        // Check name overrides first (exercises not in Wger, Swedish names, etc.)
        if let override = nameOverrides[exerciseName.lowercased()] {
            return (0, override.primary, override.secondary)
        }
        // Check erector spinae overrides (Wger lacks this muscle)
        if let override = erectorSpinaeExercises[exerciseName.lowercased()] {
            return (0, override.primary, override.secondary)
        }

        // Check alias first
        let searchName = nameAliases[exerciseName.lowercased()] ?? exerciseName
        let encoded = searchName.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? searchName
        let url = URL(string: "https://wger.de/api/v2/exercise/search/?language=2&term=\(encoded)")!

        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            let response = try JSONDecoder().decode(WgerSearchResponse.self, from: data)

            // Exact name match only (case-insensitive)
            guard let match = response.suggestions.first(where: {
                $0.data.name.lowercased() == searchName.lowercased()
            }) else { return nil }

            let raw = await fetchMuscles(baseId: match.data.baseId)
            let (primary, secondary) = applyOverrides(name: match.data.name, primary: raw.primary, secondary: raw.secondary)
            return primary.isEmpty ? nil : (match.data.baseId, primary, secondary)
        } catch {
            return nil
        }
    }
}
