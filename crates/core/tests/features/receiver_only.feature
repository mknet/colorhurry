Feature: Receiver-only shells
  A shell that cannot broadcast a color cannot be Picker. It sends
  ReceiverOnly once, before Boot. Mode select then offers only the Receiver
  choices. It starts at Receiver on channel 0, because the board Picker
  starts on channel 0. Skip follows the ring order and wraps from channel 0
  to channel 4, so the channels come in the order 0, 4, 3, 2, 1, 0.

  Background:
    Given a fresh game

  Scenario: Without ReceiverOnly, mode select still offers Picker
    Given the game has been booted
    Then the mode choice should be Picker channel 0
    When I skip 4 times
    Then the mode choice should be Picker channel 4
    When I skip
    Then the mode choice should be Receiver channel 4

  Scenario: Boot names Receiver on channel 0
    Given the shell cannot broadcast
    And the game has been booted
    Then the screen should be ModeSelect
    And the mode choice should be Receiver channel 0

  Scenario Outline: Skip cycles only the Receiver channels
    Given the shell cannot broadcast
    And the game has been booted
    When I skip <skips> times
    Then the mode choice should be Receiver channel <channel>

    Examples:
      | skips | channel |
      | 1     | 4       |
      | 2     | 3       |
      | 3     | 2       |
      | 4     | 1       |
      | 5     | 0       |
      | 6     | 4       |
      | 10    | 0       |

  Scenario: Skip plays the skip tone
    Given the shell cannot broadcast
    And the game has been booted
    When I skip
    Then the last tone should be Skip

  Scenario: Apply enters Receiver on the chosen channel
    Given the shell cannot broadcast
    And the game has been booted
    And the next random bytes are 1
    When I skip 2 times
    And I clear effect logs
    And I apply
    Then the screen should be Receiver
    And the channel should be 3
    And the countdown should be active
    And BLE should have received a Receiver op for channel 3 and Red

  Scenario: Apply without Skip enters Receiver on channel 0
    Given the shell cannot broadcast
    And the game has been booted
    And the next random bytes are 1
    When I apply
    Then the screen should be Receiver
    And the channel should be 0

  Scenario: Reset goes back to the last Receiver choice
    Given the shell cannot broadcast
    And the game has been booted
    And the next random bytes are 1
    When I skip 2 times
    And I apply
    And I go back to mode select
    Then the screen should be ModeSelect
    And the mode choice should be Receiver channel 3
    When I skip
    Then the mode choice should be Receiver channel 2

  Scenario: Reset after Receiver on channel 0 and Skip wraps to channel 4
    Given the shell cannot broadcast
    And the game has been booted
    And the next random bytes are 1
    When I skip 5 times
    And I apply
    And I go back to mode select
    And I skip
    Then the mode choice should be Receiver channel 4

  Scenario: ReceiverOnly a second time changes nothing
    Given the shell cannot broadcast
    And the game has been booted
    When I skip 3 times
    And I clear effect logs
    And the shell says that it cannot broadcast
    Then the mode choice should be Receiver channel 2
    And BLE should not have received any new ops
    And no tone should have been played
    And no delay should have been requested

  Scenario: ReceiverOnly moves a Picker choice to Receiver on channel 0
    Given the game has been booted
    When I skip 2 times
    Then the mode choice should be Picker channel 2
    When the shell says that it cannot broadcast
    Then the screen should be ModeSelect
    And the mode choice should be Receiver channel 0
    When I skip
    Then the mode choice should be Receiver channel 4

  Scenario: ReceiverOnly keeps a Receiver choice
    Given the game has been booted
    When I skip 7 times
    And the shell says that it cannot broadcast
    Then the mode choice should be Receiver channel 2

  Scenario: ReceiverOnly while paused is accepted
    Given the game has been booted
    When the app goes to the background
    And I clear effect logs
    And the shell says that it cannot broadcast
    Then the mode choice should be Receiver channel 0
    And BLE should not have received any new ops
    And no delay should have been requested
    When the app comes back
    And I skip
    Then the mode choice should be Receiver channel 4
