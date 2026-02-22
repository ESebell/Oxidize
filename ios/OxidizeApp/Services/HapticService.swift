import UIKit

enum HapticService {
    static func impact(_ style: UIImpactFeedbackGenerator.FeedbackStyle = .medium) {
        UIImpactFeedbackGenerator(style: style).impactOccurred()
    }

    static func notification(_ type: UINotificationFeedbackGenerator.FeedbackType) {
        UINotificationFeedbackGenerator().notificationOccurred(type)
    }

    static func setCompleted() {
        impact(.medium)
    }

    static func workoutFinished() {
        notification(.success)
    }

    static func timerDone() {
        notification(.warning)
    }

    static func buttonTap() {
        impact(.light)
    }

    static func error() {
        notification(.error)
    }
}
