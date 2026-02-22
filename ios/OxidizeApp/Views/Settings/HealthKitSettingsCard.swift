import SwiftUI
import HealthKit

struct HealthKitSettingsCard: View {
    @State private var status: HKAuthorizationStatus = .notDetermined

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
                            .font(.mono(size: 10, weight: .medium))
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
        status = HealthKitService.shared.authorizationStatus()
    }

    private func handleAction() {
        switch status {
        case .notDetermined:
            Task {
                await HealthKitService.shared.requestAuthorization()
                refreshStatus()
            }
        case .sharingDenied, .sharingAuthorized:
            // Open system Settings → Health → Oxidize
            if let url = URL(string: UIApplication.openSettingsURLString) {
                UIApplication.shared.open(url)
            }
        @unknown default:
            break
        }
    }

    private var statusColor: Color {
        switch status {
        case .sharingAuthorized: Theme.accentA
        case .sharingDenied: Theme.danger
        case .notDetermined: Theme.fgMuted
        @unknown default: Theme.fgMuted
        }
    }

    private var statusText: String {
        switch status {
        case .sharingAuthorized: "Ansluten"
        case .sharingDenied: "Nekad"
        case .notDetermined: "Ej ansluten"
        @unknown default: "Okänd"
        }
    }

    private var statusDescription: String {
        switch status {
        case .sharingAuthorized: "Pass och kroppsvikt synkas"
        case .sharingDenied: "Ändra i Inställningar → Hälsa"
        case .notDetermined: "Synka pass och kroppsvikt"
        @unknown default: ""
        }
    }

    private var buttonText: String {
        switch status {
        case .notDetermined: "ANSLUT"
        case .sharingDenied: "INSTÄLLNINGAR"
        case .sharingAuthorized: "INSTÄLLNINGAR"
        @unknown default: "ANSLUT"
        }
    }

    private var buttonForeground: Color {
        status == .notDetermined ? Theme.accentA : Theme.fgSecondary
    }

    private var buttonBorder: Color {
        status == .notDetermined ? Theme.accentA : Theme.border
    }
}
