import SwiftUI

struct RegisterView: View {
    @Bindable var authVM: AuthViewModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        ZStack {
            Theme.bgPrimary.ignoresSafeArea()

            VStack(spacing: 24) {
                Spacer()

                Text("SKAPA KONTO")
                    .font(.mono(size: 24, weight: .bold))
                    .foregroundStyle(Theme.fgPrimary)
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

                    SecureField("Bekräfta lösenord", text: $authVM.confirmPassword)
                        .darkInputStyle()
                }
                .padding(.horizontal, 32)

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
                    Task { await authVM.register() }
                } label: {
                    if authVM.isLoading {
                        ProgressView()
                            .tint(Theme.bgPrimary)
                            .frame(maxWidth: .infinity)
                    } else {
                        Text("REGISTRERA")
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

                Spacer()
            }
        }
        .navigationBarBackButtonHidden(false)
    }
}
