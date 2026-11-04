# shell-integration Specification

## Purpose
Defines how platform shells (board, browser, phones) drive the game, so the rules exist only once and every shell plays the same game.

## Requirements

### Requirement: Game rules live in one shared core
Mode select, picker, caller round and palette SHALL live in one platform-free `no_std` core. A shell SHALL only turn input into domain events, perform the effects the core returns and draw its view.

#### Scenario: Rules are testable on the host
- **WHEN** `cargo test -p color-hurry-core` runs on a laptop
- **THEN** mode select, picker and caller behavior is exercised without a board

#### Scenario: Board firmware uses the core
- **WHEN** the board firmware is built
- **THEN** its game behavior comes from the shared core and the firmware only maps buttons, LEDs, speaker and radio

### Requirement: Browser Caller on the shared core
The browser shell SHALL run the shared core compiled to WebAssembly as one Caller board, draw the ten LEDs as a CPB-style circle and offer Skip and Apply as buttons.

#### Scenario: Playing a round in the browser
- **WHEN** the user opens the web shell and enters Caller
- **THEN** the ring shows the target color and counts down with the same beats and tones as the board

### Requirement: Live BLE receive in the browser
Where Web Bluetooth advertisement watching is available, the browser shell SHALL let the user choose a Color Hurry board and SHALL feed every valid color advertisement from it into the core as ColorReceived.

#### Scenario: Board Picker drives the browser Caller
- **WHEN** a board Picker on the Caller's channel applies the target color
- **THEN** the browser Caller completes the task and starts the next round

### Requirement: Shells only perform effects
A shell SHALL interact with the game only by sending domain events (Skip, ActivateColor, ColorReceived, Pause, Resume, Reset and CallerOnly) and by performing the effects the core requests: render, start a timer, clear a timer, random byte, play tone and BLE role. A shell SHALL NOT read game state for control flow or decide a match.

#### Scenario: Shell forwards every advertisement
- **WHEN** the board or browser decodes any valid color advertisement
- **THEN** it sends ColorReceived to the core without comparing it to the target

#### Scenario: Shell supplies randomness
- **WHEN** the core requests a random byte
- **THEN** the shell answers with a byte from its own random source and the core picks the target

#### Scenario: Shell reports the app lifecycle
- **WHEN** an app goes to the background or comes back
- **THEN** the shell sends Pause or Resume and the core decides what stops and what continues

#### Scenario: Shell forwards the Mode button
- **WHEN** the user presses the Mode button
- **THEN** the shell sends Reset and the core decides what stops and where mode select starts

#### Scenario: Shell reports that it cannot broadcast
- **WHEN** a shell that cannot broadcast a color starts
- **THEN** it sends CallerOnly before Boot and the core decides which choices mode select offers

### Requirement: Native mobile shells on the shared core
The iOS and Android apps SHALL run the shared Rust core through FFI. They SHALL contain no game rules, tone tables or BLE payload format of their own: they map taps and BLE advertisements to domain events, perform the requested effects and draw the view.

#### Scenario: Phone as Caller
- **WHEN** the phone app is in Caller on channel 0 and a board Picker on channel 0 applies the phone's target color
- **THEN** the app completes the task and starts the next round, with the same tones as board and browser

#### Scenario: Payloads are decoded by the core
- **WHEN** the app scans manufacturer data for company `0x239A`
- **THEN** it passes the bytes to the core and only valid v4 payloads become ColorReceived

### Requirement: Native macOS and watchOS shells on the shared core
The macOS and watchOS apps SHALL run the shared Rust core through FFI, like the iOS and Android apps. They SHALL contain no game rules, tone tables or BLE payload format of their own: they map input and BLE advertisements to domain events, perform the requested effects and draw the view. The watchOS app SHALL run on Apple Watch Series 6 (`arm64_32`) and later.

#### Scenario: Mac as Caller
- **WHEN** the Mac app is in Caller on a channel and a board Picker on the same channel applies the Mac's target color
- **THEN** the app completes the task and starts the next round, with the same tones as board, browser and phone

#### Scenario: Watch as Caller
- **WHEN** the watch app is in Caller on a channel and a board Picker on the same channel applies the watch's target color
- **THEN** the app completes the task and starts the next round, with the same tones as the other shells, and the watch gives a haptic on each tone step

#### Scenario: Picker is not available
- **WHEN** the user presses Skip in mode select in the macOS or watchOS app
- **THEN** mode select names only Caller choices, as in the iOS app, and the user must use a board or an Android phone as Picker

### Requirement: Apps pause the game in the background
An app shell SHALL send `Pause` to the core when the app goes to the background and `Resume` when it comes back. The iOS, macOS and watchOS apps SHALL use the background and active scene phases and SHALL ignore the inactive phase. The Android app SHALL use `onStop` and `onStart`. The web shell SHALL use the visibility of the page. The board SHALL NOT send `Pause` or `Resume`.

