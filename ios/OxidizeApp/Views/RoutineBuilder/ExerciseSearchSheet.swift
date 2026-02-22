import SwiftUI

struct ExerciseSearchSheet: View {
    @Bindable var vm: RoutineBuilderViewModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ZStack {
                Theme.bgPrimary.ignoresSafeArea()

                VStack {
                    HStack {
                        TextField("Sök övning...", text: $vm.searchQuery)
                            .autocorrectionDisabled()
                            .darkInputStyle()

                        Button {
                            Task { await vm.searchExercises() }
                        } label: {
                            Text("SÖK")
                                .font(.mono(size: 12, weight: .bold))
                                .tracking(1)
                                .foregroundStyle(Theme.bgPrimary)
                                .padding(.horizontal, 16)
                                .padding(.vertical, 14)
                                .background(Theme.accentA)
                                .clipShape(RoundedRectangle(cornerRadius: 8))
                        }
                    }
                    .padding()

                    if vm.isSearching {
                        ProgressView()
                            .tint(Theme.accentA)
                        Spacer()
                    } else {
                        List(vm.searchResults) { exercise in
                            Button {
                                vm.addExercise(from: exercise)
                                dismiss()
                            } label: {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text(exercise.name)
                                        .font(.mono(size: 14, weight: .bold))
                                        .foregroundStyle(Theme.fgPrimary)

                                    if !exercise.primaryMuscles.isEmpty {
                                        Text(exercise.primaryMuscles.joined(separator: ", "))
                                            .font(.mono(size: 11))
                                            .foregroundStyle(Theme.fgSecondary)
                                    }

                                    if let equipment = exercise.equipment {
                                        Text(equipment)
                                            .font(.mono(size: 10))
                                            .foregroundStyle(Theme.fgMuted)
                                    }
                                }
                            }
                            .listRowBackground(Theme.bgCard)
                        }
                        .scrollContentBackground(.hidden)
                    }
                }
            }
            .navigationTitle("SÖK ÖVNING")
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
