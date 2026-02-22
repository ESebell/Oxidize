import SwiftUI

@main
struct OxidizeApp: App {
    init() {
        let monoFont = UIFont(name: "JetBrainsMono-Bold", size: 17)
            ?? UIFont.monospacedSystemFont(ofSize: 17, weight: .bold)
        let largeMono = UIFont(name: "JetBrainsMono-Bold", size: 17)
            ?? UIFont.monospacedSystemFont(ofSize: 17, weight: .bold)

        let navAppearance = UINavigationBarAppearance()
        navAppearance.configureWithTransparentBackground()
        navAppearance.backgroundColor = UIColor(Theme.bgPrimary)
        navAppearance.titleTextAttributes = [
            .font: monoFont,
            .foregroundColor: UIColor(Theme.fgPrimary)
        ]
        navAppearance.largeTitleTextAttributes = [
            .font: largeMono,
            .foregroundColor: UIColor(Theme.fgPrimary)
        ]

        UINavigationBar.appearance().standardAppearance = navAppearance
        UINavigationBar.appearance().compactAppearance = navAppearance
        UINavigationBar.appearance().scrollEdgeAppearance = navAppearance
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .preferredColorScheme(.dark)
        }
    }
}
