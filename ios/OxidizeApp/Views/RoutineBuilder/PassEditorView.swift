import SwiftUI

struct PassEditorView: View {
    @Bindable var vm: RoutineBuilderViewModel
    let passIdx: Int

    var body: some View {
        List {
            // Pass description
            Section {
                HStack {
                    Text("Namn:")
                        .font(.mono(size: 13))
                        .foregroundStyle(Theme.fgSecondary)
                    TextField("Max 8 tecken", text: Binding(
                        get: { vm.passes[passIdx].name },
                        set: { vm.passes[passIdx].name = String($0.prefix(8)) }
                    ))
                    .font(.mono(size: 13))
                    .foregroundStyle(Theme.fgPrimary)
                }
                .listRowBackground(Theme.bgCard)
                HStack {
                    Text("Beskrivning:")
                        .font(.mono(size: 13))
                        .foregroundStyle(Theme.fgSecondary)
                    TextField("T.ex. Rygg · Axlar", text: Binding(
                        get: { vm.passes[passIdx].description },
                        set: { vm.passes[passIdx].description = $0 }
                    ))
                    .font(.mono(size: 13))
                    .foregroundStyle(Theme.fgPrimary)
                }
                .listRowBackground(Theme.bgCard)
            }

            // Exercises
            Section {
                ForEach(vm.passes[passIdx].exercises.indices, id: \.self) { index in
                    ExerciseRowView(
                        exercise: $vm.passes[passIdx].exercises[index],
                        isFinisher: false,
                        onRemove: {
                            vm.removeExercise(passIdx: passIdx, exerciseIdx: index, isFinisher: false)
                        },
                        onSuperset: {
                            vm.supersetExerciseIdx = index
                            vm.showSupersetPicker = true
                        },
                        onUnlinkSuperset: {
                            vm.unlinkSuperset(exerciseIdx: index)
                        }
                    )
                    .listRowBackground(Theme.bgCard)
                }

                Button {
                    vm.addingToFinishers = false
                    vm.showExerciseSearch = true
                } label: {
                    HStack(spacing: 6) {
                        Image(systemName: "plus")
                            .font(.system(size: 12))
                        Text("LÄGG TILL ÖVNING")
                            .font(.mono(size: 11, weight: .medium))
                            .tracking(1)
                    }
                    .foregroundStyle(Theme.accentA)
                }
                .listRowBackground(Theme.bgCard)
            } header: {
                Text("ÖVNINGAR")
                    .font(.mono(size: 10, weight: .medium))
                    .tracking(2)
                    .foregroundStyle(Theme.fgMuted)
            }

            // Finishers
            Section {
                ForEach(vm.passes[passIdx].finishers.indices, id: \.self) { index in
                    ExerciseRowView(
                        exercise: $vm.passes[passIdx].finishers[index],
                        isFinisher: true,
                        onRemove: {
                            vm.removeExercise(passIdx: passIdx, exerciseIdx: index, isFinisher: true)
                        }
                    )
                    .listRowBackground(Theme.bgCard)
                }

                Button {
                    vm.addingToFinishers = true
                    vm.showExerciseSearch = true
                } label: {
                    HStack(spacing: 6) {
                        Image(systemName: "plus")
                            .font(.system(size: 12))
                        Text("LÄGG TILL FINISHER")
                            .font(.mono(size: 11, weight: .medium))
                            .tracking(1)
                    }
                    .foregroundStyle(Theme.accentB)
                }
                .listRowBackground(Theme.bgCard)
            } header: {
                Text("FINISHERS")
                    .font(.mono(size: 10, weight: .medium))
                    .tracking(2)
                    .foregroundStyle(Theme.fgMuted)
            }
        }
        .scrollContentBackground(.hidden)
        .background(Theme.bgPrimary)
        .sheet(isPresented: $vm.showSupersetPicker) {
            SupersetPickerSheet(vm: vm, passIdx: passIdx)
        }
    }
}
