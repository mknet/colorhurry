import App
import SwiftUI

/// The watch screen. The first page shows the LED ring with the mode line in its
/// centre. Skip and Apply are the bottom corner buttons. The second page shows the Bluetooth status.
struct WatchContentView: View {
    @State private var game = GameController()

    var body: some View {
        NavigationStack {
            TabView {
                gamePage
                statusPage
            }
            .tabViewStyle(.verticalPage)
        }
        .background(AppTheme.bg)
        .onAppear { game.boot() }
    }

    private var gamePage: some View {
        LedRingView(leds: game.view.leds) {
            Text(game.modeHint)
                .font(.system(size: 11))
                .foregroundStyle(AppTheme.muted)
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
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 6)
    }
}
