import SwiftUI

struct ExerciseScreen: View {
    @Bindable var vm: WorkoutViewModel

    private var exercise: ExerciseWorkoutState? { vm.currentExercise }
    private var isTimed: Bool { exercise?.exercise.durationSecs != nil }
    private var isBodyweight: Bool { exercise?.exercise.isBodyweight ?? false }
    private var isSuperset: Bool { exercise?.exercise.isSuperset ?? false }
    private var targetDuration: Int { exercise?.exercise.durationSecs ?? 30 }
    private var exerciseName: String { exercise?.exercise.name ?? "" }
    private var isDumbbell: Bool { ["Hammercurls", "Sidolyft"].contains(exerciseName) }
    private var isAlternating: Bool { ["Utfallssteg", "Dead Bug"].contains(exerciseName) }

    var body: some View {
        ScrollView {
            VStack(spacing: 24) {
                // Set progress
                Text("SET \(vm.currentSetNum) AV \(vm.totalSets)")
                    .font(.mono(size: 12, weight: .medium))
                    .foregroundStyle(Theme.fgSecondary)
                    .tracking(2)

                // Superset indicator
                if isSuperset, let partner = exercise?.exercise.supersetWith {
                    HStack(spacing: 4) {
                        Image(systemName: "arrow.triangle.2.circlepath")
                            .font(.mono(size: 12))
                        Text("SUPERSET \u{2192} \(partner)")
                            .font(.mono(size: 11, weight: .bold))
                            .tracking(1)
                    }
                    .foregroundStyle(Theme.accentB)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 6)
                    .background(Theme.accentB.opacity(0.1))
                    .clipShape(Capsule())
                }

                // Finisher indicator
                if isBodyweight {
                    Text("FINISHER")
                        .font(.mono(size: 11, weight: .bold))
                        .tracking(2)
                        .foregroundStyle(Theme.accentB)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 4)
                        .background(Theme.accentB.opacity(0.1))
                        .overlay(
                            RoundedRectangle(cornerRadius: 4)
                                .stroke(Theme.accentB, lineWidth: 1)
                        )
                        .clipShape(RoundedRectangle(cornerRadius: 4))
                }

                // Exercise name
                Text(exerciseName.uppercased())
                    .font(.mono(size: 24, weight: .bold))
                    .foregroundStyle(Theme.fgPrimary)
                    .multilineTextAlignment(.center)
                    .tracking(1)

                // Hints
                if isDumbbell {
                    Text("Lägg ihop båda hantlarnas vikt")
                        .font(.mono(size: 12))
                        .foregroundStyle(Theme.fgMuted)
                }
                if isAlternating {
                    Text("Totalt antal reps (båda sidor)")
                        .font(.mono(size: 12))
                        .foregroundStyle(Theme.fgMuted)
                }

                // Weight adjuster (non-bodyweight only)
                if !isBodyweight {
                    WeightAdjuster(
                        weight: vm.currentWeight,
                        onAdjust: { delta in vm.adjustWeight(delta: delta) }
                    )
                }

                // Timed exercise UI
                if isTimed {
                    TimerExerciseView(vm: vm, targetDuration: targetDuration)
                } else {
                    // Rep buttons
                    RepButtonGrid(vm: vm)
                }

                // Skip button
                Button {
                    vm.skipExercise()
                } label: {
                    HStack(spacing: 4) {
                        Text("HOPPA ÖVER")
                            .font(.mono(size: 11, weight: .medium))
                            .tracking(1)
                        Image(systemName: "forward.fill")
                            .font(.mono(size: 10))
                    }
                    .foregroundStyle(Theme.fgMuted)
                }
                .padding(.top, 8)
            }
            .padding()
            .opacity(vm.showTimerFlash ? 0.3 : 1.0)
            .animation(.easeInOut(duration: 0.2), value: vm.showTimerFlash)
        }
    }
}
