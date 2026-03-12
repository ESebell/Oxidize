import Foundation
import AuthenticationServices
import CryptoKit

@MainActor
@Observable
final class AuthViewModel {
    var email = ""
    var password = ""
    var confirmPassword = ""
    var isLoading = false
    var errorMessage: String?
    var isAuthenticated = false
    var currentSession: AuthSession?

    private var currentNonce: String?

    func checkSession() async {
        if let session = StorageService.shared.loadAuthSession() {
            currentSession = session
            isAuthenticated = true
            await SupabaseService.shared.checkAndRefreshSession()
            // Re-check in case session was expired
            if StorageService.shared.loadAuthSession() == nil {
                isAuthenticated = false
                currentSession = nil
            }
        }
    }

    func login() async {
        guard !email.isEmpty, !password.isEmpty else {
            errorMessage = "Fyll i alla fält"
            return
        }

        isLoading = true
        errorMessage = nil

        do {
            let session = try await SupabaseService.shared.signIn(email: email, password: password)
            currentSession = session
            isAuthenticated = true
            password = ""
        } catch {
            errorMessage = error.localizedDescription
        }

        isLoading = false
    }

    func register() async {
        guard !email.isEmpty, !password.isEmpty else {
            errorMessage = "Fyll i alla fält"
            return
        }
        guard password == confirmPassword else {
            errorMessage = "Lösenorden matchar inte"
            return
        }
        guard password.count >= 6 else {
            errorMessage = "Lösenordet måste vara minst 6 tecken"
            return
        }

        isLoading = true
        errorMessage = nil

        do {
            let session = try await SupabaseService.shared.signUp(email: email, password: password)
            currentSession = session
            isAuthenticated = true
            password = ""
            confirmPassword = ""
        } catch {
            errorMessage = error.localizedDescription
        }

        isLoading = false
    }

    func logout() async {
        await SupabaseService.shared.signOut()
        currentSession = nil
        isAuthenticated = false
        email = ""
        password = ""
    }

    func deleteAccount() async throws {
        try await SupabaseService.shared.deleteAccount()
        currentSession = nil
        isAuthenticated = false
        email = ""
        password = ""
    }

    // MARK: - Sign in with Apple

    func configureAppleSignIn(_ request: ASAuthorizationAppleIDRequest) {
        let nonce = randomNonceString()
        currentNonce = nonce
        request.requestedScopes = [.email]
        request.nonce = sha256(nonce)
    }

    func handleAppleSignIn(_ result: Result<ASAuthorization, Error>) async {
        switch result {
        case .success(let authorization):
            guard let credential = authorization.credential as? ASAuthorizationAppleIDCredential,
                  let tokenData = credential.identityToken,
                  let idToken = String(data: tokenData, encoding: .utf8),
                  let nonce = currentNonce
            else {
                errorMessage = "Kunde inte hämta Apple-inloggning"
                return
            }

            isLoading = true
            errorMessage = nil

            do {
                let session = try await SupabaseService.shared.signInWithApple(idToken: idToken, nonce: nonce)
                currentSession = session
                isAuthenticated = true
            } catch {
                errorMessage = error.localizedDescription
            }

            isLoading = false

        case .failure(let error):
            if (error as NSError).code != ASAuthorizationError.canceled.rawValue {
                errorMessage = error.localizedDescription
            }
        }
    }

    private func randomNonceString(length: Int = 32) -> String {
        let charset = Array("0123456789ABCDEFGHIJKLMNOPQRSTUVXYZabcdefghijklmnopqrstuvwxyz-._")
        var result = ""
        var remainingLength = length
        while remainingLength > 0 {
            var randoms = [UInt8](repeating: 0, count: 16)
            _ = SecRandomCopyBytes(kSecRandomDefault, randoms.count, &randoms)
            for random in randoms {
                guard remainingLength > 0 else { break }
                result.append(charset[Int(random) % charset.count])
                remainingLength -= 1
            }
        }
        return result
    }

    private func sha256(_ input: String) -> String {
        let data = Data(input.utf8)
        let hash = SHA256.hash(data: data)
        return hash.map { String(format: "%02x", $0) }.joined()
    }
}
