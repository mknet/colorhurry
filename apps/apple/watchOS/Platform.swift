import App
import AVFoundation
import Foundation
import WatchKit

/// The watchOS side of the platform hooks. `iOS/Platform.swift` and
/// `macOS/Platform.swift` declare the same API.
enum Platform {
    static let name = "watchOS"

    /// Lets tones play with other audio.
    static func prepareAudio() throws {
        try AVAudioSession.sharedInstance().setCategory(.playback, mode: .default, options: [.mixWithOthers])
        try AVAudioSession.sharedInstance().setActive(true)
    }

    /// Plays one haptic click at the start of each tone step. The watch
    /// speaker is quiet, so the haptic gives the rhythm of the tones.
    static func toneFeedback(_ steps: [ToneStep]) {
        var offsetMs: UInt32 = 0
        for step in steps {
            DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(Int(offsetMs))) {
                WKInterfaceDevice.current().play(.click)
            }
            offsetMs += step.ms
        }
    }
}
