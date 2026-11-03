import App
import SwiftUI

/// The main screen on iOS and macOS. The app owns the `GameController`, so a
/// platform wrapper view can send events to the same game.
struct ContentView: View {
    let game: GameController

    var body: some View {
        ZStack {
            RadialGradient(
                colors: [SwiftUI.Color(red: 0.11, green: 0.16, blue: 0.23), AppTheme.bg],
                center: UnitPoint(x: 0.2, y: 0),
                startRadius: 20,
                endRadius: 520
            )
            .ignoresSafeArea()

            VStack(alignment: .leading, spacing: 10) {
                HStack {
                    Text("Color Hurry")
                        .font(.system(size: 22, weight: .semibold))
                        .foregroundStyle(AppTheme.text)
                    Spacer()
                    Button(action: game.reset) {
                        Text("Mode")
                            .font(.system(size: 13, weight: .semibold))
                            .foregroundStyle(.white)
                            .padding(.horizontal, 12)
                            .padding(.vertical, 6)
                            .background(AppTheme.secondary)
                            .clipShape(Capsule())
                    }
                    .buttonStyle(.plain)
                    .accessibilityHint("Goes back to mode select")
                }

                Text("Same entry as the board: Skip cycles role/channel, Apply starts. Match a CPB over BLE (protocol v4).")
                    .font(.system(size: 13))
                    .foregroundStyle(AppTheme.muted)
                    .fixedSize(horizontal: false, vertical: true)

                Text(game.bleStatus)
                    .font(.system(size: 12))
                    .foregroundStyle(AppTheme.muted)

                VStack(alignment: .leading, spacing: 8) {
                    Text(title)
                        .font(.system(size: 17, weight: .semibold))
                        .foregroundStyle(AppTheme.text)
                    Text(game.modeHint)
                        .font(.system(size: 13))
                        .foregroundStyle(AppTheme.muted)

                    LedRingView(leds: game.view.leds)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)

                    HStack(spacing: 10) {
                        actionButton("Button A (skip)", secondary: true, action: game.skip)
                        actionButton("Button B (apply)", secondary: false, action: game.activate)
                    }
                }
                .padding(14)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(AppTheme.panel)
                .clipShape(RoundedRectangle(cornerRadius: 20, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 20, style: .continuous)
                        .stroke(AppTheme.border, lineWidth: 1)
                )
            }
            .padding(16)
        }
        .onAppear { game.boot() }
        .pausesInBackground(game)
    }

    private var title: String {
        switch game.view.screen {
        case .modeSelect: return "Mode select"
        case .picker: return "Picker"
        case .receiver: return "Receiver"
        }
    }

    private func actionButton(_ label: String, secondary: Bool, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Text(label)
                .font(.system(size: 14, weight: .semibold))
                .foregroundStyle(.white)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 14)
                .background(secondary ? AppTheme.secondary : AppTheme.primary)
                .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
        }
        .buttonStyle(.plain)
    }
}
