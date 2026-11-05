Feature: Blink and beat timing
  The core owns blink and beat periods through crux_time timers. The shell
  only advances virtual time, fires the timers that are due and frees the
  timers that the core clears.

  Background:
    Given a fresh game
    And the game has been booted

  Scenario: Boot schedules a blink timer
    Then a timer of 50 ms should have been requested

  Scenario: Advancing blink period keeps scheduling blinks
    When I advance time by 50 ms
    Then a timer of 50 ms should have been requested at least 2 times

  Scenario: After six blink ticks the mode-select LED blinks off
    # BLINK_TICKS = 6 half-periods of 50 ms → first toggle after 300 ms
    When I advance time by 300 ms
    Then LED 0 should be off

  Scenario: Skip clears the pending blink timer, and the old timer has no effect
    # The blink timer from the tick at 100 ms is due at 150 ms.
    When I advance time by 120 ms
    And I clear effect logs
    And I skip
    Then a timer should have been cleared
    And a timer of 50 ms should have been requested
    # Six ticks from the Skip at 120 ms toggle the blink at 420 ms. An old
    # timer that still fired would add ticks and toggle it earlier.
    When I advance time by 299 ms
    Then the screen should be ModeSelect
    And LED 1 should be off
    When I advance time by 1 ms
    Then LED 1 should be lit
