import App

/// The macOS side of the platform hooks. `iOS/Platform.swift` and
/// `watchOS/Platform.swift` declare the same API.
enum Platform {
    static let name = "macOS"

    /// The macOS app cannot be Picker, so this value is false. CoreBluetooth on
    /// macOS cannot advertise manufacturer data, so a board Caller never sees
    /// the color. `GameController.boot()` then sends `CallerOnly`, and mode
    /// select offers only Caller.
    static let canBroadcast = false

    /// macOS has no audio session, so this function does nothing.
    static func prepareAudio() throws {}

    /// The Mac speaker plays the tones, so macOS gives no other feedback.
    static func toneFeedback(_ steps: [ToneStep], isCancelled: @escaping @Sendable () -> Bool) {}
}
