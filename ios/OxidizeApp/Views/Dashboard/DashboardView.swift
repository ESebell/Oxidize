import SwiftUI

struct DashboardView: View {
    @Bindable var authVM: AuthViewModel
    @Binding var path: NavigationPath
    @State private var vm = DashboardViewModel()

    var body: some View {
        ZStack {
            Theme.bgPrimary.ignoresSafeArea()

            ScrollView {
                VStack(spacing: 24) {
                    // Logo + settings
                    Text("OXIDIZE")
                        .font(.mono(size: 14, weight: .bold))
                        .foregroundStyle(Theme.fgMuted)
                        .tracking(6)
                        .padding(.trailing, -6)
                        .frame(maxWidth: .infinity)
                        .overlay(alignment: .trailing) {
                            Button {
                                path.append(AppDestination.settings)
                            } label: {
                                Image(systemName: "gearshape")
                                    .font(.system(size: 16))
                                    .foregroundStyle(Theme.fgMuted)
                            }
                        }
                        .padding(.horizontal)
                        .padding(.top, 8)

                    // Stats
                    HStack(spacing: 40) {
                        VStack(spacing: 2) {
                            Text("\(vm.totalStats.totalSessions)")
                                .font(.mono(size: 36, weight: .bold))
                                .foregroundStyle(Theme.fgPrimary)
                            Text("PASS")
                                .font(.mono(size: 11, weight: .medium))
                                .foregroundStyle(Theme.fgMuted)
                                .tracking(2)
                        }
                        VStack(spacing: 2) {
                            Text("\(Int(vm.totalStats.totalVolume / 1000))")
                                .font(.mono(size: 36, weight: .bold))
                                .foregroundStyle(Theme.fgPrimary)
                            Text("TON")
                                .font(.mono(size: 11, weight: .medium))
                                .foregroundStyle(Theme.fgMuted)
                                .tracking(2)
                        }
                    }

                    // Paused workout banner
                    if let paused = vm.pausedWorkout {
                        PausedWorkoutBanner(routineName: paused.routineName) {
                            if let passName = vm.resumeWorkout() {
                                path.append(AppDestination.workout(passName: passName))
                            }
                        }
                        .padding(.horizontal)
                    }

                    // Pass buttons — sharp corners
                    if let routine = vm.activeRoutine {
                        VStack(spacing: 12) {
                            ForEach(Array(routine.passes.enumerated()), id: \.offset) { index, pass in
                                PassButton(
                                    name: pass.name,
                                    description: pass.description,
                                    color: Color.passColor(index)
                                ) {
                                    if let passName = vm.startWorkout(passName: pass.name) {
                                        path.append(AppDestination.workout(passName: passName))
                                    }
                                }
                            }
                        }
                        .padding(.horizontal)
                    }

                    Spacer().frame(height: 8)

                    // Recent session — only the latest, no card
                    if let latest = vm.recentSessions.first {
                        VStack(alignment: .leading, spacing: 8) {
                            Text("SENASTE")
                                .font(.mono(size: 11, weight: .medium))
                                .foregroundStyle(Theme.fgMuted)
                                .tracking(2)

                            HStack {
                                Text(latest.routine)
                                    .font(.mono(size: 14, weight: .bold))
                                    .foregroundStyle(passColorForSession(latest))
                                Spacer()
                                Text(formatDateShort(latest.timestamp))
                                    .font(.mono(size: 14))
                                    .foregroundStyle(Theme.fgSecondary)
                                Spacer()
                                Text(formatTime(latest.durationSecs))
                                    .font(.mono(size: 14))
                                    .foregroundStyle(Theme.fgMuted)
                            }

                            Rectangle()
                                .fill(Theme.border)
                                .frame(height: 1)
                        }
                        .padding(.horizontal)
                    }

                    // Statistik button
                    Button {
                        path.append(AppDestination.stats)
                    } label: {
                        Text("Statistik →")
                            .font(.mono(size: 14, weight: .medium))
                            .foregroundStyle(Theme.fgMuted)
                            .frame(maxWidth: .infinity)
                            .padding(.vertical, 14)
                            .overlay(
                                Rectangle()
                                    .stroke(Theme.border, lineWidth: 1)
                            )
                    }
                    .padding(.horizontal, 60)

                    Spacer().frame(height: 8)

                    // Logged in + logout
                    VStack(spacing: 10) {
                        Rectangle()
                            .fill(Theme.border)
                            .frame(width: 120, height: 1)
                        Text("inloggad: \(vm.displayName)")
                            .font(.mono(size: 12))
                            .foregroundStyle(Theme.fgMuted)
                        Button {
                            Task {
                                await authVM.logout()
                            }
                        } label: {
                            Text("logga ut")
                                .font(.mono(size: 14))
                                .foregroundStyle(Color(hex: "#00ccff"))
                        }
                    }
                }
                .padding(.vertical)
            }
        }
        .navigationBarHidden(true)
        .task {
            await vm.loadData()
        }
        .refreshable {
            await vm.loadData()
        }
        .alert("Starta nytt pass?", isPresented: $vm.showConfirmDialog) {
            Button("Nej, fortsätt pausat", role: .cancel) {}
            Button("Ja, börja om") {
                let passName = vm.confirmStartNew()
                path.append(AppDestination.workout(passName: passName))
            }
        } message: {
            Text("Du har ett pausat pass. Vill du avbryta det och starta ett nytt?")
        }
    }

    private func passColorForSession(_ session: Session) -> Color {
        guard let routine = vm.activeRoutine else { return Theme.fgPrimary }
        if let index = routine.passes.firstIndex(where: { $0.name == session.routine }) {
            return Color.passColor(index)
        }
        return Theme.fgPrimary
    }

    private func formatDateShort(_ timestamp: Int64) -> String {
        let date = Date(timeIntervalSince1970: TimeInterval(timestamp))
        let calendar = Calendar.current
        if calendar.isDateInToday(date) {
            return "Idag"
        } else if calendar.isDateInYesterday(date) {
            return "Igår"
        } else {
            let formatter = DateFormatter()
            formatter.dateFormat = "d MMM"
            formatter.locale = Locale(identifier: "sv_SE")
            return formatter.string(from: date)
        }
    }
}

struct PassButton: View {
    let name: String
    let description: String
    let color: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 4) {
                Text(name.uppercased())
                    .font(.mono(size: 20, weight: .bold))
                    .tracking(3)
                    .padding(.leading, 3)
                if !description.isEmpty {
                    Text(description)
                        .font(.mono(size: 11))
                        .opacity(0.6)
                }
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 20)
            .foregroundStyle(color)
            .overlay(
                Rectangle()
                    .stroke(color, lineWidth: 2)
            )
        }
    }
}
