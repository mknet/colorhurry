Feature: Caller round
  Caller picks a random target color, counts down with beats, and
  resets on a matching ColorReceived (same channel + named color) or
  Skip. Wrong ads are ignored. After ten beats the round explodes.

  The test shell auto-resolves Random and 0 ms delays (like a fast hardware
  shell), so a match immediately starts the next round when entropy is queued.

  Background:
    Given a fresh game
    And the game has been booted

  Scenario: Entering Caller starts a round from entropy
    Given the next random bytes are 1
    When I enter caller on channel 0
    Then the countdown should be active
    # 0 ms beat kick is resolved immediately → step advances to 1
    And the countdown step should be 1
    And BLE should have received a Caller op for channel 0 and Red
    And a delay of 0 ms should have been requested

  Scenario: Matching color ends the round and starts the next from entropy
    Given the next random bytes are 1, 4
    When I enter caller on channel 0
    And I clear effect logs
    When a color Red is received on channel 0
    Then BLE should have received Off
    And BLE should have received a Caller op for channel 0 and LightGreen
    And the countdown should be active

  Scenario: Wrong channel is ignored
    Given the next random bytes are 1
    When I enter caller on channel 0
    And I clear effect logs
    When a color Red is received on channel 1
    Then the countdown should be active
    And BLE should not have received any new ops

  Scenario: Wrong color is ignored
    Given the next random bytes are 1
    When I enter caller on channel 0
    And I clear effect logs
    When a color Blue is received on channel 0
    Then the countdown should be active
    And BLE should not have received any new ops

  Scenario: Skip aborts the round and starts a new one
    Given the next random bytes are 1, 2
    When I enter caller on channel 3
    And I clear effect logs
    When I skip
    Then BLE should have received Off
    And the countdown should be active
    And BLE should have received a Caller op for channel 3 and Orange

  Scenario: Next round starts only after the explosion has played
    Given the next random bytes are 0, 5
    When I enter caller on channel 0
    And I clear effect logs
    # Beats 1 to 9 run at 1000 ms steps. The ninth beat expires the countdown.
    When I advance time by 9000 ms
    Then an explosion tone should have been played
    And BLE should have received Off
    And the countdown should not be active
    # Last beat tone (200 ms) plus explosion (10 x 100 ms).
    And a delay of 1200 ms should have been requested
    And BLE should not have received a Caller op
    When I advance time by 1199 ms
    Then the countdown should not be active
    And BLE should not have received a Caller op
    When I advance time by 1 ms
    Then the countdown should be active
    And BLE should have received a Caller op for channel 0 and Green

  Scenario: Skip during the explosion does not start a second round
    Given the next random bytes are 0, 5, 6
    When I enter caller on channel 0
    And I advance time by 9000 ms
    And I clear effect logs
    And I skip
    Then the countdown should not be active
    And BLE should not have received a Caller op
    When I advance time by 1200 ms
    Then the countdown should be active
    And BLE should have received 1 Caller ops
    When I advance time by 100 ms
    Then BLE should have received 1 Caller ops

  Scenario: After the 0 ms kick, the next beat uses BEAT_PERIOD_MS
    Given the next random bytes are 0
    When I enter caller on channel 0
    And I clear effect logs
    When I advance time by 1000 ms
    Then a delay of 1000 ms should have been requested
    And a countdown tone should have been played
    # enter already resolved step 0 → after +1000 ms we are at step 2
    And the countdown step should be 2

  Scenario: Apply does nothing in Caller
    Given the next random bytes are 3
    When I enter caller on channel 1
    And I clear effect logs
    When I apply
    Then BLE should not have received any new ops
    And the countdown should be active

  Scenario: Match requires both channel and color
    Given the next random bytes are 7, 0
    When I enter caller on channel 4
    And I clear effect logs
    When a color Blue is received on channel 0
    Then the countdown should be active
    When a color Red is received on channel 4
    Then the countdown should be active
    When a color Blue is received on channel 4
    Then BLE should have received Off
    And BLE should have received a Caller op for channel 4 and White

  Scenario: ColorReceived before a round starts is ignored
    Given the next random bytes are 1
    When a color Red is received on channel 0
    Then the screen should be ModeSelect
    And the countdown should not be active
