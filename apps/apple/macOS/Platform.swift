import App

/// The macOS side of the platform hooks. `iOS/Platform.swift` and
/// `watchOS/Platform.swift` declare the same API.
enum Platform {
    static let name = "macOS"

    /// macOS has no audio session, so this function does nothing.
    static func prepareAudio() throws {}

    /// The Mac speaker plays the tones, so macOS gives no other feedback.
    static func toneFeedback(_ steps: [ToneStep]) {}
}
