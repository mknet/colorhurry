MEMORY
{
  /* Circuit Playground Bluefruit: S140 6.x (6.1.1), App @ 0x26000.
   * The app never enables the SoftDevice. Bluetooth runs on the SoftDevice Controller
   * (nrf-sdc), which is linked into the app. RAM starts at 0x20006000, as in the S140 7.x
   * layout. The same layout runs the nrf-sdc firmware in the crux counter-http example on
   * this board. */
  FLASH : ORIGIN = 0x26000, LENGTH = 0xC7000
  RAM   : ORIGIN = 0x20006000, LENGTH = 0x3A000
}
