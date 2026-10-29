# Agent instructions

## Git commits

### Subject line (required)

Write the subject so it completes this sentence:

> If applied, this commit will **\<subject\>**

Rules:

- Start with a **lowercase** letter (imperative verb: `add`, `fix`, `document`, …)
- No trailing period on the subject
- Prefer ~50 characters; keep the first line focused on *why* / outcome

```text
✅ add native iOS and Android shells for the board entry flow
✅ filter BLE matches by channel
✅ use TIMER4 for SoftDevice-safe delays

❌ Add native iOS and Android shells…
❌ docs for receiver BLE and channel
❌ TIMER4 for SoftDevice-safe delays
❌ non-blocking UI tones
```

### Authorship (required)

- **Never** add AI/agent `Co-authored-by` trailers (any tool or bot identity)
- **Never** set the commit author or committer to an agent identity
- Commits are authored by the human user only (existing `user.name` / `user.email`)

### Body (optional)

- Separate from the subject with a blank line
- Complete sentences are fine; explain motivation and trade-offs
- Still no agent co-author trailers
