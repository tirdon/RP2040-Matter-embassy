# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview
Matter smart home device (On/Off Light) on RP2040 Pico W in Rust. Currently on `helloworld` branch with a CYW43 LED blink test in `src/main.rs`.

## Build and flash

```sh
# Prerequisites (nightly required for build-std)
rustup default nightly
rustup target add thumbv6m-none-eabi

# Build (first build auto-downloads CYW43 firmware blobs via build.rs)
cargo build --release

# Flash option 1: debug probe (shows defmt logs via RTT)
cargo install probe-rs-tools
cargo run --release

# Flash option 2: UF2 via USB bootloader (no log output)
cargo install elf2uf2-rs
elf2uf2-rs target/thumbv6m-none-eabi/release/matter output.uf2
# Hold BOOTSEL while plugging in Pico, then:
cp output.uf2 /Volumes/RPI-RP2/
```

Skip firmware download on rebuild: `cargo build --features skip-cyw43-firmware`

## Architecture

- **Single binary** -- `src/main.rs` is the entire firmware entry point (`#![no_std]`, `#![no_main]`)
- **build.rs** -- copies `memory.x` linker script to OUT_DIR and downloads CYW43 firmware blobs from embassy-rs/embassy into `cyw43-firmware/`
- **memory.x** -- RP2040 memory layout (2MB flash, 264KB RAM)
- **.cargo/config.toml** -- sets target to `thumbv6m-none-eabi`, configures probe-rs runner, enables `build-std = ["core", "alloc"]`

## Key constraints

- **RP2040 has 264KB RAM** -- Matter stack uses ~35-50KB; keep heap small (currently 1KB for blink, use 8KB for Matter)
- **LED is behind CYW43** -- use `control.gpio_set(0, true/false)`, NOT direct GPIO25
- **CYW43 init requires NVRAM** -- `cyw43::new()` takes 5 params: `(state, pwr, spi, fw, nvram)`. The BT firmware blob is only used with `cyw43::new_with_bluetooth()` (6 params)
- **Logging is RTT-only** -- `defmt` + `rtt-target` requires a debug probe; does NOT create a USB serial device (`/dev/tty.usbmodem*` will not appear)
- **Nightly required** -- `build-std` in `.cargo/config.toml` is an unstable feature
- **No host crates in [dependencies]** -- anything pulling in `std` will fail to compile for `thumbv6m-none-eabi`; host-only tools like `probe-rs-tools` must not be in Cargo.toml

## Dependencies (patched from git)

`Cargo.toml` uses `[patch.crates-io]` to point at sysgrok forks (`next` branch):
- `rs-matter`, `rs-matter-stack`, `openthread`
- `rs-matter-embassy` is a direct git dependency (sysgrok fork)

## Code conventions

- Static allocation preferred over heap (`StaticCell`, `mk_static!` macro)
- Firmware blobs loaded via `cyw43::aligned_bytes!` macro (paths relative to source file)
- `defmt` for logging (`defmt::info!`, `defmt::debug!`), `rtt-target` as transport
- Embassy async tasks spawned via `#[embassy_executor::task]` functions
- Test commissioning data (`TEST_DEV_COMM`, `TEST_DEV_ATT`) used for Matter dev builds

## Reference

- Working Pico W example: https://github.com/sysgrok/rs-matter-embassy/tree/master/examples/rp
- CYW43 firmware source: https://github.com/embassy-rs/embassy/tree/main/cyw43-firmware
