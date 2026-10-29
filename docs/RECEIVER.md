# Receiver mode

## Setup

Position **5–9** on the ring (right half), Button **B** → receiver with channel 0–4.

The **channel color** serves only as an identity in setup — it does **not** determine the game color.

## Runtime

1. **Random color** from the 10-color spectrum, all LEDs on.
2. Immediately a **10-second countdown** (LEDs off CCW + descending tone).
3. During the countdown: BLE scan for a matching picker `CMD_COLOR`.
   - **Hit:** countdown aborts → **no** explosion → new random color.
   - **No hit:** after the countdown, **explosion** (sweep 180→45 Hz) → new random color.
4. Button **A** aborts the countdown (like a hit, without explosion).

## BLE

The receiver scans during the countdown. The picker sends `CMD_COLOR` only after color confirmation (Button B). The **channel** in the payload must match the setup (e.g. both "blue" = channel 3); otherwise the packet is ignored. RGB must match the current random color exactly.

## Files

| File | Role |
|------|------|
| `src/receiver.rs` | Loop + explosion |
| `src/countdown.rs` | LED + tone, abort via callback |
| `src/speaker.rs` | `play_explosion()` |
