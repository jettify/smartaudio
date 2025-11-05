# SmartAudio
[![CI](https://github.com/jettify/smartaudio/actions/workflows/CI.yml/badge.svg)](https://github.com/jettify/smartaudio/actions/workflows/CI.yml)
[![codecov](https://codecov.io/gh/jettify/smartaudio/graph/badge.svg?token=RCM2W4C0LB)](https://codecov.io/gh/jettify/smartaudio)
[![crates.io](https://img.shields.io/crates/v/smartaudio)](https://crates.io/crates/smartaudio)
[![docs.rs](https://img.shields.io/docsrs/smartaudio)](https://docs.rs/smartaudio/latest/smartaudio/)

This is a `no_std` platform-agnostic implementation of the TBS `SmartAudio` protocol in Rust.

## Features

* `no_std`, does not use the standard library or an allocator.
* Platform Agnostic, can be used on any MCU or platform.
* Provides a low-level interface to slice byte stream into valid frames.
* Supports `SmartAudio` protocols `1.0`, `2.0` and `2.1`.

## Usage Example

Here is a basic example of how to parse a raw `SmartAudio` frame:

```rust
use smartaudio::SmartAudioParser;

fn main() {
    let mut parser = SmartAudioParser::new();

    let raw_settings_v20: [u8; 10] = [
        0xAA, 0x55, // Headers
        0x09, // Version/Command
        0x06, // Length
        0x01, // Channel
        0x00, // Power Level
        0x1A, // Operation/Mode
        0x16, 0xE9, // Current Frequency 5865
        0x0A, // CRC8
    ];
    println!("Parssing packet:");
    for b in &raw_settings_v20[0..raw_settings_v20.len()] {
        let reult = parser.push_byte_raw(*b);
        match reult {
            Ok(Some(f)) => println!("{:?}", f),
            Err(e) => eprintln!("Error parsing packet: {:?}", e),
            _ => (),
        }
    }
}
```

## Installation

Add `smartaudio` to your `Cargo.toml`:

```toml
[dependencies]
smartaudio = "*" # replace * by the latest version of the crate.
```

Or use the command line:

```bash
cargo add smartaudio
```

## License

This project is licensed under the `Apache 2.0`. See the [LICENSE](https://github.com/jettify/smartaudio/blob/master/LICENSE) file for details.

## Protocol Specification

[tbs_smartaudio_rev09.pdf](https://www.team-blacksheep.com/tbs_smartaudio_rev09.pdf)
