import App
import AVFoundation
import Foundation

final class TonePlayer {
    private let sampleRate = 22_050.0

    // The core sends the last countdown beat and then the explosion straight after it.
    // This serial queue receives the sequences in order, and the player node plays the
    // buffers in that order without a gap, so no sequence is dropped.
    // All the state below belongs to this queue.
    private let queue = DispatchQueue(label: "colorhurry.tones", qos: .userInitiated)

    // One engine and one player node live for the whole run, because an engine
    // start is slow on macOS and would delay each sequence.
    private var engine: AVAudioEngine?
    private var player: AVAudioPlayerNode?
    private var format: AVAudioFormat?
    private var needsRestart = false
    private var configObserver: NSObjectProtocol?

    // The time at which the queued audio ends. A new sequence starts at this time,
    // or at the current time if the audio has already ended.
    private var queuedUntil = Date.distantPast

    func play(_ steps: [ToneStep]) {
        guard !steps.isEmpty else { return }
        queue.async { [weak self] in
            self?.enqueue(steps)
        }
    }

    private func enqueue(_ steps: [ToneStep]) {
        let buffer = render(steps)
        let audioReady = ensureEngineRunning()

        let now = Date()
        let start = max(now, queuedUntil)
        let length = Double(buffer.frameLength) / sampleRate
        queuedUntil = start.addingTimeInterval(length)

        // The feedback starts with the sound and does not depend on audio,
        // so it runs at the start time even when audio fails.
        queue.asyncAfter(deadline: .now() + start.timeIntervalSince(now)) {
            Platform.toneFeedback(steps)
        }

        if audioReady {
            player?.scheduleBuffer(buffer, completionHandler: nil)
        }
    }

    /// Starts the engine on the first call and again after it stops.
    /// Returns false if the engine cannot start (for example on a simulator).
    private func ensureEngineRunning() -> Bool {
        if let engine, engine.isRunning, !needsRestart { return true }

        // Best effort: if the audio session fails, try to play anyway.
        try? Platform.prepareAudio()

        if engine == nil { buildEngine() }
        guard let engine, let player else { return false }
        do {
            try engine.start()
            player.play()
            needsRestart = false
            return true
        } catch {
            return false
        }
    }

    private func buildEngine() {
        let engine = AVAudioEngine()
        let player = AVAudioPlayerNode()
        let format = AVAudioFormat(standardFormatWithSampleRate: sampleRate, channels: 1)!
        engine.attach(player)
        engine.connect(player, to: engine.mainMixerNode, format: format)
        self.engine = engine
        self.player = player
        self.format = format

        // A configuration change (for example a new output device) stops the engine
        // and drops the scheduled buffers. The next sequence starts the engine again.
        configObserver = NotificationCenter.default.addObserver(
            forName: .AVAudioEngineConfigurationChange, object: engine, queue: nil
        ) { [weak self] _ in
            self?.queue.async {
                self?.needsRestart = true
                self?.queuedUntil = .distantPast
            }
        }
    }

    private func render(_ steps: [ToneStep]) -> AVAudioPCMBuffer {
        let counts = steps.map { Int(sampleRate * Double($0.ms) / 1000.0) }
        let total = counts.reduce(0, +)
        let format = self.format
            ?? AVAudioFormat(standardFormatWithSampleRate: sampleRate, channels: 1)!
        let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: AVAudioFrameCount(max(total, 1)))!
        buffer.frameLength = AVAudioFrameCount(total)
        guard let channel = buffer.floatChannelData?[0] else { return buffer }

        var index = 0
        for (step, n) in zip(steps, counts) {
            for i in 0..<n {
                let t = Double(i) / sampleRate
                let env: Double
                if i < 40 { env = Double(i) / 40.0 }
                else if i > n - 40 { env = Double(n - i) / 40.0 }
                else { env = 1.0 }
                channel[index] = Float(sin(2 * Double.pi * Double(step.hz) * t) * 0.22 * env)
                index += 1
            }
        }
        return buffer
    }
}
