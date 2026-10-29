Feature: Mode select
  The board boots into mode select. Skip cycles role/channel on the ring;
  Apply locks the choice and enters Picker or Receiver.

  Background:
    Given a fresh game
    And the game has been booted

  Scenario: Boot lands on mode select
    Then the screen should be ModeSelect
    And the channel should be 0

  Scenario: Apply without Skip enters Picker on channel 0
    When I apply
    Then the screen should be Picker
    And the channel should be 0
    And BLE should have received a Picker op for channel 0
    And the last tone should be Apply

  Scenario Outline: Skip then Apply selects the expected role and channel
    Given the next random bytes are 0
    When I skip <skips> times
    And I apply
    Then the screen should be <screen>
    And the channel should be <channel>

    Examples:
      | skips | screen   | channel |
      | 0     | Picker   | 0       |
      | 1     | Picker   | 1       |
      | 2     | Picker   | 2       |
      | 3     | Picker   | 3       |
      | 4     | Picker   | 4       |
      | 5     | Receiver | 4       |
      | 6     | Receiver | 3       |
      | 7     | Receiver | 2       |
      | 8     | Receiver | 1       |
      | 9     | Receiver | 0       |

  Scenario: Mode step wraps after 10 Skips
    When I skip 10 times
    And I apply
    Then the screen should be Picker
    And the channel should be 0

  Scenario: Skip plays the skip tone
    When I skip
    Then the last tone should be Skip

  Scenario: Enter helpers match outline mapping
    Given the next random bytes are 7
    When I enter receiver on channel 2
    Then the screen should be Receiver
    And the channel should be 2
    And the countdown should be active
