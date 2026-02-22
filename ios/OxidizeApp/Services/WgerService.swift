import Foundation

enum WgerService {
    static func searchExercises(query: String) async throws -> [WgerExercise] {
        guard !query.isEmpty else { return [] }

        let encoded = query.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? query
        let url = URL(string: "https://wger.de/api/v2/exercise/search/?language=2&term=\(encoded)")!

        let (data, _) = try await URLSession.shared.data(from: url)
        let response = try JSONDecoder().decode(WgerSearchResponse.self, from: data)

        return Array(response.suggestions.prefix(10).map { suggestion in
            let d = suggestion.data
            let primaryMuscles = d.muscles?.map { $0.nameEn ?? $0.name } ?? []
            let secondaryMuscles = d.musclesSecondary?.map { $0.nameEn ?? $0.name } ?? []
            var imageUrl = d.image
            if let img = imageUrl, !img.hasPrefix("http") {
                imageUrl = "https://wger.de\(img)"
            }
            let equipment = d.equipment?.first?.name

            return WgerExercise(
                id: d.id,
                name: d.name,
                primaryMuscles: primaryMuscles,
                secondaryMuscles: secondaryMuscles,
                imageUrl: imageUrl,
                equipment: equipment
            )
        })
    }
}
