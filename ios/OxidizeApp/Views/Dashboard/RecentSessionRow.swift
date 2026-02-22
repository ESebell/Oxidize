import SwiftUI

struct RecentSessionRow: View {
    let session: Session

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text(session.routine)
                    .font(.mono(size: 14, weight: .bold))
                    .foregroundStyle(Theme.fgPrimary)
                Text(formatDate(session.timestamp))
                    .font(.mono(size: 12))
                    .foregroundStyle(Theme.fgSecondary)
            }
            Spacer()
            VStack(alignment: .trailing, spacing: 4) {
                Text(formatTime(session.durationSecs))
                    .font(.mono(size: 14))
                    .foregroundStyle(Theme.fgSecondary)
                Text("\(formatWeight(session.totalVolume)) kg")
                    .font(.mono(size: 12))
                    .foregroundStyle(Theme.fgMuted)
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
