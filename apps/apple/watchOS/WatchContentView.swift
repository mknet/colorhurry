import App
import SwiftUI

/// The watch screen. The first page shows the LED ring with the mode line in its
/// centre. Skip and Apply are the bottom corner buttons. The second page shows the Bluetooth status
/// and the button that goes back to mode select.
struct WatchContentView: View {
    @State private var game = GameController()
    /// The visible page: 0 is the game page and 1 is the status page.
    @State private var page = 0

    var body: some View {
        NavigationStack {
            TabView(selection: $page) {
                gamePage.tag(0)
                statusPage.tag(1)
            }
            .tabViewStyle(.verticalPage)
        }
        .background(AppTheme.bg)
        .onAppear { game.boot() }
        .pausesInBackground(game)
    }

    private var gamePage: some View {
        LedRingView(leds: game.view.leds) {
            Text(game.watchHint)
                .font(.system(size: 16, weight: .semibold))
                .foregroundStyle(AppTheme.text)
                .lineLimit(2)
                .minimumScaleFactor(0.5)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .toolbar {
            ToolbarItemGroup(placement: .bottomBar) {
                Button(action: game.skip) {
                    Image(systemName: "forward.fill")
                }
                .tint(AppTheme.secondary)
                .accessibilityLabel("Skip")
                Spacer()
                Button(action: game.activate) {
                    Image(systemName: "checkmark")
                }
                .tint(AppTheme.primary)
                .accessibilityLabel("Apply")
            }
        }
    }

    private var statusPage: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Bluetooth")
                .font(.headline)
                .foregroundStyle(AppTheme.text)
            Text(game.bleStatus)
                .font(.system(size: 13))
                .foregroundStyle(AppTheme.muted)
            Button("Change mode") {
                game.reset()
                withAnimation { page = 0 }
            }
            .tint(AppTheme.secondary)
            .padding(.top, 6)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 6)
    }
}
