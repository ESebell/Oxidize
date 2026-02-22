import SwiftUI

struct RoutineBuilderView: View {
    let routineId: String?
    @Binding var path: NavigationPath
    @State private var vm = RoutineBuilderViewModel()

    var body: some View {
        ZStack {
            Theme.bgPrimary.ignoresSafeArea()

            VStack(spacing: 0) {
                // Name and focus
                VStack(spacing: 8) {
                    TextField("Rutinnamn", text: $vm.routineName)
                        .font(.mono(size: 16, weight: .bold))
                        .darkInputStyle()
                    TextField("Fokus (t.ex. Styrka & Hypertrofi)", text: $vm.routineFocus)
                        .font(.mono(size: 14))
                        .darkInputStyle()
                }
                .padding()

                // Pass tabs
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        ForEach(Array(vm.passes.enumerated()), id: \.offset) { index, pass in
                            Button {
                                vm.selectedPassIdx = index
                            } label: {
                                Text(pass.name.uppercased())
                                    .font(.mono(size: 12, weight: .bold))
                                    .tracking(1)
                                    .padding(.horizontal, 16)
                                    .padding(.vertical, 8)
                                    .background(index == vm.selectedPassIdx ? Theme.accentA : Color.clear)
                                    .foregroundStyle(index == vm.selectedPassIdx ? Theme.bgPrimary : Theme.fgSecondary)
                                    .overlay(
                                        RoundedRectangle(cornerRadius: 4)
                                            .stroke(index == vm.selectedPassIdx ? Theme.accentA : Theme.border, lineWidth: 1)
                                    )
                                    .clipShape(RoundedRectangle(cornerRadius: 4))
                            }
                        }

                        Button {
                            vm.addPass()
                        } label: {
                            Image(systemName: "plus")
                                .font(.mono(size: 12))
                                .foregroundStyle(Theme.accentA)
                                .padding(.horizontal, 8)
                        }
                    }
                    .padding(.horizontal)
                }

                // Pass editor
                if vm.selectedPassIdx < vm.passes.count {
                    PassEditorView(vm: vm, passIdx: vm.selectedPassIdx)
                }

                // Bottom actions
                HStack {
                    if vm.routineId != nil {
                        Button {
                            vm.showDeleteConfirm = true
                        } label: {
                            Text("RADERA")
                                .font(.mono(size: 12, weight: .medium))
                                .tracking(1)
                                .foregroundStyle(Theme.danger)
                        }
                    }

                    Spacer()

                    Button {
                        vm.showAIWizard = true
                    } label: {
                        Text("AI-WIZARD")
                            .font(.mono(size: 12, weight: .medium))
                            .tracking(1)
                            .foregroundStyle(Theme.fgSecondary)
                            .padding(.horizontal, 14)
                            .padding(.vertical, 8)
                            .overlay(
                                RoundedRectangle(cornerRadius: 4)
                                    .stroke(Theme.border, lineWidth: 1)
                            )
                    }

                    Button {
                        Task {
                            await vm.saveRoutine()
                            path.removeLast()
                        }
                    } label: {
                        if vm.isSaving {
                            ProgressView()
                                .tint(Theme.bgPrimary)
                        } else {
                            Text("SPARA")
                                .font(.mono(size: 14, weight: .bold))
                                .tracking(2)
                        }
                    }
                    .foregroundStyle(Theme.bgPrimary)
                    .padding(.horizontal, 20)
                    .padding(.vertical, 10)
                    .background(Theme.accentA)
                    .clipShape(RoundedRectangle(cornerRadius: 4))
                    .disabled(vm.isSaving || vm.routineName.isEmpty)
                    .opacity(vm.routineName.isEmpty ? 0.5 : 1)
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
        .navigationTitle(routineId != nil ? "REDIGERA RUTIN" : "NY RUTIN")
        .navigationBarTitleDisplayMode(.inline)
        .task { await vm.loadRoutine(id: routineId) }
        .sheet(isPresented: $vm.showExerciseSearch) {
            ExerciseSearchSheet(vm: vm)
        }
        .sheet(isPresented: $vm.showAIWizard) {
            AIWizardSheet(vm: vm)
        }
        .alert("Radera rutin?", isPresented: $vm.showDeleteConfirm) {
            Button("Avbryt", role: .cancel) {}
            Button("Radera", role: .destructive) {
                Task {
                    await vm.deleteRoutine()
                    path.removeLast()
                }
            }
        }
    }
}
