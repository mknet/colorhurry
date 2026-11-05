package com.colorhurry.app.shell

import android.annotation.SuppressLint
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.bluetooth.le.AdvertiseCallback
import android.bluetooth.le.AdvertiseCallback.ADVERTISE_FAILED_ALREADY_STARTED
import android.bluetooth.le.AdvertiseCallback.ADVERTISE_FAILED_DATA_TOO_LARGE
import android.bluetooth.le.AdvertiseCallback.ADVERTISE_FAILED_FEATURE_UNSUPPORTED
import android.bluetooth.le.AdvertiseCallback.ADVERTISE_FAILED_INTERNAL_ERROR
import android.bluetooth.le.AdvertiseCallback.ADVERTISE_FAILED_TOO_MANY_ADVERTISERS
import android.bluetooth.le.AdvertiseData
import android.bluetooth.le.AdvertiseSettings
import android.bluetooth.le.BluetoothLeAdvertiser
import android.bluetooth.le.BluetoothLeScanner
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.Context
import android.os.Handler
import android.os.Looper
import android.util.Log
import com.colorhurry.shared.BleOp
import com.colorhurry.shared.Color
import com.colorhurry.shared.colorAdvertisement
import com.colorhurry.shared.colorCompanyId

/// Scans and advertises Color Hurry manufacturer data. Payloads are built and
/// parsed by the Rust core; this class only moves bytes.
class BleService(context: Context) {
    var onAdvertisement: ((ByteArray) -> Unit)? = null
    var onStatus: ((String) -> Unit)? = null

    private val companyId = colorCompanyId().toInt()

    private val appContext = context.applicationContext
    private val adapter: BluetoothAdapter? =
        appContext.getSystemService(BluetoothManager::class.java)?.adapter

    private var scanner: BluetoothLeScanner? = null
    private var advertiser: BluetoothLeAdvertiser? = null
    private var pickerChannel: UByte? = null
    private var scanning = false

    // The color the core wants on air, or null. A retry only runs while this
    // is still the color that failed.
    private var wantedColor: Color? = null
    private var retries = 0
    private val handler = Handler(Looper.getMainLooper())

    private val scanCallback = object : ScanCallback() {
        override fun onScanResult(callbackType: Int, result: ScanResult) {
            val mfg = result.scanRecord?.manufacturerSpecificData ?: return
            val payload = mfg.get(companyId) ?: return
            onAdvertisement?.invoke(payload)
        }

        override fun onScanFailed(errorCode: Int) {
            onStatus?.invoke("Scan failed: $errorCode")
            scanning = false
        }
    }

    private val advertiseCallback = object : AdvertiseCallback() {
        override fun onStartSuccess(settingsInEffect: AdvertiseSettings) {
            retries = 0
        }

        @SuppressLint("MissingPermission")
        override fun onStartFailure(errorCode: Int) {
            // The legacy advertiser keeps a failed callback registered, and a
            // later start with it fails with ALREADY_STARTED (3). Free it now.
            try {
                advertiser?.stopAdvertising(this)
            } catch (_: Exception) {
            }
            Log.w(TAG, "Advertise failed: $errorCode (retry $retries)")
            onStatus?.invoke("Advertise failed: ${describeAdvertiseError(errorCode)}")
            val color = wantedColor ?: return
            val retryable = errorCode == ADVERTISE_FAILED_TOO_MANY_ADVERTISERS ||
                errorCode == ADVERTISE_FAILED_INTERNAL_ERROR
            if (retryable && retries < MAX_RETRIES) {
                retries++
                handler.postDelayed({
                    if (wantedColor == color) startAdvertise(color)
                }, RETRY_DELAY_MS)
            }
        }
    }

