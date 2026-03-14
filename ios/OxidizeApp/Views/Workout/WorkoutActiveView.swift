import SwiftUI

struct WorkoutActiveView: View {
    @Bindable var vm: WorkoutViewModel
    @Binding var path: NavigationPath

    var body: some View {
        ZStack {
            Theme.bgPrimary.ignoresSafeArea()

            VStack(spacing: 0) {
                // Header
                HStack {
                    Text(vm.routineName.uppercased())
                        .font(.mono(size: 14, weight: .bold))
                        .foregroundStyle(Theme.accentA)
                        .tracking(2)

                    Spacer()

                    // Progress dots
                    Button { vm.showOverview = true } label: {
                        HStack(spacing: 4) {
                            ForEach(0..<vm.totalExercises, id: \.self) { i in
                                Circle()
                                    .fill(dotColor(for: i))
                                    .frame(width: 8, height: 8)
                                    .shadow(color: i == vm.currentIdx ? Theme.accentA.opacity(0.8) : .clear, radius: 4)
                            }
                        }
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                        .background(Color.white.opacity(0.05))
                        .clipShape(RoundedRectangle(cornerRadius: 4))
                    }

                    Spacer()

                    Text(formatTime(vm.elapsed))
                        .font(.mono(size: 14, weight: .bold))
                        .foregroundStyle(Theme.fgSecondary)
                }
                .padding()
                .background(Theme.bgSecondary)
                .overlay(
                    Rectangle()
                        .frame(height: 1)
                        .foregroundStyle(Theme.border),
                    alignment: .bottom
                )

                // Main content
                if vm.isFinished {
                    FinishScreen(vm: vm, path: $path)
                } else if vm.isResting {
                    RestScreen(vm: vm)
                } else {
                    ExerciseScreen(vm: vm)
                }

                // Footer (only if not finished)
                if !vm.isFinished {
                    HStack {
                        Button {
                            vm.pauseAndExit()
                            path.removeLast()
                        } label: {
                            HStack(spacing: 6) {
                                Image(systemName: "pause.fill")
                                    .font(.mono(size: 12))
                                Text("PAUSA")
                                    .font(.mono(size: 12, weight: .medium))
                                    .tracking(1)
                            }
                            .foregroundStyle(Theme.fgSecondary)
                            .padding(.horizontal, 16)
                            .padding(.vertical, 10)
                            .overlay(
                                RoundedRectangle(cornerRadius: 4)
                                    .stroke(Theme.border, lineWidth: 1)
                            )
                        }

                        Spacer()

                        Button {
                            vm.showCancelConfirm = true
                        } label: {
                            Text("AVSLUTA")
                                .font(.mono(size: 12, weight: .medium))
                                .tracking(1)
                                .foregroundStyle(Theme.danger)
                        }
                    }
                    .padding()
                    .background(Theme.bgSecondary)
                    .overlay(
                        Rectangle()
                            .frame(height: 1)
                            .foregroundStyle(Theme.border),
                        alignment: .top
                    )
                }
            }
        }
        .sheet(isPresented: $vm.showOverview) {
            WorkoutOverviewSheet(vm: vm)
        }
        .overlay {
            if vm.showCancelConfirm {
                CancelWorkoutModal(
                    hasLoggedSets: vm.exercises.contains { !$0.setsCompleted.isEmpty },
                    onCancel: {
                        vm.showCancelConfirm = false
                    },
                    onSaveAndFinish: {
                        vm.showCancelConfirm = false
                        vm.isFinished = true
                    },
                    onDiscard: {
                        vm.showCancelConfirm = false
                        vm.cancelWorkout()
                        path.removeLast()
                    }
                )
            }
        }
    }

    private func dotColor(for index: Int) -> Color {
        let ex = vm.exercises[safe: index]
        let isDone = (ex?.setsCompleted.count ?? 0) >= (ex?.exercise.sets ?? 0)
        let isCurrent = index == vm.currentIdx
        let isStarted = !(ex?.setsCompleted.isEmpty ?? true)

        if isDone { return Theme.accentA }
        if isCurrent { return Theme.accentA }
        if isStarted { return Theme.accentA.opacity(0.5) }
        return Theme.fgMuted
    }
}

extension Array {
    subscript(safe index: Index) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}

struct CancelWorkoutModal: View {
    let hasLoggedSets: Bool
    let onCancel: () -> Void
    let onSaveAndFinish: () -> Void
    let onDiscard: () -> Void

    var body: some View {
        ZStack {
            Color.black.opacity(0.7)
                .ignoresSafeArea()
                .onTapGesture { onCancel() }

            VStack(spacing: 0) {
                VStack(spacing: 8) {
                    Text("AVSLUTA PASS?")
                        .font(.mono(size: 16, weight: .bold))
                        .foregroundStyle(Theme.fgPrimary)
                        .tracking(2)
                }
                .padding(.top, 24)
                .padding(.bottom, 16)

                if hasLoggedSets {
                    Text("Du har loggade set. Vill du spara eller slänga passet?")
                        .font(.mono(size: 13))
                        .foregroundStyle(Theme.fgSecondary)
                        .multilineTextAlignment(.center)
                        .padding(.horizontal, 24)
                        .padding(.bottom, 24)
                } else {
                    Text("Inga set har loggats.")
                        .font(.mono(size: 13))
                        .foregroundStyle(Theme.fgSecondary)
                        .multilineTextAlignment(.center)
                        .padding(.horizontal, 24)
                        .padding(.bottom, 24)
                }

                Rectangle()
                    .fill(Theme.border)
                    .frame(height: 1)

                // Continue
                Button {
                    onCancel()
                } label: {
                    Text("FORTSÄTT TRÄNA")
                        .font(.mono(size: 13, weight: .bold))
                        .tracking(1)
                        .foregroundStyle(Theme.accentA)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 16)
                }

                if hasLoggedSets {
                    Rectangle()
                        .fill(Theme.border)
                        .frame(height: 1)

                    Button {
                        onSaveAndFinish()
                    } label: {
                        Text("SPARA & AVSLUTA")
                            .font(.mono(size: 13, weight: .bold))
                            .tracking(1)
                            .foregroundStyle(Theme.fgPrimary)
                            .frame(maxWidth: .infinity)
                            .padding(.vertical, 16)
                    }
                }

                Rectangle()
                    .fill(Theme.border)
                    .frame(height: 1)

                Button {
                    onDiscard()
                } label: {
                    Text("SLÄNG PASSET")
                        .font(.mono(size: 13, weight: .bold))
                        .tracking(1)
                        .foregroundStyle(Theme.danger)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 16)
                }
            }
            .background(Theme.bgCard)
            .overlay(
                Rectangle()
                    .stroke(Theme.border, lineWidth: 1)
            )
            .padding(.horizontal, 40)
        }
        .transition(.opacity)
        .animation(.easeInOut(duration: 0.15), value: true)
    }
}
