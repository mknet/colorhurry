import App
import SwiftUI

/// The LED ring. The optional `center` view fills the space inside the LEDs and
/// replaces the small hub circle. Without it, the ring draws the hub circle.
struct LedRingView<Center: View>: View {
    let leds: [Rgb]
    private let hasCenter: Bool
    private let center: Center

    init(leds: [Rgb], @ViewBuilder center: () -> Center) {
        self.leds = leds
        self.hasCenter = true
        self.center = center()
    }

    var body: some View {
        GeometryReader { geo in
            let size = min(geo.size.width, geo.size.height)
            ZStack {
                Circle()
                    .fill(
                        RadialGradient(
                            colors: [
                                SwiftUI.Color(red: 0.16, green: 0.21, blue: 0.26),
                                SwiftUI.Color(red: 0.08, green: 0.11, blue: 0.14),
                                SwiftUI.Color(red: 0.05, green: 0.06, blue: 0.09),
                            ],
                            center: .center,
                            startRadius: size * 0.08,
                            endRadius: size * 0.5
                        )
                    )
                    .overlay(Circle().stroke(SwiftUI.Color(red: 0.23, green: 0.29, blue: 0.36), lineWidth: 3))
                    .frame(width: size, height: size)

                if hasCenter {
                    center
                        .frame(width: size * 0.48, height: size * 0.32)
                } else {
                    Circle()
                        .fill(SwiftUI.Color(red: 0.16, green: 0.21, blue: 0.26))
                        .frame(width: size * 0.18, height: size * 0.18)
                }

                ForEach(0..<leds.count, id: \.self) { i in
                    let angle = -90.0 + Double(i) * (360.0 / Double(leds.count))
                    let orbit = size * 0.36
                    let c = leds[i]
                    let lit = SwiftUI.Color(red: Double(c.r) / 255, green: Double(c.g) / 255, blue: Double(c.b) / 255)
                    Circle()
                        .fill(lit)
                        .frame(width: size * 0.12, height: size * 0.12)
                        .shadow(color: c == Rgb(r: 0, g: 0, b: 0) ? .clear : lit.opacity(0.55), radius: 6)
                        .overlay(Circle().stroke(SwiftUI.Color(red: 0.19, green: 0.25, blue: 0.31), lineWidth: 1))
                        .offset(y: -orbit)
                        .rotationEffect(.degrees(angle))
                }
            }
            .frame(width: geo.size.width, height: geo.size.height)
        }
    }
}

extension LedRingView where Center == EmptyView {
    init(leds: [Rgb]) {
        self.leds = leds
        self.hasCenter = false
        self.center = EmptyView()
    }
}
