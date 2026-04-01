Set up the development environment for this project.

Check and install if missing:
1. `rustup default nightly`
2. `rustup target add thumbv6m-none-eabi`
3. `cargo install probe-rs-tools`
4. `cargo install elf2uf2-rs` (alternative USB flashing)

Then run `cargo build` to verify everything works and trigger CYW43 firmware download.
