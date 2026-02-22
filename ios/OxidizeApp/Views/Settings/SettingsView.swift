import SwiftUI

struct SettingsView: View {
    @Bindable var authVM: AuthViewModel
    @Binding var path: NavigationPath
    @State private var vm = SettingsViewModel()

    var body: some View {
        ZStack {
            Theme.bgPrimary.ignoresSafeArea()

            ScrollView {
                VStack(spacing: 16) {
                    // Routines
                    VStack(alignment: .leading, spacing: 10) {
                        Text("RUTINER")
                            .labelStyle()
                            .padding(.horizontal)

                        ForEach(vm.routines) { routine in
                            HStack {
                                VStack(alignment: .leading, spacing: 4) {
                                    HStack(spacing: 8) {
                                        Text(routine.name)
                                            .font(.mono(size: 14, weight: .bold))
                                            .foregroundStyle(Theme.fgPrimary)
                                        if routine.isActive {
                                            Text("AKTIV")
                                                .font(.mono(size: 9, weight: .bold))
                                                .tracking(1)
                                                .padding(.horizontal, 6)
                                                .padding(.vertical, 2)
                                                .background(Theme.accentA.opacity(0.15))
                                                .foregroundStyle(Theme.accentA)
                                                .clipShape(RoundedRectangle(cornerRadius: 3))
                                        }
                                    }
                                    Text("\(routine.passes.count) pass")
                                        .font(.mono(size: 11))
                                        .foregroundStyle(Theme.fgSecondary)
                                }

                                Spacer()

                                if !routine.isActive {
                                    Button {
                                        Task { await vm.setActiveRoutine(id: routine.id) }
                                    } label: {
                                        Text("AKTIVERA")
                                            .font(.mono(size: 10, weight: .medium))
                                            .tracking(1)
                                            .foregroundStyle(Theme.accentA)
                                            .padding(.horizontal, 10)
                                            .padding(.vertical, 6)
                                            .overlay(
                                                RoundedRectangle(cornerRadius: 4)
                                                    .stroke(Theme.accentA, lineWidth: 1)
                                            )
                                    }
                                }

                                Button {
                                    path.append(AppDestination.routineBuilder(routineId: routine.id))
                                } label: {
                                    Image(systemName: "pencil")
                                        .font(.mono(size: 12))
                                        .foregroundStyle(Theme.fgMuted)
                                }
                            }
                            .padding()
                            .background(routine.isActive ? Theme.accentA.opacity(0.05) : Theme.bgCard)
                            .overlay(
                                RoundedRectangle(cornerRadius: 4)
                                    .stroke(routine.isActive ? Theme.accentA : Theme.border, lineWidth: routine.isActive ? 2 : 1)
                            )
                            .clipShape(RoundedRectangle(cornerRadius: 4))
                            .padding(.horizontal)
                        }

                        Button {
                            path.append(AppDestination.routineBuilder(routineId: nil))
                        } label: {
                            HStack(spacing: 6) {
                                Image(systemName: "plus")
                                    .font(.mono(size: 12))
                                Text("SKAPA NY RUTIN")
                                    .font(.mono(size: 12, weight: .medium))
                                    .tracking(1)
                            }
                            .foregroundStyle(Theme.accentA)
                            .frame(maxWidth: .infinity)
                            .padding()
                            .overlay(
                                RoundedRectangle(cornerRadius: 4)
                                    .stroke(Theme.border, lineWidth: 1)
                            )
                        }
                        .padding(.horizontal)
                    }

                    // Bodyweight
                    VStack(alignment: .leading, spacing: 8) {
                        Text("KROPPSVIKT")
                            .labelStyle()
                            .padding(.horizontal)

                        Group {
                            if vm.editingWeight {
                                HStack {
                                    TextField("Vikt (kg)", text: $vm.weightInput)
                                        .keyboardType(.decimalPad)
                                        .darkInputStyle()
                                    Button {
                                        Task { await vm.saveBodyweight() }
                                    } label: {
                                        Text("SPARA")
                                            .font(.mono(size: 11, weight: .bold))
                                            .tracking(1)
                                            .foregroundStyle(Theme.bgPrimary)
                                            .padding(.horizontal, 14)
                                            .padding(.vertical, 12)
                                            .background(Theme.accentA)
                                            .clipShape(RoundedRectangle(cornerRadius: 4))
                                    }
                                    Button {
                                        vm.editingWeight = false
                                    } label: {
                                        Text("AVBRYT")
                                            .font(.mono(size: 11))
                                            .foregroundStyle(Theme.fgMuted)
                                    }
                                }
                            } else {
                                HStack {
                                    Text(vm.bodyweight.map { "\(formatWeight($0)) kg" } ?? "Ej angiven")
                                        .font(.mono(size: 14))
                                        .foregroundStyle(Theme.fgPrimary)
                                    Spacer()
                                    Button {
                                        vm.weightInput = vm.bodyweight.map { formatWeight($0) } ?? ""
                                        vm.editingWeight = true
                                    } label: {
                                        Text("ÄNDRA")
                                            .font(.mono(size: 11, weight: .medium))
                                            .tracking(1)
                                            .foregroundStyle(Theme.fgSecondary)
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
                        .padding(.horizontal)
                    }

                    // Display name
                    VStack(alignment: .leading, spacing: 8) {
                        Text("VISNINGSNAMN")
                            .labelStyle()
                            .padding(.horizontal)

                        Group {
                            if vm.editingName {
                                HStack {
                                    TextField("Namn", text: $vm.nameInput)
                                        .darkInputStyle()
                                    Button {
                                        Task { await vm.saveDisplayName() }
                                    } label: {
                                        Text("SPARA")
                                            .font(.mono(size: 11, weight: .bold))
                                            .tracking(1)
                                            .foregroundStyle(Theme.bgPrimary)
                                            .padding(.horizontal, 14)
                                            .padding(.vertical, 12)
                                            .background(Theme.accentA)
                                            .clipShape(RoundedRectangle(cornerRadius: 4))
                                    }
                                    Button {
                                        vm.editingName = false
                                    } label: {
                                        Text("AVBRYT")
                                            .font(.mono(size: 11))
                                            .foregroundStyle(Theme.fgMuted)
                                    }
                                }
                            } else {
                                HStack {
                                    Text(vm.displayName.isEmpty ? "Ej angivet" : vm.displayName)
                                        .font(.mono(size: 14))
                                        .foregroundStyle(Theme.fgPrimary)
                                    Spacer()
                                    Button {
                                        vm.nameInput = vm.displayName
                                        vm.editingName = true
                                    } label: {
                                        Text("ÄNDRA")
                                            .font(.mono(size: 11, weight: .medium))
                                            .tracking(1)
                                            .foregroundStyle(Theme.fgSecondary)
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
                        .padding(.horizontal)
                    }

                    // Account
                    VStack(alignment: .leading, spacing: 8) {
                        Text("KONTO")
                            .labelStyle()
                            .padding(.horizontal)

                        VStack(spacing: 12) {
                            if let session = authVM.currentSession {
                                HStack {
                                    Text(session.user.email)
                                        .font(.mono(size: 13))
                                        .foregroundStyle(Theme.fgSecondary)
                                    Spacer()
                                }
                            }

                            Button {
                                Task { await authVM.logout() }
                            } label: {
                                Text("LOGGA UT")
                                    .font(.mono(size: 12, weight: .medium))
                                    .tracking(1)
                                    .foregroundStyle(Theme.danger)
                                    .frame(maxWidth: .infinity)
                                    .padding()
                                    .overlay(
                                        RoundedRectangle(cornerRadius: 4)
                                            .stroke(Theme.fgMuted, lineWidth: 1)
                                    )
                            }
                        }
                        .padding()
                        .background(Theme.bgCard)
                        .overlay(
                            RoundedRectangle(cornerRadius: 4)
                                .stroke(Theme.border, lineWidth: 1)
                        )
                        .clipShape(RoundedRectangle(cornerRadius: 4))
                        .padding(.horizontal)
                    }
                }
                .padding(.vertical)
            }
        }
        .navigationTitle("INSTÄLLNINGAR")
        .navigationBarTitleDisplayMode(.inline)
        .task { await vm.loadSettings() }
    }
}
