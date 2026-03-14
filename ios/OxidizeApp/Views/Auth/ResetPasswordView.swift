import SwiftUI

struct ResetPasswordView: View {
    @Bindable var authVM: AuthViewModel

    var body: some View {
        ZStack {
            Theme.bgPrimary.ignoresSafeArea()

            VStack(spacing: 24) {
                Spacer(minLength: 40)

                Text("NYTT LÖSENORD")
                    .font(.mono(size: 24, weight: .bold))
                    .foregroundStyle(Theme.accentA)
                    .tracking(4)
                    .padding(.leading, 4)

                Text("Ange ditt nya lösenord")
                    .font(.mono(size: 13))
                    .foregroundStyle(Theme.fgMuted)

                VStack(spacing: 12) {
                    SecureField("Nytt lösenord", text: $authVM.newPassword)
                        .darkInputStyle()

                    SecureField("Bekräfta lösenord", text: $authVM.confirmNewPassword)
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
                    Task { await authVM.updatePassword() }
                } label: {
                    if authVM.isLoading {
                        ProgressView()
                            .tint(Theme.bgPrimary)
                            .frame(maxWidth: .infinity)
                    } else {
                        Text("SPARA")
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
            .frame(maxWidth: 500)
        }
    }
}
