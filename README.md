# Matter Light — Raspberry Pi Pico W (Rust) (NOT WORKING YET)

A Matter-compatible device implementation for the Raspberry Pi Pico W using the Embassy async runtime.

## Project Status

- **`src/main.rs`**: Current "Hello World" test (Blinks the onboard LED via CYW43).
- **`src/main_matter.rs`**: The full Matter On/Off Light implementation.

## Requirements

- **Rust nightly** with `thumbv6m-none-eabi` target
- **elf2uf2-rs** (for USB bootloader flashing) or **probe-rs** (for debug probes)

```sh
rustup default nightly
rustup target add thumbv6m-none-eabi
cargo install elf2uf2-rs
```

## Build & Flash (USB Bootloader)

1. **Enter BOOTSEL Mode**: Hold the **BOOTSEL** button on your Pico W while plugging it into your USB port.
2. **Build and Convert**:
   ```sh
   cargo build --release
   elf2uf2-rs target/thumbv6m-none-eabi/release/matter matter.uf2
   ```
3. **Flash**: Copy `matter.uf2` to the `RPI-RP2` volume.
   ```sh
   cp matter.uf2 /Volumes/RPI-RP2/
   ```

## Swapping Implementations

To switch back to the Matter implementation:
```sh
mv src/main.rs src/main_blink.rs
mv src/main_matter.rs src/main.rs
```

## Technologies Used

- [Embassy](https://embassy.dev): Async runtime for embedded Rust.
- [rs-matter-embassy](https://github.com/sysgrok/rs-matter-embassy): Matter protocol stack for Embassy.
- [cyw43](https://github.com/embassy-rs/embassy/tree/main/cyw43): Driver for the Pico W WiFi/BLE chip.

## License

MIT
