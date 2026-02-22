import SwiftUI

enum AppDestination: Hashable {
    case workout(passName: String)
    case stats
    case settings
    case routineBuilder(routineId: String?)
}

struct ContentView: View {
    @State private var authVM = AuthViewModel()
    @State private var path = NavigationPath()

    var body: some View {
        ZStack {
            Theme.bgPrimary.ignoresSafeArea()

            Group {
                if authVM.isAuthenticated {
                    NavigationStack(path: $path) {
                        DashboardView(authVM: authVM, path: $path)
                            .navigationDestination(for: AppDestination.self) { destination in
                                switch destination {
                                case .workout(let passName):
                                    WorkoutView(passName: passName, path: $path)
                                case .stats:
                                    StatsView()
                                case .settings:
                                    SettingsView(authVM: authVM, path: $path)
                                case .routineBuilder(let routineId):
                                    RoutineBuilderView(routineId: routineId, path: $path)
                                }
                            }
                    }
                    .tint(Theme.accentA)
                } else {
                    LoginView(authVM: authVM)
                }
            }
        }
        .task {
            await authVM.checkSession()
        }
    }
}
