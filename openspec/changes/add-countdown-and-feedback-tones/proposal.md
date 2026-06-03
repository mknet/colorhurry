# Proposal

## Why

The game is about time pressure, and the board has a speaker. Players need audible feedback for their input, and the Receiver round later needs a countdown that can be heard across the room, ending in an explosion.

## What Changes

- Skip and ActivateColor in the picker play short, distinct feedback tones.
- A countdown tone table: ten descending tones, one per second, from 680 Hz down to 200 Hz.
- An explosion sound: a sweep from 180 Hz down to 45 Hz over about one second.
- A demo sequence (guitar strum, countdown, explosion) behind the `speaker-demo` feature to try the sounds without a round.

## Capabilities

### New Capabilities
- `timing-and-tones`: what the game sounds like and when

### Modified Capabilities

## Impact

- PWM1 speaker driver (`src/speaker.rs`), `src/speaker_demo.rs`, `docs/AUDIO.md`.
