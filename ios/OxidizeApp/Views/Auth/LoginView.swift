import SwiftUI
import AuthenticationServices

struct LoginView: View {
    @Bindable var authVM: AuthViewModel
    @State private var showRegister = false

    var body: some View {
        NavigationStack {
            ZStack {
                Theme.bgPrimary.ignoresSafeArea()

                GeometryReader { geo in
                ScrollView {
                    VStack(spacing: 24) {
                        Spacer(minLength: 40)

                        Text("OXIDIZE")
                            .font(.mono(size: 32, weight: .bold))
                            .foregroundStyle(Theme.accentA)
                            .tracking(8)
                            .padding(.leading, 8)

                        Text("WORKOUT TRACKER")
                            .font(.mono(size: 12, weight: .medium))
                            .foregroundStyle(Theme.fgMuted)
                            .tracking(4)
                            .padding(.leading, 4)

                        VStack(spacing: 12) {
                            TextField("E-post", text: $authVM.email)
                                .textInputAutocapitalization(.never)
                                .keyboardType(.emailAddress)
                                .autocorrectionDisabled()
                                .darkInputStyle()

                            SecureField("Lösenord", text: $authVM.password)
                                .darkInputStyle()
                        }
                        .padding(.horizontal, 32)
                        .padding(.top, 16)

                        if let error = authVM.errorMessage {
                            Text(error)
                                .font(.mono(size: 13))
                                .foregroundStyle(Color(hex: "#ff6666"))
                                .padding(.horizontal, 16)
                                .padding(.vertical, 10)
                                .background(Theme.danger.opacity(0.15))
                                .overlay(
                                    RoundedRectangle(cornerRadius: 4)
                                        .stroke(Theme.danger, lineWidth: 1)
                                )
                                .clipShape(RoundedRectangle(cornerRadius: 4))
                                .padding(.horizontal, 32)
                        }

                        Button {
                            Task { await authVM.login() }
                        } label: {
                            if authVM.isLoading {
                                ProgressView()
                                    .tint(Theme.bgPrimary)
                                    .frame(maxWidth: .infinity)
                            } else {
                                Text("LOGGA IN")
                                    .font(.mono(size: 16, weight: .semibold))
                                    .tracking(2)
                                    .frame(maxWidth: .infinity)
                            }
                        }
                        .padding()
                        .background(Theme.accentA)
                        .foregroundStyle(Theme.bgPrimary)
                        .clipShape(RoundedRectangle(cornerRadius: 4))
                        .padding(.horizontal, 32)
                        .disabled(authVM.isLoading)
                        .opacity(authVM.isLoading ? 0.5 : 1)

                        HStack(spacing: 16) {
                            Button("Skapa konto") {
                                showRegister = true
                            }
                            .font(.mono(size: 14))
                            .foregroundStyle(Theme.accentA)

                            Text("·")
                                .foregroundStyle(Theme.fgMuted)

                            Button("Glömt lösenord?") {
                                Task { await authVM.resetPassword() }
                            }
                            .font(.mono(size: 14))
                            .foregroundStyle(Theme.fgMuted)
                        }

                        if authVM.resetSent {
                            Text("Återställningslänk skickad till din e-post")
                                .font(.mono(size: 13))
                                .foregroundStyle(Theme.accentA)
                                .padding(.horizontal, 32)
                        }

                        dividerRow

                        SignInWithAppleButton(.signIn) { request in
                            authVM.configureAppleSignIn(request)
                        } onCompletion: { result in
                            Task { await authVM.handleAppleSignIn(result) }
                        }
                        .signInWithAppleButtonStyle(.white)
                        .frame(height: 50)
                        .clipShape(RoundedRectangle(cornerRadius: 4))
                        .padding(.horizontal, 32)

                        Spacer(minLength: 40)
                    }
                    .frame(maxWidth: 500)
                    .frame(maxWidth: .infinity)
                    .frame(minHeight: geo.size.height)
                }
                .scrollBounceBehavior(.basedOnSize)
                }
            }
            .navigationDestination(isPresented: $showRegister) {
                RegisterView(authVM: authVM)
            }
        }
    }

    private var dividerRow: some View {
        HStack(spacing: 12) {
            Rectangle()
                .fill(Theme.fgMuted.opacity(0.3))
                .frame(height: 1)
            Text("ELLER")
                .font(.mono(size: 11, weight: .medium))
                .foregroundStyle(Theme.fgMuted)
                .tracking(2)
            Rectangle()
                .fill(Theme.fgMuted.opacity(0.3))
                .frame(height: 1)
        }
        .padding(.horizontal, 32)
    }
}
