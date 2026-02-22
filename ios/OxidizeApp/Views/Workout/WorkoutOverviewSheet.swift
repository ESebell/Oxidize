import SwiftUI

struct WorkoutOverviewSheet: View {
    @Bindable var vm: WorkoutViewModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ZStack {
                Theme.bgPrimary.ignoresSafeArea()

                List {
                    ForEach(Array(vm.exercises.enumerated()), id: \.offset) { index, ex in
                        let isDone = ex.setsCompleted.count >= ex.exercise.sets
                        let isCurrent = index == vm.currentIdx
                        let isStarted = !ex.setsCompleted.isEmpty

                        Button {
                            vm.jumpToExercise(idx: index)
                            dismiss()
                        } label: {
                            HStack {
                                // Status icon
                                if isDone {
                                    Image(systemName: "checkmark.circle.fill")
                                        .foregroundStyle(Theme.accentA)
                                } else if isCurrent {
                                    Image(systemName: "play.circle.fill")
                                        .foregroundStyle(Theme.accentA)
                                } else if isStarted {
                                    Image(systemName: "circle.lefthalf.filled")
                                        .foregroundStyle(Theme.accentA.opacity(0.5))
                                } else {
                                    Image(systemName: "circle")
                                        .foregroundStyle(Theme.fgMuted)
                                }

                                VStack(alignment: .leading) {
                                    Text(ex.exercise.name)
                                        .font(.mono(size: 14, weight: isCurrent ? .bold : .regular))
                                        .foregroundStyle(Theme.fgPrimary)

                                    if ex.exercise.isSuperset, let partner = ex.exercise.supersetWith {
                                        Text("SS: \(partner)")
                                            .font(.mono(size: 11))
                                            .foregroundStyle(Theme.accentB)
                                    }
                                }

                                Spacer()

                                Text("\(ex.setsCompleted.count)/\(ex.exercise.sets)")
                                    .font(.mono(size: 12))
                                    .foregroundStyle(Theme.fgSecondary)
                            }
                        }
                        .listRowBackground(Theme.bgCard)
                    }
                }
                .scrollContentBackground(.hidden)
            }
            .navigationTitle("PASS-ÖVERSIKT")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Stäng") { dismiss() }
                        .foregroundStyle(Theme.accentA)
                }
            }
        }
    }
}