#### Scenario: Watch wrist is lowered
- **WHEN** the user lowers the wrist and the watch app becomes inactive but stays in the foreground
- **THEN** the game continues

#### Scenario: App comes back
- **WHEN** the user opens the app again after it went to the background during a countdown
- **THEN** the countdown continues at the step where it stopped

#### Scenario: Browser tab is hidden
- **WHEN** the user switches to another browser tab
- **THEN** the web shell sends `Pause`, and it sends `Resume` when the tab is visible again

#### Scenario: Board is never paused
- **WHEN** the board firmware runs
- **THEN** it sends no `Pause` or `Resume`, and its game behavior does not change

### Requirement: Pause stops the sound and the haptics
On `Pause`, an app shell SHALL stop the tone that plays, drop the tones that are queued, and cancel the haptic clicks that are scheduled.

#### Scenario: Watch app is dismissed during a countdown
- **WHEN** the user dismisses the watch app during a countdown
- **THEN** no tone and no haptic click comes after the dismissal, and the countdown does not continue

#### Scenario: Phone goes to the background during the explosion
- **WHEN** the iOS or Android app goes to the background while the explosion plays
- **THEN** the explosion stops at once

### Requirement: Apps offer a way back to mode select
The iOS, macOS, watchOS, Android and web shells SHALL offer a button that sends `Reset` to the core. The iOS, macOS and Android apps SHALL show a Mode button in the header, and the macOS app SHALL also send `Reset` for the Escape key. The watchOS app SHALL show a Change mode button on the status page and SHALL scroll back to the game page after the press. The web shell SHALL show a Mode button next to Skip and Apply. The board SHALL NOT send `Reset`.

#### Scenario: Phone Caller changes the channel
- **WHEN** the user presses Mode during a countdown in the iOS or Android app
- **THEN** the tones stop, mode select shows the Caller channel of the round, and the user can press Skip to choose a different channel

#### Scenario: Watch changes the mode
- **WHEN** the user presses Change mode on the watch status page
- **THEN** the watch shows the game page with mode select at the last choice

#### Scenario: Mac uses the keyboard
- **WHEN** the user presses Escape in the macOS app
- **THEN** the app goes back to mode select

#### Scenario: Board has no reset
- **WHEN** the board firmware runs
- **THEN** it sends no `Reset`, and its game behavior does not change

### Requirement: Reset stops the sound and the haptics
Before an app shell sends `Reset`, it SHALL stop the tone that plays, drop the tones that are queued, and cancel the haptic clicks that are scheduled.

#### Scenario: Reset during the explosion
- **WHEN** the user presses Mode in the iOS or Android app while the explosion plays
- **THEN** the explosion stops at once

### Requirement: Apple apps are Caller only
The iOS, macOS and watchOS apps SHALL declare `Platform.canBroadcast` as false, because CoreBluetooth on iOS and macOS cannot advertise manufacturer data and watchOS cannot advertise at all. When `canBroadcast` is false, `GameController.boot()` SHALL send `CallerOnly` to the core before `Boot`. The Android app, the web shell and the board SHALL NOT send `CallerOnly`.

#### Scenario: Watch opens on a Caller choice
- **WHEN** the user opens the watch app
- **THEN** mode select names Caller on channel 0, and Skip cycles only the Caller channels

#### Scenario: Android keeps Picker
- **WHEN** the user opens the Android app
- **THEN** mode select offers Picker and Caller as before

#### Scenario: Board is not changed
- **WHEN** the board firmware runs
- **THEN** it sends no `CallerOnly`, and its game behavior does not change

### Requirement: Shells answer the time effects of crux_time
The core SHALL request its timers with the `crux_time` operations `NotifyAfter` and `ClearTimer`. A shell SHALL answer a `NotifyAfter` with its timer id when its duration has elapsed. A shell SHALL answer a `ClearTimer` with the same id, and SHALL free the timer that the id names. The iOS, macOS and watchOS apps SHALL use the shipped `TaskTimeHandler`, and the Android app SHALL use the shipped `CoroutineTimeHandler`. The web shell, the board and the acceptance test harness SHALL answer the two operations themselves.
Acceptance: `crates/core/tests/features/timing.feature`.

#### Scenario: A timer fires
- **WHEN** the core requests a 50 ms timer and 50 ms elapse
- **THEN** the shell answers the request with its timer id, and the core advances the blink

#### Scenario: The board frees a cleared timer
- **WHEN** the core clears a pending timer on the board
- **THEN** the board removes the timer from its list, answers the clear with the same id, and never answers the timer

#### Scenario: A late answer
- **WHEN** a native shell answers a timer after the core cleared it
- **THEN** the core ignores the answer, and the game does not change
