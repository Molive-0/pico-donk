# Pico Donk

some cool music, run in realtime on a pi pico.
Made as part of the soton ecss 2021 nanohack, in about 8-9 hours (of which at least 3 hours was getting the toolchain compiling)

Based on the template at https://github.com/rp-rs/rp2040-project-template

## Requirements
- The standard Rust tooling (cargo, rustup) which you can install from https://rustup.rs/

- Toolchain support for the cortex-m0+ processors in the rp2040 (thumbv6m-none-eabi)

## Installation of development dependencies
```
rustup target install thumbv6m-none-eabi
```

## Running

For a debug build
```
cargo run
```
For a release build
```
cargo run --release
```
  
## License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
