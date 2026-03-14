import SwiftUI

struct HealthKitSettingsCard: View {
    @State private var status: HealthKitService.HealthStatus = .notDetermined

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("APPLE HÄLSA")
                .labelStyle()
                .padding(.horizontal)

            VStack(spacing: 12) {
                HStack {
                    VStack(alignment: .leading, spacing: 4) {
                        HStack(spacing: 8) {
                            Circle()
                                .fill(statusColor)
                                .frame(width: 8, height: 8)
                            Text(statusText)
                                .font(.mono(size: 14))
                                .foregroundStyle(Theme.fgPrimary)
                        }
                        Text(statusDescription)
                            .font(.mono(size: 11))
                            .foregroundStyle(Theme.fgMuted)
                    }

                    Spacer()

                    Button {
                        handleAction()
                    } label: {
                        Text(buttonText)
                            .font(.mono(size: 11, weight: .medium))
                            .tracking(1)
                            .foregroundStyle(buttonForeground)
                            .padding(.horizontal, 10)
                            .padding(.vertical, 6)
                            .overlay(
                                RoundedRectangle(cornerRadius: 4)
                                    .stroke(buttonBorder, lineWidth: 1)
                            )
                    }
                }
            }
            .padding()
            .background(Theme.bgCard)
            .overlay(
                RoundedRectangle(cornerRadius: 4)
                    .stroke(Theme.border, lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 4))
            .padding(.horizontal)
        }
        .onAppear { refreshStatus() }
    }

    private func refreshStatus() {
        status = HealthKitService.shared.healthStatus()
    }

    private func handleAction() {
        switch status {
        case .notDetermined:
            Task {
                await HealthKitService.shared.requestAuthorization()
                refreshStatus()
            }
        case .denied, .partiallyAuthorized, .authorized:
            if let url = URL(string: UIApplication.openSettingsURLString) {
                UIApplication.shared.open(url)
            }
        }
    }

    private var statusColor: Color {
        switch status {
        case .authorized: Theme.accentA
        case .partiallyAuthorized: Theme.accentB
        case .denied: Theme.danger
        case .notDetermined: Theme.fgMuted
        }
    }

    private var statusText: String {
        switch status {
        case .authorized: "Ansluten"
        case .partiallyAuthorized: "Delvis ansluten"
        case .denied: "Nekad"
        case .notDetermined: "Ej ansluten"
        }
    }

    private var statusDescription: String {
        switch status {
        case .authorized: "Pass och kroppsvikt synkas"
        case .partiallyAuthorized: "Vissa behörigheter saknas"
        case .denied: "Ändra i Inställningar → Hälsa"
        case .notDetermined: "Synka pass och kroppsvikt"
        }
    }

    private var buttonText: String {
        switch status {
        case .notDetermined: "ANSLUT"
        default: "INSTÄLLNINGAR"
        }
    }

    private var buttonForeground: Color {
        status == .notDetermined ? Theme.accentA : Theme.fgSecondary
    }

    private var buttonBorder: Color {
        status == .notDetermined ? Theme.accentA : Theme.border
    }
}
