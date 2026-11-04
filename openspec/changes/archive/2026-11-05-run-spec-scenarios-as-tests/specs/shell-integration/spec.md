# Spec Delta

## MODIFIED Requirements

### Requirement: Game rules live in one shared core
Mode select, picker, caller round and palette SHALL live in one platform-free `no_std` core. A shell SHALL only turn input into domain events, perform the effects the core returns and draw its view.

#### Scenario: Rules are testable on the host
Manual: this run of `cargo test` is the check.
- **WHEN** `cargo test -p color-hurry-core` runs on a laptop
- **THEN** mode select, picker and caller behavior is exercised without a board

#### Scenario: Board firmware uses the core
Manual: needs the firmware build.
- **WHEN** the board firmware is built
- **THEN** its game behavior comes from the shared core and the firmware only maps buttons, LEDs, speaker and radio

### Requirement: Browser Caller on the shared core
The browser shell SHALL run the shared core compiled to WebAssembly as one Caller board, draw the ten LEDs as a CPB-style circle and offer Skip and Apply as buttons.

#### Scenario: Playing a round in the browser
Manual: needs a browser.
- **WHEN** the user opens the web shell and enters Caller
- **THEN** the ring shows the target color and counts down with the same beats and tones as the board

### Requirement: Live BLE receive in the browser
Where Web Bluetooth advertisement watching is available, the browser shell SHALL let the user choose a Color Hurry board and SHALL feed every valid color advertisement from it into the core as ColorReceived.

#### Scenario: Board Picker drives the browser Caller
Manual: needs a board and a browser with Web Bluetooth.
- **WHEN** a board Picker on the Caller's channel applies the target color
- **THEN** the browser Caller completes the task and starts the next round

### Requirement: Shells only perform effects
A shell SHALL interact with the game only by sending domain events (Skip, ActivateColor, ColorReceived, Pause, Resume, Reset and CallerOnly) and by performing the effects the core requests: render, start a timer, clear a timer, random byte, play tone and BLE role. A shell SHALL NOT read game state for control flow or decide a match.

#### Scenario: Shell forwards every advertisement
Manual: needs the board or the browser.
- **WHEN** the board or browser decodes any valid color advertisement
- **THEN** it sends ColorReceived to the core without comparing it to the target

#### Scenario: Shell supplies randomness
- **GIVEN** the board in mode select
- **WHEN** the player enters Caller on channel 0
- **AND** the shell answers the random request with 17
- **THEN** the target is Blue

#### Scenario: Shell reports the app lifecycle
Manual: needs an app.
- **WHEN** an app goes to the background or comes back
- **THEN** the shell sends Pause or Resume and the core decides what stops and what continues

#### Scenario: Shell forwards the Mode button
Manual: needs an app.
- **WHEN** the user presses the Mode button
- **THEN** the shell sends Reset and the core decides what stops and where mode select starts

#### Scenario: Shell reports that it cannot broadcast
Manual: needs an Apple app.
- **WHEN** a shell that cannot broadcast a color starts
- **THEN** it sends CallerOnly before Boot and the core decides which choices mode select offers

### Requirement: Native mobile shells on the shared core
The iOS and Android apps SHALL run the shared Rust core through FFI. They SHALL contain no game rules, tone tables or BLE payload format of their own: they map taps and BLE advertisements to domain events, perform the requested effects and draw the view.

#### Scenario: Phone as Caller
Manual: needs a board and a phone.
- **WHEN** the phone app is in Caller on channel 0 and a board Picker on channel 0 applies the phone's target color
- **THEN** the app completes the task and starts the next round, with the same tones as board and browser

#### Scenario: Payloads are decoded by the core
Manual: needs a phone.
- **WHEN** the app scans manufacturer data for company `0x239A`
- **THEN** it passes the bytes to the core and only valid v4 payloads become ColorReceived

### Requirement: Native macOS and watchOS shells on the shared core
The macOS and watchOS apps SHALL run the shared Rust core through FFI, like the iOS and Android apps. They SHALL contain no game rules, tone tables or BLE payload format of their own: they map input and BLE advertisements to domain events, perform the requested effects and draw the view. The watchOS app SHALL run on Apple Watch Series 6 (`arm64_32`) and later.

#### Scenario: Mac as Caller
Manual: needs a board and a Mac.
- **WHEN** the Mac app is in Caller on a channel and a board Picker on the same channel applies the Mac's target color
- **THEN** the app completes the task and starts the next round, with the same tones as board, browser and phone

#### Scenario: Watch as Caller
Manual: needs a board and an Apple Watch.
- **WHEN** the watch app is in Caller on a channel and a board Picker on the same channel applies the watch's target color
- **THEN** the app completes the task and starts the next round, with the same tones as the other shells, and the watch gives a haptic on each tone step

#### Scenario: Picker is not available
Manual: needs the macOS or the watchOS app.
- **WHEN** the user presses Skip in mode select in the macOS or watchOS app
- **THEN** mode select names only Caller choices, as in the iOS app, and the user must use a board or an Android phone as Picker

### Requirement: Apps pause the game in the background
An app shell SHALL send `Pause` to the core when the app goes to the background and `Resume` when it comes back. The iOS, macOS and watchOS apps SHALL use the background and active scene phases and SHALL ignore the inactive phase. The Android app SHALL use `onStop` and `onStart`. The web shell SHALL use the visibility of the page. The board SHALL NOT send `Pause` or `Resume`.

