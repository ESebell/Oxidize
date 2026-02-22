import SwiftUI

struct RepButtonGrid: View {
    @Bindable var vm: WorkoutViewModel

    private var exercise: ExerciseWorkoutState? { vm.currentExercise }

    var body: some View {
        VStack(spacing: 12) {
            Text("TRYCK ANTAL REPS")
                .font(.mono(size: 11, weight: .medium))
                .foregroundStyle(Theme.fgSecondary)
                .tracking(2)

            let (min, max) = exercise.map { parseTargetRange($0.exercise.repsTarget) } ?? (5, 8)
            let lastReps = exercise?.lastData?.reps
            let center = (min + max) / 2
            let start = Swift.max(1, center - 5)
            let end = start + 11

            LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 8), count: 4), spacing: 8) {
                ForEach(start...end, id: \.self) { rep in
                    let isTarget = rep >= min && rep <= max
                    let isLast = lastReps == rep

                    Button {
                        vm.completeSet(reps: rep)
                    } label: {
                        RoundedRectangle(cornerRadius: 4)
                            .fill(buttonBg(isTarget: isTarget, isLast: isLast))
                            .aspectRatio(1, contentMode: .fit)
                            .overlay {
                                Text("\(rep)")
                                    .font(.mono(size: 20, weight: .bold))
                                    .foregroundStyle(buttonForeground(isTarget: isTarget, isLast: isLast))
                            }
                            .overlay(
                                RoundedRectangle(cornerRadius: 4)
                                    .stroke(buttonBorder(isTarget: isTarget, isLast: isLast), lineWidth: 2)
                            )
                    }
                }
            }
            .padding(.horizontal, 24)

            Text("MÅL: \(exercise?.exercise.repsTarget ?? "")")
                .font(.mono(size: 11))
                .foregroundStyle(Theme.fgMuted)
                .tracking(1)
        }
    }

    private func buttonBorder(isTarget: Bool, isLast: Bool) -> Color {
        if isLast { return Color(hex: "#00aaff") }
        if isTarget { return Theme.accentA }
        return Theme.border
    }

    private func buttonForeground(isTarget: Bool, isLast: Bool) -> Color {
        if isLast { return Color(hex: "#00aaff") }
        if isTarget { return Theme.accentA }
        return Theme.fgSecondary
    }

    private func buttonBg(isTarget: Bool, isLast: Bool) -> Color {
        if isLast { return Color(hex: "#00aaff").opacity(0.1) }
        if isTarget { return Theme.accentA.opacity(0.05) }
        return Theme.bgCard
    }
}
