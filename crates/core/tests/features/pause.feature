Feature: Pause and resume
  A shell sends Pause when its app goes to the background and Resume when
  the app comes back. Pause stops every timer and the radio. Resume sends
  the radio op of the current screen again and continues where the game
  stopped.

  Background:
    Given a fresh game
    And the game has been booted

  Scenario: Pause switches the radio off
    Given the next random bytes are 1
    When I enter receiver on channel 0
    And I clear effect logs
    And the app goes to the background
    Then BLE should have received Off

  Scenario: A paused countdown continues at the same step
    Given the next random bytes are 1
    When I enter receiver on channel 0
    And I advance time by 1000 ms
    Then the countdown step should be 2
    When the app goes to the background
    And I clear effect logs
    # The beat that was pending at the pause expires here and has no effect.
    And I advance time by 5000 ms
    Then the countdown step should be 2
    And no tone should have been played
    And no delay should have been requested
    When the app comes back
    Then the countdown should be active
    And the countdown step should be 2
    And BLE should have received a Receiver op for channel 0 and Red
    And a delay of 1000 ms should have been requested
    # The next beat comes one full beat period after Resume.
    When I advance time by 999 ms
    Then the countdown step should be 2
    When I advance time by 1 ms
    Then the countdown step should be 3
    And a countdown tone should have been played

  Scenario: Pause during the explosion starts the next round on Resume without a second explosion
    Given the next random bytes are 0, 5
    When I enter receiver on channel 0
    And I advance time by 9000 ms
    Then an explosion tone should have been played
    When the app goes to the background
    And I clear effect logs
    And I advance time by 2000 ms
    Then the countdown should not be active
    And BLE should not have received a Receiver op
    When the app comes back
    Then the countdown should be active
    And BLE should have received a Receiver op for channel 0 and Green
    And no explosion tone should have been played
    When I advance time by 1200 ms
    Then BLE should have received 1 Receiver ops
    And no explosion tone should have been played

  Scenario: A random byte that arrives while paused is dropped, and Resume requests a new one
    Given the next random bytes are 1
    When I enter receiver on channel 0
    And the shell holds random requests
    And I skip
    And the app goes to the background
    And I clear effect logs
    And the shell answers the held random request with 2
    Then the countdown should not be active
    And BLE should not have received any new ops
    Given the next random bytes are 3
    When the shell answers random requests at once
    And the app comes back
    Then the countdown should be active
    And BLE should have received a Receiver op for channel 0 and Yellow
    And BLE should have received 1 Receiver ops

  Scenario: A random byte that arrives after Resume starts only one round
    Given the next random bytes are 1
    When I enter receiver on channel 0
    And the shell holds random requests
    And I skip
    And the app goes to the background
    And the app comes back
    And I clear effect logs
    And the shell answers the held random request with 2
    Then the countdown should be active
    And BLE should have received a Receiver op for channel 0 and Orange
    When the shell answers the held random request with 3
    Then BLE should have received 1 Receiver ops

  Scenario: Resume restores the Picker radio ops
    When I enter picker on channel 1
    And I apply
    And the app goes to the background
    And I clear effect logs
    And the app comes back
    Then BLE should have received a Picker op for channel 1
    And BLE should have received a Broadcast of Red
    And the picker should name Red as applied

  Scenario: Resume restores the Picker blink while the player selects
    When I enter picker on channel 1
    And the app goes to the background
    And I clear effect logs
    And I advance time by 500 ms
    Then no delay should have been requested
    When the app comes back
    Then BLE should have received a Picker op for channel 1
    And BLE should have received a Broadcast with no color
    And a delay of 50 ms should have been requested

  Scenario: Mode select stops blinking while paused and blinks again on Resume
    When the app goes to the background
    And I clear effect logs
    And I advance time by 500 ms
    Then no delay should have been requested
    When the app comes back
    Then a delay of 50 ms should have been requested
    And BLE should not have received any new ops

  Scenario: Input while paused is ignored
    Given the next random bytes are 1
    When I enter receiver on channel 0
    And the app goes to the background
    And I clear effect logs
    And I skip
    And I apply
    And a color Red is received on channel 0
    Then the countdown should be active
    And the countdown step should be 1
    And BLE should not have received any new ops
    And no tone should have been played

  Scenario: Skip while paused does not change the mode choice
    When the app goes to the background
    And I skip
    Then the mode choice should be Picker channel 0

  Scenario: Resume when the game is not paused does nothing
    When I clear effect logs
    And the app comes back
    Then BLE should not have received any new ops
    And no tone should have been played
    And no delay should have been requested

  Scenario: A second Pause does nothing
    When the app goes to the background
    And I clear effect logs
    And the app goes to the background
    Then BLE should not have received any new ops
