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

                // RPE selector
                VStack(spacing: 8) {
                    Text("ANSTRÄNGNING")
                        .font(.mono(size: 10, weight: .medium))
                        .foregroundStyle(Theme.fgMuted)
                        .tracking(2)

                    HStack(spacing: 8) {
                        ForEach(1...5, id: \.self) { level in
                            let isSelected = vm.selectedRPE == level
                            Button {
                                if vm.selectedRPE == level {
                                    vm.selectedRPE = nil
                                } else {
                                    vm.selectedRPE = level
                                }
                            } label: {
                                Text("\(level)")
                                    .font(.mono(size: 16, weight: .bold))
                                    .foregroundStyle(isSelected ? Theme.bgPrimary : (vm.selectedRPE == nil ? Theme.fgSecondary : Theme.fgMuted))
                                    .frame(width: 44, height: 44)
                                    .background(isSelected ? Theme.accentA : Color.clear)
                                    .overlay(
                                        Rectangle()
                                            .stroke(isSelected ? Theme.accentA : Theme.border, lineWidth: isSelected ? 2 : 1)
                                    )
                            }
                        }
                    }

                    HStack {
                        Text("LÄTT")
                            .font(.mono(size: 9))
                            .foregroundStyle(Theme.fgMuted)
                        Spacer()
                        Text("MAXIMALT")
                            .font(.mono(size: 9))
                            .foregroundStyle(Theme.fgMuted)
                    }
                    .padding(.horizontal, 4)
                    .frame(width: CGFloat(5 * 44 + 4 * 8))
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
}
