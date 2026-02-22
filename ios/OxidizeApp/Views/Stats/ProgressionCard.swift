import SwiftUI

struct ProgressionCard: View {
    let vm: StatsViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("SENASTE PASSET")
                .font(.mono(size: 12, weight: .bold))
                .foregroundStyle(Theme.fgPrimary)
                .tracking(1)

            if vm.lastSessionProgression.isEmpty {
                Text("Inga pass ännu")
                    .font(.mono(size: 13))
                    .foregroundStyle(Theme.fgSecondary)
            } else {
                ForEach(vm.lastSessionProgression, id: \.0) { name, status in
                    HStack(spacing: 0) {
                        Rectangle()
                            .fill(statusColor(status))
                            .frame(width: 3)

                        HStack {
                            Text(statusIcon(status))
                                .font(.mono(size: 16))
                            Text(name)
                                .font(.mono(size: 13))
                                .foregroundStyle(Theme.fgPrimary)
                            Spacer()
                            Text(statusLabel(status).uppercased())
                                .font(.mono(size: 10, weight: .medium))
                                .foregroundStyle(statusColor(status))
                                .tracking(1)
                        }
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                    }
                    .background(statusColor(status).opacity(0.08))
                    .clipShape(RoundedRectangle(cornerRadius: 4))
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
    }

    private func statusIcon(_ status: ProgressStatus) -> String {
        switch status {
        case .improved: "\u{1F525}"
        case .maintained: "\u{27A1}\u{FE0F}"
        case .regressed: "\u{2B07}\u{FE0F}"
        case .firstTime: "\u{1F195}"
        }
    }

    private func statusLabel(_ status: ProgressStatus) -> String {
        switch status {
        case .improved: "Förbättrad"
        case .maintained: "Bibehållen"
        case .regressed: "Sämre"
        case .firstTime: "Första gången"
        }
    }

    private func statusColor(_ status: ProgressStatus) -> Color {
        switch status {
        case .improved: Theme.progressImproved
        case .maintained: Theme.progressMaintained
        case .regressed: Theme.progressRegressed
        case .firstTime: Theme.progressNew
        }
    }
}
