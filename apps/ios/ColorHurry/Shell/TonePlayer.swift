import App
import AVFoundation
import Foundation

final class TonePlayer {
    private var engine: AVAudioEngine?
    private var busy = false

    func play(_ steps: [ToneStep]) {
        guard !steps.isEmpty, !busy else { return }
        busy = true
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            self?.renderAndPlay(steps)
            self?.busy = false
        }
    }

    private func renderAndPlay(_ steps: [ToneStep]) {
        let sampleRate = 22_050.0
        var samples: [Float] = []
        for step in steps {
            let n = Int(sampleRate * Double(step.ms) / 1000.0)
            for i in 0..<n {
                let t = Double(i) / sampleRate
                let env: Double
                if i < 40 { env = Double(i) / 40.0 }
                else if i > n - 40 { env = Double(n - i) / 40.0 }
                else { env = 1.0 }
                samples.append(Float(sin(2 * Double.pi * Double(step.hz) * t) * 0.22 * env))
            }
        }

        do {
            let engine = AVAudioEngine()
            let format = AVAudioFormat(standardFormatWithSampleRate: sampleRate, channels: 1)!
            let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: AVAudioFrameCount(samples.count))!
            buffer.frameLength = AVAudioFrameCount(samples.count)
            if let channel = buffer.floatChannelData?[0] {
                for (i, s) in samples.enumerated() { channel[i] = s }
            }
            let player = AVAudioPlayerNode()
            engine.attach(player)
            engine.connect(player, to: engine.mainMixerNode, format: format)
            try AVAudioSession.sharedInstance().setCategory(.playback, mode: .default, options: [.mixWithOthers])
            try AVAudioSession.sharedInstance().setActive(true)
            try engine.start()
            player.scheduleBuffer(buffer, completionHandler: nil)
            player.play()
            let totalMs = steps.reduce(0) { $0 + $1.ms }
            Thread.sleep(forTimeInterval: Double(totalMs) / 1000.0 + 0.03)
            engine.stop()
        } catch {
            // Best-effort on simulator / muted devices.
        }
    }
}
