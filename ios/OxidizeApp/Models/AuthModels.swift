import Foundation

struct AuthUser: Codable {
    var id: String
    var email: String
    var displayName: String?

    enum CodingKeys: String, CodingKey {
        case id, email
        case displayName = "display_name"
    }
}

struct AuthSession: Codable {
    var accessToken: String
    var refreshToken: String?
    var user: AuthUser

    enum CodingKeys: String, CodingKey {
        case user
        case accessToken = "access_token"
        case refreshToken = "refresh_token"
    }
}
