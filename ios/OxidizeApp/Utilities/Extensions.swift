import Foundation
import SwiftUI

extension Color {
    static let passColors: [Color] = [
        Color(hex: "#00ff88"),  // pass-a green
        Color(hex: "#ff6600"),  // pass-b orange
        Color(hex: "#00ccff"),  // pass-c cyan
        Color(hex: "#ff66cc"),  // pass-d magenta
        Color(hex: "#ffcc00"),  // pass-e yellow
    ]

    static func passColor(_ index: Int) -> Color {
        passColors[index % passColors.count]
    }
}

extension String {
    func uuidSimple() -> String {
        let now = UInt64(Date().timeIntervalSince1970 * 1000)
        let random = UInt64.random(in: 0..<1_000_000)
        return String(format: "%llx%llx", now, random)
    }
}

func generateId() -> String {
    let now = UInt64(Date().timeIntervalSince1970 * 1000)
    let random = UInt64.random(in: 0..<1_000_000)
    return String(format: "%llx%llx", now, random)
}

func currentTimestamp() -> Int64 {
    Int64(Date().timeIntervalSince1970)
}
