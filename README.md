# Matter Light — Raspberry Pi Pico W

A Matter-compatible On/Off Light device running on the Raspberry Pi Pico W, written in Rust.

Uses [rs-matter-embassy](https://github.com/sysgrok/rs-matter-embassy) with the Embassy async runtime, CYW43 WiFi + BLE driver, and the Matter protocol stack.

## Requirements

- **Rust nightly** with `thumbv6m-none-eabi` target
- **probe-rs** (debug probe) or **elf2uf2-rs** (USB bootloader)

```sh
rustup default nightly
rustup target add thumbv6m-none-eabi
cargo install probe-rs-tools
```

## Build & Flash

```sh
# Build (first run auto-downloads CYW43 firmware)
cargo build

# Flash via debug probe
cargo run --release
```

## What it does

- Connects to WiFi and advertises as a Matter device
- Commissions over BLE (concurrent with WiFi)
- Exposes an **On/Off Light** cluster on Endpoint 1
- Uses test commissioning data for development

## Project structure

```
.cargo/config.toml   — target, runner, build-std config
Cargo.toml           — dependencies (embassy 0.10, cyw43 0.7, rs-matter-embassy)
build.rs             — linker setup + CYW43 firmware download
memory.x             — RP2040 memory layout (2MB flash, 264KB RAM)
src/main.rs          — Matter light device implementation
```

## References

- [rs-matter-embassy examples](https://github.com/sysgrok/rs-matter-embassy/tree/master/examples/rp)
- [Embassy](https://embassy.dev)
- [Matter specification](https://csa-iot.org/all-solutions/matter/)

## License

MIT
