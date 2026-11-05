# Tasks

## 1. Android

- [ ] 1.1 Call `stopAdvertising(advertiseCallback)` in `stopAdvertise()` without the `advertising` check
- [ ] 1.2 Call `stopAdvertising` in `onStartFailure`
- [ ] 1.3 Keep the wanted color, and retry a start that failed with `2` or `4` after 1 s, up to 3 times, while the wanted color stays the same. Clear the wanted color and the pending retries at `Broadcast(None)`, `Off` and `Caller`
- [ ] 1.4 Name the error codes in words in the status, and log them with `Log.w`
- [ ] 1.5 Remove the unused scan response
- [ ] 1.6 Verify with `just android-build`

## 2. Archive

- [ ] 2.1 Apply the delta to `openspec/specs`
- [ ] 2.2 Move the change to `openspec/changes/archive/<date>-fix-android-advertise-failure`
