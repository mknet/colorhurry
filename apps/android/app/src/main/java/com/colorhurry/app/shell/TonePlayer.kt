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

    fun play(steps: List<ToneStep>) {
        if (steps.isEmpty() || busy) return
        busy = true
        thread(name = "tones") {
            try {
                render(steps)
            } finally {
                busy = false
            }
        }
    }

    private fun render(steps: List<ToneStep>) {
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
        track.play()
        val totalMs = steps.sumOf { it.ms.toLong() }
        Thread.sleep(totalMs + 30)
        track.stop()
        track.release()
    }
}
