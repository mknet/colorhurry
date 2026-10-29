Feature: Receiver round
  Receiver picks a random target color, counts down with beats, and
  resets on a matching ColorReceived (same channel + named color) or
  Skip. Wrong ads are ignored. After ten beats the round explodes.

  The test shell auto-resolves Random and 0 ms delays (like a fast hardware
  shell), so a match immediately starts the next round when entropy is queued.

  Background:
    Given a fresh game
    And the game has been booted

  Scenario: Entering Receiver starts a round from entropy
    Given the next random bytes are 1
    When I enter receiver on channel 0
    Then the countdown should be active
    # 0 ms beat kick is resolved immediately → step advances to 1
    And the countdown step should be 1
    And BLE should have received a Receiver op for channel 0 and Red
    And a delay of 0 ms should have been requested

  Scenario: Matching color ends the round and starts the next from entropy
    Given the next random bytes are 1, 4
    When I enter receiver on channel 0
    And I clear effect logs
    When a color Red is received on channel 0
    Then BLE should have received Off
    And BLE should have received a Receiver op for channel 0 and LightGreen
    And the countdown should be active

  Scenario: Wrong channel is ignored
    Given the next random bytes are 1
    When I enter receiver on channel 0
    And I clear effect logs
    When a color Red is received on channel 1
    Then the countdown should be active
    And BLE should not have received any new ops

  Scenario: Wrong color is ignored
    Given the next random bytes are 1
    When I enter receiver on channel 0
    And I clear effect logs
    When a color Blue is received on channel 0
    Then the countdown should be active
    And BLE should not have received any new ops

  Scenario: Skip aborts the round and starts a new one
    Given the next random bytes are 1, 2
    When I enter receiver on channel 3
    And I clear effect logs
    When I skip
    Then BLE should have received Off
    And the countdown should be active
    And BLE should have received a Receiver op for channel 3 and Orange

  Scenario: Full countdown explodes then starts a new round
    Given the next random bytes are 0, 5
    When I enter receiver on channel 0
    And I clear effect logs
    When the full countdown runs
    Then an explosion tone should have been played
    And BLE should have received Off
    And the countdown should be active
    And BLE should have received a Receiver op for channel 0 and Green

  Scenario: After the 0 ms kick, the next beat uses BEAT_PERIOD_MS
    Given the next random bytes are 0
    When I enter receiver on channel 0
    And I clear effect logs
    When I advance time by 1000 ms
    Then a delay of 1000 ms should have been requested
    And a countdown tone should have been played
    # enter already resolved step 0 → after +1000 ms we are at step 2
    And the countdown step should be 2

  Scenario: Apply does nothing in Receiver
    Given the next random bytes are 3
    When I enter receiver on channel 1
    And I clear effect logs
    When I apply
    Then BLE should not have received any new ops
    And the countdown should be active

  Scenario: Match requires both channel and color
    Given the next random bytes are 7, 0
    When I enter receiver on channel 4
    And I clear effect logs
    When a color Blue is received on channel 0
    Then the countdown should be active
    When a color Red is received on channel 4
    Then the countdown should be active
    When a color Blue is received on channel 4
    Then BLE should have received Off
    And BLE should have received a Receiver op for channel 4 and White

  Scenario: ColorReceived before a round starts is ignored
    Given the next random bytes are 1
    When a color Red is received on channel 0
    Then the screen should be ModeSelect
    And the countdown should not be active
