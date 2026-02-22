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

                    // Recent sessions
                    if !vm.recentSessions.isEmpty {
                        VStack(alignment: .leading, spacing: 0) {
                            Text("SENASTE PASS")
                                .font(.mono(size: 11, weight: .medium))
                                .foregroundStyle(Theme.fgMuted)
                                .tracking(2)
                                .padding(.bottom, 12)

                            ForEach(vm.recentSessions) { session in
                                VStack(spacing: 0) {
                                    HStack {
                                        Text(session.routine)
                                            .font(.mono(size: 14, weight: .bold))
                                            .foregroundStyle(Theme.fgPrimary)

                                        Spacer()

                                        Text(String(format: "%.0f kg", session.totalVolume))
                                            .font(.mono(size: 13, weight: .medium))
                                            .foregroundStyle(Theme.accentA)
                                    }

                                    HStack {
                                        Text(formatDate(session.timestamp))
                                            .font(.mono(size: 11))
                                            .foregroundStyle(Theme.fgMuted)

                                        Spacer()

                                        Text(formatTime(session.durationSecs))
                                            .font(.mono(size: 11))
                                            .foregroundStyle(Theme.fgMuted)
                                    }
                                    .padding(.top, 2)

                                    Rectangle()
                                        .fill(Theme.border)
                                        .frame(height: 1)
                                        .padding(.top, 10)
                                }
                                .padding(.bottom, 10)
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
                }
                .padding()
            }
        }
        .navigationTitle("STATISTIK")
        .navigationBarTitleDisplayMode(.inline)
        .task { vm.loadStats() }
    }
}
