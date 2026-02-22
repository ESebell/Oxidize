import SwiftUI
import Charts

struct BodyweightChartCard: View {
    let vm: StatsViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("KROPPSVIKT")
                .font(.mono(size: 12, weight: .bold))
                .foregroundStyle(Theme.fgPrimary)
                .tracking(1)

            if let latest = vm.bodyweightHistory.last {
                Text("\(formatWeight(latest.weight)) kg")
                    .font(.mono(size: 24, weight: .bold))
                    .foregroundStyle(Theme.fgPrimary)
            }

            Chart {
                ForEach(vm.bodyweightHistory, id: \.timestamp) { entry in
                    LineMark(
                        x: .value("Datum", Date(timeIntervalSince1970: TimeInterval(entry.timestamp))),
                        y: .value("Vikt", entry.weight)
                    )
                    .foregroundStyle(Theme.accentA)
                    .lineStyle(StrokeStyle(lineWidth: 1.5))
                    .interpolationMethod(.catmullRom)

                    AreaMark(
                        x: .value("Datum", Date(timeIntervalSince1970: TimeInterval(entry.timestamp))),
                        y: .value("Vikt", entry.weight)
                    )
                    .foregroundStyle(
                        LinearGradient(
                            colors: [Theme.accentA.opacity(0.15), Theme.accentA.opacity(0)],
                            startPoint: .top,
                            endPoint: .bottom
                        )
                    )
                    .interpolationMethod(.catmullRom)
                }
            }
            .frame(height: 150)
            .chartYScale(domain: yDomain)
            .chartPlotStyle { plot in
                plot.clipped()
            }
            .chartXAxis {
                AxisMarks { _ in
                    AxisGridLine().foregroundStyle(Theme.border)
                }
            }
            .chartYAxis {
                AxisMarks { _ in
                    AxisGridLine().foregroundStyle(Theme.border)
                    AxisValueLabel()
                        .font(.mono(size: 10))
                        .foregroundStyle(Theme.fgMuted)
                }
            }

            // Min / Max
            if let minW = vm.bodyweightHistory.map(\.weight).min(),
               let maxW = vm.bodyweightHistory.map(\.weight).max() {
                HStack {
                    Text("MIN: \(formatWeight(minW)) KG")
                    Spacer()
                    Text("MAX: \(formatWeight(maxW)) KG")
                }
                .font(.mono(size: 10))
                .foregroundStyle(Theme.fgMuted)
                .tracking(1)
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

    private var yDomain: ClosedRange<Double> {
        let weights = vm.bodyweightHistory.map(\.weight)
        let minW = (weights.min() ?? 70) - 2
        let maxW = (weights.max() ?? 90) + 2
        return minW...maxW
    }
}
