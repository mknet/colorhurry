# Spec Delta

## ADDED Requirements

### Requirement: Android Picker frees a failed advertiser
When the Android advertiser fails to start, the Android app SHALL free the advertiser at once, so that the next start does not fail with `ADVERTISE_FAILED_ALREADY_STARTED`. Each stop SHALL free the advertiser, also when the start has not yet succeeded.

#### Scenario: Apply after a failure
Manual: needs an Android phone.
- **WHEN** a start failed with code 2, an advertising set is free again, and the player applies a color
- **THEN** the color goes on air, and the start does not fail with code 3

#### Scenario: Skip right after Apply
Manual: needs two Android phones.
- **WHEN** the Android Picker applies a color and the player presses Skip before the start succeeds
- **THEN** no payload stays on air

### Requirement: Android Picker retries and names a failed advertise start
After a failure with `ADVERTISE_FAILED_TOO_MANY_ADVERTISERS` or `ADVERTISE_FAILED_INTERNAL_ERROR`, the Android app SHALL retry after 1 s, up to 3 times, while the core still wants the same color on air. `Broadcast` with no color, `Off` and `Caller` SHALL cancel the pending retries. The status SHALL name the error in words and its code.

#### Scenario: No free advertiser
Manual: needs an Android phone whose advertising sets are all in use.
- **WHEN** the Android Picker applies a color and the start fails with code 2
- **THEN** the status names "no free BLE advertiser (2)" and tells the user to close other Bluetooth apps or switch Bluetooth off and on
