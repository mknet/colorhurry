package com.colorhurry.app.shell

import android.annotation.SuppressLint
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.bluetooth.le.AdvertiseCallback
import android.bluetooth.le.AdvertiseData
import android.bluetooth.le.AdvertiseSettings
import android.bluetooth.le.BluetoothLeAdvertiser
import android.bluetooth.le.BluetoothLeScanner
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.Context
import android.os.ParcelUuid
import com.colorhurry.shared.BleOp
import com.colorhurry.shared.Color
import com.colorhurry.shared.colorAdvertisement
import com.colorhurry.shared.colorCompanyId
import java.util.UUID

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
    private var advertising = false

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
            advertising = true
        }

        override fun onStartFailure(errorCode: Int) {
            advertising = false
            onStatus?.invoke("Advertise failed: $errorCode")
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
                onStatus?.invoke("Picker ch${op.channel} — tap Apply to broadcast")
            }
            is BleOp.Receiver -> {
                pickerChannel = null
                stopAdvertise()
                startScan()
                onStatus?.invoke("Receiver ch${op.channel} scanning for ${op.color}…")
            }
            is BleOp.Broadcast -> {
                if (op.color == null) {
                    stopAdvertise()
                    onStatus?.invoke(
                        pickerChannel?.let { "Picker ch$it — selecting (not broadcasting)" }
                            ?: "Broadcast cleared",
                    )
                } else {
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
        // Optional scan response with a local name hint (boards ignore this).
        val scanResponse = AdvertiseData.Builder()
            .setIncludeDeviceName(false)
            .addServiceUuid(ParcelUuid(UUID.fromString("0000fff0-0000-1000-8000-00805f9b34fb")))
            .build()
        advertiser?.startAdvertising(settings, data, scanResponse, advertiseCallback)
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
        if (advertising) {
            try {
                advertiser?.stopAdvertising(advertiseCallback)
            } catch (_: Exception) {
            }
        }
        advertising = false
    }

    fun stopAll() {
        stopScan()
        stopAdvertise()
    }
}
