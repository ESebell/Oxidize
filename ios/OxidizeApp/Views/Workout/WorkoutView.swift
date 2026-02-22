import SwiftUI

struct WorkoutView: View {
    let passName: String
    @Binding var path: NavigationPath
    @State private var vm = WorkoutViewModel()
    @State private var loaded = false

    var body: some View {
        Group {
            if loaded {
                WorkoutActiveView(vm: vm, path: $path)
            } else {
                VStack {
                    ProgressView()
                        .tint(Theme.accentA)
                    Text("LADDAR PASS...")
                        .font(.mono(size: 12, weight: .medium))
                        .foregroundStyle(Theme.fgMuted)
                        .tracking(2)
                        .padding(.top, 8)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(Theme.bgPrimary)
            }
        }
        .navigationBarBackButtonHidden(true)
        .onAppear {
            guard !loaded else { return }

            // Check for paused workout
            if let paused = StorageService.shared.loadPausedWorkout(), paused.routineName == passName {
                if var data = StorageService.shared.getWorkout(passName: passName) {
                    data.exercises = paused.exercises
                    vm.setup(data: data, resumedFrom: paused.currentExerciseIdx, startElapsed: paused.elapsedSecs)
                    StorageService.shared.clearPausedWorkout()
                    loaded = true
                    return
                }
            }

            StorageService.shared.clearPausedWorkout()

            if let data = StorageService.shared.getWorkout(passName: passName) {
                vm.setup(data: data)
                loaded = true
            }
        }
        .onDisappear {
            vm.cleanup()
        }
    }
}
