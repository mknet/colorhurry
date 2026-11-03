import SwiftUI

/// The shared `ContentView` with keyboard shortcuts for the two board buttons.
/// Skip uses A and the left arrow. Apply uses B and Return. Escape goes back
/// to mode select, as the Mode button does.
struct MacContentView: View {
    @State private var game = GameController()

    var body: some View {
        ContentView(game: game)
            .frame(minWidth: 360, minHeight: 520)
            .background {
                // Buttons with no size and no opacity hold the shortcuts,
                // because one button can have only one shortcut.
                ZStack {
                    shortcut("a", action: game.skip)
                    shortcut(.leftArrow, action: game.skip)
                    shortcut("b", action: game.activate)
                    shortcut(.return, action: game.activate)
                    shortcut(.escape, action: game.reset)
                }
                .opacity(0)
                .frame(width: 0, height: 0)
                .accessibilityHidden(true)
            }
    }

    private func shortcut(_ key: KeyEquivalent, action: @escaping () -> Void) -> some View {
        Button("", action: action)
            .keyboardShortcut(key, modifiers: [])
    }
}
