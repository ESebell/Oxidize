import SwiftUI

struct WeightAdjuster: View {
    let weight: Double
    let onAdjust: (Double) -> Void

    var body: some View {
        HStack(spacing: 24) {
            Button {
                HapticService.buttonTap()
                onAdjust(-2.5)
            } label: {
                Image(systemName: "minus")
                    .font(.mono(size: 20, weight: .bold))
                    .foregroundStyle(Theme.fgSecondary)
                    .frame(width: 50, height: 50)
                    .overlay(
                        RoundedRectangle(cornerRadius: 4)
                            .stroke(Theme.border, lineWidth: 1)
                    )
            }

            VStack(spacing: 2) {
                Text(formatWeight(weight))
                    .font(.mono(size: 48, weight: .bold))
                    .foregroundStyle(Theme.fgPrimary)
                Text("KG")
                    .font(.mono(size: 12, weight: .medium))
                    .foregroundStyle(Theme.fgSecondary)
                    .tracking(2)
            }
            .frame(minWidth: 120)

            Button {
                HapticService.buttonTap()
                onAdjust(2.5)
            } label: {
                Image(systemName: "plus")
                    .font(.mono(size: 20, weight: .bold))
                    .foregroundStyle(Theme.fgSecondary)
                    .frame(width: 50, height: 50)
                    .overlay(
                        RoundedRectangle(cornerRadius: 4)
                            .stroke(Theme.border, lineWidth: 1)
                    )
            }
        }
    }
}
