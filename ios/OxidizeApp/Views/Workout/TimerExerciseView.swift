import SwiftUI

struct TimerExerciseView: View {
    @Bindable var vm: WorkoutViewModel
    let targetDuration: Int

    private let durations = [20, 25, 30, 35, 40, 45, 50, 55]

    var body: some View {
        VStack(spacing: 16) {
            if vm.timerRunning {
                // Countdown
                VStack(spacing: 12) {
                    Text(String(format: "0:%02d", vm.timerRemaining))
                        .font(.mono(size: 64, weight: .bold))
                        .foregroundStyle(Theme.accentA)
                        .shadow(color: Theme.accentA.opacity(0.5), radius: 10)

                    Button {
                        vm.stopExerciseTimer()
                    } label: {
                        Text("AVBRYT")
                            .font(.mono(size: 12, weight: .medium))
                            .tracking(1)
                            .foregroundStyle(Theme.danger)
                    }
                }
            } else {
                // Duration selector
                VStack(spacing: 12) {
                    Text("VÄLJ TID")
                        .font(.mono(size: 11, weight: .medium))
                        .foregroundStyle(Theme.fgSecondary)
                        .tracking(2)

                    LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 8), count: 4), spacing: 8) {
                        ForEach(durations, id: \.self) { d in
                            let isSelected = vm.timerSelectedDuration == d
                            let isTarget = d == targetDuration
                            let lastDuration = vm.currentExercise?.lastData?.reps

                            Button {
                                vm.timerSelectedDuration = d
                            } label: {
                                Text("\(d)s")
                                    .font(.mono(size: 14, weight: .bold))
                                    .foregroundStyle(durationForeground(isSelected: isSelected, isTarget: isTarget, isLast: lastDuration == d))
                                    .frame(maxWidth: .infinity)
                                    .frame(height: 44)
                                    .background(durationBg(isSelected: isSelected, isTarget: isTarget, isLast: lastDuration == d))
                                    .overlay(
                                        RoundedRectangle(cornerRadius: 4)
                                            .stroke(durationBorder(isSelected: isSelected, isTarget: isTarget, isLast: lastDuration == d), lineWidth: 2)
                                    )
                                    .clipShape(RoundedRectangle(cornerRadius: 4))
                            }
                        }
                    }

                    Button {
                        vm.startExerciseTimer()
                    } label: {
                        HStack(spacing: 8) {
                            Image(systemName: "play.fill")
                                .font(.mono(size: 12))
                            Text("STARTA")
                                .font(.mono(size: 16, weight: .bold))
                                .tracking(2)
                        }
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(Theme.accentA)
                        .foregroundStyle(Theme.bgPrimary)
                        .clipShape(RoundedRectangle(cornerRadius: 4))
                    }

                    Text("MÅL: \(targetDuration) SEK")
                        .font(.mono(size: 11))
                        .foregroundStyle(Theme.fgMuted)
                        .tracking(1)
                }
            }
        }
    }

    // Selected = filled (only exception)
    private func durationBorder(isSelected: Bool, isTarget: Bool, isLast: Bool) -> Color {
        if isSelected { return Theme.accentA }
        if isLast { return Color(hex: "#00aaff") }
        if isTarget { return Theme.accentA }
        return Theme.border
    }

    private func durationForeground(isSelected: Bool, isTarget: Bool, isLast: Bool) -> Color {
        if isSelected { return Theme.bgPrimary }
        if isLast { return Color(hex: "#00aaff") }
        if isTarget { return Theme.accentA }
        return Theme.fgSecondary
    }

    private func durationBg(isSelected: Bool, isTarget: Bool, isLast: Bool) -> Color {
        if isSelected { return Theme.accentA }
        if isLast { return Color(hex: "#00aaff").opacity(0.1) }
        return Theme.bgCard
    }
}
