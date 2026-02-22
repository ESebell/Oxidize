import SwiftUI

struct PausedWorkoutBanner: View {
    let routineName: String
    let onResume: () -> Void

    var body: some View {
        Button(action: onResume) {
            HStack {
                Image(systemName: "pause.circle.fill")
                    .font(.mono(size: 22))
                VStack(alignment: .leading, spacing: 2) {
                    Text("PAUSAT PASS")
                        .font(.mono(size: 12, weight: .bold))
                        .tracking(1)
                    Text(routineName)
                        .font(.mono(size: 12))
                        .opacity(0.8)
                }
                Spacer()
                Text("FORTSÄTT")
                    .font(.mono(size: 12, weight: .bold))
                    .tracking(1)
                Image(systemName: "chevron.right")
                    .font(.mono(size: 12))
            }
            .padding()
            .foregroundStyle(Theme.accentB)
            .background(Theme.accentB.opacity(0.1))
            .overlay(
                RoundedRectangle(cornerRadius: 4)
                    .stroke(Theme.accentB, lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 4))
        }
    }
}
