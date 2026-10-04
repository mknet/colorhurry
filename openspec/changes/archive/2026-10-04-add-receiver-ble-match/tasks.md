# Tasks

## 1. Radio roles

- [x] 1.1 Keep the radio off until mode select ends, verify with nRF Connect that nothing is advertised during setup
- [x] 1.2 Advertise only the applied color and withdraw it on Skip / ActivateColor, verify in nRF Connect

## 2. Receiver match

- [x] 2.1 Scan during the countdown and abort it on a matching color, verify with two boards (no explosion, next round)
- [x] 2.2 Play the explosion when the countdown expires, verify by letting a round run out
- [x] 2.3 Abort the round on Skip, verify on the board

## 3. Channels (protocol v3)

- [x] 3.1 Add the channel byte to the payload and filter scans by channel, verify two pairs on different channels do not interfere
- [x] 3.2 Update `docs/BLE.md`, `docs/RECEIVER.md` and `docs/COLOR-PICKER.md` for v3 and the receiver flow