    fun apply(op: BleOp) {
        when (op) {
            BleOp.Off -> {
                stopAll()
                onStatus?.invoke("BLE off")
            }
            is BleOp.Picker -> {
                pickerChannel = op.channel
                stopScan()
                onStatus?.invoke("Picker ch${op.channel}: tap Apply to broadcast")
            }
            is BleOp.Caller -> {
                pickerChannel = null
                cancelBroadcast()
                startScan()
                onStatus?.invoke("Caller ch${op.channel} scanning for ${op.color}…")
            }
            is BleOp.Broadcast -> {
                if (op.color == null) {
                    cancelBroadcast()
                    onStatus?.invoke(
                        pickerChannel?.let { "Picker ch$it: selecting (not broadcasting)" }
                            ?: "Broadcast cleared",
                    )
                } else {
                    handler.removeCallbacksAndMessages(null)
                    wantedColor = op.color
                    retries = 0
                    startAdvertise(op.color)
                }
            }
        }
    }

    @SuppressLint("MissingPermission")
    fun startScan() {
        val a = adapter
        if (a == null || !a.isEnabled) {
            onStatus?.invoke("Turn Bluetooth on")
            return
        }
        scanner = a.bluetoothLeScanner
        val settings = ScanSettings.Builder()
            .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
            .setReportDelay(0)
            .build()
        scanner?.startScan(null, settings, scanCallback)
        scanning = true
        onStatus?.invoke("Scanning for ColorHurry ads…")
    }

    @SuppressLint("MissingPermission")
    private fun startAdvertise(color: Color) {
        val channel = pickerChannel ?: 0u
        val payload = colorAdvertisement(channel, color.bincodeSerialize())
        if (payload.isEmpty()) return
        val a = adapter
        if (a == null || !a.isEnabled) {
            onStatus?.invoke("Turn Bluetooth on")
            return
        }
        advertiser = a.bluetoothLeAdvertiser
        if (advertiser == null) {
            onStatus?.invoke("BLE advertising not supported on this device")
            return
        }
        stopAdvertise()
        val settings = AdvertiseSettings.Builder()
            .setAdvertiseMode(AdvertiseSettings.ADVERTISE_MODE_LOW_LATENCY)
            .setTxPowerLevel(AdvertiseSettings.ADVERTISE_TX_POWER_HIGH)
            .setConnectable(false)
            .setTimeout(0)
            .build()
        val data = AdvertiseData.Builder()
            .setIncludeDeviceName(false)
            .addManufacturerData(companyId, payload)
            .build()
        // No scan response: the boards read only the advertisement, and a
        // non-scannable set needs less of the controller.
        advertiser?.startAdvertising(settings, data, advertiseCallback)
        onStatus?.invoke("Broadcasting ch$channel $color")
    }

    @SuppressLint("MissingPermission")
    private fun stopScan() {
        if (scanning) {
            try {
                scanner?.stopScan(scanCallback)
            } catch (_: Exception) {
            }
        }
        scanning = false
    }

    @SuppressLint("MissingPermission")
    private fun stopAdvertise() {
        // Always stop by callback: it is harmless when nothing runs, and it
        // also stops a start that has not yet reported success.
        try {
            advertiser?.stopAdvertising(advertiseCallback)
        } catch (_: Exception) {
        }
    }

    /// Withdraws the wanted color: stops the advertiser and drops pending retries.
    private fun cancelBroadcast() {
        wantedColor = null
        retries = 0
        handler.removeCallbacksAndMessages(null)
        stopAdvertise()
    }

    fun stopAll() {
        stopScan()
        cancelBroadcast()
    }

    private companion object {
        const val TAG = "ColorHurry"
        const val MAX_RETRIES = 3
        const val RETRY_DELAY_MS = 1000L

        fun describeAdvertiseError(code: Int): String = when (code) {
            ADVERTISE_FAILED_DATA_TOO_LARGE -> "payload too large ($code)"
            ADVERTISE_FAILED_TOO_MANY_ADVERTISERS ->
                "no free BLE advertiser ($code): close other Bluetooth apps or switch Bluetooth off and on"
            ADVERTISE_FAILED_ALREADY_STARTED -> "already started ($code)"
            ADVERTISE_FAILED_INTERNAL_ERROR -> "internal Bluetooth error ($code)"
            ADVERTISE_FAILED_FEATURE_UNSUPPORTED -> "advertising not supported ($code)"
            else -> "error $code"
        }
    }
}
