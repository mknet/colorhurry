# Tasks

## 1. Android

- [x] 1.1 Call `stopAdvertising(advertiseCallback)` in `stopAdvertise()` without the `advertising` check
- [x] 1.2 Call `stopAdvertising` in `onStartFailure`
- [x] 1.3 Keep the wanted color, and retry a start that failed with `2` or `4` after 1 s, up to 3 times, while the wanted color stays the same. Clear the wanted color and the pending retries at `Broadcast(None)`, `Off` and `Caller`
- [x] 1.4 Name the error codes in words in the status, and log them with `Log.w`
- [x] 1.5 Remove the unused scan response
- [x] 1.6 Verify with `just android-build`

## 2. Archive

- [x] 2.1 Apply the delta to `openspec/specs`
- [x] 2.2 Move the change to `openspec/changes/archive/2026-11-05-fix-android-advertise-failure`
