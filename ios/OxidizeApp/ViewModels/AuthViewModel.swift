import Foundation

@Observable
final class AuthViewModel {
    var email = ""
    var password = ""
    var confirmPassword = ""
    var isLoading = false
    var errorMessage: String?
    var isAuthenticated = false
    var currentSession: AuthSession?

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
}
