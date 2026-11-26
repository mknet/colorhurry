import { createRequire } from "node:module";
import { decodeManufacturerData, encodeAdvertisement } from "color-hurry-nodejs";

const require = createRequire(import.meta.url);

/**
 * BLE adapter: noble for scan (Caller), bleno EIR for advertise (Picker, Linux).
 */
export async function createBle() {
  let noble = null;
  try {
    noble = (await import("@abandonware/noble")).default;
  } catch (err) {
    console.warn("noble not available:", err.message);
  }

  let bleno = null;
  const canAdvertise = process.platform === "linux";
  if (canAdvertise) {
    try {
      // Optional peer dependency; install with: npm i @abandonware/bleno
      bleno = require("@abandonware/bleno");
    } catch (err) {
      console.warn("bleno not available (Picker advertise disabled):", err.message);
    }
  }

  return new BleFacade({ noble, bleno, canAdvertise: canAdvertise && !!bleno });
}

class BleFacade {
  constructor({ noble, bleno, canAdvertise }) {
    this.noble = noble;
    this.bleno = bleno;
    this.canAdvertise = canAdvertise;
    this.scanning = false;
    this.advertising = false;
    this.onColor = null;
    this.pickerChannel = 0;
    this._wired = false;
  }

  wire() {
    if (this._wired || !this.noble) return;
    this._wired = true;
    this.noble.on("stateChange", (state) => {
      if (state === "poweredOn" && this.scanning) {
        this.noble.startScanning([], true);
      } else if (state !== "poweredOn") {
        try {
          this.noble.stopScanning();
        } catch {
          /* ignore */
        }
      }
    });
    this.noble.on("discover", (peripheral) => {
      const data = peripheral?.advertisement?.manufacturerData;
      if (!data) return;
      const decoded = decodeManufacturerData(Buffer.from(data));
      if (!decoded || !this.onColor) return;
      this.onColor(decoded.channel, decoded.color);
    });
  }

  startScan(onColor) {
    this.onColor = onColor;
    if (!this.noble) return;
    this.wire();
    this.scanning = true;
    if (this.noble.state === "poweredOn") {
      this.noble.startScanning([], true);
    }
  }

  stopScan() {
    this.scanning = false;
    this.onColor = null;
    if (!this.noble) return;
    try {
      this.noble.stopScanning();
    } catch {
      /* ignore */
    }
  }

  startPicker(channel) {
    this.pickerChannel = channel;
    // No payload until Broadcast(Some).
    this.clearBroadcast();
  }

  broadcast(channel, colorIndex) {
    if (!this.bleno || !this.canAdvertise) return;
    let adv;
    try {
      adv = Buffer.from(encodeAdvertisement(channel, colorIndex));
    } catch (err) {
      console.warn("encodeAdvertisement failed:", err.message);
      return;
    }
    const start = () => {
      this.bleno.startAdvertisingWithEIRData(adv, (err) => {
        if (err) console.warn("advertise failed:", err);
        else this.advertising = true;
      });
    };
    if (this.bleno.state === "poweredOn") {
      if (this.advertising) {
        this.bleno.stopAdvertising(() => start());
      } else {
        start();
      }
    } else {
      this.bleno.once("stateChange", (state) => {
        if (state === "poweredOn") start();
      });
    }
  }

  clearBroadcast() {
    this.stopAdvertise();
  }

  stopAdvertise() {
    this.advertising = false;
    if (!this.bleno) return;
    try {
      this.bleno.stopAdvertising();
    } catch {
      /* ignore */
    }
  }

  stopAll() {
    this.stopScan();
    this.stopAdvertise();
  }
}
