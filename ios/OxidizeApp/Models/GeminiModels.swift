import Foundation

struct GeminiRequest: Codable {
    var contents: [GeminiContent]
}

struct GeminiContent: Codable {
    var parts: [GeminiPart]
}

struct GeminiPart: Codable {
    var text: String
}

struct GeminiResponse: Codable {
    var candidates: [GeminiCandidate]
}

struct GeminiCandidate: Codable {
    var content: GeminiContentResponse
}

struct GeminiContentResponse: Codable {
    var parts: [GeminiPartResponse]
}

struct GeminiPartResponse: Codable {
    var text: String
}
