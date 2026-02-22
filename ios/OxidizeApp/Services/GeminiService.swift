import Foundation

enum GeminiService {
    static let systemPrompt = """
    Du är en expert-träningsplanerare. Generera en träningsrutin i EXAKT detta JSON-format:
    {
      "name": "Rutinens namn",
      "focus": "Kort beskrivning av fokus",
      "passes": [
        {
          "name": "Pass A",
          "description": "Kort beskrivning",
          "exercises": [
            {"name": "Exercise Name", "sets": 3, "reps_target": "8-12", "is_superset": false, "is_bodyweight": false}
          ],
          "finishers": [
            {"name": "Finisher Name", "sets": 3, "reps_target": "15", "is_superset": false, "is_bodyweight": true}
          ]
        }
      ]
    }

    REGLER:
    - Passnamn: Max 8 tecken (t.ex. "Pass A", "Rygg", "Ben")
    - Övningsnamn på engelska
    - reps_target: "5-8", "10-12", "AMRAP", eller "30 sek" för tidsstyrda
    - Supersets: Sätt is_superset=true och superset_with="partnerns namn" på BÅDA övningarna
    - superset_name: Kort namn för supersetet (t.ex. "Armar", "Press/Pull")
    - Finishers: Alltid is_bodyweight=true, vanligtvis core/cardio
    - duration_secs: Sätt t.ex. 30 för tidsstyrda övningar (hopprep, mountain climbers etc)
    - Svara BARA med JSON, ingen annan text
    """

    static func generateRoutine(
        passCount: Int,
        goal: String,
        description: String,
        targetAreas: String,
        style: String,
        equipment: String,
        duration: String,
        supersets: Bool,
        finishers: Bool,
        bodyweight: Double?
    ) async throws -> (name: String, focus: String, passes: [Pass]) {
        guard let apiKey = await SupabaseService.shared.fetchApiKey() else {
            throw OxidizeError.networkError("Kunde inte hämta API-nyckel")
        }

        let bwStr = bodyweight.map { "Min kroppsvikt: \(formatWeight($0)) kg" } ?? ""
        let userPrompt = """
        Skapa en träningsrutin med dessa parametrar:
        - Antal pass: \(passCount)
        - Mål: \(goal)
        - Beskrivning: \(description)
        - Målområden: \(targetAreas)
        - Träningsstil: \(style)
        - Utrustning: \(equipment)
        - Passlängd: \(duration)
        - Supersets: \(supersets ? "Ja, gärna" : "Nej")
        - Finishers: \(finishers ? "Ja, 2-3 per pass" : "Nej")
        \(bwStr)
        """

        let responseText = try await callGemini(apiKey: apiKey, systemPrompt: systemPrompt, userPrompt: userPrompt)

        // Extract JSON from response (may be wrapped in ```json ... ```)
        let jsonString = extractJSON(from: responseText)
        guard let jsonData = jsonString.data(using: .utf8) else {
            throw OxidizeError.networkError("Ogiltigt svar från AI")
        }

        struct AIRoutine: Codable {
            var name: String
            var focus: String
            var passes: [Pass]
        }

        let routine = try JSONDecoder().decode(AIRoutine.self, from: jsonData)

        // Validate and clean superset integrity
        let cleanedPasses = routine.passes.map { pass -> Pass in
            var exercises = pass.exercises
            var finishers = pass.finishers

            // Remove broken supersets
            for i in exercises.indices {
                if exercises[i].isSuperset {
                    let partnerName = exercises[i].supersetWith ?? ""
                    let hasPartner = exercises.contains { $0.name == partnerName && $0.isSuperset }
                    if !hasPartner {
                        exercises[i].isSuperset = false
                        exercises[i].supersetWith = nil
                        exercises[i].supersetName = nil
                    }
                }
            }

            return Pass(name: pass.name, description: pass.description, exercises: exercises, finishers: finishers)
        }

        return (routine.name, routine.focus, cleanedPasses)
    }

    static func callGemini(apiKey: String, systemPrompt: String, userPrompt: String) async throws -> String {
        let fullPrompt = "\(systemPrompt)\n\nUser request: \(userPrompt)"

        let requestBody = GeminiRequest(
            contents: [GeminiContent(parts: [GeminiPart(text: fullPrompt)])]
        )

        let bodyData = try JSONEncoder().encode(requestBody)

        var request = URLRequest(url: URL(string: "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key=\(apiKey)")!)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = bodyData

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 else {
            let statusCode = (response as? HTTPURLResponse)?.statusCode ?? 0
            let body = String(data: data, encoding: .utf8) ?? ""
            print("[Gemini] API error \(statusCode): \(body)")
            throw OxidizeError.networkError("Gemini API fel \(statusCode)")
        }

        let geminiResponse = try JSONDecoder().decode(GeminiResponse.self, from: data)
        guard let text = geminiResponse.candidates.first?.content.parts.first?.text else {
            throw OxidizeError.networkError("Tomt svar från Gemini")
        }

        return text
    }

    private static func extractJSON(from text: String) -> String {
        // Remove ```json ... ``` wrapping
        var cleaned = text.trimmingCharacters(in: .whitespacesAndNewlines)
        if cleaned.hasPrefix("```json") {
            cleaned = String(cleaned.dropFirst(7))
        } else if cleaned.hasPrefix("```") {
            cleaned = String(cleaned.dropFirst(3))
        }
        if cleaned.hasSuffix("```") {
            cleaned = String(cleaned.dropLast(3))
        }
        return cleaned.trimmingCharacters(in: .whitespacesAndNewlines)
    }
}
