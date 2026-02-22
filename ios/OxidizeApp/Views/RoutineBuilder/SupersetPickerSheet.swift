import SwiftUI

struct SupersetPickerSheet: View {
    @Bindable var vm: RoutineBuilderViewModel
    let passIdx: Int
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ZStack {
                Theme.bgPrimary.ignoresSafeArea()

                List {
                    let exercises = vm.passes[passIdx].exercises
                    ForEach(Array(exercises.enumerated()), id: \.offset) { index, exercise in
                        if index != vm.supersetExerciseIdx && !exercise.isSuperset {
                            Button {
                                if let sourceIdx = vm.supersetExerciseIdx {
                                    vm.linkSuperset(exerciseIdx: sourceIdx, partnerIdx: index)
                                }
                                dismiss()
                            } label: {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text(exercise.name)
                                        .font(.mono(size: 14, weight: .bold))
                                        .foregroundStyle(Theme.fgPrimary)
                                    Text("\(exercise.sets) x \(exercise.repsTarget)")
                                        .font(.mono(size: 12))
                                        .foregroundStyle(Theme.fgSecondary)
                                }
                            }
                            .listRowBackground(Theme.bgCard)
                        }
                    }
                }
                .scrollContentBackground(.hidden)
            }
            .navigationTitle("VÄLJ PARTNER")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Avbryt") { dismiss() }
                        .foregroundStyle(Theme.accentA)
                }
            }
        }
    }
}
