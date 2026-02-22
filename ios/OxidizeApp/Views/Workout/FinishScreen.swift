import SwiftUI

struct FinishScreen: View {
    @Bindable var vm: WorkoutViewModel
    @Binding var path: NavigationPath

    var body: some View {
        let stats = vm.finishStats

        ScrollView {
            VStack(spacing: 24) {
                Spacer(minLength: 40)

                Text("\u{2713}")
                    .font(.mono(size: 64, weight: .bold))
                    .foregroundStyle(Theme.accentA)

                Text("BRA JOBBAT!")
                    .font(.mono(size: 24, weight: .bold))
                    .foregroundStyle(Theme.fgPrimary)
                    .tracking(4)
                    .padding(.leading, 4)

                Text(formatTime(vm.elapsed))
                    .font(.mono(size: 20, weight: .bold))
                    .foregroundStyle(Theme.fgSecondary)

                HStack(spacing: 24) {
                    VStack(spacing: 4) {
                        Text(String(format: "%.0f kg", stats.volume))
                            .font(.mono(size: 18, weight: .bold))
                            .foregroundStyle(Theme.fgPrimary)
                        Text("VOLYM")
                            .font(.mono(size: 10, weight: .medium))
                            .foregroundStyle(Theme.fgMuted)
                            .tracking(2)
                    }
                    VStack(spacing: 4) {
                        Text("\(stats.calories) kcal")
                            .font(.mono(size: 18, weight: .bold))
                            .foregroundStyle(Theme.fgPrimary)
                        Text("KALORIER")
                            .font(.mono(size: 10, weight: .medium))
                            .foregroundStyle(Theme.fgMuted)
                            .tracking(2)
                    }
                }

                // RPE selector (1-10, Apple's effort scale)
                VStack(spacing: 8) {
                    Text("ANSTRÄNGNING")
                        .font(.mono(size: 10, weight: .medium))
                        .foregroundStyle(Theme.fgMuted)
                        .tracking(2)

                    VStack(spacing: 6) {
                        HStack(spacing: 6) {
                            ForEach(1...5, id: \.self) { level in
                                rpeButton(level)
                            }
                        }
                        HStack(spacing: 6) {
                            ForEach(6...10, id: \.self) { level in
                                rpeButton(level)
                            }
                        }
                    }

                    if let rpe = vm.selectedRPE {
                        Text("\(rpe) — \(effortLabel(rpe))")
                            .font(.mono(size: 13, weight: .bold))
                            .foregroundStyle(effortColor(rpe))
                            .padding(.top, 2)
                    }
                }

                if vm.isSaving {
                    ProgressView()
                        .tint(Theme.accentA)
                } else {
                    Button {
                        vm.saveWorkout()
                        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                            if !vm.showSyncWarning {
                                path.removeLast()
                            }
                        }
                    } label: {
                        Text("SPARA PASS")
                            .font(.mono(size: 16, weight: .bold))
                            .tracking(2)
                            .frame(maxWidth: .infinity)
                            .padding()
                            .background(Theme.accentA)
                            .foregroundStyle(Theme.bgPrimary)
                            .clipShape(RoundedRectangle(cornerRadius: 4))
                    }
                    .padding(.horizontal, 32)

                    Button {
                        vm.isFinished = false
                    } label: {
                        Text("TILLBAKA TILL PASSET")
                            .font(.mono(size: 12, weight: .medium))
                            .tracking(1)
                            .foregroundStyle(Theme.fgMuted)
                    }
                }

                Spacer(minLength: 40)
            }
        }
        .alert("Kunde inte spara till molnet", isPresented: $vm.showSyncWarning) {
            Button("Jag förstår") {
                path.removeLast()
            }
        } message: {
            Text("Passet är sparat lokalt men kunde inte synkas till Supabase efter 3 försök. Stäng inte appen förrän den har synkat.")
        }
    }

    @ViewBuilder
    private func rpeButton(_ level: Int) -> some View {
        let isSelected = vm.selectedRPE == level
        let color = effortColor(level)
        Button {
            vm.selectedRPE = vm.selectedRPE == level ? nil : level
        } label: {
            Text("\(level)")
                .font(.mono(size: 14, weight: .bold))
                .foregroundStyle(isSelected ? Theme.bgPrimary : (vm.selectedRPE == nil ? Theme.fgSecondary : Theme.fgMuted))
                .frame(width: 38, height: 38)
                .background(isSelected ? color : Color.clear)
                .overlay(
                    Rectangle()
                        .stroke(isSelected ? color : Theme.border, lineWidth: isSelected ? 2 : 1)
                )
        }
    }

    private func effortColor(_ level: Int) -> Color {
        switch level {
        case 1...3: return Theme.accentA       // Lätt — green
        case 4...6: return Color(.systemYellow) // Måttlig
        case 7...8: return Color(.systemOrange) // Svår
        default: return Theme.danger            // Max/Extrem
        }
    }

    private func effortLabel(_ level: Int) -> String {
        switch level {
        case 1...3: return "LÄTT"
        case 4...6: return "MÅTTLIG"
        case 7...8: return "SVÅR"
        default: return "MAXIMAL"
        }
    }
}
