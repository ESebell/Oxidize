import SwiftUI

struct RestScreen: View {
    @Bindable var vm: WorkoutViewModel

    var body: some View {
        VStack(spacing: 32) {
            Spacer()

            Text("VILA")
                .font(.mono(size: 16, weight: .bold))
                .foregroundStyle(Theme.fgMuted)
                .tracking(4)
                .padding(.leading, 4)

            Text(formatTime(vm.restElapsed))
                .font(.mono(size: 56, weight: .bold))
                .foregroundStyle(Color(hex: "#ffaa00"))

            if let next = vm.currentExercise {
                VStack(spacing: 8) {
                    Text("NÄSTA")
                        .font(.mono(size: 10, weight: .medium))
                        .foregroundStyle(Theme.fgMuted)
                        .tracking(2)
                    Text(next.exercise.name.uppercased())
                        .font(.mono(size: 18, weight: .bold))
                        .foregroundStyle(Theme.fgPrimary)
                        .tracking(1)
                    Text("SET \(vm.currentSetNum) AV \(vm.totalSets)")
                        .font(.mono(size: 11))
                        .foregroundStyle(Theme.fgSecondary)
                        .tracking(1)
                }
            }

            Button {
                vm.continueWorkout()
            } label: {
                Text("FORTSÄTT")
                    .font(.mono(size: 16, weight: .bold))
                    .tracking(2)
                    .padding(.vertical, 16)
                    .padding(.horizontal, 48)
                    .background(Color.white)
                    .foregroundStyle(Theme.bgPrimary)
                    .clipShape(RoundedRectangle(cornerRadius: 4))
            }
            .padding(.horizontal, 32)

            if vm.justFinishedIdx != nil {
                Button {
                    vm.addExtraSet()
                } label: {
                    Text("+ LÄGG TILL SET")
                        .font(.mono(size: 12, weight: .medium))
                        .tracking(1)
                        .foregroundStyle(Theme.fgSecondary)
                }
            }

            Spacer()
        }
    }
}
