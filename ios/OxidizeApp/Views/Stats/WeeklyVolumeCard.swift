import SwiftUI
import Charts

struct WeeklyVolumeCard: View {
    let vm: StatsViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("VECKOVOLYM PER MUSKELGRUPP")
                .font(.mono(size: 12, weight: .bold))
                .foregroundStyle(Theme.fgPrimary)
                .tracking(1)

            ForEach(MuscleGroup.allCases, id: \.self) { muscle in
                let sets = vm.summary.weeklySets[muscle] ?? 0
                HStack {
                    Text(muscle.displayName.uppercased())
                        .font(.mono(size: 10))
                        .foregroundStyle(Theme.fgSecondary)
                        .frame(width: 70, alignment: .leading)

                    GeometryReader { geo in
                        let maxSets = 25.0
                        let width = geo.size.width * min(Double(sets) / maxSets, 1.0)

                        ZStack(alignment: .leading) {
                            RoundedRectangle(cornerRadius: 3)
                                .fill(Theme.fgMuted.opacity(0.2))

                            RoundedRectangle(cornerRadius: 3)
                                .fill(volumeColor(sets: sets))
                                .frame(width: max(width, 2))
                        }
                    }
                    .frame(height: 14)

                    Text("\(sets)")
                        .font(.mono(size: 12, weight: .bold))
                        .foregroundStyle(Theme.fgSecondary)
                        .frame(width: 30, alignment: .trailing)
                }
            }

            // Legend
            HStack(spacing: 12) {
                LegendDot(color: Theme.volNone, label: "0")
                LegendDot(color: Theme.volLow, label: "1-9")
                LegendDot(color: Theme.volOptimal, label: "10-20")
                LegendDot(color: Theme.volHigh, label: "20+")
            }
            .font(.mono(size: 10))
            .foregroundStyle(Theme.fgMuted)
            .frame(maxWidth: .infinity)
        }
        .padding()
        .background(Theme.bgCard)
        .overlay(
            RoundedRectangle(cornerRadius: 4)
                .stroke(Theme.border, lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 4))
    }

    private func volumeColor(sets: Int) -> Color {
        if sets == 0 { return Theme.volNone.opacity(0.3) }
        if sets < 10 { return Theme.volLow }
        if sets <= 20 { return Theme.volOptimal }
        return Theme.volHigh
    }
}

struct LegendDot: View {
    let color: Color
    let label: String

    var body: some View {
        HStack(spacing: 4) {
            Circle().fill(color).frame(width: 6, height: 6)
            Text(label)
        }
    }
}
