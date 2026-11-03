import SwiftUI

@main
struct ColorHurryApp: App {
    @State private var game = GameController()

    var body: some Scene {
        WindowGroup {
            ContentView(game: game)
                .preferredColorScheme(.dark)
        }
    }
}
