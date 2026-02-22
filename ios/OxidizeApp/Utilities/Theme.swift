import SwiftUI

// MARK: - Design Tokens (matches style.css)

enum Theme {
    // Backgrounds
    static let bgPrimary = Color(hex: "#050505")
    static let bgSecondary = Color(hex: "#0c0c0c")
    static let bgCard = Color(hex: "#111111")

    // Foreground
    static let fgPrimary = Color(hex: "#f5f5f5")
    static let fgSecondary = Color(hex: "#888888")
    static let fgMuted = Color(hex: "#444444")

    // Accents
    static let accentA = Color(hex: "#00ff88")
    static let accentB = Color(hex: "#ff6600")

    // Border
    static let border = Color(hex: "#222222")

    // Status
    static let danger = Color(hex: "#ff4444")

    // Volume colors (weekly volume bars)
    static let volNone = Color(hex: "#444444")
    static let volLow = Color(hex: "#ff6600")
    static let volOptimal = Color(hex: "#00ff88")
    static let volHigh = Color(hex: "#ff4444")

    // Progression colors
    static let progressImproved = Color(hex: "#00ff88")
    static let progressMaintained = Color(hex: "#00ccff")
    static let progressRegressed = Color(hex: "#aaaaaa")
    static let progressNew = Color(hex: "#00aaff")
}

// MARK: - Color hex initializer

extension Color {
    init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet(charactersIn: "#"))
        let scanner = Scanner(string: hex)
        var rgb: UInt64 = 0
        scanner.scanHexInt64(&rgb)
        self.init(
            red: Double((rgb >> 16) & 0xFF) / 255,
            green: Double((rgb >> 8) & 0xFF) / 255,
            blue: Double(rgb & 0xFF) / 255
        )
    }
}

// MARK: - JetBrains Mono (same font as web app)

extension Font {
    static func mono(size: CGFloat, weight: Weight = .regular) -> Font {
        let name: String = switch weight {
        case .bold: "JetBrainsMono-Bold"
        case .semibold: "JetBrainsMono-SemiBold"
        case .medium: "JetBrainsMono-Medium"
        default: "JetBrainsMono-Regular"
        }
        return .custom(name, size: size)
    }
}

// MARK: - View Modifiers

struct CardStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .padding()
            .background(Theme.bgCard)
            .overlay(
                RoundedRectangle(cornerRadius: 4)
                    .stroke(Theme.border, lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 4))
    }
}

struct DarkInputStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .padding()
            .background(Theme.bgSecondary)
            .overlay(
                RoundedRectangle(cornerRadius: 4)
                    .stroke(Theme.border, lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 4))
            .foregroundStyle(Theme.fgPrimary)
    }
}

extension View {
    func cardStyle() -> some View {
        modifier(CardStyle())
    }

    func darkInputStyle() -> some View {
        modifier(DarkInputStyle())
    }

    func monoFont(size: CGFloat, weight: Font.Weight = .regular) -> some View {
        self.font(.mono(size: size, weight: weight))
    }

    func labelStyle() -> some View {
        self
            .font(.mono(size: 11, weight: .medium))
            .foregroundStyle(Theme.fgMuted)
            .textCase(.uppercase)
            .tracking(2)
    }
}
