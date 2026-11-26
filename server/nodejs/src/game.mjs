import {
  Shell,
  decodeManufacturerData,
  encodeAdvertisement,
  companyId,
} from "color-hurry-nodejs";

/**
 * Owns the napi Shell, applies BLE side effects, and fans out updates to listeners.
 */
export class Game {
  constructor({ onUpdate, onBleStatus }) {
    this.shell = new Shell();
    this.onUpdate = onUpdate;
    this.onBleStatus = onBleStatus;
    this.ble = null;
    this.pickerChannel = null;
    this.broadcastColor = null;
    this.role = null; // "picker" | "caller" | null
  }

  attachBle(ble) {
    this.ble = ble;
  }

  boot() {
    this.applyUpdate(this.shell.dispatch("boot"));
  }

  dispatch(event) {
    this.applyUpdate(this.shell.dispatch(event));
  }

  ingestColor(channel, color) {
    this.applyUpdate(this.shell.ingestColor(channel, color));
  }

  poll() {
    const update = this.shell.poll();
    const hasSideEffects =
      (update.tones?.length ?? 0) > 0 || (update.ble?.length ?? 0) > 0;
    const viewChanged = !viewsEqual(this.lastView, update.view);
    if (!hasSideEffects && !viewChanged) return;
    this.applyUpdate(update);
  }

  snapshot() {
    return {
      type: "update",
      view: this.shell.view(),
      tones: [],
      bleStatus: this.lastBleStatus ?? "BLE idle",
      canAdvertise: this.ble?.canAdvertise ?? false,
      platform: process.platform,
    };
  }

  applyUpdate(update) {
    this.lastView = update.view;
    for (const op of update.ble ?? []) {
      this.applyBle(op);
    }
    this.onUpdate?.({
      type: "update",
      view: update.view,
      tones: update.tones ?? [],
      bleStatus: this.lastBleStatus ?? "BLE idle",
      canAdvertise: this.ble?.canAdvertise ?? false,
      platform: process.platform,
    });
  }

  /** Update BLE status. Pass `{ broadcast: true }` for standalone status pushes. */
  setBleStatus(text, { broadcast = true } = {}) {
    this.lastBleStatus = text;
    this.onBleStatus?.(text);
    if (broadcast) this.onUpdate?.(this.snapshot());
  }

  applyBle(op) {
    // Status only — applyUpdate broadcasts once after all BLE ops.
    const status = (text) => this.setBleStatus(text, { broadcast: false });
    if (!this.ble) {
      status("BLE not available on this machine");
      return;
    }
    switch (op.kind) {
      case "off":
        this.role = null;
        this.pickerChannel = null;
        this.broadcastColor = null;
        this.ble.stopAll();
        status("BLE off");
        break;
      case "picker":
        this.role = "picker";
        this.pickerChannel = op.channel ?? 0;
        this.broadcastColor = null;
        this.ble.stopScan();
        if (!this.ble.canAdvertise) {
          status(
            "Picker selected — manufacturer advertise needs Linux (bleno EIR). On macOS use a CPB as Picker.",
          );
          break;
        }
        this.ble.startPicker(this.pickerChannel);
        status(`Picker ready on channel ${this.pickerChannel}`);
        break;
      case "caller":
        this.role = "caller";
        this.pickerChannel = null;
        this.broadcastColor = null;
        this.ble.stopAdvertise();
        this.ble.startScan((channel, color) => this.ingestColor(channel, color));
        status(`Caller scanning (channel filter is in the core)`);
        break;
      case "broadcast":
        this.broadcastColor = op.color ?? null;
        if (this.broadcastColor == null) {
          this.ble.clearBroadcast();
          status(
            this.pickerChannel != null
              ? `Picker channel ${this.pickerChannel} (no color on air)`
              : "Broadcast cleared",
          );
          break;
        }
        if (!this.ble.canAdvertise) {
          status(
            `Would broadcast color ${this.broadcastColor} — advertise needs Linux`,
          );
          break;
        }
        this.ble.broadcast(this.pickerChannel ?? 0, this.broadcastColor);
        status(
          `Broadcasting color ${this.broadcastColor} on channel ${this.pickerChannel ?? 0}`,
        );
        break;
      default:
        status(`Unknown BLE op: ${op.kind}`);
    }
  }
}

function viewsEqual(a, b) {
  if (!a || !b) return false;
  if (
    a.screen !== b.screen ||
    a.channel !== b.channel ||
    a.countdownStep !== b.countdownStep ||
    a.countdownActive !== b.countdownActive ||
    a.pickerApplied !== b.pickerApplied ||
    a.pickerColor !== b.pickerColor
  ) {
    return false;
  }
  const am = a.modeChoice;
  const bm = b.modeChoice;
  if (!!am !== !!bm) return false;
  if (am && (am.role !== bm.role || am.channel !== bm.channel)) return false;
  if (!a.leds || !b.leds || a.leds.length !== b.leds.length) return false;
  for (let i = 0; i < a.leds.length; i++) {
    const x = a.leds[i];
    const y = b.leds[i];
    if (x.r !== y.r || x.g !== y.g || x.b !== y.b) return false;
  }
  return true;
}

export { decodeManufacturerData, encodeAdvertisement, companyId };
