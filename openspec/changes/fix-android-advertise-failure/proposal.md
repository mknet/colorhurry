# Proposal

## Why

The Android Picker sends nothing. The status first shows `Advertise failed: 2`, then `Advertise failed: 3`.

- `2` is `ADVERTISE_FAILED_TOO_MANY_ADVERTISERS`: the controller has no free advertising set, because other apps or the system hold them.
- `3` is `ADVERTISE_FAILED_ALREADY_STARTED`, and it is our bug. The legacy `BluetoothLeAdvertiser.startAdvertising` keeps the callback in its map also when the start fails, and only `stopAdvertising(callback)` removes it. `BleService.stopAdvertise()` calls `stopAdvertising` only while the `advertising` flag is set, and a failure clears that flag. So each later Apply fails with `3`, also when a set is free again, until the app restarts.

The same flag lets a withdrawn color go on air: if Skip comes before `onStartSuccess`, the stop is skipped.

## What Changes

- `stopAdvertise()` always calls `stopAdvertising(advertiseCallback)`. The call does nothing when no set runs, and it clears a stuck entry.
- `onStartFailure` calls `stopAdvertising` on its callback at once.
- After a failure with `2` or `4` (`ADVERTISE_FAILED_INTERNAL_ERROR`), the shell retries after 1 s, up to 3 times, while the color that it wants on air stays the same. `Broadcast(None)`, `Off` and `Caller` cancel the pending retries.
- The status names the error in words, for example `Advertise failed: no free BLE advertiser (2): close other Bluetooth apps or switch Bluetooth off and on`. The shell also logs the code.
- The unused scan response (service UUID `0xFFF0`) goes. The set is then non-scannable and needs less of the controller. The boards ignore the scan response.

## Capabilities

### New Capabilities

### Modified Capabilities
- `shell-integration`: two new requirements say how the Android Picker frees, retries and names a failed start of the advertiser

## Impact

- Android: `apps/android/app/src/main/java/com/colorhurry/app/shell/BleService.kt`.
- No core, FFI, Apple or board changes.
