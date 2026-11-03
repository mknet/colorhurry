package com.colorhurry.app.shell

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.colorhurry.shared.Ble
import com.colorhurry.shared.Color as SpectrumColor
import com.colorhurry.shared.Core
import com.colorhurry.shared.Delay
import com.colorhurry.shared.EffectHandler
import com.colorhurry.shared.Event
import com.colorhurry.shared.ModeRole
import com.colorhurry.shared.PlayTone
import com.colorhurry.shared.Random
import com.colorhurry.shared.Screen
import com.colorhurry.shared.ViewModel
import com.colorhurry.shared.colorReceivedEvent
import java.security.SecureRandom
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/// Holds the Rust core (Crux bridge over BoltFFI) and turns taps and BLE ads
/// into domain events. No game rule lives here: the core decides, the shell
/// only performs the effects it asks for.
class GameController(app: Application) : AndroidViewModel(app) {
    private val tones = TonePlayer()
    private val bleService = BleService(app)
    private val rng = SecureRandom()
    private val _bleStatus = MutableStateFlow("BLE idle")

    /// The shell's side of the effect protocol: one method per operation the
    /// core declares. `Core` resolves `delay` and `random` with what they return.
    private val handler = object : EffectHandler {
        override suspend fun delay(operation: Delay) {
            kotlinx.coroutines.delay(operation.millis.toLong())
        }

        override suspend fun random(operation: Random): UByte = rng.nextInt(256).toUByte()

        override fun playTone(operation: PlayTone) {
            val sequence = operation.sequence
            tones.play(sequence.steps.take(sequence.len.toInt()))
        }

        override fun ble(operation: Ble) = bleService.apply(operation.op)
    }

    /// `viewModelScope` runs on the main thread, so effects and BLE calls do too.
    private val core = Core(handler, viewModelScope)

    val view: StateFlow<ViewModel> = core.view
    val bleStatus: StateFlow<String> = _bleStatus.asStateFlow()

    init {
        bleService.onAdvertisement = { payload -> received(payload) }
        bleService.onStatus = { text -> _bleStatus.value = text }
    }

    fun boot() = core.update(Event.Boot)
    fun skip() = core.update(Event.Skip)
    fun activate() = core.update(Event.ActivateColor)

    /// Stops the tone that plays, then pauses the core. The core stops its
    /// timers and the radio.
    fun pause() {
        tones.stop()
        core.update(Event.Pause)
    }

    /// Lets the core continue where it stopped.
    fun resume() = core.update(Event.Resume)

    /// Manufacturer payload after the company ID; the core owns the wire format.
    private fun received(payload: ByteArray) {
        val bytes = colorReceivedEvent(payload)
        if (bytes.isEmpty()) return
        core.update(Event.bincodeDeserialize(bytes))
    }

    fun modeHint(view: ViewModel): String = when (view.screen) {
        Screen.MODESELECT -> view.modeChoice?.let { choice ->
            val role = if (choice.role == ModeRole.PICKER) "Picker" else "Receiver"
            "Apply enters $role ch${choice.channel} · Skip for next"
        } ?: "Mode select · Skip cycles role/channel · Apply enters"
        Screen.PICKER -> {
            val color = view.pickerColor?.let(::colorName) ?: "?"
            if (view.pickerApplied) {
                "Picker ch${view.channel} · broadcasting $color · Skip or Apply stops"
            } else {
                "Picker ch${view.channel} · $color · Skip for next · Apply broadcasts"
            }
        }
        Screen.RECEIVER -> if (view.countdownActive) {
            "Receiver ch${view.channel} · countdown ${view.countdownStep}/10"
        } else {
            "Receiver ch${view.channel} · waiting for match"
        }
    }

    override fun onCleared() {
        tones.stop()
        bleService.stopAll()
        super.onCleared()
    }
}

/** Reader-friendly name, for example "Light green". */
private fun colorName(color: SpectrumColor): String = when (color) {
    SpectrumColor.WHITE -> "White"
    SpectrumColor.RED -> "Red"
    SpectrumColor.ORANGE -> "Orange"
    SpectrumColor.YELLOW -> "Yellow"
    SpectrumColor.LIGHTGREEN -> "Light green"
    SpectrumColor.GREEN -> "Green"
    SpectrumColor.LIGHTBLUE -> "Light blue"
    SpectrumColor.BLUE -> "Blue"
    SpectrumColor.PURPLE -> "Purple"
    SpectrumColor.PINK -> "Pink"
}
