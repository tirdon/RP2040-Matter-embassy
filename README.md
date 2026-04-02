# Matter Light — Raspberry Pi Pico W (Rust)

A Matter-compatible On/Off Light device for the Raspberry Pi Pico W, built with Embassy and rs-matter-embassy.

## Requirements

- **Rust nightly** with `thumbv6m-none-eabi` target
- **elf2uf2-rs** (USB bootloader flashing) or **probe-rs** (debug probe flashing + logs)

```sh
rustup default nightly
rustup target add thumbv6m-none-eabi
```

## Build & Flash

### Option 1: UF2 via USB Bootloader

1. Build the firmware:
   ```sh
   cargo build --release
   elf2uf2-rs target/thumbv6m-none-eabi/release/matter output.uf2
   ```
2. Hold **BOOTSEL** on the Pico W while plugging it into USB.
3. Copy the UF2 to the mounted volume:
   ```sh
   cp output.uf2 /Volumes/RPI-RP2/
   ```

### Option 2: Debug Probe (with log output)

Requires a Picoprobe, CMSIS-DAP adapter, or similar SWD debug probe.

```sh
cargo install probe-rs-tools
cargo run --release
```

Log output (via `defmt`/RTT) is only available with a debug probe.

## Technologies

- [Embassy](https://embassy.dev) — async runtime for embedded Rust
- [rs-matter-embassy](https://github.com/sysgrok/rs-matter-embassy) — Matter protocol stack for Embassy
- [CYW43](https://github.com/embassy-rs/embassy/tree/main/cyw43) — Pico W WiFi/BLE driver

## License

MIT
