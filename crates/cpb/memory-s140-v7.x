MEMORY
{
  /* Circuit Playground Bluefruit: S140 7.x, App @ 0x27000.
   * The app never enables the SoftDevice. Bluetooth runs on the SoftDevice Controller
   * (nrf-sdc), which is linked into the app. */
  FLASH : ORIGIN = 0x27000, LENGTH = 0xC6000
  RAM   : ORIGIN = 0x20006000, LENGTH = 0x3A000
}
