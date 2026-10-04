# Proposal

## Why

With SoftDevice S140 running, the app timer on TIMER2 collides with the radio stack, and tones played with `await` stall button polling and the LED ring. Before the Receiver can scan during its countdown, timing and sound must work next to the radio.

## What Changes

- The app time base moves to TIMER4, which S140 leaves to the application.
- Feedback tones become non-blocking: a tone service ends them in the background while input and LEDs keep running.

## Capabilities

### New Capabilities

### Modified Capabilities
- `timing-and-tones`: timing keeps its period next to the radio stack, and tones no longer block input

## Impact

- `src/time.rs`, `src/speaker.rs`, call sites in `mode_select.rs` and `color_picker.rs`.
