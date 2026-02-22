import SwiftUI

struct LoginView: View {
    @Bindable var authVM: AuthViewModel
    @State private var showRegister = false

    var body: some View {
        NavigationStack {
            ZStack {
                Theme.bgPrimary.ignoresSafeArea()

                VStack(spacing: 24) {
                    Spacer()

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

                    Button("Skapa konto") {
                        showRegister = true
                    }
                    .font(.mono(size: 14))
                    .foregroundStyle(Theme.accentA)

                    Spacer()
                }
            }
            .navigationDestination(isPresented: $showRegister) {
                RegisterView(authVM: authVM)
            }
        }
    }
}
