import App
import AVFoundation

/// The iOS side of the platform hooks. `macOS/Platform.swift` and
/// `watchOS/Platform.swift` declare the same API.
enum Platform {
    static let name = "iOS"

    /// Lets tones play with other audio and when the ring switch is on silent.
    static func prepareAudio() throws {
        try AVAudioSession.sharedInstance().setCategory(.playback, mode: .default, options: [.mixWithOthers])
        try AVAudioSession.sharedInstance().setActive(true)
    }

    /// The phone speaker plays the tones, so iOS gives no other feedback.
    static func toneFeedback(_ steps: [ToneStep]) {}
}
