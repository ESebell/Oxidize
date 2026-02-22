import Foundation

func formatTime(_ totalSeconds: Int64) -> String {
    let mins = totalSeconds / 60
    let secs = totalSeconds % 60
    return String(format: "%d:%02d", mins, secs)
}

func formatDate(_ timestamp: Int64) -> String {
    let date = Date(timeIntervalSince1970: TimeInterval(timestamp))
    let now = Date()
    let calendar = Calendar.current

    if calendar.isDateInToday(date) {
        return "Idag"
    } else if calendar.isDateInYesterday(date) {
        return "Igår"
    } else {
        let days = calendar.dateComponents([.day], from: date, to: now).day ?? 0
        return "\(days) dgr sen"
    }
}

func formatWeight(_ weight: Double) -> String {
    if weight == weight.rounded() {
        return String(format: "%.0f", weight)
    } else {
        return String(format: "%.1f", weight)
    }
}

func parseTargetRange(_ target: String) -> (min: Int, max: Int) {
    let trimmed = target.trimmingCharacters(in: .whitespaces).uppercased()

    if trimmed == "AMRAP" {
        return (1, 30)
    }

    if trimmed.contains("-") {
        let parts = trimmed.split(separator: "-")
        if parts.count == 2,
           let min = Int(parts[0].trimmingCharacters(in: .whitespaces)),
           let max = Int(parts[1].trimmingCharacters(in: .whitespaces)) {
            return (min, max)
        }
    }

    if let n = Int(trimmed) {
        return (n, n)
    }

    return (5, 8)
}

func parseTargetReps(_ target: String) -> Int {
    parseTargetRange(target).min
}
