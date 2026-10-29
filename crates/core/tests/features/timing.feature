Feature: Blink and beat timing
  The core owns blink and beat periods via Delay effects. The shell only
  advances virtual time and resolves due requests.

  Background:
    Given a fresh game
    And the game has been booted

  Scenario: Boot schedules a blink delay
    Then a delay of 50 ms should have been requested

  Scenario: Advancing blink period keeps scheduling blinks
    When I advance time by 50 ms
    Then a delay of 50 ms should have been requested at least 2 times

  Scenario: After six blink ticks the mode-select LED blinks off
    # BLINK_TICKS = 6 half-periods of 50 ms → first toggle after 300 ms
    When I advance time by 300 ms
    Then LED 0 should be off

  Scenario: Stale blink delays after Skip are ignored without crashing
    When I skip
    And I advance time by 500 ms
    Then the screen should be ModeSelect