#### Scenario: Watch wrist is lowered
Manual: needs an Apple Watch.
- **WHEN** the user lowers the wrist and the watch app becomes inactive but stays in the foreground
- **THEN** the game continues

#### Scenario: App comes back
Manual: needs an app.
- **WHEN** the user opens the app again after it went to the background during a countdown
- **THEN** the countdown continues at the step where it stopped

#### Scenario: Browser tab is hidden
Manual: needs a browser.
- **WHEN** the user switches to another browser tab
- **THEN** the web shell sends `Pause`, and it sends `Resume` when the tab is visible again

#### Scenario: Board is never paused
Manual: needs the board.
- **WHEN** the board firmware runs
- **THEN** it sends no `Pause` or `Resume`, and its game behavior does not change

### Requirement: Pause stops the sound and the haptics
On `Pause`, an app shell SHALL stop the tone that plays, drop the tones that are queued, and cancel the haptic clicks that are scheduled.

#### Scenario: Watch app is dismissed during a countdown
Manual: needs an Apple Watch.
- **WHEN** the user dismisses the watch app during a countdown
- **THEN** no tone and no haptic click comes after the dismissal, and the countdown does not continue

#### Scenario: Phone goes to the background during the explosion
Manual: needs a phone.
- **WHEN** the iOS or Android app goes to the background while the explosion plays
- **THEN** the explosion stops at once

### Requirement: Apps offer a way back to mode select
The iOS, macOS, watchOS, Android and web shells SHALL offer a button that sends `Reset` to the core. The iOS, macOS and Android apps SHALL show a Mode button in the header, and the macOS app SHALL also send `Reset` for the Escape key. The watchOS app SHALL show a Change mode button on the status page and SHALL scroll back to the game page after the press. The web shell SHALL show a Mode button next to Skip and Apply. The board SHALL NOT send `Reset`.

#### Scenario: Phone Caller changes the channel
Manual: needs a phone.
- **WHEN** the user presses Mode during a countdown in the iOS or Android app
- **THEN** the tones stop, mode select shows the Caller channel of the round, and the user can press Skip to choose a different channel

#### Scenario: Watch changes the mode
Manual: needs an Apple Watch.
- **WHEN** the user presses Change mode on the watch status page
- **THEN** the watch shows the game page with mode select at the last choice

#### Scenario: Mac uses the keyboard
Manual: needs a Mac.
- **WHEN** the user presses Escape in the macOS app
- **THEN** the app goes back to mode select

#### Scenario: Board has no reset
Manual: needs the board.
- **WHEN** the board firmware runs
- **THEN** it sends no `Reset`, and its game behavior does not change

### Requirement: Reset stops the sound and the haptics
Before an app shell sends `Reset`, it SHALL stop the tone that plays, drop the tones that are queued, and cancel the haptic clicks that are scheduled.

#### Scenario: Reset during the explosion
Manual: needs a phone.
- **WHEN** the user presses Mode in the iOS or Android app while the explosion plays
- **THEN** the explosion stops at once

### Requirement: Apple apps are Caller only
The iOS, macOS and watchOS apps SHALL declare `Platform.canBroadcast` as false, because CoreBluetooth on iOS and macOS cannot advertise manufacturer data and watchOS cannot advertise at all. When `canBroadcast` is false, `GameController.boot()` SHALL send `CallerOnly` to the core before `Boot`. The Android app, the web shell and the board SHALL NOT send `CallerOnly`.

#### Scenario: Watch opens on a Caller choice
Manual: needs an Apple Watch.
- **WHEN** the user opens the watch app
- **THEN** mode select names Caller on channel 0, and Skip cycles only the Caller channels

#### Scenario: Android keeps Picker
Manual: needs an Android phone.
- **WHEN** the user opens the Android app
- **THEN** mode select offers Picker and Caller as before

#### Scenario: Board is not changed
Manual: needs the board.
- **WHEN** the board firmware runs
- **THEN** it sends no `CallerOnly`, and its game behavior does not change

### Requirement: Shells answer the time effects of crux_time
The core SHALL request its timers with the `crux_time` operations `NotifyAfter` and `ClearTimer`. A shell SHALL answer a `NotifyAfter` with its timer id when its duration has elapsed. A shell SHALL answer a `ClearTimer` with the same id, and SHALL free the timer that the id names. The iOS, macOS and watchOS apps SHALL use the shipped `TaskTimeHandler`, and the Android app SHALL use the shipped `CoroutineTimeHandler`. The web shell, the board and the acceptance test harness SHALL answer the two operations themselves.

#### Scenario: A timer fires
- **GIVEN** the board in mode select
- **WHEN** 50 ms have passed
- **THEN** a 50 ms timer is requested

#### Scenario: The board frees a cleared timer
Manual: needs the board.
- **WHEN** the core clears a pending timer on the board
- **THEN** the board removes the timer from its list, answers the clear with the same id, and never answers the timer

#### Scenario: A late answer
Manual: needs a native shell.
- **WHEN** a native shell answers a timer after the core cleared it
- **THEN** the core ignores the answer, and the game does not change
