Feature: Reset to mode select
  An app shell sends Reset when the player asks to change the mode. Reset
  stops every timer, the round and the radio, and goes back to mode select.
  Mode select starts at the role and channel that the player entered last,
  so one Apply enters them again.

  Background:
    Given a fresh game
    And the game has been booted

  Scenario: Reset from a Picker that selects a color
    When I enter picker on channel 2
    And I skip
    And I clear effect logs
    And I go back to mode select
    Then the screen should be ModeSelect
    And the mode choice should be Picker channel 2
    And there should be no picker color
    And BLE should have received Off
    And a timer of 50 ms should have been requested
    And no tone should have been played

  Scenario: Reset from a Picker that broadcasts
    When I enter picker on channel 1
    And I apply
    And I clear effect logs
    And I go back to mode select
    Then the screen should be ModeSelect
    And the mode choice should be Picker channel 1
    And there should be no picker color
    And BLE should have received Off

  Scenario: Reset from a Caller countdown
    Given the next random bytes are 1
    When I enter caller on channel 3
    And I advance time by 1000 ms
    Then the countdown step should be 2
    When I clear effect logs
    And I go back to mode select
    Then the screen should be ModeSelect
    And the mode choice should be Caller channel 3
    And the countdown should not be active
    And BLE should have received Off
    And no explosion tone should have been played
    And a timer should have been cleared
    # Reset cleared the beat that was pending, so no beat comes here.
    When I advance time by 12000 ms
    Then the screen should be ModeSelect
    And the mode choice should be Caller channel 3
    And no tone should have been played
    And BLE should not have received a Caller op

  Scenario: Apply after Reset enters the same Caller with a fresh round
    Given the next random bytes are 1, 4
    When I enter caller on channel 3
    And I advance time by 3000 ms
    And I go back to mode select
    And I clear effect logs
    And I apply
    Then the screen should be Caller
    And the channel should be 3
    And the countdown should be active
    And the countdown step should be 1
    And BLE should have received a Caller op for channel 3 and LightGreen

  Scenario: Apply after Reset enters the same Picker
    When I enter picker on channel 4
    And I apply
    And I go back to mode select
    And I clear effect logs
    And I apply
    Then the screen should be Picker
    And the channel should be 4
    And the picker should name LightGreen as selected
    And BLE should have received a Picker op for channel 4
    And BLE should have received a Broadcast with no color

  Scenario: Skip after Reset moves on from the last choice
    When I enter picker on channel 4
    And I go back to mode select
    And I skip
    Then the mode choice should be Caller channel 4

  Scenario: Reset in mode select keeps the choice
    When I skip 3 times
    And I clear effect logs
    And I go back to mode select
    Then the screen should be ModeSelect
    And the mode choice should be Picker channel 3
    And a timer of 50 ms should have been requested
    And no tone should have been played

  Scenario: Reset while paused is ignored
    Given the next random bytes are 1
    When I enter caller on channel 0
    And the app goes to the background
    And I clear effect logs
    And I go back to mode select
    Then the screen should be Caller
    And the countdown should be active
    And BLE should not have received any new ops
    And no timer should have been requested
