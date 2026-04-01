# Matter on Raspberry Pi Pico W

## Project overview
Matter smart home device (On/Off Light) running on RP2040 Pico W using Rust.

## Stack
- **rs-matter-embassy** (sysgrok fork) — Matter protocol + embassy integration
- **embassy 0.10** — async embedded runtime
- **cyw43 0.7** — Pico W WiFi + BLE driver
- **probe-rs** — flash/debug tool

## Build
```sh
# Requires nightly Rust
rustup default nightly
rustup target add thumbv6m-none-eabi
cargo install probe-rs-tools

# Build (first run downloads CYW43 firmware automatically)
cargo build

# Flash via debug probe
cargo run
```

## Key constraints
- **RP2040 has 264KB RAM** — Matter stack is ~35-50KB, keep heap small (8KB)
- **LED is behind CYW43** — use `control.gpio_set(0, true/false)`, NOT GPIO25 directly
- **Nightly required** — `.cargo/config.toml` uses `build-std = ["core", "alloc"]`
- **No std** — `#![no_std]`, `#![no_main]`, use `embedded-alloc` for heap

## Dependencies (patched from git)
- `rs-matter` → sysgrok/rs-matter `next` branch
- `rs-matter-stack` → sysgrok/rs-matter-stack `next` branch
- `openthread` → sysgrok/openthread `next` branch

## Reference
- Example source: https://github.com/sysgrok/rs-matter-embassy/tree/master/examples/rp
- CYW43 firmware: auto-downloaded from embassy-rs/embassy by `build.rs`

## Code conventions
- Static allocation preferred over heap (`mk_static!` macro, `StaticCell`)
- Firmware blobs loaded via `cyw43::aligned_bytes!` macro (paths relative to source file)
- `defmt` for logging, `rtt-target` as transport
- Test commissioning data used for dev (`TEST_DEV_COMM`, `TEST_DEV_ATT`)
