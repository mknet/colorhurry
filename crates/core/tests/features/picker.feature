Feature: Color Picker
  After entering Picker, Skip cycles the spectrum and Apply broadcasts
  the named color on the chosen channel (protocol v4).

  Background:
    Given a fresh game
    And the game has been booted

  Scenario: Entering Picker clears broadcast and arms picker BLE
    When I enter picker on channel 1
    Then BLE should have received a Picker op for channel 1
    And BLE should have received a Broadcast with no color

  Scenario: Apply broadcasts White (first spectrum color)
    When I enter picker on channel 0
    And I clear effect logs
    And I apply
    Then BLE should have received a Broadcast of White
    And all LEDs should show White
    And the last tone should be Apply

  Scenario: Skip cycles to Red then Apply broadcasts Red
    When I enter picker on channel 0
    And I clear effect logs
    And I skip
    And I apply
    Then BLE should have received a Broadcast of Red
    And all LEDs should show Red

  Scenario Outline: Spectrum index maps to named color broadcast
    # Entering Picker seeds picker_selected from the channel index; use ch 0
    # so Skip count equals the spectrum index.
    When I enter picker on channel 0
    And I clear effect logs
    And I skip <skips> times
    And I apply
    Then BLE should have received a Broadcast of <color>

    Examples:
      | skips | color      |
      | 0     | White      |
      | 1     | Red        |
      | 2     | Orange     |
      | 3     | Yellow     |
      | 4     | LightGreen |
      | 5     | Green      |
      | 6     | LightBlue  |
      | 7     | Blue       |
      | 8     | Purple     |
      | 9     | Pink       |

  Scenario: Skip after Apply withdraws the broadcast
    When I enter picker on channel 0
    And I apply
    And I clear effect logs
    And I skip
    Then BLE should have received a Broadcast with no color

  Scenario: Apply after Apply also withdraws the broadcast
    When I enter picker on channel 2
    And I apply
    And I clear effect logs
    And I apply
    Then BLE should have received a Broadcast with no color

  Scenario: ColorReceived is ignored while Picker
    When I enter picker on channel 0
    And I clear effect logs
    When a color Red is received on channel 0
    Then BLE should not have received any new ops
    And the screen should be Picker
