import SwiftUI

/// Pauses the game when the app goes to the background and resumes it when
/// the app is active again.
///
/// The modifier ignores the inactive phase. A lowered watch wrist or the iOS
/// Control Center makes the app inactive, and the game continues then.
struct PauseInBackground: ViewModifier {
    let game: GameController
    @Environment(\.scenePhase) private var scenePhase

    func body(content: Content) -> some View {
        content.onChange(of: scenePhase) { _, phase in
            switch phase {
            case .background: game.pause()
            case .active: game.resume()
            default: break
            }
        }
    }
}

extension View {
    /// Pauses `game` while the app is in the background.
    func pausesInBackground(_ game: GameController) -> some View {
        modifier(PauseInBackground(game: game))
    }
}
