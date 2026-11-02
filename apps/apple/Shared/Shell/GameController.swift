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

    func boot() { core.update(.boot) }
    func skip() { core.update(.skip) }
    func activate() { core.update(.activateColor) }

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
            return "Mode select · Skip cycles role/channel · Apply enters"
        case .picker:
            return "Picker ch\(view.channel) · Skip color · Apply broadcasts or withdraws"
        case .receiver:
            return view.countdownActive
                ? "Receiver ch\(view.channel) · countdown \(view.countdownStep)/10"
                : "Receiver ch\(view.channel) · waiting for match"
        }
    }
}

/// The shell's side of the effect protocol: one method per operation the core
/// declares. `Core` resolves `delay` and `random` with what they return.
final class GameHandler: EffectHandler, @unchecked Sendable {
    let tones = TonePlayer()
    let bleService = BleService()

    func delay(_ operation: Delay) async {
        try? await Task.sleep(for: .milliseconds(Int(operation.millis)))
    }

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
