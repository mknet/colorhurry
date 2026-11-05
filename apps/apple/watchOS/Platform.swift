import App
import AVFoundation
import Foundation
import WatchKit

/// The watchOS side of the platform hooks. `iOS/Platform.swift` and
/// `macOS/Platform.swift` declare the same API.
enum Platform {
    static let name = "watchOS"

    /// The watchOS app cannot be Picker, so this value is false. watchOS cannot
    /// advertise over Bluetooth at all, so a board Caller never sees the
    /// color. `GameController.boot()` then sends `CallerOnly`, and mode
    /// select offers only Caller.
    static let canBroadcast = false

    /// Lets tones play with other audio.
    static func prepareAudio() throws {
        try AVAudioSession.sharedInstance().setCategory(.playback, mode: .default, options: [.mixWithOthers])
        try AVAudioSession.sharedInstance().setActive(true)
    }

    /// Plays one haptic click at the start of each tone step. The watch
    /// speaker is quiet, so the haptic gives the rhythm of the tones.
    /// Each click calls `isCancelled` first, and plays only if it returns false.
    static func toneFeedback(_ steps: [ToneStep], isCancelled: @escaping @Sendable () -> Bool) {
        var offsetMs: UInt32 = 0
        for step in steps {
            DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(Int(offsetMs))) {
                guard !isCancelled() else { return }
                WKInterfaceDevice.current().play(.click)
            }
            offsetMs += step.ms
        }
    }
}
