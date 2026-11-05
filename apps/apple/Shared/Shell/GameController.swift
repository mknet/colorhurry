import App
import Foundation
import Observation
import Shared

/// Holds the Rust core (Crux bridge over BoltFFI) and turns taps and BLE ads
/// into domain events. No game rule lives here: the core decides, the shell
/// only performs the effects it asks for.
@MainActor
@Observable
final class GameController {
    @ObservationIgnored private let handler: GameHandler
    @ObservationIgnored private let core: Core

    init() {
        handler = GameHandler()
        core = Core(handler: handler)
        handler.bleService.onAdvertisement = { [weak self] payload in
            self?.received(payload)
        }
    }

    var view: ViewModel { core.view }
    var bleStatus: String { handler.bleService.status }

    /// Starts the core. If the platform cannot broadcast a color, the core
    /// first learns that mode select must offer only Caller.
    func boot() {
        if !Platform.canBroadcast {
            core.update(.callerOnly)
        }
        core.update(.boot)
    }

    func skip() { core.update(.skip) }
    func activate() { core.update(.activateColor) }

    /// Stops the tones and haptics that are queued, then pauses the core.
    /// The core stops its timers and the radio.
    func pause() {
        handler.tones.stop()
        core.update(.pause)
    }

    /// Lets the core continue where it stopped.
    func resume() { core.update(.resume) }

    /// Stops the tones and haptics that are queued, then sends the player
    /// back to mode select. The core stops the round and the radio.
    func reset() {
        handler.tones.stop()
        core.update(.reset)
    }

    /// Manufacturer payload after the company ID; the core owns the wire format.
    private func received(_ payload: Data) {
        let bytes = colorReceivedEvent(payload: payload)
        guard !bytes.isEmpty,
              let event = try? Event.bincodeDeserialize(input: [UInt8](bytes))
        else { return }
        core.update(event)
    }

    var modeHint: String {
        switch view.screen {
        case .modeSelect:
            guard let choice = view.modeChoice else {
                return "Mode select · Skip cycles role/channel · Apply enters"
            }
            let role = choice.role == .picker ? "Picker" : "Caller"
            return "Apply enters \(role) ch\(choice.channel) · Skip for next"
        case .picker:
            let color = view.pickerColor?.displayName ?? "?"
            return view.pickerApplied
                ? "Picker ch\(view.channel) · \(Platform.name) cannot send \(color) · use a board as Picker"
                : "Picker ch\(view.channel) · \(color) · Skip for next · Apply broadcasts"
        case .caller:
            return view.countdownActive
                ? "Caller ch\(view.channel) · countdown \(view.countdownStep)/10"
                : "Caller ch\(view.channel) · waiting for match"
        }
    }
}

/// The shell's side of the effect protocol: one method per operation the core
/// declares. `Core` resolves the timers and `random` with what they return.
final class GameHandler: EffectHandler, @unchecked Sendable {
    let tones = TonePlayer()
    let bleService = BleService()
    /// The timer handler that `crux_time` ships. A cleared timer stops here.
    let time = TaskTimeHandler()

    func timeNotifyAfter(_ operation: NotifyAfter) async -> TimerId { await time.notifyAfter(operation) }
    func timeClear(_ operation: ClearTimer) async -> TimerId { await time.clear(operation) }

    func random(_ operation: Random) async -> UInt8 {
        var byte: UInt8 = 0
        _ = SecRandomCopyBytes(kSecRandomDefault, 1, &byte)
        return byte
    }

    func playTone(_ operation: PlayTone) {
        let sequence = operation.sequence
        tones.play(Array(sequence.steps.prefix(Int(sequence.len))))
    }

    func ble(_ operation: Ble) {
        // `Core` dispatches notifications on the main actor.
        MainActor.assumeIsolated { bleService.apply(operation.op) }
    }
}

extension Color {
    /// Reader-friendly name, for example "Light green".
    var displayName: String {
        switch self {
        case .white: "White"
        case .red: "Red"
        case .orange: "Orange"
        case .yellow: "Yellow"
        case .lightGreen: "Light green"
        case .green: "Green"
        case .lightBlue: "Light blue"
        case .blue: "Blue"
        case .purple: "Purple"
        case .pink: "Pink"
        }
    }
}
