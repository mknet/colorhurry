import App
import CoreBluetooth
import Foundation
import Observation
import Shared

/// Central-only on iOS: scan manufacturer ads from CPB / Android pickers.
/// iOS cannot advertise custom manufacturer data that boards understand.
/// Payloads go to the Rust core undecoded; this class only moves bytes.
@Observable
final class BleService: NSObject {
    var status: String = "BLE idle"

    @ObservationIgnored private var central: CBCentralManager!
    @ObservationIgnored private var pickerChannel: UInt8?
    @ObservationIgnored private let companyId = colorCompanyId()
    @ObservationIgnored var onAdvertisement: ((Data) -> Void)?

    override init() {
        super.init()
        central = CBCentralManager(delegate: self, queue: .main)
    }

    func apply(_ op: BleOp) {
        switch op {
        case .off:
            stopScan()
            status = "BLE off"
        case .picker(let channel):
            pickerChannel = channel
            stopScan()
            status = "Picker ch\(channel): iOS cannot advertise manufacturer data; use a CPB (or Android) as Picker"
        case .receiver(let channel, let color):
            pickerChannel = nil
            startScan()
            status = "Receiver ch\(channel) scanning for \(color)…"
        case .broadcast(let color):
            if color == nil {
                status = pickerChannel.map { "Picker ch\($0): selecting (not broadcasting)" } ?? "Broadcast cleared"
            } else if let color {
                status = "Cannot broadcast \(color) from iOS (no manufacturer AD). Pair with CPB Picker instead."
            }
        }
    }

    func startScan() {
        guard central.state == .poweredOn else {
            status = "Turn Bluetooth on"
            return
        }
        // Scan all; filter Adafruit company ID in advertisements.
        central.scanForPeripherals(withServices: nil, options: [
            CBCentralManagerScanOptionAllowDuplicatesKey: true,
        ])
        status = "Scanning for ColorHurry ads…"
    }

    func stopScan() {
        if central.isScanning { central.stopScan() }
    }
}

extension BleService: CBCentralManagerDelegate {
    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        switch central.state {
        case .poweredOn:
            if status.contains("scanning") || status.contains("Receiver") {
                startScan()
            }
        case .poweredOff:
            status = "Bluetooth off"
        case .unauthorized:
            status = "Bluetooth permission denied"
        default:
            status = "Bluetooth: \(String(describing: central.state))"
        }
    }

    func centralManager(
        _ central: CBCentralManager,
        didDiscover peripheral: CBPeripheral,
        advertisementData: [String: Any],
        rssi RSSI: NSNumber
    ) {
        guard let mfg = advertisementData[CBAdvertisementDataManufacturerDataKey] as? Data,
              mfg.count > 2
        else { return }

        // CoreBluetooth manufacturer data = company ID (LE) + payload
        let company = UInt16(mfg[0]) | (UInt16(mfg[1]) << 8)
        guard company == companyId else { return }
        onAdvertisement?(Data(mfg.dropFirst(2)))
    }
}
