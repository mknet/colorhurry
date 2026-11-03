import SwiftUI

@main
struct ColorHurryMacApp: App {
    var body: some Scene {
        WindowGroup {
            MacContentView()
                .preferredColorScheme(.dark)
        }
        .defaultSize(width: 480, height: 640)
    }
}
