package com.colorhurry.app.shell

import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioTrack
import com.colorhurry.shared.ToneStep
import kotlin.concurrent.thread
import kotlin.math.PI
import kotlin.math.sin

class TonePlayer {
    @Volatile private var busy = false

    // The lock guards `generation` and `current`. Each call to `stop()` starts a new
    // generation. A sequence keeps the generation from the time it was played, and
    // does not start if `stop()` came after it.
    private val lock = Any()
    private var generation = 0L
    private var current: AudioTrack? = null

    fun play(steps: List<ToneStep>) {
        if (steps.isEmpty() || busy) return
        busy = true
        val gen = synchronized(lock) { generation }
        thread(name = "tones") {
            try {
                render(steps, gen)
            } finally {
                busy = false
            }
        }
    }

    /// Stops the tone that plays and drops the sequences that have not started.
    fun stop() {
        synchronized(lock) {
            generation++
            current?.let { track -> runCatching { track.stop() } }
        }
    }

    private fun render(steps: List<ToneStep>, gen: Long) {
        val sampleRate = 22_050
        val samples = ArrayList<Short>()
        for (step in steps) {
            val n = (sampleRate * step.ms.toDouble() / 1000.0).toInt()
            for (i in 0 until n) {
                val t = i.toDouble() / sampleRate
                val env = when {
                    i < 40 -> i / 40.0
                    i > n - 40 -> (n - i) / 40.0
                    else -> 1.0
                }
                val s = (sin(2 * PI * step.hz.toDouble() * t) * 0.22 * env * Short.MAX_VALUE).toInt()
                    .coerceIn(Short.MIN_VALUE.toInt(), Short.MAX_VALUE.toInt())
                samples.add(s.toShort())
            }
        }
        if (samples.isEmpty()) return
        val track = AudioTrack.Builder()
            .setAudioAttributes(
                AudioAttributes.Builder()
                    .setUsage(AudioAttributes.USAGE_GAME)
                    .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
                    .build(),
            )
            .setAudioFormat(
                AudioFormat.Builder()
                    .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                    .setSampleRate(sampleRate)
                    .setChannelMask(AudioFormat.CHANNEL_OUT_MONO)
                    .build(),
            )
            .setBufferSizeInBytes(samples.size * 2)
            .setTransferMode(AudioTrack.MODE_STATIC)
            .build()
        val buf = ShortArray(samples.size) { samples[it] }
        track.write(buf, 0, buf.size)
        val started = synchronized(lock) {
            if (gen == generation) {
                current = track
                track.play()
                true
            } else {
                false
            }
        }
        if (started) {
            val totalMs = steps.sumOf { it.ms.toLong() }
            Thread.sleep(totalMs + 30)
            synchronized(lock) {
                if (current === track) current = null
                runCatching { track.stop() }
            }
        }
        track.release()
    }
}
