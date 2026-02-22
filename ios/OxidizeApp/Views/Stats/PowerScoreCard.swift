import SwiftUI
import Charts

struct PowerScoreCard: View {
    let vm: StatsViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            // Hero
            VStack(spacing: 4) {
                Text(String(format: "%.0f", vm.summary.powerScore))
                    .font(.mono(size: 48, weight: .bold))
                    .foregroundStyle(Theme.fgPrimary)
                Text("KG STYRKETOTAL")
                    .font(.mono(size: 11, weight: .medium))
                    .foregroundStyle(Theme.fgMuted)
                    .tracking(2)

                if vm.bodyweight > 0 {
                    let ratio = vm.summary.powerScore / vm.bodyweight
                    Text(String(format: "%.1fx KROPPSVIKT", ratio))
                        .font(.mono(size: 11))
                        .foregroundStyle(Theme.fgSecondary)
                        .tracking(1)
                }
            }
            .frame(maxWidth: .infinity)

            // Big 4 breakdown
            LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 10), count: 2), spacing: 10) {
                ForEach(BIG_FOUR, id: \.self) { lift in
                    let e1rm = vm.summary.e1rmByExercise[lift] ?? 0
                    HStack(spacing: 0) {
                        Rectangle()
                            .fill(Theme.accentA)
                            .frame(width: 3)
                        VStack(spacing: 4) {
                            Text(lift.uppercased())
                                .font(.mono(size: 10))
                                .foregroundStyle(Theme.fgSecondary)
                                .tracking(1)
                            Text(String(format: "%.0f kg", e1rm))
                                .font(.mono(size: 16, weight: .bold))
                                .foregroundStyle(Theme.fgPrimary)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(8)
                    }
                    .background(Theme.bgSecondary)
                    .clipShape(RoundedRectangle(cornerRadius: 4))
                }
            }

            // Power score history chart
            if vm.powerScoreHistory.count > 1 {
                Chart {
                    ForEach(Array(vm.powerScoreHistory.suffix(10).enumerated()), id: \.offset) { index, point in
                        BarMark(
                            x: .value("Session", index),
                            y: .value("Score", point.1)
                        )
                        .foregroundStyle(
                            LinearGradient(
                                colors: [Theme.accentA, Theme.accentA.opacity(0.3)],
                                startPoint: .top,
                                endPoint: .bottom
                            )
                        )
                    }
                }
                .frame(height: 80)
                .chartYAxis(.hidden)
                .chartXAxis(.hidden)
            }
        }
        .padding()
        .background(
            LinearGradient(
                colors: [Theme.bgCard, Theme.accentA.opacity(0.05)],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        )
        .overlay(
            RoundedRectangle(cornerRadius: 4)
                .stroke(Theme.accentA.opacity(0.3), lineWidth: 2)
        )
        .clipShape(RoundedRectangle(cornerRadius: 4))
    }
}
