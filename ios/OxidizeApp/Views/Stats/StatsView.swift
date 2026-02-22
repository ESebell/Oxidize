import SwiftUI
import Charts

struct StatsView: View {
    @State private var vm = StatsViewModel()

    var body: some View {
        ZStack {
            Theme.bgPrimary.ignoresSafeArea()

            ScrollView {
                VStack(spacing: 16) {
                    PowerScoreCard(vm: vm)
                    WeeklyVolumeCard(vm: vm)
                    ProgressionCard(vm: vm)
                    if !vm.bodyweightHistory.isEmpty {
                        BodyweightChartCard(vm: vm)
                    }
                }
                .padding()
            }
        }
        .navigationTitle("STATISTIK")
        .navigationBarTitleDisplayMode(.inline)
        .task { vm.loadStats() }
    }
}
